use crate::handler::{PerfHandler, PerfHandlerIn, PerfHandlerOut};
use libp2p::{
    core::{
        connection::{ConnectionId, ListenerId},
        ConnectedPoint,
    },
    swarm::{
        DialError, IntoProtocolsHandler, NetworkBehaviour, NetworkBehaviourAction, NotifyHandler,
        PollParameters, ProtocolsHandler,
    },
    Multiaddr, PeerId,
};
use std::error;
use std::fmt;
use std::task::{Context, Poll};
use std::time::Duration;

#[derive(Default)]
pub struct Perf {
    connected_peers: Vec<(PeerId, Direction)>,
    outbox: Vec<
        NetworkBehaviourAction<
            <Self as NetworkBehaviour>::OutEvent,
            <Self as NetworkBehaviour>::ProtocolsHandler,
        >,
    >,
}

enum Direction {
    Incoming,
    Outgoing,
}

impl NetworkBehaviour for Perf {
    type ProtocolsHandler = PerfHandler;

    type OutEvent = PerfEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        PerfHandler::default()
    }

    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        vec![]
    }

    fn inject_connected(&mut self, peer_id: &PeerId) {
        for (peer, direction) in &self.connected_peers {
            if peer == peer_id && matches!(direction, Direction::Outgoing) {
                self.outbox.push(NetworkBehaviourAction::NotifyHandler {
                    peer_id: peer_id.clone(),
                    event: PerfHandlerIn::StartPerf,
                    handler: NotifyHandler::Any,
                })
            }
        }
    }

    fn inject_disconnected(&mut self, _peer_id: &PeerId) {}

    fn inject_connection_established(
        &mut self,
        peer_id: &PeerId,
        _: &ConnectionId,
        connected_point: &ConnectedPoint,
    ) {
        let direction = match connected_point {
            ConnectedPoint::Dialer { .. } => Direction::Outgoing,
            ConnectedPoint::Listener { .. } => Direction::Incoming,
        };

        self.connected_peers.push((peer_id.clone(), direction));
    }

    fn inject_connection_closed(
        &mut self,
        _: &PeerId,
        _: &ConnectionId,
        _: &ConnectedPoint,
        _handler: PerfHandler,
    ) {
    }

    fn inject_event(
        &mut self,
        _peer_id: PeerId,
        _connection: ConnectionId,
        event: <<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::OutEvent,
    ) {
        match event {
            PerfHandlerOut::PerfRunDone(duration, transfered) => self.outbox.push(
                NetworkBehaviourAction::GenerateEvent(PerfEvent::PerfRunDone(duration, transfered)),
            ),
        }
    }

    fn inject_dial_failure(&mut self, _peer_id: &PeerId, _handler: PerfHandler, error: DialError) {
        panic!("inject dial failure: {:?}", error);
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
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ProtocolsHandler>> {
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
