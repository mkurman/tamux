use zorai_protocol::{WorkspaceActor, AGENT_ID_SWAROG};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceActorPickerMode {
    Assignee,
    Reviewer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkspaceActorPickerOption {
    pub(super) label: String,
    pub(super) actor: Option<WorkspaceActor>,
}

impl WorkspaceActorPickerMode {
    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Assignee => "WORKSPACE ASSIGNEE",
            Self::Reviewer => "WORKSPACE REVIEWER",
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Self::Assignee => "assignee",
            Self::Reviewer => "reviewer",
        }
    }
}

pub(super) fn workspace_actor_picker_options(
    mode: WorkspaceActorPickerMode,
    subagents: &crate::state::SubAgentsState,
) -> Vec<WorkspaceActorPickerOption> {
    let mut options = vec![WorkspaceActorPickerOption {
        label: "none".to_string(),
        actor: None,
    }];
    if mode == WorkspaceActorPickerMode::Reviewer {
        options.push(WorkspaceActorPickerOption {
            label: "user".to_string(),
            actor: Some(WorkspaceActor::User),
        });
    }
    options.push(WorkspaceActorPickerOption {
        label: "svarog".to_string(),
        actor: Some(WorkspaceActor::Agent(AGENT_ID_SWAROG.to_string())),
    });
    options.extend(builtin_workspace_actor_options());
    options.extend(
        subagents
            .entries
            .iter()
            .filter(|entry| entry.enabled)
            .map(|entry| WorkspaceActorPickerOption {
                label: if entry.name.trim().is_empty() {
                    entry.id.clone()
                } else {
                    entry.name.clone()
                },
                actor: Some(WorkspaceActor::Subagent(entry.id.clone())),
            }),
    );
    options
}

fn builtin_workspace_actor_options() -> impl Iterator<Item = WorkspaceActorPickerOption> {
    crate::state::subagents::BUILTIN_PERSONA_ROLE_CHOICES
        .iter()
        .filter(|choice| {
            !matches!(
                choice.id,
                zorai_protocol::AGENT_ID_SWAROG | zorai_protocol::AGENT_ID_RAROG | "rod"
            )
        })
        .map(|choice| WorkspaceActorPickerOption {
            label: choice.label.to_string(),
            actor: Some(WorkspaceActor::Subagent(choice.id.to_string())),
        })
}

pub(super) fn workspace_actor_picker_body(
    task_id: &str,
    mode: WorkspaceActorPickerMode,
    options: &[WorkspaceActorPickerOption],
    cursor: usize,
) -> String {
    let short_id = task_id.chars().take(12).collect::<String>();
    let mut body = format!("Task: {short_id}\nSelect {}:\n\n", mode.noun());
    for (index, option) in options.iter().enumerate() {
        let marker = if index == cursor { ">" } else { " " };
        body.push_str(&format!("{marker} {}\n", option.label));
    }
    body.push_str("\nEnter select - Esc cancel");
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{SubAgentEntry, SubAgentsState};

    fn subagent(id: &str, name: &str, enabled: bool) -> SubAgentEntry {
        SubAgentEntry {
            id: id.to_string(),
            name: name.to_string(),
            provider: "openai".to_string(),
            model: "gpt-5.4".to_string(),
            role: None,
            enabled,
            builtin: false,
            immutable_identity: false,
            disable_allowed: true,
            delete_allowed: true,
            protected_reason: None,
            reasoning_effort: None,
            openrouter_provider_order: String::new(),
            openrouter_provider_ignore: String::new(),
            openrouter_allow_fallbacks: true,
            raw_json: None,
        }
    }

    #[test]
    fn assignee_picker_excludes_user_and_disabled_subagents() {
        let mut subagents = SubAgentsState::new();
        subagents.entries = vec![subagent("qa", "QA", true), subagent("off", "Off", false)];

        let options =
            workspace_actor_picker_options(WorkspaceActorPickerMode::Assignee, &subagents);
        let labels = options
            .iter()
            .map(|option| option.label.as_str())
            .collect::<Vec<_>>();

        assert!(labels.starts_with(&["none", "svarog", "Weles"]));
        assert!(labels.contains(&"Mokosh"));
        assert!(labels.contains(&"QA"));
        assert!(!labels.contains(&"Off"));
    }

    #[test]
    fn reviewer_picker_includes_user() {
        let subagents = SubAgentsState::new();
        let options =
            workspace_actor_picker_options(WorkspaceActorPickerMode::Reviewer, &subagents);

        assert_eq!(options[0].label, "none");
        assert_eq!(options[1].label, "user");
        assert_eq!(options[2].label, "svarog");
        assert!(options.iter().any(|option| option.label == "Mokosh"
            && option.actor == Some(WorkspaceActor::Subagent("mokosh".to_string()))));
    }

    #[test]
    fn picker_body_marks_selected_option() {
        let subagents = SubAgentsState::new();
        let options =
            workspace_actor_picker_options(WorkspaceActorPickerMode::Reviewer, &subagents);

        let body = workspace_actor_picker_body(
            "workspace-task-123",
            WorkspaceActorPickerMode::Reviewer,
            &options,
            1,
        );

        assert!(body.contains("Task: workspace-ta"));
        assert!(body.contains("> user"));
    }
}
