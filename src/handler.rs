use bytes::BytesMut;
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use libp2p::{
    core::upgrade::{InboundUpgrade, OutboundUpgrade},
    swarm::{
        KeepAlive, NegotiatedSubstream, ProtocolsHandler, ProtocolsHandlerEvent,
        ProtocolsHandlerUpgrErr, SubstreamProtocol,
    },
};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use crate::protocol::PerfProtocolConfig;

// iPerf works by writing an array of len bytes a number of times. Default is
// 128 KB for TCP, 8 KB for UDP.
//
// https://iperf.fr/iperf-doc.php
const BUFFER_SIZE: usize = 128_000;

#[derive(Default)]
pub struct PerfHandler {
    outbox: Vec<
        ProtocolsHandlerEvent<
            <Self as ProtocolsHandler>::OutboundProtocol,
            <Self as ProtocolsHandler>::OutboundOpenInfo,
            <Self as ProtocolsHandler>::OutEvent,
            <Self as ProtocolsHandler>::Error,
        >,
    >,
    perf_runs:
        FuturesUnordered<
            PerfRun<
                <<Self as ProtocolsHandler>::InboundProtocol as InboundUpgrade<
                    NegotiatedSubstream,
                >>::Output,
                <<Self as ProtocolsHandler>::OutboundProtocol as OutboundUpgrade<
                    NegotiatedSubstream,
                >>::Output,
            >,
        >,
}

enum PerfRun<I, O> {
    Running {
        start: Option<std::time::Instant>,
        transfered: usize,
        substream: PerfRunStream<I, O>,
    },
    Closing {
        duration: Duration,
        transfered: usize,
        substream: O,
    },
    Done {
        duration: std::time::Duration,
        transfered: usize,
    },
    Poisoned,
}

impl<I, O> PerfRun<I, O> {
    fn new(substream: PerfRunStream<I, O>) -> Self {
        PerfRun::Running {
            start: None,
            transfered: 0,
            substream,
        }
    }
}

enum PerfRunStream<I, O> {
    Receiver(I),
    Sender(O),
}

impl<I, O> Unpin for PerfRun<I, O> {}

impl<
        I: Stream<Item = Result<BytesMut, std::io::Error>> + Unpin,
        O: Sink<std::io::Cursor<Vec<u8>>> + Unpin,
    > Future for PerfRun<I, O>
{
    type Output = (Duration, usize);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        loop {
            match std::mem::replace(&mut *self, PerfRun::Poisoned) {
                PerfRun::Running {
                    start,
                    transfered,
                    substream: PerfRunStream::Sender(mut substream),
                } => {
                    if let Some(start) = start {
                        if start.elapsed() >= Duration::from_secs(10) {
                            // TODO: Do we need a closing to flush?
                            *self = PerfRun::Closing {
                                duration: start.elapsed(),
                                transfered,
                                substream,
                            };

                            continue;
                        }
                    }

                    if let Poll::Pending = substream.poll_ready_unpin(cx) {
                        *self = PerfRun::Running {
                            start,
                            transfered,
                            substream: PerfRunStream::Sender(substream),
                        };
                        return Poll::Pending;
                    }

                    let start = start.or_else(|| Some(Instant::now()));

                    if substream
                        .start_send_unpin(std::io::Cursor::new([0; BUFFER_SIZE].to_vec()))
                        .is_err()
                    {
                        panic!("sending failed");
                    }

                    let transfered = transfered + BUFFER_SIZE;

                    *self = PerfRun::Running {
                        start,
                        transfered,
                        substream: PerfRunStream::Sender(substream),
                    };
                }
                PerfRun::Running {
                    start,
                    transfered,
                    substream: PerfRunStream::Receiver(mut substream),
                } => match substream.poll_next_unpin(cx) {
                    Poll::Ready(Some(msg)) => {
                        let start = start.or_else(|| Some(Instant::now()));
                        let len = msg.unwrap().len();
                        let transfered = transfered + len;

                        *self = PerfRun::Running {
                            start,
                            transfered,
                            substream: PerfRunStream::Receiver(substream),
                        };
                    }
                    Poll::Ready(None) => {
                        *self = PerfRun::Done {
                            duration: start.unwrap().elapsed(),
                            transfered,
                        };

                        continue;
                    }
                    Poll::Pending => {
                        *self = PerfRun::Running {
                            start,
                            transfered,
                            substream: PerfRunStream::Receiver(substream),
                        };
                        return Poll::Pending;
                    }
                },
                PerfRun::Closing {
                    duration,
                    transfered,
                    mut substream,
                } => match substream.poll_flush_unpin(cx) {
                    Poll::Ready(Ok(())) => {
                        match substream.poll_close_unpin(cx) {
                            Poll::Ready(Ok(())) => {}
                            _ => panic!("unxpected"),
                        };
                        drop(substream);
                        std::thread::sleep(Duration::from_secs(1));
                        *self = PerfRun::Done {
                            duration,
                            transfered,
                        };
                        continue;
                    }
                    Poll::Ready(Err(_)) => panic!("Got error while closing substream"),
                    Poll::Pending => {
                        *self = PerfRun::Closing {
                            duration,
                            transfered,
                            substream,
                        };
                        return Poll::Pending;
                    }
                },
                PerfRun::Done {
                    duration,
                    transfered,
                } => {
                    return Poll::Ready((duration, transfered));
                }
                PerfRun::Poisoned => panic!("PerfRun::Poisoned"),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PerfHandlerIn {
    StartPerf,
}

pub enum PerfHandlerOut {
    PerfRunDone(Duration, usize),
}

impl ProtocolsHandler for PerfHandler {
    type InEvent = PerfHandlerIn;
    /// Custom event that can be produced by the handler and that will be returned to the outside.
    type OutEvent = PerfHandlerOut;
    /// The type of errors returned by [`ProtocolsHandler::poll`].
    type Error = io::Error;
    /// The inbound upgrade for the protocol(s) used by the handler.
    type InboundProtocol = PerfProtocolConfig;
    /// The outbound upgrade for the protocol(s) used by the handler.
    type OutboundProtocol = PerfProtocolConfig;
    /// The type of additional information passed to an `OutboundSubstreamRequest`.
    type OutboundOpenInfo = ();

    /// The [`InboundUpgrade`](libp2p_core::upgrade::InboundUpgrade) to apply on inbound
    /// substreams to negotiate the desired protocols.
    ///
    /// > **Note**: The returned `InboundUpgrade` should always accept all the generally
    /// >           supported protocols, even if in a specific context a particular one is
    /// >           not supported, (eg. when only allowing one substream at a time for a protocol).
    /// >           This allows a remote to put the list of supported protocols in a cache.
    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol> {
        SubstreamProtocol::new(PerfProtocolConfig {})
    }

    /// Injects the output of a successful upgrade on a new inbound substream.
    fn inject_fully_negotiated_inbound(
        &mut self,
        substream: <Self::InboundProtocol as InboundUpgrade<NegotiatedSubstream>>::Output,
    ) {
        self.perf_runs
            .push(PerfRun::new(PerfRunStream::Receiver(substream)));
    }

    /// Injects the output of a successful upgrade on a new outbound substream.
    ///
    /// The second argument is the information that was previously passed to
    /// [`ProtocolsHandlerEvent::OutboundSubstreamRequest`].
    fn inject_fully_negotiated_outbound(
        &mut self,
        substream: <Self::OutboundProtocol as OutboundUpgrade<NegotiatedSubstream>>::Output,
        _info: Self::OutboundOpenInfo,
    ) {
        self.perf_runs
            .push(PerfRun::new(PerfRunStream::Sender(substream)));
    }

    /// Injects an event coming from the outside in the handler.
    fn inject_event(&mut self, event: Self::InEvent) {
        match event {
            PerfHandlerIn::StartPerf => {
                self.outbox
                    .push(ProtocolsHandlerEvent::OutboundSubstreamRequest {
                        protocol: SubstreamProtocol::new(PerfProtocolConfig {}),
                        info: (),
                    })
            }
        }
    }

    /// Indicates to the handler that upgrading a substream to the given protocol has failed.
    fn inject_dial_upgrade_error(
        &mut self,
        _info: Self::OutboundOpenInfo,
        error: ProtocolsHandlerUpgrErr<
            <Self::OutboundProtocol as OutboundUpgrade<NegotiatedSubstream>>::Error,
        >,
    ) {
        panic!("{:?}", error);
    }

    /// Returns until when the connection should be kept alive.
    ///
    /// This method is called by the `Swarm` after each invocation of
    /// [`ProtocolsHandler::poll`] to determine if the connection and the associated
    /// `ProtocolsHandler`s should be kept alive as far as this handler is concerned
    /// and if so, for how long.
    ///
    /// Returning [`KeepAlive::No`] indicates that the connection should be
    /// closed and this handler destroyed immediately.
    ///
    /// Returning [`KeepAlive::Until`] indicates that the connection may be closed
    /// and this handler destroyed after the specified `Instant`.
    ///
    /// Returning [`KeepAlive::Yes`] indicates that the connection should
    /// be kept alive until the next call to this method.
    ///
    /// > **Note**: The connection is always closed and the handler destroyed
    /// > when [`ProtocolsHandler::poll`] returns an error. Furthermore, the
    /// > connection may be closed for reasons outside of the control
    /// > of the handler.
    fn connection_keep_alive(&self) -> KeepAlive {
        KeepAlive::Yes
    }

    fn poll(
        &mut self,
        cx: &mut Context,
    ) -> Poll<
        ProtocolsHandlerEvent<
            Self::OutboundProtocol,
            Self::OutboundOpenInfo,
            Self::OutEvent,
            Self::Error,
        >,
    > {
        if let Some(event) = self.outbox.pop() {
            return Poll::Ready(event);
        }

        match self.perf_runs.poll_next_unpin(cx) {
            Poll::Ready(Some((duration, transfered))) => {
                return Poll::Ready(ProtocolsHandlerEvent::Custom(PerfHandlerOut::PerfRunDone(
                    duration, transfered,
                )));
            }
            // No Futures within `self.perf_runs`.
            Poll::Ready(None) => {}
            Poll::Pending => {}
        }

        Poll::Pending
    }
}
