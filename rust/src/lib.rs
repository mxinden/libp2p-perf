mod behaviour;
mod handler;
mod protocol;

pub use behaviour::{Perf, PerfEvent};
use futures::executor::block_on;

use libp2p::{
    core::{
        self,
        either::EitherOutput,
        muxing::StreamMuxerBox,
        transport::{choice::OrTransport, Transport},
        upgrade::{InboundUpgradeExt, OptionalUpgrade, OutboundUpgradeExt, SelectUpgrade},
    },
    dns, identity, noise,
    plaintext::PlainText2Config,
    quic::{QuicConfig, QuicTransport, TlsCrypto},
    tcp, yamux, PeerId,
};

#[derive(Debug)]
pub enum TcpTransportSecurity {
    Noise,
    Plaintext,
    All,
}

impl std::str::FromStr for TcpTransportSecurity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "noise" => Ok(Self::Noise),
            "plaintext" => Ok(Self::Plaintext),
            "all" => Ok(Self::All),
            _ => Err("Expected one of 'noise', 'plaintext' or 'all'.".to_string()),
        }
    }
}

impl std::fmt::Display for TcpTransportSecurity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn build_transport(
    keypair: identity::Keypair,
    tcp_transport_security: TcpTransportSecurity,
) -> std::io::Result<core::transport::Boxed<(PeerId, StreamMuxerBox)>> {
    let tcp_transport = {
        let mut yamux_config = yamux::YamuxConfig::default();
        yamux_config.set_window_update_mode(yamux::WindowUpdateMode::on_read());

        // The default TCP receive window (minimum, default, maximum) on my OS
        // (Debian) is:
        //
        // $ cat /proc/sys/net/ipv4/tcp_rmem
        // 4096    131072  6291456
        //
        // Possible Bandwidth of a connection ignoring all overheads of TCP would be
        // calculated with:
        //
        // Bandwidth (mBit/s) = (Receive window in bit) / (latency in s) / 1_000_000
        //
        // Ping latency via `localhost` is around 0.09 ms:
        //
        // $ ping localhost
        // 64 bytes from localhost (::1): icmp_seq=2 ttl=64 time=0.095 ms
        // 64 bytes from localhost (::1): icmp_seq=3 ttl=64 time=0.087 ms
        //
        // Thus the bandwidth with the maximum receive window would be:
        //
        // ((6291456*8) / (0,09/1000)) / 1000000 = 559_240 mBit/s
        //
        // An iperf run on localhost achieves around 60 gBit/sec:
        //
        // $ iperf -c 127.0.0.1
        // [  3]  0.0-10.0 sec  68.4 GBytes  58.8 Gbits/sec
        //
        // A libp2p-perf run with the default yamux receive window settings (256
        // kByte) achieves a bandwidth of 30 mBit/s:
        //
        // $ cargo run --bin client --release -- --server-address /ip4/127.0.0.1/tcp/9992
        // Interval        Transfer        Bandwidth
        // 0 s - 10.08 s   35 MBytes       27.78 MBit/s
        //
        // With the yamux receive window set to the OS max receive window (6291456
        // bytes) libp2p-perf runs as fast as 500 mBit/s:
        //
        // $ cargo run --bin client --release -- --server-address /ip4/127.0.0.1/tcp/9992
        // Interval        Transfer        Bandwidth
        // 0 s - 10.00 s   614 MBytes      491.19 MBit/s
        //
        // Set to golang default of 16MiB
        // (https://github.com/libp2p/go-libp2p-yamux/blob/35d571287404f972dc626e2de2980ef2c8178b26/transport.go#L15).
        yamux_config.set_receive_window_size(16 * 1024 * 1024);
        yamux_config.set_max_buffer_size(16 * 1024 * 1024);

        let tcp_transport_security_config = match tcp_transport_security {
            TcpTransportSecurity::Plaintext => {
                let plaintext = PlainText2Config {
                    local_public_key: keypair.public(),
                };

                SelectUpgrade::new(OptionalUpgrade::<noise::NoiseAuthenticated<noise::XX,noise::X25519Spec,()>>::none(), OptionalUpgrade::some(plaintext))
            }
            TcpTransportSecurity::Noise => {
                let noise = noise::NoiseConfig::xx(
                    noise::Keypair::<noise::X25519Spec>::new()
                        .into_authentic(&keypair)
                        .unwrap(),
                )
                .into_authenticated();

                SelectUpgrade::new(
                    OptionalUpgrade::some(noise),
                    OptionalUpgrade::<PlainText2Config>::none(),
                )
            }
            TcpTransportSecurity::All => {
                let noise = noise::NoiseConfig::xx(
                    noise::Keypair::<noise::X25519Spec>::new()
                        .into_authentic(&keypair)
                        .unwrap(),
                )
                .into_authenticated();

                let plaintext = PlainText2Config {
                    local_public_key: keypair.public(),
                };

                SelectUpgrade::new(
                    OptionalUpgrade::some(noise),
                    OptionalUpgrade::some(plaintext),
                )
            }
        };

        let transport = block_on(dns::DnsConfig::system(tcp::TcpConfig::new().nodelay(true)))?;

        transport
            .upgrade(core::upgrade::Version::V1Lazy)
            .authenticate(
                tcp_transport_security_config
                    .map_inbound(move |result| match result {
                        EitherOutput::First((peer_id, o)) => (peer_id, EitherOutput::First(o)),
                        EitherOutput::Second((peer_id, o)) => (peer_id, EitherOutput::Second(o)),
                    })
                    .map_outbound(move |result| match result {
                        EitherOutput::First((peer_id, o)) => (peer_id, EitherOutput::First(o)),
                        EitherOutput::Second((peer_id, o)) => (peer_id, EitherOutput::Second(o)),
                    }),
            )
            .multiplex(yamux_config)
            .map(|(peer, muxer), _| (peer, StreamMuxerBox::new(muxer)))
    };

    let quic_transport = {
        block_on(QuicTransport::new(
            QuicConfig::<TlsCrypto>::new(keypair),
            "/ip4/0.0.0.0/udp/0/quic".parse().unwrap(),
        ))
        .unwrap()
    };

    Ok(OrTransport::new(quic_transport, tcp_transport)
        .map(|either_output, _| match either_output {
            EitherOutput::First((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
            EitherOutput::Second((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
        })
        .boxed())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::LocalPool;

    use futures::prelude::*;
    use futures::stream::StreamExt;
    use futures::task::Spawn;
    use libp2p::core::multiaddr::Multiaddr;
    use libp2p::swarm::{Swarm, SwarmEvent};

    use std::time::Duration;

    #[test]
    fn it_works() {
        let mut pool = LocalPool::new();
        let _ = env_logger::try_init();

        let mut sender = {
            let key = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(key.public());

            let transport = build_transport(key, TcpTransportSecurity::Plaintext).unwrap();
            let perf = Perf::default();
            Swarm::new(transport, perf, local_peer_id)
        };

        let mut receiver = {
            let key = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(key.public());

            let transport = build_transport(key, TcpTransportSecurity::Plaintext).unwrap();
            let perf = Perf::default();
            Swarm::new(transport, perf, local_peer_id)
        };
        let receiver_address: Multiaddr = "/ip6/::1/tcp/0".parse().unwrap();

        // Wait for receiver to bind to listen address.
        let receiver_listen_addr = pool.run_until(async {
            let id = receiver.listen_on(receiver_address.clone()).unwrap();
            match receiver.next().await.unwrap() {
                SwarmEvent::NewListenAddr {
                    listener_id,
                    address,
                    ..
                } if listener_id == id => address,
                _ => panic!("Unexpected event."),
            }
        });

        pool.spawner()
            .spawn_obj(
                async move {
                    loop {
                        receiver.next().await;
                    }
                }
                .boxed()
                .into(),
            )
            .unwrap();

        sender.dial_addr(receiver_listen_addr).unwrap();

        pool.run_until(async move {
            loop {
                match sender.next().await.unwrap() {
                    SwarmEvent::Behaviour(PerfEvent::PerfRunDone(duration, _transfered)) => {
                        if duration < Duration::from_secs(10) {
                            panic!("Expected test to run at least 10 seconds.")
                        }

                        if duration > Duration::from_secs(11) {
                            panic!("Expected test to run roughly 10 seconds.")
                        }

                        break;
                    }
                    _ => {}
                }
            }
        });
    }
}
