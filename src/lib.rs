mod behaviour;
mod handler;
mod protocol;

pub use behaviour::{Perf, PerfEvent};

use libp2p::{
    core::{self, Transport},
    dns, identity, noise, tcp, PeerId,
};
use libp2p_yamux as yamux;

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

    Ok(dns::DnsConfig::new(tcp::TcpConfig::new())?
        .upgrade(core::upgrade::Version::V1)
        .authenticate(
            noise::NoiseConfig::ix(
                noise::Keypair::<noise::X25519>::new()
                    .into_authentic(&keypair)
                    .unwrap(),
            )
            .into_authenticated(),
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
