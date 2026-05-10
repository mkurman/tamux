use super::*;
use super::super::build_model;
use ratatui::backend::TestBackend;
#[test]
fn goal_composer_mission_control_preflight_renders_stable_sections() {
    let mut model = build_model();
    model.width = 100;
    model.height = 40;
    model.open_new_goal_view();
    model.goal_mission_control.prompt_text = "Ship the next release".to_string();
    model.goal_mission_control.preset_source_label = "Previous goal snapshot".to_string();
    model.goal_mission_control.save_as_default_pending = true;

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("goal mission control preflight render should succeed");

    let buffer = terminal.backend().buffer();
    let rendered = (0..model.height)
        .map(|y| {
            (0..model.width)
                .filter_map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        rendered.contains("MISSION CONTROL"),
        "preflight should render the Mission Control title"
    );
    assert!(
        rendered.contains("Goal prompt"),
        "preflight should render a prompt section"
    );
    assert!(
        rendered.contains("Main model"),
        "preflight should render the selected main model section"
    );
    assert!(
        rendered.contains("Agent roster"),
        "preflight should render the goal agent roster section"
    );
    assert!(
        rendered.contains("Preset source"),
        "preflight should render the preset source label"
    );
    assert!(
        rendered.contains("Save as default"),
        "preflight should render the save-as-default toggle state"
    );
    assert!(
        rendered.contains("Ship the next release"),
        "preflight should include the current goal prompt"
    );
    assert!(
        !rendered.contains("Describe the goal in the input below"),
        "old composer helper text should no longer be rendered"
    );
}

#[test]
fn mission_control_roster_render_shows_live_now_and_pending_next_turn_labels() {
    let mut model = build_model();
    model.width = 100;
    model.height = 40;
    model.main_pane_view = MainPaneView::GoalComposer;
    model.goal_mission_control =
        goal_mission_control::GoalMissionControlState::from_main_assignment(
            task::GoalAgentAssignment {
                role_id: zorai_protocol::AGENT_ID_SWAROG.to_string(),
                enabled: true,
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                reasoning_effort: Some("medium".to_string()),
                inherit_from_main: false,
            },
            vec![
                task::GoalAgentAssignment {
                    role_id: zorai_protocol::AGENT_ID_SWAROG.to_string(),
                    enabled: true,
                    provider: "openai".to_string(),
                    model: "gpt-5.4".to_string(),
                    reasoning_effort: Some("medium".to_string()),
                    inherit_from_main: false,
                },
                task::GoalAgentAssignment {
                    role_id: "reviewer".to_string(),
                    enabled: true,
                    provider: "openai".to_string(),
                    model: "gpt-5.4".to_string(),
                    reasoning_effort: Some("low".to_string()),
                    inherit_from_main: false,
                },
            ],
            "Goal runtime roster",
        );
    model.goal_mission_control.runtime_goal_run_id = Some("goal-1".to_string());
    model.goal_mission_control.active_runtime_assignment_index = Some(0);
    model.goal_mission_control.pending_role_assignments = Some(vec![
        task::GoalAgentAssignment {
            role_id: zorai_protocol::AGENT_ID_SWAROG.to_string(),
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-5.4".to_string(),
            reasoning_effort: Some("medium".to_string()),
            inherit_from_main: false,
        },
        task::GoalAgentAssignment {
            role_id: "reviewer".to_string(),
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-5.4-mini".to_string(),
            reasoning_effort: Some("low".to_string()),
            inherit_from_main: false,
        },
    ]);
    model.goal_mission_control.pending_runtime_apply_modes = vec![
        None,
        Some(goal_mission_control::RuntimeAssignmentApplyMode::NextTurn),
    ];

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("mission control roster render should succeed");

    let buffer = terminal.backend().buffer();
    let rendered = (0..model.height)
        .map(|y| {
            (0..model.width)
                .filter_map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains("live now"), "{rendered}");
    assert!(rendered.contains("pending next turn"), "{rendered}");
    assert!(rendered.contains("reviewer"), "{rendered}");
    assert!(rendered.contains("gpt-5.4-mini"), "{rendered}");
}
