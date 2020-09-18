use futures::prelude::*;
use libp2p::core::upgrade::{InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use std::io;
use std::{borrow::Cow, iter};

const PROTOCOL_NAME: &[u8] = b"/perf/0.1.0";

pub struct PerfProtocolConfig {}

impl UpgradeInfo for PerfProtocolConfig {
    type Info = Cow<'static, [u8]>;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(Cow::Borrowed(PROTOCOL_NAME))
    }
}

impl<C> InboundUpgrade<C> for PerfProtocolConfig
where
    C: AsyncRead + AsyncWrite + Unpin,
{
    type Output = C;
    type Future = future::Ready<Result<Self::Output, io::Error>>;
    type Error = io::Error;

    fn upgrade_inbound(self, incoming: C, _: Self::Info) -> Self::Future {
        future::ok(incoming)
    }
}

impl<C> OutboundUpgrade<C> for PerfProtocolConfig
where
    C: AsyncRead + AsyncWrite + Unpin,
{
    type Output = C;
    type Future = future::Ready<Result<Self::Output, io::Error>>;
    type Error = io::Error;

    fn upgrade_outbound(self, incoming: C, _: Self::Info) -> Self::Future {
        future::ok(incoming)
    }
}
