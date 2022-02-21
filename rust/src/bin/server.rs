use futures::prelude::*;
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::{identity, Multiaddr, PeerId};
use libp2p_perf::{build_transport, Perf, TcpTransportSecurity};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "libp2p-perf server",
    about = "The iPerf equivalent for the libp2p ecosystem."
)]
struct Opt {
    #[structopt(long)]
    listen_address: Vec<Multiaddr>,

    #[structopt(long)]
    private_key_pkcs8: Option<PathBuf>,
}

fn setup_global_subscriber() -> impl Drop {
    use tracing_flame::FlameLayer;
    use tracing_subscriber::{prelude::*, fmt};

    let filter_layer = tracing_subscriber::EnvFilter::from_default_env();

    let fmt_format = tracing_subscriber::fmt::format()
        .pretty()
        .with_thread_ids(false)
        .without_time();
    let fmt_layer = fmt::Layer::default().event_format(fmt_format);

    let (flame_layer, _guard) = FlameLayer::with_file("./tracing.server.folded").unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(flame_layer)
        .try_init()
        .ok();
    _guard
}

#[async_std::main]
async fn main() {
    // env_logger::init();
    let _guard = setup_global_subscriber();
    log_panics::init();
    let opt = Opt::from_args();

    let key = if let Some(path) = opt.private_key_pkcs8 {
        let mut bytes = std::fs::read(path).unwrap();
        identity::Keypair::rsa_from_pkcs8(&mut bytes).unwrap()
    } else {
        identity::Keypair::generate_ed25519()
    };
    let local_peer_id = PeerId::from(key.public());

    println!("Local peer id: {:?}", local_peer_id);

    let transport = build_transport(
        key,
        TcpTransportSecurity::All,
        Some("/ip4/0.0.0.0/udp/9992/quic".parse().unwrap()),
    )
    .unwrap();
    let perf = Perf::default();
    let mut server = SwarmBuilder::new(transport, perf, local_peer_id.clone())
        .executor(Box::new(|f| {
            async_std::task::spawn(f);
        }))
        .build();

    assert!(
        !opt.listen_address.is_empty(),
        "Provide at least one listen address."
    );
    for addr in opt.listen_address {
        println!("about to listen on {:?}", addr);
        server.listen_on(addr).unwrap();
    }

    loop {
        match server.next().await.unwrap() {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}.", address);
            }
            e => {
                println!("{:?}", e);
            }
        }
    }
}
