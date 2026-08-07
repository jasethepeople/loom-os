use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoomConfig {
    pub llm: LlmConfig,
    pub gpu: GpuConfig,
    pub ipc: IpcConfig,
    pub plugins: PluginConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmConfig {
    pub model_path: String,
    pub tokenizer_path: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuConfig {
    pub backend: String,
    pub shader_cache_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpcConfig {
    pub transport: String,
    pub buffer_size: usize,
    pub nng_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    pub auto_reload: bool,
    pub wasm_dir: String,
}

impl LoomConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

impl Default for LoomConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                model_path: "./models/model.safetensors".into(),
                tokenizer_path: "./models/tokenizer.json".into(),
                max_tokens: 512,
                temperature: 0.7,
                top_p: 0.9,
            },
            gpu: GpuConfig {
                backend: "vulkan".into(),
                shader_cache_size: 50,
            },
            ipc: IpcConfig {
                transport: "mpsc".into(),
                buffer_size: 1024,
                nng_address: "tcp://127.0.0.1:9001".into(),
            },
            plugins: PluginConfig {
                auto_reload: true,
                wasm_dir: "./plugins/bin".into(),
            },
        }
    }
}
