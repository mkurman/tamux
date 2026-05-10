use super::*;
use serde::{Deserialize, Serialize};

pub use crate::agent::memory_context::PromptMemoryInjectionState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentThread {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    pub title: String,
    pub messages: Vec<AgentMessage>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_transport: Option<ApiTransport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_assistant_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadExecutionProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestSkillDiscoveryState {
    pub query: String,
    pub confidence_tier: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_skill: Option<String>,
    pub recommended_action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh_next_step: Option<crate::agent::skill_mesh::types::SkillMeshNextStep>,
    #[serde(default)]
    pub mesh_requires_approval: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh_approval_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_skill_identifier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_rationale: Option<String>,
    #[serde(default, skip_serializing_if = "bool_is_false")]
    pub discovery_pending: bool,
    #[serde(default)]
    pub skill_read_completed: bool,
    #[serde(default)]
    pub compliant: bool,
    pub updated_at: u64,
}

pub(crate) const MAX_PERSISTED_SKILL_DISCOVERY_QUERY_CHARS: usize = 160;

pub(crate) fn compact_skill_discovery_query_for_persistence(query: &str) -> String {
    let mut compact = query
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if compact.chars().count() > MAX_PERSISTED_SKILL_DISCOVERY_QUERY_CHARS {
        compact = compact
            .chars()
            .take(MAX_PERSISTED_SKILL_DISCOVERY_QUERY_CHARS)
            .collect::<String>()
            .trim()
            .to_string();
    }
    compact
}

fn bool_is_false(value: &bool) -> bool {
    !*value
}

impl LatestSkillDiscoveryState {
    pub(crate) fn compact_query_for_persistence(&mut self) {
        self.query = compact_skill_discovery_query_for_persistence(&self.query);
        if let Some(approval_id) = self.mesh_approval_id.as_ref() {
            if approval_id.starts_with("skill-mesh-approval:") {
                self.mesh_approval_id = Some(format!("skill-mesh-approval:{}", self.query));
            }
        }
    }

    pub fn is_discovery_pending(&self) -> bool {
        self.discovery_pending || self.confidence_tier.eq_ignore_ascii_case("pending")
    }

    pub fn effective_mesh_next_step(&self) -> crate::agent::skill_mesh::types::SkillMeshNextStep {
        if self.is_discovery_pending() {
            return crate::agent::skill_mesh::types::SkillMeshNextStep::JustifySkillSkip;
        }
        self.mesh_next_step
            .unwrap_or_else(|| self.legacy_mesh_next_step())
    }

    pub fn requires_skill_read_before_progress(&self) -> bool {
        !self.is_discovery_pending()
            && matches!(
                self.effective_mesh_next_step(),
                crate::agent::skill_mesh::types::SkillMeshNextStep::ReadSkill
            )
    }

    pub fn has_advisory_skill_read(&self) -> bool {
        !self.is_discovery_pending()
            && matches!(
                self.effective_mesh_next_step(),
                crate::agent::skill_mesh::types::SkillMeshNextStep::ChooseOrBypass
            )
    }

    fn legacy_mesh_next_step(&self) -> crate::agent::skill_mesh::types::SkillMeshNextStep {
        if self.confidence_tier.eq_ignore_ascii_case("strong") {
            crate::agent::skill_mesh::types::SkillMeshNextStep::ReadSkill
        } else if self
            .recommended_action
            .trim()
            .starts_with(crate::agent::skill_mesh::types::SkillMeshNextStep::ReadSkill.as_str())
        {
            crate::agent::skill_mesh::types::SkillMeshNextStep::ChooseOrBypass
        } else {
            crate::agent::skill_mesh::types::SkillMeshNextStep::JustifySkillSkip
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Stable unique ID for persistence and deletion. Generated on creation.
    #[serde(default = "generate_message_id")]
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_blocks: Vec<AgentContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weles_review: Option<WelesReviewMeta>,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_transport: Option<ApiTransport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_message: Option<CompletionUpstreamMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_final_result: Option<CompletionProviderFinalResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_agent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(default)]
    pub message_kind: AgentMessageKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compaction_strategy: Option<CompactionStrategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compaction_payload: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offloaded_payload_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_output_preview_path: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structural_refs: Vec<String>,
    #[serde(default)]
    pub pinned_for_compaction: bool,
    pub timestamp: u64,
}

pub fn generate_message_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentContentBlock {
    Text {
        text: String,
    },
    Image {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        data_url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
    Audio {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        data_url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
}

impl AgentContentBlock {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }
}

pub fn content_blocks_text_projection(blocks: &[AgentContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|block| match block {
            AgentContentBlock::Text { text } => Some(text.trim().to_string()),
            AgentContentBlock::Image { mime_type, .. } => Some(format!(
                "[Image attachment{}]",
                mime_type
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .map(|value| format!(": {value}"))
                    .unwrap_or_default()
            )),
            AgentContentBlock::Audio { mime_type, .. } => Some(format!(
                "[Audio attachment{}]",
                mime_type
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .map(|value| format!(": {value}"))
                    .unwrap_or_default()
            )),
        })
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

impl AgentMessage {
    pub fn user(content: impl Into<String>, now: u64) -> Self {
        Self {
            id: generate_message_id(),
            role: MessageRole::User,
            content: content.into(),
            content_blocks: Vec::new(),
            tool_calls: None,
            tool_call_id: None,
            tool_name: None,
            tool_arguments: None,
            tool_status: None,
            weles_review: None,
            input_tokens: 0,
            output_tokens: 0,
            cost: None,
            provider: None,
            model: None,
            api_transport: None,
            response_id: None,
            upstream_message: None,
            provider_final_result: None,
            author_agent_id: None,
            author_agent_name: None,
            reasoning: None,
            message_kind: AgentMessageKind::Normal,
            compaction_strategy: None,
            compaction_payload: None,
            offloaded_payload_id: None,
            tool_output_preview_path: None,
            structural_refs: Vec::new(),
            pinned_for_compaction: false,
            timestamp: now,
        }
    }

    pub fn user_with_blocks(
        content: impl Into<String>,
        content_blocks: Vec<AgentContentBlock>,
        now: u64,
    ) -> Self {
        let mut message = Self::user(content, now);
        message.content_blocks = content_blocks;
        if message.content.trim().is_empty() && !message.content_blocks.is_empty() {
            message.content = content_blocks_text_projection(&message.content_blocks);
        }
        message
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentMessageKind {
    #[default]
    Normal,
    CompactionArtifact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: ToolFunction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weles_review: Option<WelesReviewMeta>,
}

impl ToolCall {
    pub fn with_default_weles_review(id: String, function: ToolFunction) -> Self {
        Self {
            id,
            function,
            weles_review: Some(WelesReviewMeta {
                weles_reviewed: false,
                verdict: WelesVerdict::Allow,
                reasons: vec!["governance_not_run".to_string()],
                audit_id: None,
                security_override_mode: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub name: String,
    pub content: String,
    pub is_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weles_review: Option<WelesReviewMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_approval: Option<ToolPendingApproval>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredVisibleThreadContinuation {
    pub agent_id: String,
    pub task_id: Option<String>,
    pub preferred_session_hint: Option<String>,
    pub llm_user_content: String,
    pub queued_at_ms: u64,
    pub force_compaction: bool,
    pub rerun_participant_observers_after_turn: bool,
    pub internal_delegate_sender: Option<String>,
    pub internal_delegate_message: Option<String>,
}

impl DeferredVisibleThreadContinuation {
    pub(crate) fn same_request_as(&self, other: &Self) -> bool {
        self.agent_id == other.agent_id
            && self.task_id == other.task_id
            && self.preferred_session_hint == other.preferred_session_hint
            && self.llm_user_content == other.llm_user_content
            && self.force_compaction == other.force_compaction
            && self.rerun_participant_observers_after_turn
                == other.rerun_participant_observers_after_turn
            && self.internal_delegate_sender == other.internal_delegate_sender
            && self.internal_delegate_message == other.internal_delegate_message
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPendingApproval {
    pub approval_id: String,
    pub execution_id: String,
    pub command: String,
    pub rationale: String,
    pub risk_level: String,
    pub blast_radius: String,
    pub reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunctionDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
