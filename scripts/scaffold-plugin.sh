#!/usr/bin/env bash
set -e

PLUGIN_NAME="$1"
if [ -z "$PLUGIN_NAME" ]; then
    echo "Usage: $0 <plugin-name>"
    exit 1
fi

echo "Scaffolding plugin: $PLUGIN_NAME"
cargo new "plugins/$PLUGIN_NAME" --lib

cat >> "plugins/$PLUGIN_NAME/Cargo.toml" << EOF

[dependencies]
shared = { path = "../../shared" }
sdk = { path = "../../sdk" }
serde = { workspace = true }
serde_json = { workspace = true }
EOF

cat > "plugins/$PLUGIN_NAME/src/lib.rs" << 'EOF'
use shared::*;
use sdk::plugin_manifest;

pub struct MyPlugin;

impl LoomPlugin for MyPlugin {
    fn manifest(&self) -> PluginManifest {
        plugin_manifest!(
            "my-plugin-001",
            "My Plugin",
            "0.1.0",
            "Loom OS",
            ["compute"],
            "Description here."
        )
    }

    fn execute(&self, ctx: ProjectContext, task: Task) -> Result<TaskResult, String> {
        Ok(TaskResult {
            task_id: task.task_id,
            success: true,
            output: format!("Hello from {}", ctx.project_name).into_bytes(),
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
EOF

echo "Plugin scaffolded at plugins/$PLUGIN_NAME/"
echo "Implement your logic in plugins/$PLUGIN_NAME/src/lib.rs"
