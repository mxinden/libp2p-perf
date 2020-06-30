use futures::future::poll_fn;
use futures::prelude::*;
use libp2p::{identity, Multiaddr, PeerId, Swarm};
use libp2p_perf::{build_transport, Perf};
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

fn main() {
    let opt = Opt::from_args();

    let key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(key.public());

    let transport = build_transport(key).unwrap();
    let perf = Perf::default();
    let mut server = Swarm::new(transport, perf, local_peer_id);

    Swarm::listen_on(&mut server, opt.listen_address).unwrap();
    let listening = false;

    futures::executor::block_on(poll_fn(|cx| match server.poll_next_unpin(cx) {
        Poll::Ready(e) => panic!(
            "Not expecting server swarm to return any event but got {:?}.",
            e
        ),
        Poll::Pending => {
            if !listening {
                if let Some(a) = Swarm::listeners(&server).next() {
                    println!("Listening on {:?}.", a);
                }
            }

            return Poll::Pending;
        }
    }))
}
