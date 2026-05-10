use super::whatsapp_modal_esc_sends_stop_and_closes_to_clicking_rendered_settings::*;
use crate::app::*;
use zorai_shared::providers::*;
#[test]
fn approval_center_keyboard_resolves_selected_approval() {
    let (mut model, mut daemon_rx) = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model
        .approval
        .reduce(crate::state::ApprovalAction::ApprovalRequired(
            crate::state::PendingApproval {
                approval_id: "approval-1".to_string(),
                task_id: "task-1".to_string(),
                task_title: Some("Task".to_string()),
                thread_id: Some("thread-1".to_string()),
                thread_title: Some("Thread".to_string()),
                workspace_id: Some(model.config.honcho_workspace_id.clone()),
                rationale: None,
                reasons: Vec::new(),
                command: "git push".to_string(),
                risk_level: crate::state::RiskLevel::High,
                blast_radius: "repo".to_string(),
                received_at: 1,
                seen_at: None,
            },
        ));
    model.toggle_approval_center();

    let quit = model.handle_key_modal(
        KeyCode::Char('a'),
        KeyModifiers::NONE,
        modal::ModalKind::ApprovalCenter,
    );

    assert!(!quit);
    assert!(model.approval.pending_approvals().is_empty());
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected approval rules refresh command"),
        DaemonCommand::ListTaskApprovalRules
    ));
    assert!(matches!(
        daemon_rx.try_recv().expect("expected approval resolution command"),
        DaemonCommand::ResolveTaskApproval {
            approval_id,
            decision
        } if approval_id == "approval-1" && decision == "allow_once"
    ));
}

#[test]
fn goal_approval_reject_from_overlay_opens_followup_prompt_without_resolving_yet() {
    let (mut model, mut daemon_rx) = make_model();
    seed_goal_approval_overlay(&mut model, "approval-1", "goal-1", "thread-1");

    let quit = model.handle_key_modal(
        KeyCode::Char('n'),
        KeyModifiers::NONE,
        modal::ModalKind::ApprovalOverlay,
    );

    assert!(!quit);
    assert_eq!(
        model.modal.top(),
        Some(modal::ModalKind::GoalApprovalRejectPrompt)
    );
    assert!(
        daemon_rx.try_recv().is_err(),
        "reject follow-up should not resolve approval until the operator chooses rewrite or stop"
    );
}

#[test]
fn goal_approval_rewrite_choice_rejects_and_prefills_guidance_input() {
    let (mut model, mut daemon_rx) = make_model();
    seed_goal_approval_overlay(&mut model, "approval-1", "goal-1", "thread-1");
    let _ = model.handle_key_modal(
        KeyCode::Char('n'),
        KeyModifiers::NONE,
        modal::ModalKind::ApprovalOverlay,
    );

    let quit = model.handle_key_modal(
        KeyCode::Char('r'),
        KeyModifiers::NONE,
        modal::ModalKind::GoalApprovalRejectPrompt,
    );

    assert!(!quit);
    assert!(matches!(
        daemon_rx.try_recv().expect("expected approval rejection command"),
        DaemonCommand::ResolveTaskApproval {
            approval_id,
            decision
        } if approval_id == "approval-1" && decision == "reject"
    ));
    assert!(
        daemon_rx.try_recv().is_err(),
        "rewrite path should not stop the goal"
    );
    assert_eq!(model.focus, FocusArea::Input);
    assert!(model
        .input
        .buffer()
        .contains("Rewrite the blocked goal step"));
    assert_eq!(model.modal.top(), None);
}

#[test]
fn goal_approval_stop_choice_rejects_and_stops_goal() {
    let (mut model, mut daemon_rx) = make_model();
    seed_goal_approval_overlay(&mut model, "approval-1", "goal-1", "thread-1");
    let _ = model.handle_key_modal(
        KeyCode::Char('n'),
        KeyModifiers::NONE,
        modal::ModalKind::ApprovalOverlay,
    );

    let quit = model.handle_key_modal(
        KeyCode::Char('s'),
        KeyModifiers::NONE,
        modal::ModalKind::GoalApprovalRejectPrompt,
    );

    assert!(!quit);
    assert!(matches!(
        daemon_rx.try_recv().expect("expected approval rejection command"),
        DaemonCommand::ResolveTaskApproval {
            approval_id,
            decision
        } if approval_id == "approval-1" && decision == "reject"
    ));
    assert!(matches!(
        daemon_rx.try_recv().expect("expected stop goal command"),
        DaemonCommand::ControlGoalRun {
            goal_run_id,
            action,
            step_index: None
        } if goal_run_id == "goal-1" && action == "stop"
    ));
    assert_eq!(model.modal.top(), None);
}

#[test]
fn approval_center_mouse_click_executes_approve_once() {
    let (mut model, mut daemon_rx) = make_model();
    model.width = 120;
    model.height = 40;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model
        .approval
        .reduce(crate::state::ApprovalAction::ApprovalRequired(
            crate::state::PendingApproval {
                approval_id: "approval-1".to_string(),
                task_id: "task-1".to_string(),
                task_title: Some("Task".to_string()),
                thread_id: Some("thread-1".to_string()),
                thread_title: Some("Thread".to_string()),
                workspace_id: Some(model.config.honcho_workspace_id.clone()),
                rationale: Some("Needed".to_string()),
                reasons: vec!["network".to_string()],
                command: "git push".to_string(),
                risk_level: crate::state::RiskLevel::High,
                blast_radius: "repo".to_string(),
                received_at: 1,
                seen_at: None,
            },
        ));
    model.toggle_approval_center();
    let (_, area) = model
        .current_modal_area()
        .expect("approval center modal area should exist");
    let click = (area.y..area.y.saturating_add(area.height))
        .flat_map(|row| {
            (area.x..area.x.saturating_add(area.width)).map(move |column| (column, row))
        })
        .find(|(column, row)| {
            widgets::approval_center::hit_test(
                area,
                &model.approval,
                model.chat.active_thread_id(),
                model.current_workspace_id(),
                ratatui::layout::Position::new(*column, *row),
            ) == Some(
                widgets::approval_center::ApprovalCenterHitTarget::ApproveOnce(
                    "approval-1".to_string(),
                ),
            )
        })
        .expect("approve-once button should be hittable");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: click.0,
        row: click.1,
        modifiers: KeyModifiers::NONE,
    });

    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected approval rules refresh command"),
        DaemonCommand::ListTaskApprovalRules
    ));
    assert!(matches!(
        daemon_rx.try_recv().expect("expected approval resolution command"),
        DaemonCommand::ResolveTaskApproval {
            approval_id,
            decision
        } if approval_id == "approval-1" && decision == "allow_once"
    ));
}

#[test]
fn approval_center_keyboard_creates_always_approve_rule() {
    let (mut model, mut daemon_rx) = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model
        .approval
        .reduce(crate::state::ApprovalAction::ApprovalRequired(
            crate::state::PendingApproval {
                approval_id: "approval-1".to_string(),
                task_id: "task-1".to_string(),
                task_title: Some("Task".to_string()),
                thread_id: Some("thread-1".to_string()),
                thread_title: Some("Thread".to_string()),
                workspace_id: Some(model.config.honcho_workspace_id.clone()),
                rationale: None,
                reasons: Vec::new(),
                command: "orchestrator_policy_escalation".to_string(),
                risk_level: crate::state::RiskLevel::Medium,
                blast_radius: "thread".to_string(),
                received_at: 1,
                seen_at: None,
            },
        ));
    model.toggle_approval_center();

    let quit = model.handle_key_modal(
        KeyCode::Char('w'),
        KeyModifiers::NONE,
        modal::ModalKind::ApprovalCenter,
    );

    assert!(!quit);
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected approval rules refresh command"),
        DaemonCommand::ListTaskApprovalRules
    ));
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected rule creation command"),
        DaemonCommand::CreateTaskApprovalRule { approval_id } if approval_id == "approval-1"
    ));
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected approval resolution command"),
        DaemonCommand::ResolveTaskApproval {
            approval_id,
            decision
        } if approval_id == "approval-1" && decision == "allow_once"
    ));
}

#[test]
fn approval_center_keyboard_revokes_saved_rule() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .approval
        .reduce(crate::state::ApprovalAction::SetRules(vec![
            crate::state::approval::SavedApprovalRule {
                id: "rule-1".to_string(),
                command: "orchestrator_policy_escalation".to_string(),
                created_at: 1,
                last_used_at: Some(2),
                use_count: 3,
            },
        ]));
    model
        .approval
        .reduce(crate::state::ApprovalAction::SetFilter(
            crate::state::ApprovalFilter::SavedRules,
        ));
    model.toggle_approval_center();

    let quit = model.handle_key_modal(
        KeyCode::Char('r'),
        KeyModifiers::NONE,
        modal::ModalKind::ApprovalCenter,
    );

    assert!(!quit);
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected approval rules refresh command"),
        DaemonCommand::ListTaskApprovalRules
    ));
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected revoke rule command"),
        DaemonCommand::RevokeTaskApprovalRule { rule_id } if rule_id == "rule-1"
    ));
}

#[test]
fn command_palette_plugins_install_seeds_terminal_command() {
    let (mut model, _daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));
    model
        .modal
        .reduce(modal::ModalAction::SetQuery("plugins install".into()));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::CommandPalette,
    );

    assert!(!quit);
    assert_eq!(model.input.buffer(), "zorai install plugin ");
}

#[test]
fn command_palette_skills_install_seeds_terminal_command() {
    let (mut model, _daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));
    model
        .modal
        .reduce(modal::ModalAction::SetQuery("skills install".into()));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::CommandPalette,
    );

    assert!(!quit);
    assert_eq!(model.input.buffer(), "zorai skill import ");
}

#[test]
fn command_palette_guidelines_install_seeds_terminal_command() {
    let (mut model, _daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));
    model
        .modal
        .reduce(modal::ModalAction::SetQuery("guidelines install".into()));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::CommandPalette,
    );

    assert!(!quit);
    assert_eq!(model.input.buffer(), "zorai guideline install ");
}

#[test]
fn stacked_modal_pop_only_cleans_whatsapp_when_top() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::WhatsAppLink));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::CommandPalette,
    );
    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::WhatsAppLink));
    assert!(daemon_rx.try_recv().is_err());

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::WhatsAppLink,
    );
    assert!(!quit);
    assert!(model.modal.top().is_none());
    assert!(matches!(
        daemon_rx.try_recv().expect("expected stop command"),
        DaemonCommand::WhatsAppLinkStop
    ));
    assert!(matches!(
        daemon_rx.try_recv().expect("expected unsubscribe command"),
        DaemonCommand::WhatsAppLinkUnsubscribe
    ));
}

#[test]
fn stacked_modal_pop_preserves_connected_whatsapp_session() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .set_whatsapp_link_connected(Some("+48663977535".to_string()));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::WhatsAppLink));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::CommandPalette,
    );
    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::WhatsAppLink));
    assert!(daemon_rx.try_recv().is_err());

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::WhatsAppLink,
    );
    assert!(!quit);
    assert!(model.modal.top().is_none());
    assert!(matches!(
        daemon_rx.try_recv().expect("expected unsubscribe command"),
        DaemonCommand::WhatsAppLinkUnsubscribe
    ));
    assert!(daemon_rx.try_recv().is_err());
}

#[test]
fn selecting_custom_provider_does_not_chain_into_model_picker() {
    let (mut model, _daemon_rx) = make_model();
    let custom_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| provider.id == PROVIDER_ID_CUSTOM)
        .expect("custom provider to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::Provider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model.modal.set_picker_item_count(
        widgets::provider_picker::available_provider_defs(&model.auth).len(),
    );
    if custom_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(custom_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.provider, PROVIDER_ID_CUSTOM);
    assert_ne!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
}
