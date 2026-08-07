use shared::{PluginManifest, PluginHealth, HealthStatus};
use std::collections::HashMap;

/// Manages the lifecycle of all loaded plugins.
/// Thread-safe via RwLock in the kernel.
pub struct PluginRegistry {
    manifests: HashMap<String, PluginManifest>,
    health: HashMap<String, PluginHealth>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            manifests: HashMap::new(),
            health: HashMap::new(),
        }
    }

    pub fn register(&mut self, manifest: PluginManifest) {
        self.health.insert(
            manifest.id.clone(),
            PluginHealth {
                status: HealthStatus::Healthy,
                last_heartbeat: chrono::Utc::now(),
                memory_mb: 0.0,
                uptime_secs: 0,
            },
        );
        self.manifests.insert(manifest.id.clone(), manifest);
    }

    pub fn unregister(&mut self, id: &str) {
        self.manifests.remove(id);
        self.health.remove(id);
    }

    pub fn list(&self) -> Vec<&PluginManifest> {
        self.manifests.values().collect()
    }

    pub fn get(&self, id: &str) -> Option<&PluginManifest> {
        self.manifests.get(id)
    }

    pub fn health_snapshot(&self) -> &HashMap<String, PluginHealth> {
        &self.health
    }

    pub fn count(&self) -> usize {
        self.manifests.len()
    }
}
