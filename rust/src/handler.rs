use futures::prelude::*;
use futures::stream::FuturesUnordered;
use libp2p::{
    core::upgrade::{InboundUpgrade, OutboundUpgrade},
    swarm::{
        handler::{
            ConnectionEvent, DialUpgradeError, FullyNegotiatedInbound, FullyNegotiatedOutbound,
            ListenUpgradeError,
        },
        ConnectionHandler, ConnectionHandlerEvent, KeepAlive, NegotiatedSubstream,
        SubstreamProtocol,
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

const MSG: &[u8] = &[0u8; BUFFER_SIZE];

#[derive(Default)]
pub struct PerfHandler {
    outbox: Vec<
        ConnectionHandlerEvent<
            <Self as ConnectionHandler>::OutboundProtocol,
            <Self as ConnectionHandler>::OutboundOpenInfo,
            <Self as ConnectionHandler>::OutEvent,
            <Self as ConnectionHandler>::Error,
        >,
    >,
    perf_runs:
        FuturesUnordered<
            PerfRun<
                <<Self as ConnectionHandler>::InboundProtocol as InboundUpgrade<
                    NegotiatedSubstream,
                >>::Output,
                <<Self as ConnectionHandler>::OutboundProtocol as OutboundUpgrade<
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
    ClosingWriter {
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
    // Receiver + void buffer.
    Receiver(I, Vec<u8>),
    Sender(O),
}

impl<I, O> Unpin for PerfRun<I, O> {}

impl<I, O> Future for PerfRun<I, O>
where
    I: AsyncRead + AsyncWrite + Unpin,
    O: AsyncRead + AsyncWrite + Unpin,
{
    type Output = (Duration, usize);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        loop {
            match std::mem::replace(&mut *self, PerfRun::Poisoned) {
                PerfRun::Running {
                    mut start,
                    transfered,
                    substream: PerfRunStream::Sender(mut substream),
                } => {
                    match start {
                        Some(start) => {
                            if start.elapsed() >= Duration::from_secs(10) {
                                *self = PerfRun::ClosingWriter {
                                    duration: start.elapsed(),
                                    transfered,
                                    substream,
                                };

                                continue;
                            }
                        }
                        None => start = Some(Instant::now()),
                    }

                    match Pin::new(&mut substream).poll_write(cx, MSG) {
                        Poll::Ready(Ok(n)) => {
                            *self = PerfRun::Running {
                                start,
                                transfered: transfered + n,
                                substream: PerfRunStream::Sender(substream),
                            };
                        }
                        Poll::Ready(Err(e)) => panic!("Unexpected error {:?}", e),
                        Poll::Pending => {
                            *self = PerfRun::Running {
                                start,
                                transfered,
                                substream: PerfRunStream::Sender(substream),
                            };
                            return Poll::Pending;
                        }
                    }
                }
                PerfRun::Running {
                    mut start,
                    transfered,
                    substream: PerfRunStream::Receiver(mut substream, mut void_buf),
                } => match Pin::new(&mut substream).poll_read(cx, &mut void_buf) {
                    Poll::Ready(Ok(n)) => {
                        start = start.or_else(|| Some(Instant::now()));
                        if n == 0 {
                            *self = PerfRun::Done {
                                duration: start.unwrap().elapsed(),
                                transfered,
                            };
                        } else {
                            *self = PerfRun::Running {
                                start,
                                transfered: transfered + n,
                                substream: PerfRunStream::Receiver(substream, void_buf),
                            };
                        }
                    }
                    Poll::Ready(Err(e)) => panic!("Unexpected error {:?}", e),
                    Poll::Pending => {
                        *self = PerfRun::Running {
                            start,
                            transfered,
                            substream: PerfRunStream::Receiver(substream, void_buf),
                        };
                        return Poll::Pending;
                    }
                },
                PerfRun::ClosingWriter {
                    duration,
                    transfered,
                    mut substream,
                } => match Pin::new(&mut substream).poll_flush(cx) {
                    Poll::Ready(Ok(())) => match Pin::new(&mut substream).poll_close(cx) {
                        Poll::Ready(Ok(())) => {
                            drop(substream);
                            std::thread::sleep(Duration::from_secs(1));
                            *self = PerfRun::Done {
                                duration,
                                transfered,
                            };
                        }
                        Poll::Ready(Err(_)) => panic!("Failed to close connection"),
                        Poll::Pending => {
                            *self = PerfRun::ClosingWriter {
                                duration,
                                transfered,
                                substream,
                            };
                            return Poll::Pending;
                        }
                    },
                    Poll::Ready(Err(_)) => panic!("Got error while closing substream"),
                    Poll::Pending => {
                        *self = PerfRun::ClosingWriter {
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

#[derive(Debug)]
pub enum PerfHandlerOut {
    PerfRunDone(Duration, usize),
}

impl ConnectionHandler for PerfHandler {
    type InEvent = PerfHandlerIn;
    /// Custom event that can be produced by the handler and that will be returned to the outside.
    type OutEvent = PerfHandlerOut;
    /// The type of errors returned by [`ConnectionHandler::poll`].
    type Error = io::Error;
    /// The inbound upgrade for the protocol(s) used by the handler.
    type InboundProtocol = PerfProtocolConfig;
    /// The outbound upgrade for the protocol(s) used by the handler.
    type OutboundProtocol = PerfProtocolConfig;
    /// The type of additional information returned from `listen_protocol`.
    type InboundOpenInfo = ();
    /// The type of additional information passed to an `OutboundSubstreamRequest`.
    type OutboundOpenInfo = ();

    /// The [`InboundUpgrade`](libp2p_core::upgrade::InboundUpgrade) to apply on inbound
    /// substreams to negotiate the desired protocols.
    ///
    /// > **Note**: The returned `InboundUpgrade` should always accept all the generally
    /// >           supported protocols, even if in a specific context a particular one is
    /// >           not supported, (eg. when only allowing one substream at a time for a protocol).
    /// >           This allows a remote to put the list of supported protocols in a cache.
    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        SubstreamProtocol::new(PerfProtocolConfig {}, ())
    }

    fn on_behaviour_event(&mut self, event: Self::InEvent) {
        match event {
            PerfHandlerIn::StartPerf => {
                self.outbox
                    .push(ConnectionHandlerEvent::OutboundSubstreamRequest {
                        protocol: SubstreamProtocol::new(PerfProtocolConfig {}, ()),
                    })
            }
        }
    }

    fn on_connection_event(
        &mut self,
        event: ConnectionEvent<
            Self::InboundProtocol,
            Self::OutboundProtocol,
            Self::InboundOpenInfo,
            Self::OutboundOpenInfo,
        >,
    ) {
        match event {
            ConnectionEvent::FullyNegotiatedInbound(event) => {
                let FullyNegotiatedInbound { protocol, .. } = event;
                self.perf_runs.push(PerfRun::new(PerfRunStream::Receiver(
                    protocol,
                    vec![0; BUFFER_SIZE],
                )));
            }
            ConnectionEvent::FullyNegotiatedOutbound(event) => {
                let FullyNegotiatedOutbound { protocol, .. } = event;
                self.perf_runs
                    .push(PerfRun::new(PerfRunStream::Sender(protocol)));
            }
            ConnectionEvent::DialUpgradeError(event) => {
                let DialUpgradeError { error, .. } = event;
                panic!("{:?}", error);
            }
            ConnectionEvent::ListenUpgradeError(event) => {
                let ListenUpgradeError { error, .. } = event;
                panic!("listener upgrade error {:?}", error);
            }
            ConnectionEvent::AddressChange(_) => {}
        }
    }

    /// Returns until when the connection should be kept alive.
    ///
    /// This method is called by the `Swarm` after each invocation of
    /// [`ConnectionHandler::poll`] to determine if the connection and the associated
    /// `ConnectionHandler`s should be kept alive as far as this handler is concerned
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
    /// > when [`ConnectionHandler::poll`] returns an error. Furthermore, the
    /// > connection may be closed for reasons outside of the control
    /// > of the handler.
    fn connection_keep_alive(&self) -> KeepAlive {
        KeepAlive::Yes
    }

    fn poll(
        &mut self,
        cx: &mut Context,
    ) -> Poll<
        ConnectionHandlerEvent<
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
                return Poll::Ready(ConnectionHandlerEvent::Custom(PerfHandlerOut::PerfRunDone(
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
