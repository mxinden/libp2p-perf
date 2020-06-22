
mod behaviour;
mod handler;
mod protocol;

#[cfg(test)]
mod tests {
    use crate::behaviour::Perf;
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
        let mut sender = {
            let key = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(key.public());

            let transport = build_transport(key).unwrap();
            let perf = Perf{};
            Swarm::new(transport, perf, local_peer_id)
        };

        let mut receiver = {
            let key = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(key.public());

            let transport = build_transport(key).unwrap();
            let perf = Perf{};
            Swarm::new(transport, perf, local_peer_id)
        };

        Swarm::listen_on(&mut sender, "/ip4/0.0.0.0/tcp/9991".parse().unwrap()).unwrap();
        Swarm::listen_on(&mut receiver, "/ip4/0.0.0.0/tcp/9992".parse().unwrap()).unwrap();

        async_std::task::block_on(future::poll_fn(|cx| -> Poll<()> {
            loop {
                println!("polling");
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

        println!("done");

        Swarm::dial_addr(&mut sender, "/ip4/127.0.0.1/tcp/9992".parse().unwrap()).unwrap();

        let sender_task = async_std::task::spawn(future::poll_fn(move |cx: &mut Context|  -> Poll<()>{
            loop {
                match sender.poll_next_unpin(cx) {
                    Poll::Ready(e) => panic!("{:?}", e),
                    Poll::Pending => break,
                }
            }

            Poll::Pending
        }));

        let receiver_task = async_std::task::spawn(future::poll_fn(move |cx: &mut Context| -> Poll<()> {
            loop {
                match receiver.poll_next_unpin(cx) {
                    Poll::Ready(e) => panic!("{:?}", e),
                    Poll::Pending => break,
                }
            }

            Poll::Pending
        }));

        async_std::task::block_on(async move {
            sender_task.await;
            receiver_task.await;
        });
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
