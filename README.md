# Loom OS

A high-performance, modular runtime for AI-powered creator tools. All tools run as hot-swappable WebAssembly plugins sharing one centralized inference engine — the **single-model-context architecture** means plugins never each load their own large model. MIT / Apache-2.0 dual licensed.

## Features

- **Centralized LLM inference** — one shared model context (per the README, via `candle` with CUDA/Metal/CPU fallback) used by every plugin.
- **GPU compute dispatch** — a `wgpu`-based compute pipeline across Vulkan, Metal, DX12, and WebGPU.
- **Hot-swappable WASM plugins** — plugins loaded via Wasmtime without restarting the kernel; a plugin SDK (`sdk/`) and scaffolding script (`scripts/scaffold-plugin.sh`) are included. The repo ships 10 plugins (`plugins/`): repurpose, ethos-guardian, thumbnail-engine, fact-checker, dubbing-studio, aether-audio, video-pipeline, content-scorer, momentum-orchestrator, forge-framework.
- **High-performance IPC** — Tokio `mpsc` backbone with nanomsg (`nng`) and Unix Domain Socket transports.
- **Terminal UI** — `ratatui` dashboard (in `tui/`) for plugin health, GPU metrics, and system logs.
- **CLI** (`cli/`) — plugin management, LLM queries, and GPU task dispatch.
- **Docs** — `docs/API.md`, `docs/ARCHITECTURE.md`, `docs/PLUGINS.md`; example config at `config/loom.toml`.

## Tech stack

Rust 1.75+ (Cargo workspace: `core`, `shared`, `sdk`, `tui`, `cli`, `plugins/*`), Tokio, Serde, Wasmtime, wgpu, candle, ratatui, tracing.

## Getting started

From the repo (scripts at `scripts/`):

- `scripts/build-all.sh` — build the whole workspace
- `cargo build --release` / `cargo run` in `cli/` or `tui/`
- `scripts/scaffold-plugin.sh` — scaffold a new plugin from the SDK
- `scripts/setup-models.sh` — fetch model weights
- The repo's full README has installation, quick start, and configuration sections; consult it for environment details.

## Project structure

```
.
├── core/        # async runtime kernel
├── shared/      # shared types/messages
├── sdk/         # plugin SDK
├── plugins/     # 10 WASM plugins (audio, video, text, scoring)
├── cli/         # command-line interface
├── tui/         # ratatui terminal dashboard
├── docs/        # API, architecture, plugin docs
├── config/      # loom.toml example
└── scripts/     # build, scaffold, model setup
```

## Status

**Real project.** A substantial Rust workspace with docs and scripts. The repo ships a long-form README (abstract, architecture, plugin table, API reference); this is a condensed version.
