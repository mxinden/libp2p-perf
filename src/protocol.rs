use futures::prelude::*;
use futures_codec::Framed;
use libp2p::core::upgrade::{InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use std::io;
use std::{borrow::Cow, iter};
use unsigned_varint::codec::UviBytes;

pub struct PerfProtocolConfig {}

impl UpgradeInfo for PerfProtocolConfig {
    type Info = Cow<'static, [u8]>;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        // TODO: Rename to `perf`.
        iter::once(Cow::Borrowed(b"/backpressure"))
    }
}

impl<C> InboundUpgrade<C> for PerfProtocolConfig
where
    C: AsyncRead + AsyncWrite + Unpin,
{
    type Output = Framed<C, UviBytes<io::Cursor<Vec<u8>>>>;
    type Future = future::Ready<Result<Self::Output, io::Error>>;
    type Error = io::Error;

    fn upgrade_inbound(self, incoming: C, _: Self::Info) -> Self::Future {
        let codec = UviBytes::default();
        // codec.set_max_len(self.max_packet_size);

        future::ok(Framed::new(incoming, codec))
    }
}

impl<C> OutboundUpgrade<C> for PerfProtocolConfig
where
    C: AsyncRead + AsyncWrite + Unpin,
{
    type Output = Framed<C, UviBytes<io::Cursor<Vec<u8>>>>;
    type Future = future::Ready<Result<Self::Output, io::Error>>;
    type Error = io::Error;

    fn upgrade_outbound(self, incoming: C, _: Self::Info) -> Self::Future {
        let codec = UviBytes::default();
        // codec.set_max_len(self.max_packet_size);

        future::ok(Framed::new(incoming, codec))
    }
}
