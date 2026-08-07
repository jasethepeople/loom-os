//! # Loom OS Kernel
//!
//! The central orchestrator. Manages plugin lifecycle, LLM inference,
//! GPU compute dispatch, and IPC messaging across all Loom OS components.
//!
//! ## Architecture
//! ```text
//! Kernel Event Loop (Tokio)
//!    ├─ Control Channel  → Plugin Registry + Wasm Runtime
//!    ├─ LLM Channel      → LlmEngine (candle)
//!    ├─ GPU Channel      → GpuEngine (wgpu)
//!    ├─ Audio Channel    → Audio Pipeline
//!    ├─ Video Channel    → Video Pipeline
//!    ├─ Data Channel     → Plugin Execution
//!    └─ Log Channel      → Unified Logging
//! ```

use shared::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, error, debug};

mod plugin_registry;
mod wasm_runtime;
mod llm_bridge;
mod gpu_scheduler;
mod ipc_bus;
mod config;

use plugin_registry::PluginRegistry;
use wasm_runtime::WasmRuntime;
use llm_bridge::LlmEngine;
use gpu_scheduler::GpuEngine;
use ipc_bus::IpcBus;
use config::LoomConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        )
        .init();

    info!(r#"
    ╔══════════════════════════════════════════════════════════════════╗
    ║                                                                  ║
    ║     ██╗      ██████╗  ██████╗ ███╗   ███╗     ██████╗ ███████╗  ║
    ║     ██║     ██╔═══██╗██╔═══██╗████╗ ████║    ██╔═══██╗██╔════╝  ║
    ║     ██║     ██║   ██║██║   ██║██╔████╔██║    ██║   ██║███████╗  ║
    ║     ██║     ██║   ██║██║   ██║██║╚██╔╝██║    ██║   ██║╚════██║  ║
    ║     ███████╗╚██████╔╝╚██████╔╝██║ ╚═╝ ██║    ╚██████╔╝███████║  ║
    ║     ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝     ╚═╝     ╚═════╝ ╚══════╝  ║
    ║                                                                  ║
    ║           v0.2.0 — High-Performance Modular Runtime              ║
    ║         LLM Bridge · GPU Compute · Wasm Plugin Runtime           ║
    ╚══════════════════════════════════════════════════════════════════╝
    "#);

    // ── Load Configuration ───────────────────────────────
    let config = LoomConfig::load("./config/loom.toml")
        .unwrap_or_else(|e| {
            warn!("Config load failed ({}), using defaults", e);
            LoomConfig::default()
        });
    info!("Configuration loaded: {:?}", config);

    // ── Initialize Subsystems ────────────────────────────
    let registry = Arc::new(RwLock::new(PluginRegistry::new()));
    let wasm_runtime = Arc::new(WasmRuntime::new()?);

    // LLM Engine
    let llm_engine = if Path::new(&config.llm.model_path).exists() {
        info!("Initializing LLM engine with: {}", config.llm.model_path);
        let (engine, mut result_rx) = LlmEngine::new(
            Path::new(&config.llm.model_path),
            Path::new(&config.llm.tokenizer_path),
        ).await?;

        tokio::spawn(async move {
            while let Some((task_id, response)) = result_rx.recv().await {
                info!("[LLM] {} → {} tokens ({})",
                    task_id, response.tokens_used, response.finish_reason);
            }
        });
        Some(Arc::new(engine))
    } else {
        warn!("No model at {} — LLM bridge offline", config.llm.model_path);
        None
    };

    // GPU Engine
    let gpu_engine = match GpuEngine::new().await {
        Ok(engine) => {
            info!("GPU scheduler online: {} ({:?})",
                engine.adapter_name(), engine.backend());
            Some(Arc::new(engine))
        }
        Err(e) => {
            warn!("GPU scheduler offline: {}", e);
            None
        }
    };

    // ── IPC Bus ──────────────────────────────────────────
    let (tx, mut rx) = mpsc::channel::<LoomMessage>(config.ipc.buffer_size);
    let ipc = Arc::new(IpcBus::new(tx.clone()));

    // Start nng/UDS listener if configured
    if config.ipc.transport == "nng" {
        let ipc_clone = Arc::clone(&ipc);
        tokio::spawn(async move {
            if let Err(e) = ipc_clone.start_nng_listener("tcp://127.0.0.1:9001").await {
                error!("nng listener failed: {}", e);
            }
        });
    }

    // ── Health Monitor ───────────────────────────────────
    let reg_clone = Arc::clone(&registry);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            let reg = reg_clone.read().await;
            for (id, health) in reg.health_snapshot().iter() {
                match health.status {
                    HealthStatus::Failed => warn!("[HEALTH] {} FAILED", id),
                    HealthStatus::Degraded => warn!("[HEALTH] {} DEGRADED", id),
                    _ => debug!("[HEALTH] {} OK", id),
                }
            }
        }
    });

    // ── Main Event Loop ──────────────────────────────────
    info!("Kernel online. Listening for events...");
    info!("Press Ctrl+C to shutdown gracefully.");

    while let Some(msg) = rx.recv().await {
        debug!("Message on {:?} from {}", msg.channel, msg.source);

        match msg.channel {
            Channel::Control => {
                handle_control(msg, &registry, &wasm_runtime).await;
            }
            Channel::Llm => {
                if let Some(ref engine) = llm_engine {
                    let engine = Arc::clone(engine);
                    tokio::spawn(async move {
                        if let Payload::Task(task) = msg.payload {
                            engine.route(task).await;
                        }
                    });
                } else {
                    warn!("LLM request dropped — engine offline");
                }
            }
            Channel::Gpu => {
                if let Some(ref engine) = gpu_engine {
                    let engine = Arc::clone(engine);
                    tokio::spawn(async move {
                        if let Payload::Task(task) = msg.payload {
                            match engine.enqueue(task).await {
                                Ok(result) => info!(
                                    "[GPU] {} complete in {:.1}ms",
                                    result.task_id,
                                    result.metrics.get("gpu_time_ms").unwrap_or(&0.0)
                                ),
                                Err(e) => error!("[GPU] Task failed: {}", e),
                            }
                        }
                    });
                } else {
                    warn!("GPU request dropped — scheduler offline");
                }
            }
            Channel::Audio => {
                info!("[Audio] Task received — routing to audio pipeline");
                // Audio pipeline integration point
            }
            Channel::Video => {
                info!("[Video] Task received — routing to video pipeline");
                // Video pipeline integration point
            }
            Channel::Data => {
                handle_data(msg, &registry).await;
            }
            Channel::Log => {
                info!("[{}] {:?}", msg.source, msg.payload);
            }
        }
    }

    info!("Kernel shutting down gracefully.");
    Ok(())
}

async fn handle_control(
    msg: LoomMessage,
    registry: &Arc<RwLock<PluginRegistry>>,
    wasm: &Arc<WasmRuntime>,
) {
    match msg.payload {
        Payload::Command(cmd) => {
            info!("[Control] {} from {} — verb: {}", msg.id, msg.source, cmd.verb);
            match cmd.verb.as_str() {
                "load_plugin" => {
                    if let Some(path) = cmd.args.get("wasm_path") {
                        let mut reg = registry.write().await;
                        match wasm.load_module(path).await {
                            Ok(manifest) => {
                                reg.register(manifest);
                                info!("✓ Plugin loaded: {}", path);
                            }
                            Err(e) => error!("✗ Plugin load failed: {}", e),
                        }
                    }
                }
                "unload_plugin" => {
                    if let Some(id) = cmd.args.get("plugin_id") {
                        let mut reg = registry.write().await;
                        reg.unregister(id);
                        info!("✓ Plugin {} unloaded", id);
                    }
                }
                "list_plugins" => {
                    let reg = registry.read().await;
                    info!("Active plugins ({}):", reg.list().len());
                    for m in reg.list() {
                        info!("  → {} v{} — {:?}", m.name, m.version, m.capabilities);
                    }
                }
                "system_status" => {
                    info!("[Status] Kernel: OK | Registry: active | IPC: online");
                }
                _ => warn!("Unknown verb: {}", cmd.verb),
            }
        }
        _ => {}
    }
}

async fn handle_data(msg: LoomMessage, registry: &Arc<RwLock<PluginRegistry>>) {
    if let Payload::Task(task) = msg.payload {
        let reg = registry.read().await;
        if let Some(_manifest) = reg.get(&task.plugin_id) {
            let ctx = ProjectContext {
                project_name: "loom_default".into(),
                root_dir: "./data".into(),
                platform_targets: vec!["web".into(), "gpu".into()],
                metadata: HashMap::new(),
            };
            info!("[Data] Task {} for plugin {} dispatched", task.task_id, task.plugin_id);
        }
    }
}
