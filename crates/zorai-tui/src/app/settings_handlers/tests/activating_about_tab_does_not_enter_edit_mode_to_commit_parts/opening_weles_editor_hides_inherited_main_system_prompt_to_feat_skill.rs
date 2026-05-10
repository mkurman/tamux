use super::super::make_model;
use super::*;
use crate::app::TuiModel;
use crate::state::settings::SettingsTab;
use crossterm::event::{KeyCode, KeyModifiers};
#[test]
fn opening_weles_editor_hides_inherited_main_system_prompt() {
    let (mut model, _daemon_rx) = make_model();
    model.config.agent_config_raw = Some(serde_json::json!({
        "system_prompt": "Main Svarog prompt",
        "builtin_sub_agents": {
            "weles": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "gpt-5.4-mini"
            }
        }
    }));
    model.subagents.entries = vec![crate::state::SubAgentEntry {
        id: "weles_builtin".to_string(),
        name: "WELES".to_string(),
        provider: PROVIDER_ID_OPENAI.to_string(),
        model: "gpt-5.4-mini".to_string(),
        role: Some("governance".to_string()),
        enabled: true,
        builtin: true,
        immutable_identity: true,
        disable_allowed: false,
        delete_allowed: false,
        protected_reason: Some("Daemon-owned WELES registry entry".to_string()),
        reasoning_effort: Some("medium".to_string()),
        openrouter_provider_order: String::new(),
        openrouter_provider_ignore: String::new(),
        openrouter_allow_fallbacks: true,
        raw_json: Some(serde_json::json!({
            "id": "weles_builtin",
            "name": "WELES",
            "provider": PROVIDER_ID_OPENAI,
            "model": "gpt-5.4-mini",
            "role": "governance",
            "system_prompt": "Main Svarog prompt",
            "enabled": true,
            "builtin": true,
            "immutable_identity": true,
            "disable_allowed": false,
            "delete_allowed": false,
            "protected_reason": "Daemon-owned WELES registry entry",
            "reasoning_effort": "medium",
            "created_at": 0
        })),
    }];

    model.open_subagent_editor_existing();

    assert_eq!(
        model
            .subagents
            .editor
            .as_ref()
            .map(|editor| editor.system_prompt.as_str()),
        Some("")
    );
}

pub fn focus_settings_field(model: &mut TuiModel, tab: SettingsTab, field_name: &str) {
    model.settings.reduce(SettingsAction::SwitchTab(tab));
    let count = model.settings.field_count_with_config(&model.config);
    for _ in 0..count {
        if model.settings.current_field_name_with_config(&model.config) == field_name {
            return;
        }
        model.settings.reduce(SettingsAction::NavigateField(1));
    }
    panic!("field {field_name} not found in {:?}", tab);
}

fn raw_json_string(value: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for part in path {
        current = current.get(*part)?;
    }
    current.as_str().map(str::to_string)
}

fn raw_json_bool(value: &serde_json::Value, path: &[&str]) -> Option<bool> {
    let mut current = value;
    for part in path {
        current = current.get(*part)?;
    }
    current.as_bool()
}

#[test]
fn feature_toggle_fields_emit_expected_config_updates() {
    let cases = [
        (
            "feat_tier_override",
            "/tier/user_override",
            "\"familiar\"",
            Some((vec!["tier", "user_override"], "familiar")),
            None,
        ),
        (
            "feat_security_level",
            "/managed_security_level",
            "\"strict\"",
            Some((vec!["managed_security_level"], "strict")),
            None,
        ),
        (
            "feat_check_stale_todos",
            "/heartbeat/check_stale_todos",
            "false",
            None,
            Some((vec!["heartbeat", "check_stale_todos"], false)),
        ),
        (
            "feat_check_stuck_goals",
            "/heartbeat/check_stuck_goals",
            "false",
            None,
            Some((vec!["heartbeat", "check_stuck_goals"], false)),
        ),
        (
            "feat_check_unreplied_messages",
            "/heartbeat/check_unreplied_messages",
            "false",
            None,
            Some((vec!["heartbeat", "check_unreplied_messages"], false)),
        ),
        (
            "feat_check_repo_changes",
            "/heartbeat/check_repo_changes",
            "false",
            None,
            Some((vec!["heartbeat", "check_repo_changes"], false)),
        ),
        (
            "feat_consolidation_enabled",
            "/consolidation/enabled",
            "false",
            None,
            Some((vec!["consolidation", "enabled"], false)),
        ),
        (
            "feat_skill_recommendation_enabled",
            "/skill_recommendation/enabled",
            "false",
            None,
            Some((vec!["skill_recommendation", "enabled"], false)),
        ),
        (
            "feat_skill_background_community_search",
            "/skill_recommendation/background_community_search",
            "false",
            None,
            Some((
                vec!["skill_recommendation", "background_community_search"],
                false,
            )),
        ),
        (
            "feat_audio_stt_enabled",
            "/audio/stt/enabled",
            "true",
            None,
            Some((vec!["audio", "stt", "enabled"], true)),
        ),
        (
            "feat_audio_tts_enabled",
            "/audio/tts/enabled",
            "true",
            None,
            Some((vec!["audio", "tts", "enabled"], true)),
        ),
    ];

    for (field, expected_key_path, expected_value_json, expected_string, expected_bool) in cases {
        let (mut model, mut daemon_rx) = make_model();
        model.config.agent_config_raw = Some(serde_json::json!({}));
        model
            .modal
            .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
        focus_settings_field(&mut model, SettingsTab::Features, field);

        let quit = model.handle_key_modal(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
            modal::ModalKind::Settings,
        );
        assert!(!quit, "settings modal should remain open for {field}");

        match daemon_rx.try_recv() {
            Ok(DaemonCommand::SetConfigItem {
                key_path,
                value_json,
            }) => {
                assert_eq!(key_path, expected_key_path, "wrong key path for {field}");
                assert_eq!(
                    value_json, expected_value_json,
                    "wrong serialized value for {field}"
                );
            }
            other => panic!("expected SetConfigItem for {field}, got {other:?}"),
        }

        let raw = model
            .config
            .agent_config_raw
            .as_ref()
            .expect("feature toggles should keep raw config");
        if let Some((path, expected)) = expected_string {
            assert_eq!(raw_json_string(raw, &path), Some(expected.to_string()));
        }
        if let Some((path, expected)) = expected_bool {
            assert_eq!(raw_json_bool(raw, &path), Some(expected));
        }
    }
}

#[test]
fn feature_edit_fields_start_with_saved_values_and_submit_expected_updates() {
    let cases = [
        (
            "feat_heartbeat_cron",
            serde_json::json!({"heartbeat": {"cron": "*/30 * * * *"}}),
            "*/30 * * * *",
            "/heartbeat/cron",
            "\"*/30 * * * *\"",
        ),
        (
            "feat_heartbeat_quiet_start",
            serde_json::json!({"heartbeat": {"quiet_start": "21:30"}}),
            "21:30",
            "/heartbeat/quiet_start",
            "\"21:30\"",
        ),
        (
            "feat_heartbeat_quiet_end",
            serde_json::json!({"heartbeat": {"quiet_end": "06:30"}}),
            "06:30",
            "/heartbeat/quiet_end",
            "\"06:30\"",
        ),
        (
            "feat_decay_half_life_hours",
            serde_json::json!({"consolidation": {"decay_half_life_hours": 72.0}}),
            "72",
            "/consolidation/decay_half_life_hours",
            "72",
        ),
        (
            "feat_heuristic_promotion_threshold",
            serde_json::json!({"consolidation": {"heuristic_promotion_threshold": 9}}),
            "9",
            "/consolidation/heuristic_promotion_threshold",
            "9",
        ),
        (
            "feat_skill_community_preapprove_timeout_secs",
            serde_json::json!({"skill_recommendation": {"community_preapprove_timeout_secs": 45}}),
            "45",
            "/skill_recommendation/community_preapprove_timeout_secs",
            "45",
        ),
        (
            "feat_skill_suggest_global_enable_after_approvals",
            serde_json::json!({"skill_recommendation": {"suggest_global_enable_after_approvals": 6}}),
            "6",
            "/skill_recommendation/suggest_global_enable_after_approvals",
            "6",
        ),
    ];

    for (field, raw, expected_buffer, expected_key_path, expected_value_json) in cases {
        let (mut model, mut daemon_rx) = make_model();
        model.config.agent_config_raw = Some(raw);
        model
            .modal
            .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
        focus_settings_field(&mut model, SettingsTab::Features, field);

        let quit = model.handle_key_modal(
            KeyCode::Enter,
            KeyModifiers::NONE,
            modal::ModalKind::Settings,
        );
        assert!(
            !quit,
            "settings modal should stay open while starting edit for {field}"
        );
        assert_eq!(model.settings.editing_field(), Some(field));
        assert_eq!(model.settings.edit_buffer(), expected_buffer);

        let quit = model.handle_key_modal(
            KeyCode::Enter,
            KeyModifiers::NONE,
            modal::ModalKind::Settings,
        );
        assert!(
            !quit,
            "settings modal should stay open while committing edit for {field}"
        );
        assert_eq!(model.settings.editing_field(), None);

        match daemon_rx.try_recv() {
            Ok(DaemonCommand::SetConfigItem {
                key_path,
                value_json,
            }) => {
                assert_eq!(key_path, expected_key_path, "wrong key path for {field}");
                assert_eq!(
                    value_json, expected_value_json,
                    "wrong serialized value for {field}"
                );
            }
            other => panic!("expected SetConfigItem for {field}, got {other:?}"),
        }
    }
}

#[test]
fn feat_skill_recommendation_toggles_write_new_daemon_paths() {
    let cases = [
        (
            "feat_skill_recommendation_enabled",
            "/skill_recommendation/enabled",
            "false",
            vec!["skill_recommendation", "enabled"],
        ),
        (
            "feat_skill_background_community_search",
            "/skill_recommendation/background_community_search",
            "false",
            vec!["skill_recommendation", "background_community_search"],
        ),
    ];

    for (field, expected_key_path, expected_value_json, raw_path) in cases {
        let (mut model, mut daemon_rx) = make_model();
        model.config.agent_config_raw = Some(serde_json::json!({}));
        model
            .modal
            .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
        focus_settings_field(&mut model, SettingsTab::Features, field);

        let quit = model.handle_key_modal(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
            modal::ModalKind::Settings,
        );
        assert!(!quit, "settings modal should remain open for {field}");

        match daemon_rx.try_recv() {
            Ok(DaemonCommand::SetConfigItem {
                key_path,
                value_json,
            }) => {
                assert_eq!(key_path, expected_key_path, "wrong key path for {field}");
                assert_eq!(
                    value_json, expected_value_json,
                    "wrong serialized value for {field}"
                );
            }
            other => panic!("expected SetConfigItem for {field}, got {other:?}"),
        }

        let raw = model
            .config
            .agent_config_raw
            .as_ref()
            .expect("feature toggles should keep raw config");
        assert_eq!(raw_json_bool(raw, &raw_path), Some(false));
    }
}

#[test]
fn feat_skill_recommendation_numeric_fields_write_new_daemon_paths() {
    let cases = [
        (
            "feat_skill_community_preapprove_timeout_secs",
            serde_json::json!({"skill_recommendation": {"community_preapprove_timeout_secs": 45}}),
            "45",
            "/skill_recommendation/community_preapprove_timeout_secs",
            "45",
        ),
        (
            "feat_skill_suggest_global_enable_after_approvals",
            serde_json::json!({"skill_recommendation": {"suggest_global_enable_after_approvals": 6}}),
            "6",
            "/skill_recommendation/suggest_global_enable_after_approvals",
            "6",
        ),
    ];

    for (field, raw, expected_buffer, expected_key_path, expected_value_json) in cases {
        let (mut model, mut daemon_rx) = make_model();
        model.config.agent_config_raw = Some(raw);
        model
            .modal
            .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
        focus_settings_field(&mut model, SettingsTab::Features, field);

        let quit = model.handle_key_modal(
            KeyCode::Enter,
            KeyModifiers::NONE,
            modal::ModalKind::Settings,
        );
        assert!(
            !quit,
            "settings modal should stay open while starting edit for {field}"
        );
        assert_eq!(model.settings.editing_field(), Some(field));
        assert_eq!(model.settings.edit_buffer(), expected_buffer);

        let quit = model.handle_key_modal(
            KeyCode::Enter,
            KeyModifiers::NONE,
            modal::ModalKind::Settings,
        );
        assert!(
            !quit,
            "settings modal should stay open while committing edit for {field}"
        );
        assert_eq!(model.settings.editing_field(), None);

        match daemon_rx.try_recv() {
            Ok(DaemonCommand::SetConfigItem {
                key_path,
                value_json,
            }) => {
                assert_eq!(key_path, expected_key_path, "wrong key path for {field}");
                assert_eq!(
                    value_json, expected_value_json,
                    "wrong serialized value for {field}"
                );
            }
            other => panic!("expected SetConfigItem for {field}, got {other:?}"),
        }
    }
}
