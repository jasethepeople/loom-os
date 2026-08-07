use shared::{LlmRequest, LlmResponse, Task};
use candle_core::{Device, Tensor, DType};
use candle_transformers::generation::LogitsProcessor;
use tokenizers::Tokenizer;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error, debug, warn};
use anyhow::Result;

/// Centralized LLM inference engine.
/// All plugins route LLM requests through this single instance,
/// preventing VRAM duplication across the ecosystem.
pub struct LlmEngine {
    device: Device,
    tokenizer: Arc<Tokenizer>,
    config: GenerationConfig,
    backend: Arc<RwLock<Box<dyn ModelBackend>>>,
    result_tx: mpsc::Sender<(String, LlmResponse)>,
}

#[derive(Clone, Debug)]
pub struct GenerationConfig {
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_p: f64,
    pub repeat_penalty: f64,
    pub seed: u64,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
            seed: 42,
        }
    }
}

pub trait ModelBackend: Send + Sync {
    fn forward(&mut self, input: &Tensor, pos: usize) -> Result<Tensor>;
    fn clear_kv_cache(&mut self);
    fn dtype(&self) -> DType;
    fn num_layers(&self) -> usize;
}

pub struct CandleBackend {
    model: candle_transformers::models::llama::Model,
    cache: candle_transformers::models::llama::Cache,
}

impl CandleBackend {
    pub fn load(path: &Path, device: &Device) -> Result<Self> {
        info!("Loading Candle model from {:?}", path);
        let config = candle_transformers::models::llama::Config::config_7b_v2(false);
        let vb = candle_nn::VarBuilder::from_pth(path, DType::F16, device)?;
        let cache = candle_transformers::models::llama::Cache::new(true, &config, device)?;
        let model = candle_transformers::models::llama::Model::load(vb, &config)?;
        info!("Model loaded: {} layers, dtype {:?}", config.num_hidden_layers, DType::F16);
        Ok(Self { model, cache })
    }
}

impl ModelBackend for CandleBackend {
    fn forward(&mut self, input: &Tensor, pos: usize) -> Result<Tensor> {
        let logits = self.model.forward(input, pos)?;
        Ok(logits)
    }
    fn clear_kv_cache(&mut self) { self.cache.clear(); }
    fn dtype(&self) -> DType { DType::F16 }
    fn num_layers(&self) -> usize { self.cache.num_layers() }
}

pub struct QuantizedBackend;

impl ModelBackend for QuantizedBackend {
    fn forward(&mut self, _input: &Tensor, _pos: usize) -> Result<Tensor> {
        anyhow::bail!("Quantized backend: implement gguf_file::Content::read + ModelWeights::from_gguf")
    }
    fn clear_kv_cache(&mut self) {}
    fn dtype(&self) -> DType { DType::F32 }
    fn num_layers(&self) -> usize { 0 }
}

impl LlmEngine {
    pub async fn new(model_path: &Path, tokenizer_path: &Path) -> Result<(Self, mpsc::Receiver<(String, LlmResponse)>)> {
        let device = if candle_core::utils::cuda_is_available() {
            info!("CUDA available — using GPU 0");
            Device::new_cuda(0)?
        } else if candle_core::utils::metal_is_available() {
            info!("Metal available — using Apple Silicon");
            Device::new_metal(0)?
        } else {
            warn!("No GPU detected — falling back to CPU (slow)");
            Device::Cpu
        };

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Tokenizer load failed: {}", e))?;

        let backend: Box<dyn ModelBackend> = if model_path.extension().map(|e| e == "gguf").unwrap_or(false) {
            info!("Detected GGUF — using quantized backend");
            Box::new(QuantizedBackend)
        } else {
            info!("Detected standard weights — using Candle backend");
            Box::new(CandleBackend::load(model_path, &device)?)
        };

        let (result_tx, result_rx) = mpsc::channel(100);

        let engine = Self {
            device,
            tokenizer: Arc::new(tokenizer),
            config: GenerationConfig::default(),
            backend: Arc::new(RwLock::new(backend)),
            result_tx,
        };

        Ok((engine, result_rx))
    }

    pub async fn route(&self, task: Task) {
        let req: LlmRequest = match serde_json::from_slice(&task.payload) {
            Ok(r) => r,
            Err(e) => {
                error!("Task {}: deserialize error: {}", task.task_id, e);
                return;
            }
        };

        match self.generate(&req).await {
            Ok(response) => {
                if let Err(e) = self.result_tx.send((task.task_id.to_string(), response)).await {
                    error!("Result send failed: {}", e);
                }
            }
            Err(e) => error!("Generation failed for {}: {}", task.task_id, e),
        }
    }

    pub async fn generate(&self, req: &LlmRequest) -> Result<LlmResponse> {
        let mut backend = self.backend.write().await;
        backend.clear_kv_cache();

        let encoding = self.tokenizer.encode(req.prompt.clone(), true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
        let mut tokens = encoding.get_ids().to_vec();
        let mut logits_processor = LogitsProcessor::new(
            req.temperature as f64,
            Some(req.temperature as f64),
            Some(req.temperature as f64),
        );

        let mut generated_tokens = 0usize;
        let mut output_text = String::new();
        let start_pos = tokens.len();

        for pos in 0..req.max_tokens {
            let input = Tensor::new(&tokens[..], &self.device)?.unsqueeze(0)?;
            let logits = backend.forward(&input, start_pos + pos)?;
            let logits = logits.i((0, logits.dim(1)? - 1))?;
            let next_token = logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens += 1;

            if let Some(text) = self.tokenizer.decode(&[next_token], false).ok() {
                output_text.push_str(&text);
                debug!("Token {}: {}", pos, text.trim());
            }

            if req.stop_sequences.iter().any(|s| output_text.contains(s)) {
                info!("Stop sequence hit at token {}", pos);
                break;
            }

            if next_token == self.tokenizer.token_to_id("</s>").unwrap_or(2) {
                info!("EOS at position {}", pos);
                break;
            }
        }

        info!("Generation complete: {} tokens", generated_tokens);

        Ok(LlmResponse {
            text: output_text.trim().to_string(),
            tokens_used: generated_tokens,
            finish_reason: if generated_tokens >= req.max_tokens { "length".into() } else { "stop".into() },
            model_id: req.model_id.clone(),
        })
    }

    pub async fn swap_model(&self, new_backend: Box<dyn ModelBackend>) {
        let mut backend = self.backend.write().await;
        *backend = new_backend;
        info!("Model backend hot-swapped");
    }
}
