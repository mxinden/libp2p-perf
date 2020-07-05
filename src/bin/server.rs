use futures::future::poll_fn;
use futures::prelude::*;
use libp2p::{identity, Multiaddr, PeerId, Swarm};
use libp2p::swarm::SwarmBuilder;
use libp2p_perf::{build_transport, Perf, Executor};
use std::task::Poll;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "libp2p-perf server",
    about = "The iPerf equivalent for the libp2p ecosystem."
)]
struct Opt {
    #[structopt(long)]
    listen_address: Multiaddr,
}

#[async_std::main]
async fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    let key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(key.public());

    let transport = build_transport(key).unwrap();
    let perf = Perf::default();
    let mut server = SwarmBuilder::new(transport, perf, local_peer_id).executor(Box::new(Executor{})).build();

    Swarm::listen_on(&mut server, opt.listen_address).unwrap();
    let mut listening = false;

    poll_fn(|cx| loop {
        match server.poll_next_unpin(cx) {
            Poll::Ready(Some(e)) => println!("{}", e),
            Poll::Ready(None) => panic!("Unexpected server termination."),
            Poll::Pending => {
                if !listening {
                    if let Some(a) = Swarm::listeners(&server).next() {
                        println!("Listening on {:?}.", a);
                        listening = true;
                    }
                }

                return Poll::Pending;
            }
        }
    }).await
}
