use crate::handler::{PerfHandler, PerfHandlerIn, PerfHandlerOut};
use libp2p::{
    swarm::{
        behaviour::{ConnectionEstablished, ListenerClosed, ListenerError},
        ConnectionId, FromSwarm, NetworkBehaviour, NetworkBehaviourAction, NotifyHandler,
        PollParameters,
    },
    Multiaddr, PeerId,
};
use std::fmt;
use std::task::{Context, Poll};
use std::time::Duration;

#[derive(Default)]
pub struct Perf {
    outbox: Vec<NetworkBehaviourAction<PerfEvent, PerfHandlerIn>>,
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

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        match event {
            FromSwarm::ConnectionEstablished(event) => {
                let ConnectionEstablished {
                    endpoint, peer_id, ..
                } = event;
                if endpoint.is_dialer() {
                    self.outbox.push(NetworkBehaviourAction::NotifyHandler {
                        peer_id,
                        event: PerfHandlerIn::StartPerf,
                        handler: NotifyHandler::Any,
                    })
                };
            }
            FromSwarm::DialFailure(_) => {
                panic!("inject dial failure");
            }
            FromSwarm::ListenerError(event) => {
                let ListenerError { err, .. } = event;
                panic!("listener error {:?}", err);
            }
            FromSwarm::ListenerClosed(event) => {
                let ListenerClosed { reason, .. } = event;
                panic!("listener closed {:?}", reason);
            }
            _ => {}
        }
    }

    fn on_connection_handler_event(
        &mut self,
        _peer_id: PeerId,
        _connection_id: ConnectionId,
        event: PerfHandlerOut,
    ) {
        match event {
            //PerfHandlerOut::PerfRunDone(duration, transfered) => {},
            PerfHandlerOut::PerfRunDone(duration, transfered) => self.outbox.push(
                NetworkBehaviourAction::GenerateEvent(PerfEvent::PerfRunDone(duration, transfered)),
            ),
        }
    }

    fn poll(
        &mut self,
        _cx: &mut Context,
        _params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<PerfEvent, PerfHandlerIn>> {
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
