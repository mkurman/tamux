use super::flow::{
    parse_fetch_models_terminal_response, parse_set_config_item_response,
    setup_probe_from_config_json, SetupProbe,
};
use super::*;

#[test]
fn test_select_list_wraps_index() {
    let len = 4usize;
    let mut idx = 0usize;
    if idx == 0 {
        idx = len.saturating_sub(1);
    } else {
        idx -= 1;
    }
    assert_eq!(idx, 3);

    idx = 3;
    idx += 1;
    if idx >= len {
        idx = 0;
    }
    assert_eq!(idx, 0);

    idx = 1;
    idx += 1;
    if idx >= len {
        idx = 0;
    }
    assert_eq!(idx, 2);
}

#[test]
fn test_is_local_provider() {
    assert!(is_local_provider("ollama"));
    assert!(is_local_provider("lmstudio"));
    assert!(!is_local_provider("anthropic"));
    assert!(!is_local_provider("openai"));
}

#[test]
fn test_security_default_for_tier() {
    assert_eq!(default_security_index("newcomer"), 0);
    assert_eq!(default_security_index("familiar"), 1);
    assert_eq!(default_security_index("power_user"), 2);
    assert_eq!(default_security_index("expert"), 2);
    assert_eq!(default_security_index("unknown"), 1);
}

#[test]
fn test_tier_shows_optional_steps() {
    assert!(!tier_shows_step("newcomer", "model"));
    assert!(!tier_shows_step("newcomer", "data_dir"));
    assert!(!tier_shows_step("newcomer", "advanced_agents"));
    assert!(tier_shows_step("familiar", "model"));
    assert!(tier_shows_step("familiar", "data_dir"));
    assert!(tier_shows_step("familiar", "advanced_agents"));
    assert!(tier_shows_step("power_user", "model"));
    assert!(tier_shows_step("power_user", "advanced_agents"));
    assert!(tier_shows_step("expert", "model"));
    assert!(tier_shows_step("expert", "advanced_agents"));
}

#[test]
fn test_security_level_from_index() {
    assert_eq!(
        security_level_from_index(0),
        ("highest", "Approve risky actions")
    );
    assert_eq!(
        security_level_from_index(1),
        ("moderate", "Approve risky actions")
    );
    assert_eq!(
        security_level_from_index(2),
        ("lowest", "Approve destructive only")
    );
    assert_eq!(
        security_level_from_index(3),
        ("yolo", "Minimize interruptions")
    );
    assert_eq!(
        security_level_from_index(99),
        ("moderate", "Approve risky actions")
    );
}

#[test]
fn test_post_setup_action_from_index() {
    assert_eq!(post_setup_action_from_index(0), PostSetupAction::LaunchTui);
    assert_eq!(
        post_setup_action_from_index(1),
        PostSetupAction::LaunchElectron
    );
    assert_eq!(post_setup_action_from_index(2), PostSetupAction::NotNow);
}

#[test]
fn test_post_setup_choices_include_not_now() {
    let choices = post_setup_choices();
    assert_eq!(choices.len(), 3);
    assert_eq!(choices[0].0, "TUI");
    assert_eq!(choices[1].0, "Electron");
    assert_eq!(choices[2].0, "Not now");
}

#[test]
fn submit_key_recognizes_control_m_as_enter() {
    assert!(terminal_ui::is_submit_key(
        KeyCode::Char('m'),
        KeyModifiers::CONTROL
    ));
}

#[test]
fn submit_key_recognizes_standard_enter() {
    assert!(terminal_ui::is_submit_key(KeyCode::Enter, KeyModifiers::NONE));
}

#[test]
fn submit_key_does_not_treat_plain_m_as_enter() {
    assert!(!terminal_ui::is_submit_key(
        KeyCode::Char('m'),
        KeyModifiers::NONE
    ));
}

#[test]
fn raw_mode_guard_requests_only_disambiguate_enhancement_flag() {
    // Only DISAMBIGUATE_ESCAPE_CODES is requested so that arrow-key sequences
    // remain unambiguous while Enter continues to arrive as a plain \r / \n.
    // The more aggressive flags caused Enter to be re-encoded as a CSI-u
    // escape sequence that some terminals (e.g. Yakuake/Konsole) emit in a
    // shape crossterm does not reliably map to KeyCode::Enter — see issue #53.
    let flags = wizard_keyboard_enhancement_flags();

    assert!(flags.contains(crossterm::event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES));
    assert!(!flags.contains(crossterm::event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES));
    assert!(!flags.contains(crossterm::event::KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS));
    assert!(!flags.contains(
        crossterm::event::KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
    ));
}

#[test]
fn submit_key_recognizes_carriage_return_char() {
    assert!(terminal_ui::is_submit_key(
        KeyCode::Char('\r'),
        KeyModifiers::NONE
    ));
}

#[test]
fn submit_key_recognizes_newline_char() {
    assert!(terminal_ui::is_submit_key(
        KeyCode::Char('\n'),
        KeyModifiers::NONE
    ));
}

#[test]
fn submit_key_recognizes_control_j_as_enter() {
    assert!(terminal_ui::is_submit_key(
        KeyCode::Char('j'),
        KeyModifiers::CONTROL
    ));
}

#[test]
fn submit_key_does_not_treat_plain_j_as_enter() {
    assert!(!terminal_ui::is_submit_key(
        KeyCode::Char('j'),
        KeyModifiers::NONE
    ));
}

#[test]
fn test_gateway_choice_items_include_whatsapp_and_skip() {
    let items = gateway_choice_items();
    assert_eq!(items.len(), 5);
    assert_eq!(items[3], ("WhatsApp", "whatsapp"));
    assert_eq!(items[4], ("Skip", ""));
}

#[test]
fn test_whatsapp_timeout_choice_mapping() {
    let choices = whatsapp_timeout_choices();
    assert_eq!(choices.len(), 2);
    assert!(whatsapp_timeout_retry_selected(0));
    assert!(!whatsapp_timeout_retry_selected(1));
}

#[test]
fn whatsapp_setup_accepts_multiline_or_csv_contacts() {
    let parsed = parse_whatsapp_setup_allowlist("+48 123 456 789, 15551230000\n+44 20 7946 0958");

    assert_eq!(
        parsed,
        Some(vec![
            "48123456789".to_string(),
            "15551230000".to_string(),
            "442079460958".to_string(),
        ])
    );
}

#[test]
fn whatsapp_setup_rejects_empty_allowlist() {
    assert_eq!(
        parse_whatsapp_setup_allowlist("\n , invalid , device "),
        None
    );
    assert_eq!(parse_whatsapp_setup_allowlist("   \n  "), None);
}

#[test]
fn whatsapp_setup_cancellation_paths_stop_without_retry() {
    assert_eq!(
        resolve_whatsapp_allowlist_prompt(WhatsAppAllowlistPromptOutcome::Cancelled),
        WhatsAppAllowlistPromptResolution::Cancel
    );
    assert_eq!(
        resolve_whatsapp_allowlist_prompt(WhatsAppAllowlistPromptOutcome::EndOfInput),
        WhatsAppAllowlistPromptResolution::Cancel
    );
}

#[test]
fn whatsapp_setup_invalid_submission_requests_retry() {
    assert_eq!(
        resolve_whatsapp_allowlist_prompt(WhatsAppAllowlistPromptOutcome::Submitted(
            "\n , invalid , device ".to_string()
        )),
        WhatsAppAllowlistPromptResolution::Retry(
            "Enter at least one valid WhatsApp phone number before linking."
        )
    );
}

#[test]
fn whatsapp_setup_persists_allowlist_before_gateway_enable() {
    let writes = whatsapp_gateway_config_writes("+48 123 456 789").expect("valid writes");

    assert_eq!(writes.len(), 2);
    assert_eq!(writes[0].key_path, "/gateway/whatsapp_allowed_contacts");
    assert_eq!(writes[0].value_json, "[\"48123456789\"]");
    assert_eq!(writes[1].key_path, "/gateway/enabled");
    assert_eq!(writes[1].value_json, "true");
}

#[test]
fn wizard_declares_async_command_capability_on_connect() {
    let messages = wizard_startup_messages();
    assert_eq!(messages.len(), 1);
    match &messages[0] {
        ClientMessage::AgentDeclareAsyncCommandCapability { capability } => {
            assert_eq!(capability.version, 1);
            assert!(capability.supports_operation_acceptance);
        }
        other => panic!("expected async command capability declaration, got {other:?}"),
    }
}

#[test]
fn wizard_ignores_async_command_capability_ack_messages() {
    let ignored = should_ignore_wizard_message(&DaemonMessage::AgentAsyncCommandCapabilityAck {
        capability: amux_protocol::AsyncCommandCapability {
            version: 1,
            supports_operation_acceptance: true,
        },
    });
    assert!(ignored, "handshake ack should not surface to setup flows");

    let forwarded = should_ignore_wizard_message(&DaemonMessage::AgentProviderAuthStates {
        states_json: "[]".to_string(),
    });
    assert!(
        !forwarded,
        "real daemon payloads must still reach setup flows"
    );
}

#[test]
fn provider_validation_terminal_response_ignores_operation_acceptance() {
    let response = parse_provider_validation_terminal_response(DaemonMessage::OperationAccepted {
        operation_id: "op-provider-validation-1".to_string(),
        kind: "provider_validation".to_string(),
        dedup: None,
        revision: 1,
    });
    assert!(response.is_none(), "operation acceptance is not terminal");
}

#[test]
fn provider_validation_terminal_response_extracts_result_payload() {
    let response =
        parse_provider_validation_terminal_response(DaemonMessage::AgentProviderValidation {
            operation_id: Some("op-provider-validation-1".to_string()),
            provider_id: "openai".to_string(),
            valid: false,
            error: Some("bad key".to_string()),
            models_json: None,
        })
        .expect("terminal response")
        .expect("successful parse");

    assert_eq!(response, (false, Some("bad key".to_string())));
}

#[test]
fn fetch_models_terminal_response_ignores_operation_acceptance() {
    let response = parse_fetch_models_terminal_response(DaemonMessage::OperationAccepted {
        operation_id: "op-fetch-models-1".to_string(),
        kind: "fetch_models".to_string(),
        dedup: None,
        revision: 1,
    });
    assert!(response.is_none(), "operation acceptance is not terminal");
}

#[test]
fn fetch_models_terminal_response_extracts_models_json() {
    let response = parse_fetch_models_terminal_response(DaemonMessage::AgentModelsResponse {
        operation_id: Some("op-fetch-models-1".to_string()),
        models_json: "[\"gpt-5.4\"]".to_string(),
    })
    .expect("terminal response")
    .expect("successful parse");

    assert_eq!(response, "[\"gpt-5.4\"]".to_string());
}

#[test]
fn config_set_response_completes_on_operation_acceptance() {
    let response = parse_set_config_item_response(DaemonMessage::OperationAccepted {
        operation_id: "op-config-set-1".to_string(),
        kind: "config_set_item".to_string(),
        dedup: None,
        revision: 1,
    });
    assert!(
        response.is_some(),
        "operation acceptance should complete config writes"
    );
    response
        .expect("terminal response")
        .expect("successful config-set acknowledgement");
}

#[test]
fn setup_probe_marks_ready_when_provider_is_persisted() {
    assert_eq!(
        setup_probe_from_config_json(r#"{"provider":"openai"}"#),
        SetupProbe::Ready
    );
}

#[test]
fn setup_probe_requires_setup_without_provider() {
    assert_eq!(
        setup_probe_from_config_json(r#"{"provider":""}"#),
        SetupProbe::NeedsSetup
    );
    assert_eq!(
        setup_probe_from_config_json(r#"{"providers":{"openai":{}}}"#),
        SetupProbe::NeedsSetup
    );
}

#[test]
fn setup_probe_treats_invalid_config_as_needing_setup() {
    assert_eq!(
        setup_probe_from_config_json("{not-json"),
        SetupProbe::NeedsSetup
    );
}
