mod behaviour;
mod handler;
mod protocol;

pub use behaviour::{Perf, PerfEvent};

use futures::future::Future;
use libp2p::core::Executor as TExecutor;
use libp2p::{
    core::{self, Transport},
    dns, identity, noise, tcp, yamux, PeerId,
};
use std::pin::Pin;

pub struct Executor {}

impl TExecutor for Executor {
    fn exec(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        async_std::task::spawn(future);
    }
}

pub fn build_transport(
    keypair: identity::Keypair,
) -> std::io::Result<
    impl Transport<
            Output = (
                PeerId,
                impl core::muxing::StreamMuxer<
                        OutboundSubstream = impl Send,
                        Substream = impl Send,
                        Error = impl Into<std::io::Error>,
                    > + Send
                    + Sync,
            ),
            Error = impl std::error::Error + Send,
            Listener = impl Send,
            Dial = impl Send,
            ListenerUpgrade = impl Send,
        > + Clone,
> {
    let mut yamux_config = yamux::Config::default();
    yamux_config.set_window_update_mode(yamux::WindowUpdateMode::OnRead);

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
    yamux_config.set_receive_window(6_291_456);
    yamux_config.set_max_buffer_size(6_291_456);

    Ok(dns::DnsConfig::new(tcp::TcpConfig::new())?
        .upgrade(core::upgrade::Version::V1)
        .authenticate(
            // noise::NoiseConfig::xx(
            //     noise::Keypair::<noise::X25519Spec>::new()
            //         .into_authentic(&keypair)
            //         .unwrap(),
            // )
            // .into_authenticated(),
            libp2p::plaintext::PlainText2Config {
                local_public_key: keypair.public(),
            }
        )
        .multiplex(yamux_config)
        .map(|(peer, muxer), _| (peer, core::muxing::StreamMuxerBox::new(muxer))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::poll_fn;
    use futures::prelude::*;
    use libp2p::Swarm;
    use std::task::Poll;
    use std::time::Duration;

    #[test]
    fn it_works() {
        let _ = env_logger::try_init();
        let mut sender = {
            let key = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(key.public());

            let transport = build_transport(key).unwrap();
            let perf = Perf::default();
            Swarm::new(transport, perf, local_peer_id)
        };

        let mut receiver = {
            let key = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(key.public());

            let transport = build_transport(key).unwrap();
            let perf = Perf::default();
            Swarm::new(transport, perf, local_peer_id)
        };

        Swarm::listen_on(&mut sender, "/ip4/0.0.0.0/tcp/9991".parse().unwrap()).unwrap();
        Swarm::listen_on(&mut receiver, "/ip4/0.0.0.0/tcp/9992".parse().unwrap()).unwrap();

        // Wait for receiver to bind to port.
        async_std::task::block_on(poll_fn(|cx| -> Poll<()> {
            match receiver.poll_next_unpin(cx) {
                Poll::Ready(e) => panic!("{:?}", e),
                Poll::Pending => {
                    if let Some(a) = Swarm::listeners(&receiver).next() {
                        println!("{:?}", a);
                        return Poll::Ready(());
                    }

                    Poll::Pending
                }
            }
        }));

        Swarm::dial_addr(&mut sender, "/ip4/127.0.0.1/tcp/9992".parse().unwrap()).unwrap();

        let sender_task = async_std::task::spawn(poll_fn(move |cx| -> Poll<()> {
            match sender.poll_next_unpin(cx) {
                Poll::Ready(Some(PerfEvent::PerfRunDone(duration, _transfered))) => {
                    if duration < Duration::from_secs(10) {
                        panic!("Expected test to run at least 10 seconds.")
                    }

                    if duration > Duration::from_secs(11) {
                        panic!("Expected test to run roughly 10 seconds.")
                    }

                    Poll::Ready(())
                }
                Poll::Ready(None) => panic!("unexpected stream close"),
                Poll::Pending => Poll::Pending,
            }
        }));

        async_std::task::spawn(poll_fn(move |cx| -> Poll<()> {
            receiver.poll_next_unpin(cx).map(|e| println!("{:?}", e))
        }));

        // Don't block on receiver task. When the sender drops the substream it
        // itself is also dropped right afterwards. Thus the substream closing
        // might not reach the receiver, but instead the receiver will just
        // learn about the connection being closed. In that case the perfrun on
        // the receiver side is never finished.
        async_std::task::block_on(sender_task);
    }
}
