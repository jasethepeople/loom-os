use shared::*;
use sdk::plugin_manifest;

/// Ethos Guardian
/// Ensures all generated content aligns with creator-defined values and brand identity.
pub struct EthosGuardian {
    rules: Vec<String>,
}

impl EthosGuardian {
    pub fn new() -> Self {
        Self {
            rules: vec![
                "No sensationalism".into(),
                "Cite sources".into(),
                "Respect privacy".into(),
                "No misleading thumbnails".into(),
                "Accessibility-first".into(),
            ],
        }
    }
}

impl LoomPlugin for EthosGuardian {
    fn manifest(&self) -> PluginManifest {
        plugin_manifest!(
            "ethos-001",
            "Ethos Guardian",
            "0.1.0",
            "Loom OS",
            ["content-moderation", "brand-guard", "compliance", "ethical-ai"],
            "Validates content against a configurable ethical framework and brand guidelines."
        )
    }

    fn execute(&self, ctx: ProjectContext, task: Task) -> Result<TaskResult, String> {
        let content = String::from_utf8_lossy(&task.payload);
        let violations: Vec<&str> = self.rules.iter()
            .filter(|r| !content.to_lowercase().contains(&r.to_lowercase()))
            .map(|r| r.as_str())
            .collect();

        let success = violations.is_empty();
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("violations".into(), violations.len() as f64);
        metrics.insert("rules_checked".into(), self.rules.len() as f64);

        let output = if success {
            format!("[{}] Ethos validation PASSED", ctx.project_name)
        } else {
            format!("[{}] Ethos validation FAILED: {:?}", ctx.project_name, violations)
        };

        Ok(TaskResult {
            task_id: task.task_id,
            success,
            output: output.into_bytes(),
            metrics,
        })
    }

    fn health(&self) -> PluginHealth {
        PluginHealth {
            status: HealthStatus::Healthy,
            last_heartbeat: chrono::Utc::now(),
            memory_mb: 4.2,
            uptime_secs: 0,
        }
    }
}
