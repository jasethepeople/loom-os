//! # Loom OS SDK
//!
//! The Plugin Development Kit. All Loom plugins derive from this crate.
//!
//! ## Quick Start
//! ```rust,ignore
//! use sdk::{plugin_manifest, LoomPlugin, ProjectContext, Task, TaskResult};
//!
//! pub struct MyPlugin;
//! impl LoomPlugin for MyPlugin {
//!     fn manifest(&self) -> PluginManifest {
//!         plugin_manifest!("my-plugin", "My Plugin", "0.1.0", "Me", ["compute"])
//!     }
//!     fn execute(&self, ctx: ProjectContext, task: Task) -> Result<TaskResult, String> {
//!         // Your logic here
//!         Ok(TaskResult { ... })
//!     }
//! }
//! ```

pub use shared::*;

/// Convenience macro for plugin authors to declare their manifest.
#[macro_export]
macro_rules! plugin_manifest {
    ($id:expr, $name:expr, $version:expr, $author:expr, [$($cap:expr),*]) => {
        PluginManifest {
            id: $id.to_string(),
            name: $name.to_string(),
            version: $version.to_string(),
            author: $author.to_string(),
            capabilities: vec![$($cap.to_string()),*],
            wasm_path: None,
            description: String::new(),
        }
    };
    ($id:expr, $name:expr, $version:expr, $author:expr, [$($cap:expr),*], $desc:expr) => {
        PluginManifest {
            id: $id.to_string(),
            name: $name.to_string(),
            version: $version.to_string(),
            author: $author.to_string(),
            capabilities: vec![$($cap.to_string()),*],
            wasm_path: None,
            description: $desc.to_string(),
        }
    };
}

/// Neuro-Marketing scoring engine using CLIP-like semantic scoring.
pub struct ScoringEngine {
    pub ethos_definition: String,
    pub threshold: f64,
}

impl ScoringEngine {
    pub fn new(ethos: &str) -> Self {
        Self {
            ethos_definition: ethos.to_string(),
            threshold: 0.6,
        }
    }

    pub fn with_threshold(mut self, t: f64) -> Self {
        self.threshold = t;
        self
    }

    /// Score an asset against the defined Ethos.
    /// In production, this dispatches to a quantized CLIP model via candle.
    pub fn score(&self, _asset: &[u8]) -> f64 {
        // Placeholder: integrate candle + CLIP
        // Real implementation would:
        // 1. Encode image through CLIP vision encoder
        // 2. Encode ethos text through CLIP text encoder
        // 3. Compute cosine similarity
        0.85
    }

    pub fn passes(&self, asset: &[u8]) -> bool {
        self.score(asset) >= self.threshold
    }
}

/// Zero-copy buffer pool for high-throughput frame passing.
pub struct FramePool {
    pub capacity: usize,
    pub pool: std::sync::Mutex<Vec<Vec<u8>>>,
}

impl FramePool {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            pool: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn acquire(&self) -> Vec<u8> {
        let mut pool = self.pool.lock().unwrap();
        pool.pop().unwrap_or_else(|| Vec::with_capacity(self.capacity))
    }

    pub fn release(&self, mut buf: Vec<u8>) {
        buf.clear();
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < 32 {
            pool.push(buf);
        }
    }
}
