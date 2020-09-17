use futures::future::poll_fn;
use futures::prelude::*;
use libp2p::swarm::SwarmBuilder;
use libp2p::{identity, Multiaddr, PeerId, Swarm};
use libp2p_perf::{build_transport, Executor, Perf};
use log::{warn};
use std::task::Poll;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "libp2p-perf client",
    about = "The iPerf equivalent for the libp2p ecosystem."
)]
struct Opt {
    #[structopt(long)]
    server_address: Multiaddr,
}

#[async_std::main]
async fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    let key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(key.public());

    let transport = build_transport(key).unwrap();
    let perf = Perf::default();
    let mut client = SwarmBuilder::new(transport, perf, local_peer_id)
        .executor(Box::new(Executor {}))
        .build();

    // Hack as Swarm::dial_addr does not accept Multiaddr with PeerId.
    let mut server_address = opt.server_address;
    if matches!(server_address.iter().last(), Some(libp2p::core::multiaddr::Protocol::P2p(_))) {
        warn!("Ignoring provided PeerId.");
        server_address.pop().unwrap();
    } 

    Swarm::dial_addr(&mut client, server_address).unwrap();

    poll_fn(|cx| match client.poll_next_unpin(cx) {
        Poll::Ready(Some(e)) => {
            println!("{}", e);

            // TODO: Fix hack
            //
            // Performance run timer has already been stopped. Wait for a second
            // to make sure the receiving side of the substream on the server is
            // closed before the whole connection is dropped.
            std::thread::sleep(std::time::Duration::from_secs(1));

            Poll::Ready(())
        }
        Poll::Ready(None) => panic!("Client finished unexpectedly."),
        Poll::Pending => Poll::Pending,
    })
    .await
}
