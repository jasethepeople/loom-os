# Plugin Development Guide

## Creating a New Plugin

```bash
cargo new plugins/my-plugin --lib
echo 'sdk = { path = "../../sdk" }' >> plugins/my-plugin/Cargo.toml
```

## Implementing the LoomPlugin Trait

```rust
use shared::*;
use sdk::plugin_manifest;

pub struct MyPlugin;

impl LoomPlugin for MyPlugin {
    fn manifest(&self) -> PluginManifest {
        plugin_manifest!(
            "my-plugin-001",
            "My Plugin",
            "0.1.0",
            "Your Name",
            ["capability-1", "capability-2"],
            "Description of what this plugin does."
        )
    }

    fn execute(&self, ctx: ProjectContext, task: Task) -> Result<TaskResult, String> {
        // Your logic here
        Ok(TaskResult {
            task_id: task.task_id,
            success: true,
            output: b"done".to_vec(),
            metrics: std::collections::HashMap::new(),
        })
    }

    fn health(&self) -> PluginHealth {
        PluginHealth {
            status: HealthStatus::Healthy,
            last_heartbeat: chrono::Utc::now(),
            memory_mb: 0.0,
            uptime_secs: 0,
        }
    }
}
```

## Compiling to WebAssembly

```bash
cargo build --target wasm32-wasi --release
cp target/wasm32-wasi/release/my-plugin.wasm plugins/bin/
```

## Loading at Runtime

```bash
loom-cli plugin load ./plugins/bin/my-plugin.wasm
```

## Available Capabilities

| Capability | Description |
|-----------|-------------|
| `content-analysis` | Text/media semantic analysis |
| `gpu-compute` | Requires GPU shader dispatch |
| `llm-routing` | Sends prompts to central LLM bridge |
| `audio-synthesis` | Generates procedural audio |
| `video-transcode` | Video processing pipeline |
| `image-gen` | Image generation and manipulation |
| `neuro-marketing` | CLIP-based asset scoring |
| `code-generation` | Runtime code synthesis |
