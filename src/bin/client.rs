use futures::future::poll_fn;
use futures::prelude::*;
use libp2p::{identity, Multiaddr, PeerId, Swarm};
use libp2p::swarm::SwarmBuilder;
use libp2p_perf::{build_transport, Perf, Executor};
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
    let mut client = SwarmBuilder::new(transport, perf, local_peer_id).executor(Box::new(Executor{})).build();

    Swarm::dial_addr(&mut client, opt.server_address).unwrap();

    poll_fn(|cx| match client.poll_next_unpin(cx) {
        Poll::Ready(Some(e)) => {
            println!("{}", e);

            Poll::Ready(())
        }
        Poll::Ready(None) => panic!("Client finished unexpectedly."),
        Poll::Pending => Poll::Pending,
    }).await
}
