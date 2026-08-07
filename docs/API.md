# Loom OS API Reference

## Message Types

### LoomMessage
The canonical envelope for all IPC.

```rust
pub struct LoomMessage {
    pub id: LoomId,           // UUID v4
    pub source: String,       // Sender identifier
    pub target: String,       // Recipient identifier
    pub channel: Channel,     // Control | Data | Log | Gpu | Llm | Audio | Video
    pub payload: Payload,     // See below
    pub timestamp: DateTime<Utc>,
}
```

### Channels

| Channel | Purpose |
|---------|---------|
| `Control` | Plugin lifecycle, system commands |
| `Data` | General plugin execution |
| `Log` | Unified logging |
| `Gpu` | GPU compute dispatch |
| `Llm` | LLM inference requests |
| `Audio` | Audio synthesis pipeline |
| `Video` | Video processing pipeline |

### Payload Variants

```rust
pub enum Payload {
    Command(Command),       // Control verbs
    Task(Task),             // Plugin execution
    Result(TaskResult),     // Execution result
    Heartbeat,              // Health ping
    Error(String),          // Error report
    StreamChunk(StreamChunk), // Streaming data
}
```

## LLM Request Format

```json
{
  "model_id": "mistral-7b-instruct",
  "prompt": "Explain quantum computing in simple terms",
  "max_tokens": 512,
  "temperature": 0.7,
  "top_p": 0.9,
  "stop_sequences": ["

"],
  "stream": false
}
```

## GPU Request Format

```json
{
  "shader_id": "particle_update",
  "input_buffers": [[1.0, 2.0, 3.0, ...]],
  "output_size": 1024,
  "params": {"gravity": 9.8, "damping": 0.995}
}
```
