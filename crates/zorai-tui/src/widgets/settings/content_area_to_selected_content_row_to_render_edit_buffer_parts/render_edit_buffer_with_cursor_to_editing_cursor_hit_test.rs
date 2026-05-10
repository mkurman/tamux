use super::super::advanced_single_line_edit_layout_to_subagent_row_action_offsets::*;
use crate::providers;
use crate::state::concierge::ConciergeState;
use crate::state::config::ConfigState;
use crate::state::modal::ModalState;
use crate::state::settings::{PluginSettingsState, SettingsState, SettingsTab};
use crate::state::subagents::SubAgentsState;
use crate::theme::ThemeTokens;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsHitTarget {
    Tab(SettingsTab),
    Field(usize),
    AuthProviderItem(usize),
    AuthAction {
        index: usize,
        action: AuthTabAction,
    },
    SubAgentListItem(usize),
    SubAgentAction(SubAgentTabAction),
    SubAgentRowAction {
        index: usize,
        action: SubAgentTabAction,
    },
    EditCursor {
        line: usize,
        col: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthTabAction {
    Primary,
    Test,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubAgentTabAction {
    Add,
    Edit,
    Delete,
    Toggle,
}

pub(crate) const TAB_LABELS: [&str; 12] = [
    "Auth", "Svar", "Rar", "Tools", "Search", "Chat", "GW", "Sub", "Feat", "Adv", "Plug", "About",
];
pub(crate) const TAB_DIVIDER: &str = " | ";

#[derive(Debug, Clone, Copy)]
pub(crate) struct VisibleTab {
    pub(crate) tab: SettingsTab,
    pub(crate) index: usize,
    pub(crate) start_x: u16,
    pub(crate) end_x: u16,
}

pub(crate) fn render_edit_buffer_with_cursor(text: &str, cursor: usize) -> String {
    let cursor = cursor.min(text.chars().count());
    let mut out = String::with_capacity(text.len() + 3);
    let byte_cursor = text
        .char_indices()
        .nth(cursor)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len());
    out.push_str(&text[..byte_cursor]);
    out.push('\u{2588}');
    out.push_str(&text[byte_cursor..]);
    out
}

pub(crate) fn render_edit_line_with_cursor(text: &str, cursor_col: usize) -> String {
    let mut out = String::with_capacity(text.len() + 3);
    let mut inserted = false;
    for (col, ch) in text.chars().enumerate() {
        if col == cursor_col {
            out.push('\u{2588}');
            inserted = true;
        }
        out.push(ch);
    }
    if !inserted {
        out.push('\u{2588}');
    }
    out
}

pub(crate) fn clip_inline_text(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_string();
    }
    let tail: String = chars[chars.len().saturating_sub(max_chars)..]
        .iter()
        .collect();
    format!("…{}", tail)
}

pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    settings: &SettingsState,
    config: &ConfigState,
    modal: &ModalState,
    auth: &crate::state::auth::AuthState,
    subagents: &SubAgentsState,
    concierge: &ConciergeState,
    tier: &crate::state::tier::TierState,
    plugin_settings: &PluginSettingsState,
    scroll: usize,
    theme: &ThemeTokens,
) {
    let block = Block::default()
        .title(" SETTINGS ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme.accent_secondary);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 5 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let active = settings.active_tab();
    let tab_index = active_tab_index(active);
    let tabs = visible_tabs(chunks[0], tab_index);
    frame.render_widget(
        Paragraph::new(render_tabs_line(&tabs, settings, theme)),
        chunks[0],
    );

    let sep = Line::from(Span::styled(
        "\u{2500}".repeat(chunks[1].width as usize),
        theme.fg_dim,
    ));
    frame.render_widget(Paragraph::new(sep), chunks[1]);

    let content_lines = render_tab_content(
        chunks[2].width,
        settings,
        config,
        modal,
        auth,
        subagents,
        concierge,
        tier,
        plugin_settings,
        theme,
    );
    let paragraph = Paragraph::new(content_lines).scroll((scroll.min(u16::MAX as usize) as u16, 0));
    frame.render_widget(paragraph, chunks[2]);

    let hints = if settings.is_editing() {
        Line::from(vec![
            Span::raw(" "),
            Span::styled("Enter", theme.fg_active),
            Span::styled(" confirm  ", theme.fg_dim),
            Span::styled("Esc", theme.fg_active),
            Span::styled(" cancel", theme.fg_dim),
        ])
    } else {
        let mut spans = vec![
            Span::raw(" "),
            Span::styled("↑↓", theme.fg_active),
            Span::styled(" navigate  ", theme.fg_dim),
        ];
        if settings_field_can_activate(settings, config) {
            spans.push(Span::styled("Enter", theme.fg_active));
            spans.push(Span::styled(" edit/select  ", theme.fg_dim));
        }
        if settings_field_can_toggle(settings, config) {
            spans.push(Span::styled("Space", theme.fg_active));
            spans.push(Span::styled(" toggle  ", theme.fg_dim));
        }
        spans.extend([
            Span::styled("Tab", theme.fg_active),
            Span::styled(" switch tab  ", theme.fg_dim),
            Span::styled("Esc", theme.fg_active),
            Span::styled(" close", theme.fg_dim),
        ]);
        Line::from(spans)
    };
    frame.render_widget(Paragraph::new(hints), chunks[3]);
}

pub(crate) fn settings_field_can_activate(settings: &SettingsState, config: &ConfigState) -> bool {
    let field = settings.current_field_name_with_config(config);
    if field.is_empty() || matches!(field, "snapshot_stats") {
        return false;
    }
    if settings.active_tab() == SettingsTab::Advanced
        && field == "context_window_tokens"
        && !providers::model_uses_context_window_override(
            &config.provider,
            &config.auth_source,
            &config.model,
            &config.custom_model_name,
        )
    {
        return false;
    }
    !matches!(
        field,
        "managed_sandbox_enabled"
            | "gateway_enabled"
            | "web_search_enabled"
            | "enable_streaming"
            | "enable_conversation_memory"
            | "anticipatory_enabled"
            | "anticipatory_morning_brief"
            | "anticipatory_predictive_hydration"
            | "anticipatory_stuck_detection"
            | "operator_model_enabled"
            | "operator_model_allow_message_statistics"
            | "operator_model_allow_approval_learning"
            | "operator_model_allow_attention_tracking"
            | "operator_model_allow_implicit_feedback"
            | "collaboration_enabled"
            | "compliance_sign_all_events"
            | "tool_synthesis_enabled"
            | "tool_synthesis_require_activation"
            | "auto_compact_context"
            | "auto_retry"
            | "snapshot_auto_cleanup"
            | "workspace_repo_monitor_enabled"
            | "feat_check_stale_todos"
            | "feat_check_stuck_goals"
            | "feat_check_unreplied_messages"
            | "feat_check_repo_changes"
            | "feat_consolidation_enabled"
            | "feat_skill_recommendation_enabled"
            | "feat_skill_background_community_search"
            | "feat_audio_stt_enabled"
            | "feat_audio_tts_enabled"
            | "feat_embedding_enabled"
    ) && !field.starts_with("tool_")
}

pub(crate) fn settings_field_can_toggle(settings: &SettingsState, config: &ConfigState) -> bool {
    let field = settings.current_field_name_with_config(config);
    matches!(
        field,
        "managed_sandbox_enabled"
            | "managed_security_level"
            | "gateway_enabled"
            | "web_search_enabled"
            | "enable_streaming"
            | "enable_conversation_memory"
            | "enable_honcho_memory"
            | "anticipatory_enabled"
            | "anticipatory_morning_brief"
            | "anticipatory_predictive_hydration"
            | "anticipatory_stuck_detection"
            | "operator_model_enabled"
            | "operator_model_allow_message_statistics"
            | "operator_model_allow_approval_learning"
            | "operator_model_allow_attention_tracking"
            | "operator_model_allow_implicit_feedback"
            | "collaboration_enabled"
            | "compliance_sign_all_events"
            | "tool_synthesis_enabled"
            | "tool_synthesis_require_activation"
            | "auto_compact_context"
            | "compaction_strategy"
            | "compaction_weles_provider"
            | "compaction_weles_reasoning_effort"
            | "compaction_custom_provider"
            | "compaction_custom_auth_source"
            | "compaction_custom_api_transport"
            | "compaction_custom_reasoning_effort"
            | "auto_retry"
            | "snapshot_auto_cleanup"
            | "workspace_repo_monitor_enabled"
            | "feat_tier_override"
            | "feat_security_level"
            | "feat_check_stale_todos"
            | "feat_check_stuck_goals"
            | "feat_check_unreplied_messages"
            | "feat_check_repo_changes"
            | "feat_consolidation_enabled"
            | "feat_skill_recommendation_enabled"
            | "feat_skill_background_community_search"
            | "feat_audio_stt_enabled"
            | "feat_audio_tts_enabled"
            | "feat_embedding_enabled"
            | "whatsapp_link_device"
            | "whatsapp_relink_device"
    ) || field.starts_with("tool_")
}

pub(crate) fn hit_test(
    area: Rect,
    settings: &SettingsState,
    config: &ConfigState,
    auth: &crate::state::auth::AuthState,
    subagents: &SubAgentsState,
    scroll: usize,
    mouse: Position,
) -> Option<SettingsHitTarget> {
    let block = Block::default()
        .title(" SETTINGS ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double);
    let inner = block.inner(area);
    if inner.height < 5
        || mouse.x < inner.x
        || mouse.x >= inner.x.saturating_add(inner.width)
        || mouse.y < inner.y
        || mouse.y >= inner.y.saturating_add(inner.height)
    {
        return None;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    if mouse.y == chunks[0].y {
        if let Some(tab) = tab_hit_test(chunks[0], settings.active_tab(), mouse.x) {
            return Some(SettingsHitTarget::Tab(tab));
        }
        return None;
    }

    if mouse.y < chunks[2].y || mouse.y >= chunks[2].y.saturating_add(chunks[2].height) {
        return None;
    }

    if let Some((line, col)) = editing_cursor_hit_test(chunks[2], settings, config, scroll, mouse) {
        return Some(SettingsHitTarget::EditCursor { line, col });
    }

    if matches!(settings.active_tab(), SettingsTab::Auth) {
        return auth_hit_test(chunks[2], auth, scroll, mouse);
    }

    if matches!(settings.active_tab(), SettingsTab::SubAgents) {
        return subagents_hit_test(chunks[2], subagents, scroll, mouse);
    }

    let row = mouse.y.saturating_sub(chunks[2].y) as usize + scroll;
    match settings_row_hit(settings, config, subagents, row) {
        Some((_, Some(subagent_index))) => {
            Some(SettingsHitTarget::SubAgentListItem(subagent_index))
        }
        Some((field, None)) => Some(SettingsHitTarget::Field(field)),
        None => None,
    }
}

pub(crate) fn tab_hit_test(
    tab_area: Rect,
    active_tab: SettingsTab,
    mouse_x: u16,
) -> Option<SettingsTab> {
    visible_tabs(tab_area, active_tab_index(active_tab))
        .into_iter()
        .find(|tab| mouse_x >= tab.start_x && mouse_x < tab.end_x)
        .map(|tab| tab.tab)
}

pub(crate) fn active_tab_index(tab: SettingsTab) -> usize {
    match tab {
        SettingsTab::Auth => 0,
        SettingsTab::Provider | SettingsTab::Agent => 1,
        SettingsTab::Concierge => 2,
        SettingsTab::Tools => 3,
        SettingsTab::WebSearch => 4,
        SettingsTab::Chat => 5,
        SettingsTab::Gateway => 6,
        SettingsTab::SubAgents => 7,
        SettingsTab::Features => 8,
        SettingsTab::Advanced => 9,
        SettingsTab::Plugins => 10,
        SettingsTab::About => 11,
    }
}

pub(crate) fn visible_tabs(tab_area: Rect, active_index: usize) -> Vec<VisibleTab> {
    let all = SettingsTab::all();
    let divider_width = TAB_DIVIDER.chars().count() as u16;
    let total_width = |start: usize, end: usize| -> u16 {
        (start..=end)
            .map(|idx| TAB_LABELS[idx].chars().count() as u16)
            .sum::<u16>()
            .saturating_add(divider_width.saturating_mul((end - start) as u16))
    };

    let mut start = 0usize;
    let mut end = all.len().saturating_sub(1);
    let available = tab_area.width.saturating_sub(2);

    while start < active_index && total_width(start, end) > available {
        start += 1;
    }
    while end > active_index && total_width(start, end) > available {
        end = end.saturating_sub(1);
    }
    while total_width(start, end) > available && start < end {
        if active_index.saturating_sub(start) > end.saturating_sub(active_index) {
            start += 1;
        } else {
            end = end.saturating_sub(1);
        }
    }

    let prefix = if start > 0 { "« " } else { "" };
    let mut x = tab_area.x.saturating_add(prefix.chars().count() as u16);

    let mut visible = Vec::new();
    for idx in start..=end {
        let width = TAB_LABELS[idx].chars().count() as u16;
        visible.push(VisibleTab {
            tab: all[idx],
            index: idx,
            start_x: x,
            end_x: x.saturating_add(width),
        });
        x = x.saturating_add(width);
        if idx < end {
            x = x.saturating_add(divider_width);
        }
    }
    visible
}

pub(crate) fn render_tabs_line(
    tabs: &[VisibleTab],
    settings: &SettingsState,
    theme: &ThemeTokens,
) -> Line<'static> {
    let active_index = active_tab_index(settings.active_tab());
    let mut spans = Vec::new();
    if tabs.first().is_some_and(|tab| tab.index > 0) {
        spans.push(Span::styled("« ", theme.fg_dim));
    }
    for (idx, tab) in tabs.iter().enumerate() {
        let style = if tab.index == active_index {
            theme.fg_active
        } else {
            theme.fg_dim
        };
        spans.push(Span::styled(TAB_LABELS[tab.index], style));
        if idx + 1 < tabs.len() {
            spans.push(Span::styled(TAB_DIVIDER, theme.fg_dim));
        }
    }
    if tabs
        .last()
        .is_some_and(|tab| tab.index + 1 < TAB_LABELS.len())
    {
        spans.push(Span::styled(" »", theme.fg_dim));
    }
    Line::from(spans)
}

pub(crate) fn editing_cursor_hit_test(
    content_area: Rect,
    settings: &SettingsState,
    config: &ConfigState,
    scroll: usize,
    mouse: Position,
) -> Option<(usize, usize)> {
    let field = settings.editing_field()?;
    let row = mouse.y.saturating_sub(content_area.y) as usize + scroll;
    let rel_x = mouse.x.saturating_sub(content_area.x) as usize;

    if settings.is_textarea() {
        let (text_start_row, text_start_col) = textarea_edit_layout(settings, config, field)?;
        let line_count = settings.edit_buffer().split('\n').count().max(1);
        let row_end = text_start_row + line_count;
        if row < text_start_row || row > row_end {
            return None;
        }
        let line = (row - text_start_row).min(line_count.saturating_sub(1));
        let col = rel_x.saturating_sub(text_start_col);
        return Some((line, col));
    }

    let (field_row, start_col) = single_line_edit_layout(settings, config, field)?;
    if row == field_row {
        return Some((0, rel_x.saturating_sub(start_col)));
    }
    None
}
