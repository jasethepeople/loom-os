//! # Loom OS Shared Library
//!
//! Canonical data structures, traits, and message types for all Loom OS components.
//! This crate defines the contract between the kernel, plugins, and external interfaces.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for any entity in the Loom ecosystem.
pub type LoomId = uuid::Uuid;

/// The canonical message envelope for all IPC in Loom OS.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoomMessage {
    pub id: LoomId,
    pub source: String,
    pub target: String,
    pub channel: Channel,
    pub payload: Payload,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl LoomMessage {
    pub fn new(source: &str, target: &str, channel: Channel, payload: Payload) -> Self {
        Self {
            id: LoomId::new_v4(),
            source: source.to_string(),
            target: target.to_string(),
            channel,
            payload,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Channel {
    Control,
    Data,
    Log,
    Gpu,
    Llm,
    Audio,
    Video,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    Command(Command),
    Task(Task),
    Result(TaskResult),
    Heartbeat,
    Error(String),
    StreamChunk(StreamChunk),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub verb: String,
    pub args: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub task_id: LoomId,
    pub plugin_id: String,
    pub payload: Vec<u8>,
    pub priority: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskResult {
    pub task_id: LoomId,
    pub success: bool,
    pub output: Vec<u8>,
    pub metrics: HashMap<String, f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamChunk {
    pub task_id: LoomId,
    pub chunk: Vec<u8>,
    pub is_final: bool,
}

/// Project context passed to every plugin execution.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectContext {
    pub project_name: String,
    pub root_dir: String,
    pub platform_targets: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Core trait that ALL plugins must implement.
pub trait LoomPlugin: Send + Sync {
    fn manifest(&self) -> PluginManifest;
    fn execute(&self, ctx: ProjectContext, task: Task) -> Result<TaskResult, String>;
    fn health(&self) -> PluginHealth;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub capabilities: Vec<String>,
    pub wasm_path: Option<String>,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginHealth {
    pub status: HealthStatus,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub memory_mb: f64,
    pub uptime_secs: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Failed,
    Unknown,
}

/// GPU compute request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GpuRequest {
    pub shader_id: String,
    pub input_buffers: Vec<Vec<f32>>,
    pub output_size: usize,
    pub params: HashMap<String, f32>,
}

/// LLM inference request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LlmRequest {
    pub model_id: String,
    pub prompt: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub stop_sequences: Vec<String>,
    pub stream: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub tokens_used: usize,
    pub finish_reason: String,
    pub model_id: String,
}

/// Audio generation request for the Aether module.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioRequest {
    pub seed: u64,
    pub duration_secs: f32,
    pub sample_rate: u32,
    pub waveform_type: WaveformType,
    pub effects: Vec<AudioEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WaveformType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
    Granular,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AudioEffect {
    Reverb { room_size: f32, damping: f32 },
    Delay { time_ms: f32, feedback: f32 },
    Filter { cutoff_hz: f32, resonance: f32 },
}

/// Video processing request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoRequest {
    pub input_path: String,
    pub output_path: String,
    pub operations: Vec<VideoOperation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VideoOperation {
    Transcode { codec: String, bitrate: u32 },
    ExtractFrames { fps: f32, format: String },
    ApplyFilter { filter_id: String, params: HashMap<String, f32> },
    Concat { sources: Vec<String> },
}
