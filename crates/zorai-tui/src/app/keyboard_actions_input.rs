use super::*;
use crossterm::event::{
    KeyCode, KeyModifiers,
};
impl TuiModel {
    pub(super) fn handle_input_key_action(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        ctrl: bool,
    ) -> Option<bool> {
        match code {
            KeyCode::Char('j') if ctrl && self.focus == FocusArea::Input => {
                {
                    self.input.reduce(input::InputAction::InsertNewline);
                };
                Some(false)
            }
            KeyCode::Char('l') if ctrl && self.focus == FocusArea::Input => {
                {
                    self.toggle_voice_capture();
                };
                Some(false)
            }
            KeyCode::Char('p') if ctrl && self.focus == FocusArea::Chat => {
                {
                    self.speak_latest_assistant_message();
                };
                Some(false)
            }
            KeyCode::Enter => {
                return Some(self.handle_enter_key(modifiers));
            }
            KeyCode::Backspace
                if (ctrl && modifiers.contains(KeyModifiers::SHIFT))
                    || modifiers.contains(KeyModifiers::ALT) =>
            {
                if self.focus == FocusArea::Input && !self.attachments.is_empty() {
                    let removed = self.attachments.len();
                    self.attachments.clear();
                    self.status_line = format!("Cleared {removed} attachment(s)");
                }
                Some(false)
            }
            KeyCode::Backspace if ctrl => {
                {
                    if self.focus == FocusArea::Input {
                        self.input.reduce(input::InputAction::DeleteWord);
                    }
                };
                Some(false)
            }
            KeyCode::Char('h') if ctrl && self.focus == FocusArea::Input => {
                {
                    self.input.reduce(input::InputAction::DeleteWord);
                };
                Some(false)
            }
            KeyCode::Backspace => {
                {
                    if self.focus == FocusArea::Input {
                        if self.input.buffer().is_empty()
                            && !self.attachments.is_empty()
                            && self.modal.top() != Some(modal::ModalKind::CommandPalette)
                        {
                            if let Some(att) = self.attachments.pop() {
                                self.status_line = format!("Removed: {}", att.filename);
                            }
                        } else {
                            self.input.reduce(input::InputAction::Backspace);
                            if self.modal.top() == Some(modal::ModalKind::CommandPalette) {
                                self.modal.reduce(modal::ModalAction::SetQuery(
                                    self.input.buffer().to_string(),
                                ));
                            }
                        }
                    }
                };
                Some(false)
            }
            KeyCode::Delete => {
                {
                    if self.focus == FocusArea::Chat {
                        if let Some(sel) = self.chat.selected_message() {
                            self.request_delete_message(sel);
                        }
                    }
                };
                Some(false)
            }
            KeyCode::Char('/') if self.focus != FocusArea::Input => {
                {
                    self.input.set_mode(input::InputMode::Insert);
                    self.focus = FocusArea::Input;
                    self.open_command_palette(None);
                };
                Some(false)
            }
            KeyCode::Char('w') if ctrl && self.focus == FocusArea::Input => {
                {
                    self.input.reduce(input::InputAction::DeleteWord);
                };
                Some(false)
            }
            KeyCode::Char('v' | 'V') if ctrl => {
                self.paste_from_clipboard();
                Some(false)
            }
            KeyCode::Char('\u{16}') => {
                self.paste_from_clipboard();
                Some(false)
            }
            KeyCode::Insert if modifiers.contains(KeyModifiers::SHIFT) => {
                {
                    self.paste_from_clipboard();
                };
                Some(false)
            }
            KeyCode::Char('c')
                if self.focus == FocusArea::Chat && self.chat.selected_message().is_some() =>
            {
                {
                    if let Some(sel) = self.chat.selected_message() {
                        self.copy_message(sel);
                    }
                };
                Some(false)
            }
            KeyCode::Char(c) => {
                {
                    if self.focus == FocusArea::Input {
                        if c == '/'
                            && self.input.buffer().is_empty()
                            && self.modal.top() != Some(modal::ModalKind::CommandPalette)
                        {
                            self.open_command_palette(None);
                        } else {
                            self.input.reduce(input::InputAction::InsertChar(c));
                            if self.modal.top() == Some(modal::ModalKind::CommandPalette) {
                                self.modal.reduce(modal::ModalAction::SetQuery(
                                    self.input.buffer().to_string(),
                                ));
                            }
                        }
                    } else {
                        self.focus = FocusArea::Input;
                        self.input.set_mode(input::InputMode::Insert);
                        self.input.reduce(input::InputAction::InsertChar(c));
                    }
                };
                Some(false)
            }
            _ => None,
        }
    }
}
