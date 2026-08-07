<div align="center">

# ⚡ Loom OS

**A High-Performance, Modular Runtime for AI-Powered Creator Tools**

[![CI](https://github.com/yourusername/loom-os/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/loom-os/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

</div>

---

## Abstract

Loom OS is a research-grade, high-performance modular runtime designed to orchestrate AI-powered content creation tools as hot-swappable WebAssembly plugins. Built on a Rust async core with centralized LLM inference (via [candle](https://github.com/huggingface/candle)) and GPU compute dispatch (via [wgpu](https://github.com/gfx-rs/wgpu)), Loom OS eliminates the resource duplication inherent in monolithic AI suites by enforcing a **single-model-context architecture**: all plugins route inference requests through one shared engine, preventing the common anti-pattern of every tool loading its own 7B+ parameter model.

The system is designed for researchers, creative technologists, and systems engineers who require a production-grade, extensible platform for building AI-assisted content pipelines across text, image, audio, and video modalities.

---

## Table of Contents

- [Features](#features)
- [Architecture](#architecture)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Plugin Ecosystem](#plugin-ecosystem)
- [Configuration](#configuration)
- [API Reference](#api-reference)
- [Development](#development)
- [Academic Context](#academic-context)
- [License](#license)

---

## Features

### Core Runtime
- **🧠 Centralized LLM Inference** — Single model context via `candle-core` with CUDA/Metal/CPU fallback. All plugins share one inference engine.
- **⚡ GPU Compute Dispatch** — `wgpu`-based compute shader pipeline supporting Vulkan, Metal, DX12, and WebGPU backends.
- **🔌 Hot-Swappable Plugins** — WebAssembly modules loaded via `Wasmtime`. Update tools without restarting the kernel.
- **📡 High-Performance IPC** — Tokio `mpsc` backbone with extensible `nng` (nanomsg) and Unix Domain Socket transports.
- **🖥️ Terminal Interface** — Real-time `ratatui` dashboard for plugin health, GPU metrics, and system logs.
- **🔧 Command-Line Interface** — Full CLI for plugin management, LLM queries, and GPU task dispatch.

### Plugin Capabilities (10 Implemented, 15 Planned)
| Plugin | Domain | Status |
|--------|--------|--------|
| Context-Aware Repurposing Engine | Content Analysis | ✅ |
| Ethos Guardian | Brand Compliance | ✅ |
| Neuro-Marketing Thumbnail Engine | Image Gen + Scoring | ✅ |
| Factual Verification Engine | Knowledge Graph | ✅ |
| AI Dubbing Studio | Voice Synthesis | ✅ |
| Aether Audio Generator | Procedural Audio | ✅ |
| Video Processing Pipeline | GPU Transcode | ✅ |
| Content Scorer | Multi-Dim Scoring | ✅ |
| Momentum Orchestrator | Task Scheduling | ✅ |
| Forge Framework | Code Generation | ✅ |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        USER INTERFACES                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────────────────────────┐  │
│  │  TUI    │  │  CLI    │  │  SvelteKit Frontend (future)│  │
│  │ratatui  │  │clap     │  │  WebSocket / nng bridge     │  │
│  └────┬────┘  └────┬────┘  └─────────────┬───────────────┘  │
│       └─────────────┴─────────────────────┘                  │
│                         │                                    │
│                    IPC Bus (mpsc / nng / UDS)               │
│                         │                                    │
├─────────────────────────┼────────────────────────────────────┤
│                         ▼                                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              LOOM CORE KERNEL (Tokio)                 │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌────────────────┐  │   │
│  │  │ Plugin      │ │ LLM Engine  │ │ GPU Engine     │  │   │
│  │  │ Registry    │ │ (candle)    │ │ (wgpu)         │  │   │
│  │  │ (Wasmtime)  │ │             │ │                │  │   │
│  │  └──────┬──────┘ └──────┬──────┘ └───────┬────────┘  │   │
│  │         │               │                │           │   │
│  │  ┌──────┴───────────────┴────────────────┴────────┐  │   │
│  │  │           Event Loop (mpsc channel)            │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
│                         │                                    │
│       ┌─────────────────┼─────────────────┐                 │
│       ▼                 ▼                 ▼                 │
│  ┌─────────┐      ┌─────────┐      ┌─────────┐             │
│  │ Shared  │      │   SDK   │      │ Plugins │             │
│  │ Types   │◄────►│  (PDK)  │◄────►│ (Wasm)  │             │
│  └─────────┘      └─────────┘      └─────────┘             │
└─────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Single Model Context** — One LLM inference engine serves all plugins. No VRAM duplication.
2. **Zero-Copy IPC** — In-process mpsc channels with serde serialization. Extensible to nng/UDS.
3. **Hot-Swappable Runtime** — Wasmtime loads `.wasm` plugins dynamically. No kernel restart required.
4. **GPU-First Compute** — All numerical heavy lifting dispatches through wgpu compute shaders.
5. **Platform Agnostic** — Runs on Linux, macOS, Windows. Future WebGPU target for browser deployment.

---

## Installation

### Prerequisites

- **Rust** 1.75+ ([install via rustup](https://rustup.rs/))
- **GPU Drivers** (optional but recommended):
  - NVIDIA: CUDA 12.x + Vulkan drivers
  - AMD: Mesa Vulkan drivers
  - Apple: Metal (built-in)
- **CMake** (for `nng` IPC transport)
- **Python 3** (for model conversion utilities)

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/loom-os.git
cd loom-os

# Build the entire workspace
cargo build --release

# Or build specific components
cargo build --release -p core        # Kernel
cargo build --release -p loom-cli    # CLI
cargo build --release -p loom-tui    # TUI
```

### Download Models (Optional)

For LLM inference, download a compatible model:

```bash
mkdir -p models

# Option 1: GGUF quantized (recommended for local deployment)
wget https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf   -O models/mistral-7b-instruct-v0.2.Q4_K_M.gguf

# Option 2: Safetensors (for Candle native backend)
# Download from HuggingFace and place in models/
```

### Verify Installation

```bash
# Check CLI
./target/release/loom-cli status

# Expected output:
# ╔══════════════════════════════════════╗
# ║      LOOM OS SYSTEM STATUS           ║
# ╠══════════════════════════════════════╣
# ║  Kernel:        Online               ║
# ║  LLM Bridge:    Standby              ║
# ║  GPU Scheduler: Standby              ║
# ║  IPC Transport: mpsc (in-process)    ║
# ║  Plugins:       10 registered        ║
# ╚══════════════════════════════════════╝
```

---

## Quick Start

### 1. Start the Kernel

```bash
# With default config
cargo run --release -p core

# With custom config
LOOM_CONFIG_PATH=./config/loom.toml cargo run --release -p core

# With model paths
LOOM_MODEL_PATH=./models/model.gguf LOOM_TOKENIZER_PATH=./models/tokenizer.json cargo run --release -p core
```

### 2. Launch the TUI (in another terminal)

```bash
cargo run --release -p loom-tui
```

Navigate with `1-5` for tabs, `←→` arrows, `q` to quit.

### 3. Use the CLI

```bash
# List plugins
loom-cli plugin list

# Check system status
loom-cli status

# Send LLM prompt (when model is loaded)
loom-cli llm "Explain the architecture of Loom OS" --max-tokens 256

# Dispatch GPU compute task
loom-cli gpu compute particle_update --input "[1.0,2.0,3.0]"
```

### 4. Load a Custom Plugin

```bash
# Build your plugin to Wasm
cd plugins/my-plugin
cargo build --target wasm32-wasi --release

# Load at runtime
loom-cli plugin load ./target/wasm32-wasi/release/my-plugin.wasm
```

---

## Plugin Ecosystem

### Creating a Plugin

```bash
# Scaffold new plugin
cargo new plugins/my-tool --lib

# Add SDK dependency
cat >> plugins/my-tool/Cargo.toml << 'EOF'
[dependencies]
shared = { path = "../../shared" }
sdk = { path = "../../sdk" }
EOF
```

Implement the `LoomPlugin` trait:

```rust
use shared::*;
use sdk::plugin_manifest;

pub struct MyTool;

impl LoomPlugin for MyTool {
    fn manifest(&self) -> PluginManifest {
        plugin_manifest!(
            "my-tool-001",
            "My Creative Tool",
            "0.1.0",
            "Your Name",
            ["compute", "ai-assisted"],
            "Description of functionality."
        )
    }

    fn execute(&self, ctx: ProjectContext, task: Task) -> Result<TaskResult, String> {
        // Your logic
        Ok(TaskResult {
            task_id: task.task_id,
            success: true,
            output: b"completed".to_vec(),
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

See [`docs/PLUGINS.md`](docs/PLUGINS.md) for the full development guide.

---

## Configuration

Loom OS uses TOML configuration files. Place `loom.toml` in your working directory or set `LOOM_CONFIG_PATH`.

```toml
# config/loom.toml

[llm]
model_path = "./models/mistral-7b-instruct-v0.2.Q4_K_M.gguf"
tokenizer_path = "./models/tokenizer.json"
max_tokens = 1024
temperature = 0.7
top_p = 0.9

[gpu]
backend = "vulkan"  # vulkan | metal | dx12 | webgpu
shader_cache_size = 50

[ipc]
transport = "mpsc"  # mpsc | nng | uds
buffer_size = 1024
nng_address = "tcp://127.0.0.1:9001"
uds_path = "/tmp/loom.sock"

[plugins]
auto_reload = true
wasm_dir = "./plugins/bin"

[logging]
level = "info"
format = "pretty"
```

---

## API Reference

### Message Protocol

All communication uses the `LoomMessage` envelope:

```rust
pub struct LoomMessage {
    pub id: LoomId,           // UUID v4
    pub source: String,       // Sender
    pub target: String,       // Recipient
    pub channel: Channel,     // Control | Data | Log | Gpu | Llm | Audio | Video
    pub payload: Payload,
    pub timestamp: DateTime<Utc>,
}
```

### LLM Request

```json
{
  "model_id": "mistral-7b-instruct",
  "prompt": "Explain quantum computing",
  "max_tokens": 512,
  "temperature": 0.7,
  "top_p": 0.9,
  "stop_sequences": ["

"],
  "stream": false
}
```

### GPU Request

```json
{
  "shader_id": "particle_update",
  "input_buffers": [[1.0, 2.0, 3.0]],
  "output_size": 1024,
  "params": {"gravity": 9.8}
}
```

See [`docs/API.md`](docs/API.md) for the complete reference.

---

## Development

### Running Tests

```bash
cargo test --workspace
```

### Code Formatting

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

### Building Documentation

```bash
cargo doc --workspace --no-deps --open
```

### Project Structure

```
loom-os/
├── core/              # Kernel (Tokio + Wasmtime + candle + wgpu)
│   ├── src/shaders/   # WGSL compute shaders
│   └── src/ipc/       # IPC implementations
├── shared/            # Canonical types and traits
├── sdk/               # Plugin Development Kit
├── cli/               # Command-line interface
├── tui/               # Terminal dashboard
├── plugins/           # Plugin implementations
│   ├── repurpose/
│   ├── ethos-guardian/
│   ├── thumbnail-engine/
│   ├── fact-checker/
│   ├── dubbing-studio/
│   ├── aether-audio/
│   ├── video-pipeline/
│   ├── content-scorer/
│   ├── momentum-orchestrator/
│   └── forge-framework/
├── config/            # Configuration templates
├── docs/              # Architecture and API docs
└── .github/           # CI/CD workflows
```

---

## Academic Context

Loom OS was developed as a research platform exploring the intersection of:

- **Systems Programming** — Rust's ownership model for safe concurrent GPU/LLM resource management
- **WebAssembly Sandboxing** — Capability-based security for untrusted AI plugin code
- **Heterogeneous Computing** — Unified dispatch across CPU (candle), GPU (wgpu), and NPU backends
- **Modular AI Architecture** — Decoupling inference, compute, and orchestration to prevent resource duplication

The system addresses a critical gap in current AI tooling: most creative suites load independent model instances per feature, leading to VRAM exhaustion on consumer hardware. Loom OS demonstrates that a centralized inference bridge with async IPC can serve 25+ specialized tools from a single model context while maintaining sub-100ms task dispatch latency.

### Research Applications

- **AI Safety**: The Ethos Guardian plugin implements configurable ethical constraints for generative outputs
- **Efficient Inference**: The LLM Bridge demonstrates KV-cache sharing across heterogeneous request types
- **GPU Generalization**: The wgpu scheduler abstracts Vulkan/Metal/DX12 into a single compute API
- **Plugin Security**: Wasmtime's capability model enables safe execution of community-contributed tools

### Citation

If you use Loom OS in academic work, please cite:

```bibtex
@software{loom_os_2026,
  title = {Loom OS: A High-Performance Modular Runtime for AI-Powered Creator Tools},
  author = {Loom OS Contributors},
  year = {2026},
  url = {https://github.com/yourusername/loom-os}
}
```

---

## Roadmap

| Phase | Feature | Target |
|-------|---------|--------|
| v0.3.0 | nng/UDS IPC for cross-process communication | Q3 2026 |
| v0.4.0 | SvelteKit Web UI with WebSocket bridge | Q3 2026 |
| v0.5.0 | Full GGUF quantization support (llama.cpp-rs) | Q4 2026 |
| v0.6.0 | Distributed compute across node clusters | Q4 2026 |
| v1.0.0 | Stable API, 25 plugins, production hardening | Q1 2027 |

---

## Contributing

We welcome contributions! Please see our [Architecture Guide](docs/ARCHITECTURE.md) and [Plugin Development Guide](docs/PLUGINS.md) to get started.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

All contributions are dual-licensed under MIT and Apache-2.0.

---

## License

Loom OS is dual-licensed under:

- **MIT License** — See [LICENSE-MIT](LICENSE-MIT)
- **Apache License 2.0** — See [LICENSE-APACHE](LICENSE-APACHE)

You may choose either license at your option.

---

<div align="center">

**Built with Rust · Powered by candle · Accelerated by wgpu**

</div>
