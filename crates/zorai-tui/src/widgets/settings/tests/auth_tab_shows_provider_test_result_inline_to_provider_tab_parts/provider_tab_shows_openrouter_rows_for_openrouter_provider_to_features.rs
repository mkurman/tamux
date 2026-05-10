use crate::state::config::{ConfigAction, ConfigState};
use crate::state::modal::ModalState;
use crate::state::settings::{
    SettingsAction, SettingsState, SettingsTab,
};
use crate::state::subagents::SubAgentsState;
use crate::theme::ThemeTokens;
use crate::widgets::settings::render_tab_content;
use crate::widgets::settings::{
    render_agent_tab,
    render_concierge_tab, render_provider_tab,
};
use zorai_shared::providers::*;
#[test]
fn provider_tab_shows_openrouter_rows_for_openrouter_provider() {
    let mut settings = SettingsState::new();
    settings.reduce(crate::state::settings::SettingsAction::SwitchTab(
        SettingsTab::Provider,
    ));
    let mut config = ConfigState::new();
    config.provider = PROVIDER_ID_OPENROUTER.to_string();
    config.openrouter_provider_order = "anthropic".to_string();
    config.openrouter_provider_ignore = "deepinfra".to_string();

    let lines = render_provider_tab(&settings, &config, &ThemeTokens::default());
    let text = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("OR Prefer"));
    assert!(text.contains("anthropic"));
    assert!(text.contains("OR Exclude"));
    assert!(text.contains("deepinfra"));
    assert!(text.contains("OR Fallbacks"));
    assert!(text.contains("OR Cache"));
}

#[test]
fn provider_tab_mentions_swarog() {
    let mut settings = SettingsState::new();
    settings.reduce(crate::state::settings::SettingsAction::SwitchTab(
        SettingsTab::Provider,
    ));
    let config = ConfigState::new();

    let lines = render_provider_tab(&settings, &config, &ThemeTokens::default());
    let text = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Svarog"));
}

#[test]
fn agent_tab_uses_swarog_label() {
    let mut settings = SettingsState::new();
    settings.reduce(crate::state::settings::SettingsAction::SwitchTab(
        SettingsTab::Agent,
    ));
    let config = ConfigState::new();

    let lines = render_agent_tab(&settings, &config, &ThemeTokens::default());
    let text = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Svarog"));
}

#[test]
fn agent_tab_includes_provider_controls() {
    let mut settings = SettingsState::new();
    settings.reduce(crate::state::settings::SettingsAction::SwitchTab(
        SettingsTab::Agent,
    ));
    let config = ConfigState::new();

    let lines = render_agent_tab(&settings, &config, &ThemeTokens::default());
    let text = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Svarog Provider"));
    assert!(text.contains("Provider"));
    assert!(text.contains("System Prompt"));
}

#[test]
fn agent_tab_shows_fixed_swarog_name() {
    let mut settings = SettingsState::new();
    settings.reduce(crate::state::settings::SettingsAction::SwitchTab(
        SettingsTab::Agent,
    ));
    let config = ConfigState::new();

    let lines = render_agent_tab(&settings, &config, &ThemeTokens::default());
    let text = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Svarog"));
    assert!(!text.contains("Agent Name"));
    assert!(!text.contains("Svarog Name [Enter: edit]"));
}

#[test]
fn rarog_tab_uses_rarog_label() {
    let mut settings = SettingsState::new();
    settings.reduce(crate::state::settings::SettingsAction::SwitchTab(
        SettingsTab::Concierge,
    ));
    let concierge = crate::state::concierge::ConciergeState::new();

    let lines = render_concierge_tab(&settings, &concierge, &ThemeTokens::default());
    let text = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Rarog"));
    assert!(!text.contains("Concierge"));
}

#[test]
fn concierge_tab_renders_reasoning_effort_field() {
    let mut settings = SettingsState::new();
    settings.reduce(crate::state::settings::SettingsAction::SwitchTab(
        SettingsTab::Concierge,
    ));
    let mut concierge = crate::state::concierge::ConciergeState::new();
    concierge.reasoning_effort = Some("xhigh".to_string());

    let lines = render_concierge_tab(&settings, &concierge, &ThemeTokens::default());
    let text = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Reasoning"));
    assert!(text.contains("xhigh"));
}

#[test]
fn protected_weles_row_hides_delete_and_disable_actions() {
    let mut settings = SettingsState::new();
    settings.reduce(crate::state::settings::SettingsAction::SwitchTab(
        SettingsTab::SubAgents,
    ));
    let config = ConfigState::new();
    let modal = ModalState::new();
    let auth = crate::state::auth::AuthState::new();
    let mut subagents = SubAgentsState::new();
    subagents
        .entries
        .push(crate::state::subagents::SubAgentEntry {
            id: "weles_builtin".to_string(),
            name: "WELES".to_string(),
            provider: PROVIDER_ID_OPENAI.to_string(),
            model: "gpt-5.4-mini".to_string(),
            role: Some("testing".to_string()),
            enabled: true,
            builtin: true,
            immutable_identity: true,
            disable_allowed: false,
            delete_allowed: false,
            protected_reason: Some("Daemon-owned governance agent".to_string()),
            reasoning_effort: Some("medium".to_string()),
            openrouter_provider_order: String::new(),
            openrouter_provider_ignore: String::new(),
            openrouter_allow_fallbacks: true,
            raw_json: Some(serde_json::json!({ "id": "weles_builtin" })),
        });
    let concierge = crate::state::concierge::ConciergeState::new();
    let tier = crate::state::tier::TierState::from_tier("power_user");
    let plugin_settings = crate::state::settings::PluginSettingsState::new();

    let lines = render_tab_content(
        100,
        &settings,
        &config,
        &modal,
        &auth,
        &subagents,
        &concierge,
        &tier,
        &plugin_settings,
        &ThemeTokens::default(),
    );
    let text = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        text.contains("[built-in]"),
        "expected protected marker, got: {text}"
    );
    assert!(
        text.contains("[Locked]"),
        "expected locked action label, got: {text}"
    );
    assert!(
        text.contains("[Protected]"),
        "expected protected action label, got: {text}"
    );
    assert!(
        !text.contains("[Delete]"),
        "protected WELES row should hide delete action: {text}"
    );
    assert!(
        !text.contains("[Disable]"),
        "protected WELES row should hide disable action: {text}"
    );
}

#[test]
fn subagent_editor_renders_reasoning_effort_field() {
    let mut settings = SettingsState::new();
    settings.reduce(crate::state::settings::SettingsAction::SwitchTab(
        SettingsTab::SubAgents,
    ));
    let config = ConfigState::new();
    let modal = ModalState::new();
    let auth = crate::state::auth::AuthState::new();
    let mut subagents = SubAgentsState::new();
    let mut editor = crate::state::subagents::SubAgentEditorState::new(
        Some("weles_builtin".to_string()),
        1,
        PROVIDER_ID_OPENAI.to_string(),
        "gpt-5.4-mini".to_string(),
    );
    editor.name = "WELES".to_string();
    editor.reasoning_effort = Some("medium".to_string());
    editor.field = crate::state::subagents::SubAgentEditorField::ReasoningEffort;
    subagents.editor = Some(editor);
    let concierge = crate::state::concierge::ConciergeState::new();
    let tier = crate::state::tier::TierState::from_tier("power_user");
    let plugin_settings = crate::state::settings::PluginSettingsState::new();

    let lines = render_tab_content(
        100,
        &settings,
        &config,
        &modal,
        &auth,
        &subagents,
        &concierge,
        &tier,
        &plugin_settings,
        &ThemeTokens::default(),
    );
    let text = lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Reasoning"));
    assert!(text.contains("medium"));
}

use serde_json::json;

fn make_config_with_audio() -> ConfigState {
    let mut config = ConfigState::new();
    let raw = json!({
        "provider": "openai",
        "model": "gpt-4o",
        "extra": {
            "audio_stt_enabled": true,
            "audio_stt_provider": "openai",
            "audio_stt_model": "whisper-1",
            "audio_tts_enabled": true,
            "audio_tts_provider": "openai",
            "audio_tts_model": "tts-1",
            "audio_tts_voice": "alloy"
        }
    });
    config.reduce(ConfigAction::ConfigRawReceived(raw));
    config
}

fn make_config_without_audio() -> ConfigState {
    let mut config = ConfigState::new();
    let raw = json!({
        "provider": "openai",
        "model": "gpt-4o",
        "extra": {}
    });
    config.reduce(ConfigAction::ConfigRawReceived(raw));
    config
}

#[test]
fn features_tab_includes_audio_section_when_enabled() {
    let mut settings = SettingsState::new();
    settings.reduce(SettingsAction::SwitchTab(SettingsTab::Features));
    let config = make_config_with_audio();
    let tier = crate::state::tier::TierState::from_tier("base");
    let plugin_settings = crate::state::settings::PluginSettingsState::new();
    let modal = crate::state::modal::ModalState::new();
    let auth = crate::state::auth::AuthState::new();
    let subagents = crate::state::subagents::SubAgentsState::new();
    let concierge = crate::state::concierge::ConciergeState::new();

    let lines = render_tab_content(
        80,
        &settings,
        &config,
        &modal,
        &auth,
        &subagents,
        &concierge,
        &tier,
        &plugin_settings,
        &ThemeTokens::default(),
    );

    let audio_header = lines
        .iter()
        .find(|l| l.spans.iter().any(|s| s.content.contains("Audio")));
    assert!(
        audio_header.is_some(),
        "Audio section header should appear in Features tab"
    );
}

#[test]
fn features_tab_shows_stt_provider_when_configured() {
    let mut settings = SettingsState::new();
    settings.reduce(SettingsAction::SwitchTab(SettingsTab::Features));
    let config = make_config_with_audio();
    let tier = crate::state::tier::TierState::from_tier("base");
    let plugin_settings = crate::state::settings::PluginSettingsState::new();
    let modal = crate::state::modal::ModalState::new();
    let auth = crate::state::auth::AuthState::new();
    let subagents = crate::state::subagents::SubAgentsState::new();
    let concierge = crate::state::concierge::ConciergeState::new();

    let lines = render_tab_content(
        80,
        &settings,
        &config,
        &modal,
        &auth,
        &subagents,
        &concierge,
        &tier,
        &plugin_settings,
        &ThemeTokens::default(),
    );

    let stt_line = lines.iter().find(|l| {
        let text: String = l.spans.iter().map(|s| s.content.clone()).collect();
        text.contains("STT") && text.contains("openai")
    });
    assert!(stt_line.is_some(), "STT line with provider should appear");
}

#[test]
fn features_tab_shows_tts_voice_when_configured() {
    let mut settings = SettingsState::new();
    settings.reduce(SettingsAction::SwitchTab(SettingsTab::Features));
    let config = make_config_with_audio();
    let tier = crate::state::tier::TierState::from_tier("base");
    let plugin_settings = crate::state::settings::PluginSettingsState::new();
    let modal = crate::state::modal::ModalState::new();
    let auth = crate::state::auth::AuthState::new();
    let subagents = crate::state::subagents::SubAgentsState::new();
    let concierge = crate::state::concierge::ConciergeState::new();

    let lines = render_tab_content(
        80,
        &settings,
        &config,
        &modal,
        &auth,
        &subagents,
        &concierge,
        &tier,
        &plugin_settings,
        &ThemeTokens::default(),
    );

    let tts_line = lines.iter().find(|l| {
        let text: String = l.spans.iter().map(|s| s.content.clone()).collect();
        text.contains("TTS") && text.contains("alloy")
    });
    assert!(tts_line.is_some(), "TTS line with voice should appear");
}

#[test]
fn features_tab_shows_hotkey_hints() {
    let mut settings = SettingsState::new();
    settings.reduce(SettingsAction::SwitchTab(SettingsTab::Features));
    let config = make_config_with_audio();
    let tier = crate::state::tier::TierState::from_tier("base");
    let plugin_settings = crate::state::settings::PluginSettingsState::new();
    let modal = crate::state::modal::ModalState::new();
    let auth = crate::state::auth::AuthState::new();
    let subagents = crate::state::subagents::SubAgentsState::new();
    let concierge = crate::state::concierge::ConciergeState::new();

    let lines = render_tab_content(
        80,
        &settings,
        &config,
        &modal,
        &auth,
        &subagents,
        &concierge,
        &tier,
        &plugin_settings,
        &ThemeTokens::default(),
    );

    let hint_line = lines.iter().find(|l| {
        l.spans
            .iter()
            .any(|s| s.content.contains("Ctrl+L") || s.content.contains("Ctrl+P"))
    });
    assert!(hint_line.is_some(), "Hotkey hints should appear");
}

#[test]
fn features_tab_audio_defaults_when_missing() {
    let mut settings = SettingsState::new();
    settings.reduce(SettingsAction::SwitchTab(SettingsTab::Features));
    let config = make_config_without_audio();
    let tier = crate::state::tier::TierState::from_tier("base");
    let plugin_settings = crate::state::settings::PluginSettingsState::new();
    let modal = crate::state::modal::ModalState::new();
    let auth = crate::state::auth::AuthState::new();
    let subagents = crate::state::subagents::SubAgentsState::new();
    let concierge = crate::state::concierge::ConciergeState::new();

    let lines = render_tab_content(
        80,
        &settings,
        &config,
        &modal,
        &auth,
        &subagents,
        &concierge,
        &tier,
        &plugin_settings,
        &ThemeTokens::default(),
    );

    let stt_line = lines
        .iter()
        .find(|l| l.spans.iter().any(|s| s.content.contains("STT")));
    assert!(
        stt_line.is_some(),
        "STT line should appear even when disabled"
    );

    let content: String = stt_line
        .unwrap()
        .spans
        .iter()
        .map(|s| s.content.clone())
        .collect();
    assert!(
        content.contains("off") || content.contains("disabled") || !content.contains("openai"),
        "STT should show disabled/default state"
    );
}
