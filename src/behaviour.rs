use crate::handler::PerfHandler;
use futures::prelude::*;
use futures_codec::Framed;
use libp2p::{
    core::{
        connection::{ConnectionId, ListenerId},
        ConnectedPoint,
    },
    swarm::{
        IntoProtocolsHandler, NetworkBehaviour, NetworkBehaviourAction, PollParameters,
        ProtocolsHandler,
    },
    Multiaddr, PeerId,
};
use std::error;
use std::task::{Context, Poll};

pub struct Perf {}

impl NetworkBehaviour for Perf {
    type ProtocolsHandler = PerfHandler;

    type OutEvent = ();

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        println!("NetworkBehaviour::new_handler");
        PerfHandler {}
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        vec![]
    }

    fn inject_connected(&mut self, peer_id: &PeerId) {
        panic!("inject connected");
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        panic!("inject disconnected");
    }

    fn inject_connection_established(&mut self, _: &PeerId, _: &ConnectionId, _: &ConnectedPoint) {
        panic!("inject connection_established");
    }

    fn inject_connection_closed(&mut self, _: &PeerId, _: &ConnectionId, _: &ConnectedPoint) {}

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        connection: ConnectionId,
        event: <<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::OutEvent,
    ) {
        panic!("inject_event");
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

    fn poll(&mut self, cx: &mut Context, params: &mut impl PollParameters)
-> Poll<NetworkBehaviourAction<<<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::InEvent, Self::OutEvent>>{
        println!("NetworkBehaviour::poll called");
        return Poll::Pending;
    }
}
