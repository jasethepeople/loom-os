# Contributing to Loom OS

Thank you for your interest in contributing to Loom OS! This document provides guidelines for participating in the project.

## Code of Conduct

Be respectful, constructive, and inclusive. All contributions are welcome regardless of experience level.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your work
4. Make your changes
5. Run tests and formatting
6. Submit a pull request

## Development Setup

```bash
git clone https://github.com/yourusername/loom-os.git
cd loom-os
rustup update
cargo build --release
```

## Coding Standards

- Follow the Rust API Guidelines
- Run `cargo fmt` before committing
- Run `cargo clippy --all-targets -- -D warnings`
- Write doc comments for all public APIs
- Add tests for new functionality

## Commit Messages

Use conventional commits format:

```
feat: add new plugin capability
docs: update API reference
fix: resolve GPU memory leak
refactor: simplify IPC message routing
test: add integration tests for LLM bridge
```

## Pull Request Process

1. Update documentation if you change APIs
2. Add tests for new features
3. Ensure CI passes (check, test, fmt, clippy)
4. Request review from maintainers
5. Address feedback promptly

## Plugin Contributions

To contribute a new plugin:

1. Create a new crate in `plugins/`
2. Implement the `LoomPlugin` trait
3. Add documentation and examples
4. Update the plugin registry in `core/src/plugin_registry.rs` if needed
5. Submit a PR with a description of the plugin's capabilities

## Questions?

Open a GitHub Discussion or reach out via the project's communication channels.
