
mod behaviour;
mod handler;
mod protocol;

#[cfg(test)]
mod tests {
    use crate::behaviour::{Perf, PerfEvent};
    use futures::prelude::*;
    use libp2p::{
        core::{
            self,
            Transport,
        },
        identity, secio,
        dns,
        tcp, PeerId, Swarm,
    };
    use libp2p_yamux as yamux;
    use std::task::{Context, Poll};

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

        async_std::task::block_on(future::poll_fn(|cx| -> Poll<()> {
            loop {
                match receiver.poll_next_unpin(cx) {
                    Poll::Ready(e) => panic!("{:?}", e),
                    Poll::Pending => {
                        if let Some(a) = Swarm::listeners(&receiver).next() {
                            println!("{:?}", a);
                            break;
                        }
                    }
                }
            }

            Poll::Ready(())
        }));

        Swarm::dial_addr(&mut sender, "/ip4/127.0.0.1/tcp/9992".parse().unwrap()).unwrap();

        let sender_task = async_std::task::spawn(future::poll_fn(move |cx: &mut Context|  -> Poll<()>{
            loop {
                match sender.poll_next_unpin(cx) {
                    Poll::Ready(Some(PerfEvent::PerfRunDone(duration, transfered))) => {
                        println!("Duration {:?}, transfered {:?} rate {:?}", duration, transfered, (transfered / 1024 / 1024) as f64 / duration.as_secs_f64());
                        return Poll::Ready(());
                    }
                    Poll::Ready(None) => panic!("unexpected stream close"),
                    Poll::Pending => break,
                }
            }

            Poll::Pending
        }));

        async_std::task::spawn(future::poll_fn(move |cx: &mut Context| -> Poll<()> {
            receiver.poll_next_unpin(cx).map(|e| println!("{:?}", e))
        }));

        // Don't block on receiver task. When the sender drops the substream it
        // itself is also dropped right afterwards. Thus the substream closing
        // might not reach the receiver, but instead the receiver will just
        // learn about the connection being closed. In that case the perfrun on
        // the receiver side is never finished.
        async_std::task::block_on(sender_task);
    }



    fn build_transport(
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
        Ok(dns::DnsConfig::new(tcp::TcpConfig::new())?
            .upgrade(core::upgrade::Version::V1)
            .authenticate(secio::SecioConfig::new(keypair))
            .multiplex(yamux::Config::default())
            .map(|(peer, muxer), _| (peer, core::muxing::StreamMuxerBox::new(muxer))))
    }
}
