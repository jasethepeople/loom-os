# Loom OS Architecture

## Overview

Loom OS is a high-performance, modular runtime designed to host AI-powered creator tools as hot-swappable plugins. It is built on a Rust async core with WebAssembly plugin support, centralized LLM inference, and GPU compute dispatch.

## Design Principles

1. **Single Model Context**: All plugins route LLM requests through one shared inference engine, preventing VRAM duplication.
2. **Zero-Copy IPC**: High-throughput message passing via Tokio mpsc (extensible to nng/UDS).
3. **Hot-Swappable Plugins**: WebAssembly modules can be loaded and unloaded at runtime without kernel restart.
4. **GPU-First Compute**: All heavy numerical work dispatches through wgpu compute shaders.
5. **Platform Agnostic**: Runs on Linux, macOS, Windows, and eventually WebGPU-enabled browsers.

## Component Diagram

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

## Message Flow

1. **Plugin sends Task** → IPC Bus → Kernel Event Loop
2. **Kernel routes by Channel**:
   - `Control` → Plugin Registry
   - `Llm` → LlmEngine (single model context)
   - `Gpu` → GpuEngine (wgpu compute dispatch)
   - `Data` → Plugin execution context
3. **Result returns** via async `mpsc` back to caller

## Plugin Lifecycle

```
┌─────────┐    load_module()    ┌─────────────┐    register()    ┌──────────┐
│  .wasm  │ ──────────────────► │ WasmRuntime │ ───────────────► │ Registry │
│  file   │                     │  (Wasmtime) │                  │          │
└─────────┘                     └─────────────┘                  └────┬─────┘
                                                                    │
                              execute()                             │
                              ◄─────────────────────────────────────┘
                              health() — every 5s
```

## Security Model

- Plugins run in Wasmtime sandbox with WASI capabilities
- No filesystem access without explicit capability grants
- LLM inference is centralized — plugins cannot load arbitrary models
- GPU compute shaders are pre-compiled and cached — no runtime shader injection
