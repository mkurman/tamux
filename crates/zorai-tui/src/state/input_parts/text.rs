
use super::InputState;

impl InputState {
    /// Replace input buffer with the given text and place cursor at end.
    pub fn set_text(&mut self, text: &str) {
        self.clear_text();
        self.paste_blocks.clear();
        for ch in text.chars() {
            self.textarea.insert_char(ch);
        }
        self.sync_buffer_cache();
    }
}
