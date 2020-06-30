use crate::handler::{PerfHandler, PerfHandlerIn, PerfHandlerOut};
use libp2p::{
    core::{
        connection::{ConnectionId, ListenerId},
        ConnectedPoint,
    },
    swarm::{
        IntoProtocolsHandler, NetworkBehaviour, NetworkBehaviourAction, NotifyHandler,
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
    outbox: Vec<NetworkBehaviourAction<<<<Self as NetworkBehaviour>::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::InEvent, <Self as NetworkBehaviour>::OutEvent>>
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

    fn inject_connection_closed(&mut self, _: &PeerId, _: &ConnectionId, _: &ConnectedPoint) {}

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

    fn inject_addr_reach_failure(
        &mut self,
        _peer_id: Option<&PeerId>,
        _addr: &Multiaddr,
        error: &dyn error::Error,
    ) {
        panic!("inject addr reach failure: {:?}", error);
    }

    fn inject_dial_failure(&mut self, _peer_id: &PeerId) {
        panic!("inject dial failure");
    }

    fn inject_new_listen_addr(&mut self, _addr: &Multiaddr) {}

    fn inject_expired_listen_addr(&mut self, _addr: &Multiaddr) {}

    fn inject_new_external_addr(&mut self, _addr: &Multiaddr) {}

    fn inject_listener_error(&mut self, _id: ListenerId, err: &(dyn std::error::Error + 'static)) {
        panic!("listener error {:?}", err);
    }

    fn inject_listener_closed(&mut self, _id: ListenerId, reason: Result<(), &std::io::Error>) {
        panic!("listener closed {:?}", reason);
    }

    fn poll(&mut self, _cx: &mut Context, _params: &mut impl PollParameters)
-> Poll<NetworkBehaviourAction<<<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::InEvent, Self::OutEvent>>{
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
                "Interval\t\tTransfer\tBandwidth\n{:?} - {:?}\t{:?} mb\t{:?} mb/s",
                Duration::from_secs(0),
                duration,
                transfered / 1000 / 1000,
                (transfered / 1000 / 1000) as f64 / duration.as_secs_f64()
            ),
        }
    }
}
