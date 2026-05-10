use super::*;
use serde::Serialize;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub(crate) struct BackgroundSubsystemMetricsSnapshot {
    pub(crate) current_depth: usize,
    pub(crate) max_depth: usize,
    pub(crate) rejection_count: u64,
    pub(crate) accepted_count: u64,
    pub(crate) started_count: u64,
    pub(crate) completed_count: u64,
    pub(crate) failed_count: u64,
    pub(crate) accepted_to_started_samples: u64,
    pub(crate) started_to_terminal_samples: u64,
    pub(crate) last_accepted_to_started_ms: Option<u64>,
    pub(crate) last_started_to_terminal_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
struct OperationTimingState {
    subsystem: BackgroundSubsystem,
    accepted_at: std::time::Instant,
    started_at: Option<std::time::Instant>,
}

#[derive(Default)]
pub(crate) struct BackgroundSubsystemMetrics {
    per_subsystem: std::sync::Mutex<
        std::collections::HashMap<BackgroundSubsystem, BackgroundSubsystemMetricsSnapshot>,
    >,
    inflight_operations: std::sync::Mutex<std::collections::HashMap<String, OperationTimingState>>,
}

pub(crate) fn subsystem_metrics() -> &'static BackgroundSubsystemMetrics {
    static METRICS: std::sync::OnceLock<BackgroundSubsystemMetrics> = std::sync::OnceLock::new();
    METRICS.get_or_init(BackgroundSubsystemMetrics::default)
}

impl BackgroundSubsystemMetrics {
    pub(crate) fn snapshot_for(
        &self,
        subsystem: BackgroundSubsystem,
    ) -> BackgroundSubsystemMetricsSnapshot {
        self.per_subsystem
            .lock()
            .expect("subsystem metrics mutex poisoned")
            .get(&subsystem)
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn all_snapshots_json(&self) -> String {
        let mut json = serde_json::Map::new();
        for subsystem in BackgroundSubsystem::ALL {
            json.insert(
                subsystem.metric_key().to_string(),
                serde_json::to_value(self.snapshot_for(subsystem)).unwrap_or_default(),
            );
        }
        serde_json::Value::Object(json).to_string()
    }

    pub(crate) fn record_depth(&self, subsystem: BackgroundSubsystem, depth: usize) {
        let mut metrics = self
            .per_subsystem
            .lock()
            .expect("subsystem metrics mutex poisoned");
        let snapshot = metrics.entry(subsystem).or_default();
        snapshot.current_depth = depth;
        snapshot.max_depth = snapshot.max_depth.max(depth);
    }

    pub(crate) fn record_rejection(&self, subsystem: BackgroundSubsystem) {
        let mut metrics = self
            .per_subsystem
            .lock()
            .expect("subsystem metrics mutex poisoned");
        let snapshot = metrics.entry(subsystem).or_default();
        snapshot.rejection_count = snapshot.rejection_count.saturating_add(1);
    }

    pub(crate) fn record_operation_accepted(&self, kind: &str, operation_id: &str) {
        let Some(subsystem) = subsystem_for_operation_kind(kind) else {
            return;
        };

        {
            let mut metrics = self
                .per_subsystem
                .lock()
                .expect("subsystem metrics mutex poisoned");
            let snapshot = metrics.entry(subsystem).or_default();
            snapshot.accepted_count = snapshot.accepted_count.saturating_add(1);
        }

        self.inflight_operations
            .lock()
            .expect("subsystem inflight metrics mutex poisoned")
            .insert(
                operation_id.to_string(),
                OperationTimingState {
                    subsystem,
                    accepted_at: std::time::Instant::now(),
                    started_at: None,
                },
            );
    }

    pub(crate) fn record_operation_started(&self, operation_id: &str) {
        let mut inflight = self
            .inflight_operations
            .lock()
            .expect("subsystem inflight metrics mutex poisoned");
        let Some(state) = inflight.get_mut(operation_id) else {
            return;
        };

        let elapsed_ms = state.accepted_at.elapsed().as_millis() as u64;
        state.started_at = Some(std::time::Instant::now());

        let mut metrics = self
            .per_subsystem
            .lock()
            .expect("subsystem metrics mutex poisoned");
        let snapshot = metrics.entry(state.subsystem).or_default();
        snapshot.started_count = snapshot.started_count.saturating_add(1);
        snapshot.accepted_to_started_samples =
            snapshot.accepted_to_started_samples.saturating_add(1);
        snapshot.last_accepted_to_started_ms = Some(elapsed_ms);
    }

    pub(crate) fn record_operation_terminal(&self, operation_id: &str, failed: bool) {
        let mut inflight = self
            .inflight_operations
            .lock()
            .expect("subsystem inflight metrics mutex poisoned");
        let Some(state) = inflight.remove(operation_id) else {
            return;
        };

        let elapsed_ms = state
            .started_at
            .map(|started_at| started_at.elapsed().as_millis() as u64);

        let mut metrics = self
            .per_subsystem
            .lock()
            .expect("subsystem metrics mutex poisoned");
        let snapshot = metrics.entry(state.subsystem).or_default();
        if failed {
            snapshot.failed_count = snapshot.failed_count.saturating_add(1);
        } else {
            snapshot.completed_count = snapshot.completed_count.saturating_add(1);
        }
        if let Some(elapsed_ms) = elapsed_ms {
            snapshot.started_to_terminal_samples =
                snapshot.started_to_terminal_samples.saturating_add(1);
            snapshot.last_started_to_terminal_ms = Some(elapsed_ms);
        }
    }

    #[cfg(test)]
    pub(crate) fn reset_for_tests(&self) {
        self.per_subsystem
            .lock()
            .expect("subsystem metrics mutex poisoned")
            .clear();
        self.inflight_operations
            .lock()
            .expect("subsystem inflight metrics mutex poisoned")
            .clear();
    }
}

impl BackgroundSubsystem {
    pub(crate) fn metric_key(self) -> &'static str {
        match self {
            BackgroundSubsystem::ConciergeWork => "concierge_work",
            BackgroundSubsystem::AgentWork => "agent_work",
            BackgroundSubsystem::ProviderIo => "provider_io",
            BackgroundSubsystem::PluginIo => "plugin_io",
            BackgroundSubsystem::ConfigReconcile => "config_reconcile",
        }
    }
}

fn subsystem_for_operation_kind(kind: &str) -> Option<BackgroundSubsystem> {
    match kind {
        OPERATION_KIND_CONCIERGE_WELCOME => Some(BackgroundSubsystem::ConciergeWork),
        OPERATION_KIND_PROVIDER_VALIDATION | OPERATION_KIND_FETCH_MODELS => {
            Some(BackgroundSubsystem::ProviderIo)
        }
        OPERATION_KIND_PLUGIN_OAUTH_START
        | OPERATION_KIND_PLUGIN_API_CALL
        | OPERATION_KIND_SKILL_PUBLISH
        | OPERATION_KIND_SKILL_IMPORT => Some(BackgroundSubsystem::PluginIo),
        OPERATION_KIND_EXPLAIN_ACTION
        | OPERATION_KIND_SYNTHESIZE_TOOL
        | OPERATION_KIND_START_DIVERGENT_SESSION => Some(BackgroundSubsystem::AgentWork),
        OPERATION_KIND_CONFIG_SET_ITEM
        | OPERATION_KIND_SET_PROVIDER_MODEL
        | OPERATION_KIND_SET_SUB_AGENT
        | OPERATION_KIND_REMOVE_SUB_AGENT => Some(BackgroundSubsystem::ConfigReconcile),
        _ => None,
    }
}
