use super::*;
impl SettingsState {
    pub fn reduce(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::Open => {
                self.active_tab = SettingsTab::Auth;
                self.field_cursor = 0;
                self.editing_field = None;
                self.edit_buffer.clear();
                self.edit_cursor = 0;
                self.dropdown_open = false;
                self.dropdown_cursor = 0;
                self.dirty = false;
            }

            SettingsAction::Close => {
                self.editing_field = None;
                self.edit_buffer.clear();
                self.edit_cursor = 0;
                self.dropdown_open = false;
            }

            SettingsAction::SwitchTab(tab) => {
                self.active_tab = tab;
                self.field_cursor = 0;
                self.editing_field = None;
                self.edit_buffer.clear();
                self.edit_cursor = 0;
                self.dropdown_open = false;
                self.dropdown_cursor = 0;
            }

            SettingsAction::NavigateField(delta) => {
                let count = self.field_count();
                if delta > 0 {
                    self.field_cursor =
                        (self.field_cursor + delta as usize).min(count.saturating_sub(1));
                } else {
                    self.field_cursor = self.field_cursor.saturating_sub((-delta) as usize);
                }
            }

            SettingsAction::EditField => {
                let field_name = self.current_field_name().to_string();
                if !field_name.is_empty() {
                    self.editing_field = Some(field_name);
                    self.dirty = true;
                }
            }

            SettingsAction::InsertChar(c) => {
                if self.editing_field.is_some() {
                    self.edit_buffer.insert(self.edit_cursor, c);
                    self.edit_cursor += c.len_utf8();
                }
            }

            SettingsAction::Backspace => {
                if self.editing_field.is_some() && self.edit_cursor > 0 {
                    let prev = self.edit_buffer[..self.edit_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.edit_buffer.drain(prev..self.edit_cursor);
                    self.edit_cursor = prev;
                }
            }

            SettingsAction::MoveCursorLeft => self.move_cursor_left(),

            SettingsAction::MoveCursorRight => self.move_cursor_right(),

            SettingsAction::MoveCursorUp => self.move_cursor_up(),

            SettingsAction::MoveCursorDown => self.move_cursor_down(),

            SettingsAction::MoveCursorHome => self.move_cursor_home(),

            SettingsAction::MoveCursorEnd => self.move_cursor_end(),

            SettingsAction::SetCursor(pos) => {
                self.edit_cursor = pos.min(self.edit_buffer.len());
            }

            SettingsAction::SetCursorLineCol(line, col) => {
                self.edit_cursor = self.line_col_to_offset(line, col);
            }

            SettingsAction::ConfirmEdit => {
                self.editing_field = None;
                self.textarea_mode = false;
            }

            SettingsAction::CancelEdit => {
                self.editing_field = None;
                self.textarea_mode = false;
                self.edit_buffer.clear();
                self.edit_cursor = 0;
            }

            SettingsAction::ToggleCheckbox => {
                self.dirty = true;
            }

            SettingsAction::SelectRadio => {
                self.dirty = true;
            }

            SettingsAction::OpenDropdown => {
                self.dropdown_open = true;
                self.dropdown_cursor = 0;
            }

            SettingsAction::NavigateDropdown(delta) => {
                if self.dropdown_open {
                    if delta > 0 {
                        self.dropdown_cursor = self.dropdown_cursor.saturating_add(delta as usize);
                    } else {
                        self.dropdown_cursor =
                            self.dropdown_cursor.saturating_sub((-delta) as usize);
                    }
                }
            }

            SettingsAction::SelectDropdown => {
                self.dropdown_open = false;
                self.dirty = true;
            }

            SettingsAction::Save => {
                self.dirty = false;
                self.editing_field = None;
                self.edit_buffer.clear();
                self.edit_cursor = 0;
            }
        }
    }
}
