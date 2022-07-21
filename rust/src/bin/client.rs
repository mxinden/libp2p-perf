use futures::prelude::*;
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::{identity, Multiaddr, PeerId};
use libp2p_perf::{build_transport, Perf, TcpTransportSecurity};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "libp2p-perf client",
    about = "The iPerf equivalent for the libp2p ecosystem."
)]
struct Opt {
    #[structopt(long)]
    server_address: Multiaddr,

    #[structopt(long)]
    tcp_transport_security: Option<TcpTransportSecurity>,
}

fn setup_global_subscriber() {
    let filter_layer = tracing_subscriber::EnvFilter::from_default_env();
    tracing_subscriber::fmt()
        .with_env_filter(filter_layer)
        .try_init()
        .ok();
}

#[async_std::main]
async fn main() {
    setup_global_subscriber();
    let opt = Opt::from_args();

    let key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(key.public());

    println!("Local peer id: {:?}", local_peer_id);

    let transport = build_transport(
        key,
        opt.tcp_transport_security
            .unwrap_or(TcpTransportSecurity::Noise),
    )
    .unwrap();
    let perf = Perf::default();
    let mut client = SwarmBuilder::new(transport, perf, local_peer_id)
        .executor(Box::new(|f| {
            async_std::task::spawn(f);
        }))
        .build();

    client.dial(opt.server_address).unwrap();

    let mut remote_peer_id = None;

    loop {
        match client.next().await.expect("Infinite stream.") {
            SwarmEvent::Behaviour(e) => {
                println!("{}", e);

                if let Some(peer_id) = remote_peer_id.take() {
                    client.disconnect_peer_id(peer_id).unwrap();
                }
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                remote_peer_id = Some(peer_id);
            }
            e @ SwarmEvent::ConnectionClosed { .. } => {
                println!("{:?}", e);
                break;
            }
            e => panic!("{:?}", e),
        }
    }
}
