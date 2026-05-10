use serde::{Deserialize, Serialize};

use super::capability_tier::TierConfig;

pub type WhatsAppLinkRuntimeEvent = super::whatsapp_link::WhatsAppLinkEvent;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WelesVerdict {
    Allow,
    Block,
    FlagOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WelesReviewMeta {
    pub weles_reviewed: bool,
    pub verdict: WelesVerdict,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_override_mode: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WelesHealthState {
    Healthy,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WelesHealthStatus {
    pub state: WelesHealthState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub checked_at: u64,
}

mod agent_event;
mod browser_profiles;
mod config_core;
mod config_skill;
mod custom_provider_catalog;
mod custom_provider_runtime;
mod goal_dossier;
mod goal_types;
mod heartbeat_misc;
mod provider_basics;
mod provider_catalog_a;
mod provider_catalog_b;
mod provider_registry;
mod runtime_config;
mod runtime_profiles;
mod task_types;
mod thread_message_types;

pub use agent_event::*;
pub use browser_profiles::*;
pub use config_core::*;
pub use config_skill::*;
pub use custom_provider_catalog::*;
pub use custom_provider_runtime::*;
pub use goal_dossier::*;
pub use goal_types::*;
pub use heartbeat_misc::*;
pub use provider_basics::*;
pub use provider_catalog_a::*;
pub use provider_catalog_b::*;
pub use provider_registry::*;
pub use runtime_config::*;
pub use runtime_profiles::*;
pub use task_types::*;
pub use thread_message_types::*;

#[cfg(test)]
mod tests;
