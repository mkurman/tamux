use super::*;
use crate::widgets;
use crossterm::event::{
    KeyCode, KeyModifiers,
};
impl TuiModel {
    pub(crate) fn open_command_palette(&mut self, seed_query: Option<String>) {
        self.modal
            .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));
        if let Some(query) = seed_query {
            self.modal.reduce(modal::ModalAction::SetQuery(query));
        }
    }

    pub(crate) fn matches_shift_char(
        code: KeyCode,
        modifiers: KeyModifiers,
        expected: char,
    ) -> bool {
        modifiers.contains(KeyModifiers::SHIFT)
            && matches!(code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&expected))
    }

    pub(crate) fn pinned_shortcut_scope_active(&self) -> bool {
        !self.sidebar_uses_goal_sidebar()
            && self.sidebar_visible()
            && self.sidebar.active_tab() == sidebar::SidebarTab::Pinned
            && self.chat.active_thread_has_pinned_messages()
    }

    fn sidebar_navigation_tabs(&self) -> Vec<sidebar::SidebarTab> {
        widgets::sidebar::visible_tabs(&self.tasks, &self.chat, self.chat.active_thread_id())
    }

    pub(crate) fn step_sidebar_tab(&mut self, delta: i32) {
        if self.sidebar_uses_goal_sidebar() {
            self.step_goal_sidebar_tab(delta);
            return;
        }

        let tabs = self.sidebar_navigation_tabs();
        let Some(last_index) = tabs.len().checked_sub(1) else {
            return;
        };
        let current_index = tabs
            .iter()
            .position(|tab| *tab == self.sidebar.active_tab())
            .unwrap_or(0);
        let next_index = (current_index as i32 + delta).clamp(0, last_index as i32) as usize;
        self.activate_sidebar_tab(tabs[next_index]);
    }

    pub(crate) fn arm_pinned_shortcut_leader(&mut self) {
        self.pending_pinned_shortcut_leader = Some(PendingPinnedShortcutLeader::Active);
        self.status_line = "Pinned shortcuts: J jump, U unpin".to_string();
        self.show_input_notice(
            "Pinned shortcuts: Ctrl+K J jump, Ctrl+K U unpin",
            InputNoticeKind::Success,
            60,
            true,
        );
    }

    pub(crate) fn handle_pending_pinned_shortcut_leader(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> bool {
        if self.pending_pinned_shortcut_leader.is_none() {
            return false;
        }
        self.pending_pinned_shortcut_leader = None;

        if !self.pinned_shortcut_scope_active() {
            return false;
        }

        match code {
            KeyCode::Esc => {
                self.status_line = "Pinned shortcut cancelled".to_string();
                true
            }
            KeyCode::Char(ch)
                if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                match ch.to_ascii_lowercase() {
                    'j' => {
                        self.handle_sidebar_enter();
                        true
                    }
                    'u' => {
                        self.unpin_selected_sidebar_message();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
