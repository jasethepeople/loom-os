use shared::*;
use sdk::{plugin_manifest, ScoringEngine};

/// Content Scorer
/// Multi-dimensional scoring engine for creative assets.
pub struct ContentScorer {
    engines: Vec<ScoringEngine>,
}

impl ContentScorer {
    pub fn new() -> Self {
        Self {
            engines: vec![
                ScoringEngine::new("Visual appeal and composition").with_threshold(0.60),
                ScoringEngine::new("Brand consistency and voice").with_threshold(0.75),
                ScoringEngine::new("Engagement potential").with_threshold(0.55),
            ],
        }
    }
}

impl LoomPlugin for ContentScorer {
    fn manifest(&self) -> PluginManifest {
        plugin_manifest!(
            "scorer-001",
            "Content Scorer",
            "0.1.0",
            "Loom OS",
            ["multi-dimensional-scoring", "asset-evaluation", "engagement-prediction", "clip-integration"],
            "Scores creative assets across visual, brand, and engagement dimensions."
        )
    }

    fn execute(&self, ctx: ProjectContext, task: Task) -> Result<TaskResult, String> {
        let mut scores = Vec::new();
        let mut total = 0.0;

        for engine in &self.engines {
            let s = engine.score(&task.payload);
            scores.push(s);
            total += s;
        }

        let avg = total / self.engines.len() as f64;
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("visual_score".into(), scores[0]);
        metrics.insert("brand_score".into(), scores[1]);
        metrics.insert("engagement_score".into(), scores[2]);
        metrics.insert("composite".into(), avg);

        let output = format!(
            "[{}] Content scored | Composite: {:.2} | Visual: {:.2} | Brand: {:.2} | Engagement: {:.2}",
            ctx.project_name, avg, scores[0], scores[1], scores[2]
        );

        Ok(TaskResult {
            task_id: task.task_id,
            success: avg >= 0.65,
            output: output.into_bytes(),
            metrics,
        })
    }

    fn health(&self) -> PluginHealth {
        PluginHealth {
            status: HealthStatus::Healthy,
            last_heartbeat: chrono::Utc::now(),
            memory_mb: 32.0,
            uptime_secs: 0,
        }
    }
}
