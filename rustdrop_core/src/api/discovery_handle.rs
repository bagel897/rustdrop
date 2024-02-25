use std::ops::ControlFlow;

use flume::Receiver;
use tokio::runtime::Handle;
use tracing::{error, info, info_span, instrument, span};

use crate::{
    mediums::{Discover, Discovery},
    Context, Device, Outgoing, RustdropResult, SenderEvent,
};

#[derive(Debug)]
pub struct DiscoveryHandle {
    device: Device,
    context: Context,
    discoveries: Receiver<Discover>,
}
impl DiscoveryHandle {
    pub fn new(device: Device, context: Context, discoveries: Receiver<Discover>) -> Self {
        Self {
            device,
            context,
            discoveries,
        }
    }
    pub fn send_file(
        &self,
        outgoing: Outgoing,
        handle: &Handle,
    ) -> RustdropResult<Receiver<SenderEvent>> {
        info!("Running client");
        let (tx, rx) = flume::unbounded();
        let cloned = self.context.clone();
        let discoveries = self.discoveries.clone();
        self.context.spawn_on(
            async move {
                while let Ok(discovery) = discoveries.recv_async().await {
                    if let ControlFlow::Break(_) = send_to(discovery, &cloned, &outgoing, &tx).await
                    {
                        break;
                    }
                }
            },
            handle,
        );
        info!("Done sending");
        Ok(rx)
    }
    pub fn device(&self) -> &Device {
        &self.device
    }
}
#[instrument(fields(discovery=?discovery), skip_all)]
async fn send_to(
    discovery: Discover,
    cloned: &Context,
    outgoing: &Outgoing,
    tx: &flume::Sender<SenderEvent>,
) -> ControlFlow<()> {
    let context = cloned.clone();
    let res = match discovery {
        Discover::Wlan(discovery) => {
            discovery
                .send_to(context, outgoing.clone(), tx.clone())
                .await
        }
        Discover::Bluetooth(discovery) => {
            discovery
                .send_to(context, outgoing.clone(), tx.clone())
                .await
        }
    };

    if let Err(e) = res {
        error!("{}", e);
    } else {
        return ControlFlow::Break(());
    }
    ControlFlow::Continue(())
}
