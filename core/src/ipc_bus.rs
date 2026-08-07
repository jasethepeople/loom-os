use shared::LoomMessage;
use tokio::sync::mpsc::Sender;
use tracing::{info, error};

/// High-performance message bus.
/// Current: Tokio mpsc (in-process, zero-copy).
/// Future: nng (nanomsg) for cross-process / network IPC.
pub struct IpcBus {
    tx: Sender<LoomMessage>,
}

impl IpcBus {
    pub fn new(tx: Sender<LoomMessage>) -> Self {
        Self { tx }
    }

    pub async fn send(&self, msg: LoomMessage) -> Result<(), String> {
        self.tx.send(msg).await.map_err(|e| e.to_string())
    }

    /// Start an nng listener for external clients (SvelteKit, CLI, etc.)
    pub async fn start_nng_listener(&self, address: &str) -> anyhow::Result<()> {
        info!("Starting nng listener on {}", address);
        // Production: use nng::Socket::Rep0, bind to address,
        // deserialize incoming messages, forward to tx channel
        info!("nng listener stub — implement with nng crate");
        Ok(())
    }

    /// Send a message via Unix Domain Socket
    pub async fn send_uds(&self, _path: &str, msg: LoomMessage) -> Result<(), String> {
        // Production: serialize to JSON, write to Unix socket
        self.send(msg).await
    }
}
