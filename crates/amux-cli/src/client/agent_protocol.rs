use anyhow::Result;
use serde::Deserialize;

/// Gracefully deserialize context_messages. Malformed entries are dropped instead of failing the whole command.
fn deserialize_context_messages<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<amux_protocol::AgentDbMessage>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let raw: Option<Vec<serde_json::Value>> = Option::deserialize(deserializer)?;
    match raw {
        None => Ok(None),
        Some(arr) => {
            let parsed: Vec<amux_protocol::AgentDbMessage> = arr
                .into_iter()
                .filter_map(|v| serde_json::from_value(v).ok())
                .collect();
            if parsed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(parsed))
            }
        }
    }
}

/// Commands for the agent bridge (JSON over stdin).
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(super) enum AgentBridgeCommand {
    SendMessage {
        thread_id: Option<String>,
        content: String,
        session_id: Option<String>,
        #[serde(default, deserialize_with = "deserialize_context_messages")]
        context_messages: Option<Vec<amux_protocol::AgentDbMessage>>,
    },
    InternalDelegate {
        thread_id: Option<String>,
        target_agent_id: String,
        content: String,
        session_id: Option<String>,
    },
    ThreadParticipantCommand {
        thread_id: String,
        target_agent_id: String,
        action: String,
        instruction: Option<String>,
        session_id: Option<String>,
    },
    StopStream {
        thread_id: String,
    },
    ListThreads,
    GetThread {
        thread_id: String,
    },
    DeleteThread {
        thread_id: String,
    },
    PinThreadMessageForCompaction {
        thread_id: String,
        message_id: String,
    },
    UnpinThreadMessageForCompaction {
        thread_id: String,
        message_id: String,
    },
    AddTask {
        title: String,
        description: String,
        priority: Option<String>,
        command: Option<String>,
        session_id: Option<String>,
        scheduled_at: Option<u64>,
        #[serde(default)]
        dependencies: Vec<String>,
    },
    CancelTask {
        task_id: String,
    },
    ListTasks,
    ListRuns,
    GetRun {
        run_id: String,
    },
    StartGoalRun {
        goal: String,
        title: Option<String>,
        thread_id: Option<String>,
        session_id: Option<String>,
        priority: Option<String>,
        client_request_id: Option<String>,
        autonomy_level: Option<String>,
    },
    ListGoalRuns,
    GetGoalRun {
        goal_run_id: String,
    },
    ControlGoalRun {
        goal_run_id: String,
        action: String,
        step_index: Option<usize>,
    },
    ListTodos,
    GetTodos {
        thread_id: String,
    },
    GetWorkContext {
        thread_id: String,
    },
    GetGitDiff {
        repo_path: String,
        file_path: Option<String>,
    },
    GetFilePreview {
        path: String,
        max_bytes: Option<usize>,
    },
    GetConfig,
    SetConfigItem {
        key_path: String,
        value_json: String,
    },
    SetProviderModel {
        provider_id: String,
        model: String,
    },
    SetTargetAgentProviderModel {
        target_agent_id: String,
        provider_id: String,
        model: String,
    },
    HeartbeatGetItems,
    HeartbeatSetItems {
        items_json: String,
    },
    ResolveTaskApproval {
        approval_id: String,
        decision: String,
    },
    ValidateProvider {
        provider_id: String,
        base_url: String,
        api_key: String,
        auth_source: String,
    },
    GetProviderAuthStates,
    #[serde(rename = "openai-codex-auth-status")]
    GetOpenAICodexAuthStatus,
    #[serde(rename = "openai-codex-auth-login")]
    LoginOpenAICodex,
    #[serde(rename = "openai-codex-auth-logout")]
    LogoutOpenAICodex,
    LoginProvider {
        provider_id: String,
        api_key: String,
        #[serde(default)]
        base_url: String,
        #[serde(default)]
        auth_source: Option<String>,
    },
    LogoutProvider {
        provider_id: String,
    },
    SetSubAgent {
        sub_agent_json: String,
    },
    RemoveSubAgent {
        sub_agent_id: String,
    },
    ListSubAgents,
    GetConciergeConfig,
    SetConciergeConfig {
        config_json: String,
    },
    DismissConciergeWelcome,
    RequestConciergeWelcome,
    AuditDismiss {
        entry_id: String,
    },
    QueryAudits {
        #[serde(default)]
        action_types: Option<Vec<String>>,
        #[serde(default)]
        since: Option<u64>,
        #[serde(default)]
        limit: Option<usize>,
    },
    GetProvenanceReport {
        #[serde(default)]
        limit: Option<u32>,
    },
    GetMemoryProvenanceReport {
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        limit: Option<u32>,
    },
    ConfirmMemoryProvenanceEntry {
        entry_id: String,
    },
    RetractMemoryProvenanceEntry {
        entry_id: String,
    },
    GetCollaborationSessions {
        #[serde(default)]
        parent_task_id: Option<String>,
    },
    VoteOnCollaborationDisagreement {
        parent_task_id: String,
        disagreement_id: String,
        task_id: String,
        position: String,
        #[serde(default)]
        confidence: Option<f64>,
    },
    GetStatus,
    InspectPrompt {
        #[serde(default)]
        agent_id: Option<String>,
    },
    SetTierOverride {
        tier: Option<String>,
    },
    #[serde(rename = "plugin-list")]
    PluginList,
    #[serde(rename = "plugin-get")]
    PluginGetDetail {
        name: String,
    },
    #[serde(rename = "plugin-enable")]
    PluginEnableCmd {
        name: String,
    },
    #[serde(rename = "plugin-disable")]
    PluginDisableCmd {
        name: String,
    },
    #[serde(rename = "plugin-get-settings")]
    PluginGetSettings {
        name: String,
    },
    #[serde(rename = "plugin-update-settings")]
    PluginUpdateSettings {
        plugin_name: String,
        key: String,
        value: String,
        is_secret: bool,
    },
    #[serde(rename = "plugin-test-connection")]
    PluginTestConnection {
        name: String,
    },
    #[serde(rename = "plugin-oauth-start")]
    PluginOAuthStart {
        name: String,
    },
    WhatsAppLinkStart,
    WhatsAppLinkStop,
    WhatsAppLinkStatus,
    WhatsAppLinkSubscribe,
    WhatsAppLinkUnsubscribe,
    StartOperatorProfileSession {
        kind: String,
    },
    NextOperatorProfileQuestion {
        session_id: String,
    },
    SubmitOperatorProfileAnswer {
        session_id: String,
        question_id: String,
        answer_json: String,
    },
    SkipOperatorProfileQuestion {
        session_id: String,
        question_id: String,
        #[serde(default)]
        reason: Option<String>,
    },
    DeferOperatorProfileQuestion {
        session_id: String,
        question_id: String,
        #[serde(default)]
        defer_until_unix_ms: Option<u64>,
    },
    #[serde(rename = "answer-question")]
    AnswerQuestion {
        question_id: String,
        answer: String,
    },
    GetOperatorProfileSummary,
    SetOperatorProfileConsent {
        consent_key: String,
        granted: bool,
    },
    ExplainAction {
        action_id: String,
        #[serde(default)]
        step_index: Option<usize>,
    },
    StartDivergentSession {
        problem_statement: String,
        thread_id: String,
        #[serde(default)]
        goal_run_id: Option<String>,
        #[serde(default)]
        custom_framings_json: Option<String>,
    },
    GetDivergentSession {
        session_id: String,
    },
    ListGeneratedTools,
    RunGeneratedTool {
        tool_name: String,
        args_json: String,
    },
    PromoteGeneratedTool {
        tool_name: String,
    },
    ActivateGeneratedTool {
        tool_name: String,
    },
    RetireGeneratedTool {
        tool_name: String,
    },
    #[serde(rename = "agent-get-statistics")]
    GetStatistics {
        window: amux_protocol::AgentStatisticsWindow,
    },
    Shutdown,
}
