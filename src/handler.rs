use futures::prelude::*;
use libp2p::{
    core::upgrade::{InboundUpgrade, OutboundUpgrade},
    swarm::{
        KeepAlive, NegotiatedSubstream, ProtocolsHandler, ProtocolsHandlerEvent,
        ProtocolsHandlerUpgrErr, SubstreamProtocol,
    },
};
use std::io;
use std::task::{Context, Poll};

use crate::protocol::PerfProtocolConfig;

pub struct PerfHandler {}

impl ProtocolsHandler for PerfHandler {
    type InEvent = ();
    /// Custom event that can be produced by the handler and that will be returned to the outside.
    type OutEvent = ();
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
        println!("listen_protocol");
        SubstreamProtocol::new(PerfProtocolConfig {})
    }

    /// Injects the output of a successful upgrade on a new inbound substream.
    fn inject_fully_negotiated_inbound(
        &mut self,
        protocol: <Self::InboundProtocol as InboundUpgrade<NegotiatedSubstream>>::Output,
    ) {
        panic!("yeah, got an inbound substream");
    }

    /// Injects the output of a successful upgrade on a new outbound substream.
    ///
    /// The second argument is the information that was previously passed to
    /// [`ProtocolsHandlerEvent::OutboundSubstreamRequest`].
    fn inject_fully_negotiated_outbound(
        &mut self,
        protocol: <Self::OutboundProtocol as OutboundUpgrade<NegotiatedSubstream>>::Output,
        info: Self::OutboundOpenInfo,
    ) {
        panic!("yeah, got an outbound substream");
    }

    /// Injects an event coming from the outside in the handler.
    fn inject_event(&mut self, event: Self::InEvent) {
        panic!("inject_event");
    }

    /// Indicates to the handler that upgrading a substream to the given protocol has failed.
    fn inject_dial_upgrade_error(
        &mut self,
        info: Self::OutboundOpenInfo,
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

    /// Should behave like `Stream::poll()`.
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
        println!("ProtocolsHandler::poll");
        return Poll::Pending;
    }
}
