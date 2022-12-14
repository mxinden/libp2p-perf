use crate::handler::{PerfHandler, PerfHandlerIn, PerfHandlerOut};
use libp2p::{
    core::{connection::ConnectionId, transport::ListenerId, ConnectedPoint},
    swarm::{
        ConnectionHandler, DialError, IntoConnectionHandler, NetworkBehaviour,
        NetworkBehaviourAction, NotifyHandler, PollParameters,
    },
    Multiaddr, PeerId,
};
use std::fmt;
use std::task::{Context, Poll};
use std::time::Duration;

#[derive(Default)]
pub struct Perf {
    outbox: Vec<
        NetworkBehaviourAction<
            <Self as NetworkBehaviour>::OutEvent,
            <Self as NetworkBehaviour>::ConnectionHandler,
        >,
    >,
}

impl NetworkBehaviour for Perf {
    type ConnectionHandler = PerfHandler;

    type OutEvent = PerfEvent;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        PerfHandler::default()
    }

    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        vec![]
    }

    fn inject_connection_established(
        &mut self,
        peer_id: &PeerId,
        _: &ConnectionId,
        connected_point: &ConnectedPoint,
        _failed_addresses: Option<&Vec<Multiaddr>>,
        _other_established: usize,
    ) {
        if connected_point.is_dialer() {
            self.outbox.push(NetworkBehaviourAction::NotifyHandler {
                peer_id: *peer_id,
                event: PerfHandlerIn::StartPerf,
                handler: NotifyHandler::Any,
            })
        };
    }

    fn inject_event(
        &mut self,
        _peer_id: PeerId,
        _connection: ConnectionId,
        event: <<Self::ConnectionHandler as IntoConnectionHandler>::Handler as ConnectionHandler>::OutEvent,
    ) {
        match event {
            PerfHandlerOut::PerfRunDone(duration, transfered) => self.outbox.push(
                NetworkBehaviourAction::GenerateEvent(PerfEvent::PerfRunDone(duration, transfered)),
            ),
        }
    }

    fn inject_dial_failure(
        &mut self,
        _peer_id: Option<PeerId>,
        _handler: PerfHandler,
        _error: &DialError,
    ) {
        panic!("inject dial failure");
    }

    fn inject_new_listen_addr(&mut self, _: ListenerId, _addr: &Multiaddr) {}

    fn inject_expired_listen_addr(&mut self, _: ListenerId, _addr: &Multiaddr) {}

    fn inject_new_external_addr(&mut self, _addr: &Multiaddr) {}

    fn inject_listener_error(&mut self, _id: ListenerId, err: &(dyn std::error::Error + 'static)) {
        panic!("listener error {:?}", err);
    }

    fn inject_listener_closed(&mut self, _id: ListenerId, reason: Result<(), &std::io::Error>) {
        panic!("listener closed {:?}", reason);
    }

    fn poll(
        &mut self,
        _cx: &mut Context,
        _params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        if let Some(action) = self.outbox.pop() {
            return Poll::Ready(action);
        }

        Poll::Pending
    }
}

#[derive(Debug, Clone)]
pub enum PerfEvent {
    PerfRunDone(Duration, usize),
}

impl fmt::Display for PerfEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PerfEvent::PerfRunDone(duration, transfered) => write!(
                f,
                "Interval\tTransfer\tBandwidth\n\
                 0 s - {:.2} s\t{:?} MBytes\t{:.2} MBit/s",
                duration.as_secs_f64(),
                transfered / 1000 / 1000,
                (transfered / 1000 / 1000 * 8) as f64 / duration.as_secs_f64()
            ),
        }
    }
}
