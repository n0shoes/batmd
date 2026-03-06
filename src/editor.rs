use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct Editor {
    pub lines: Vec<String>,
    /// Cursor position as character index (not byte index)
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl Editor {
    pub fn new(content: String) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(String::from).collect()
        };

        Self {
            lines,
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn current_line(&self) -> &str {
        &self.lines[self.cursor_row]
    }

    /// Number of characters in the current line
    fn current_line_char_len(&self) -> usize {
        self.lines[self.cursor_row].chars().count()
    }

    /// Number of characters in a given line
    fn line_char_len(&self, row: usize) -> usize {
        self.lines[row].chars().count()
    }

    /// Convert character index to byte index for a given line
    fn char_to_byte(&self, row: usize, char_idx: usize) -> usize {
        self.lines[row]
            .char_indices()
            .nth(char_idx)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or(self.lines[row].len())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                // Ctrl-A: move to beginning of line
                KeyCode::Char('a') => {
                    self.cursor_col = 0;
                }
                // Ctrl-E: move to end of line
                KeyCode::Char('e') => {
                    self.cursor_col = self.current_line_char_len();
                }
                // Ctrl-K: kill from cursor to end of line
                KeyCode::Char('k') => {
                    let row = self.cursor_row;
                    let col = self.cursor_col;
                    if col < self.line_char_len(row) {
                        let byte_idx = self.char_to_byte(row, col);
                        self.lines[row].truncate(byte_idx);
                    } else if row + 1 < self.lines.len() {
                        // Join with next line
                        let next = self.lines.remove(row + 1);
                        self.lines[row].push_str(&next);
                    }
                }
                // Ctrl-D: delete character under cursor
                KeyCode::Char('d') => {
                    self.delete_forward();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete_forward(),
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Home => self.cursor_col = 0,
            KeyCode::End => self.cursor_col = self.current_line_char_len(),
            KeyCode::Tab => {
                for _ in 0..4 {
                    self.insert_char(' ');
                }
            }
            _ => {}
        }
    }

    fn insert_char(&mut self, c: char) {
        let row = self.cursor_row;
        let byte_idx = self.char_to_byte(row, self.cursor_col);
        self.lines[row].insert(byte_idx, c);
        self.cursor_col += 1;
    }

    fn insert_newline(&mut self) {
        let row = self.cursor_row;
        let byte_idx = self.char_to_byte(row, self.cursor_col);
        let remainder = self.lines[row][byte_idx..].to_string();
        self.lines[row].truncate(byte_idx);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.lines.insert(self.cursor_row, remainder);
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            let byte_idx = self.char_to_byte(self.cursor_row, self.cursor_col);
            // Find the byte range of the character to remove
            let ch = self.lines[self.cursor_row][byte_idx..].chars().next().unwrap();
            let end = byte_idx + ch.len_utf8();
            self.lines[self.cursor_row].replace_range(byte_idx..end, "");
        } else if self.cursor_row > 0 {
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.line_char_len(self.cursor_row);
            self.lines[self.cursor_row].push_str(&current_line);
        }
    }

    fn delete_forward(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        if col < self.line_char_len(row) {
            let byte_idx = self.char_to_byte(row, col);
            let ch = self.lines[row][byte_idx..].chars().next().unwrap();
            let end = byte_idx + ch.len_utf8();
            self.lines[row].replace_range(byte_idx..end, "");
        } else if row + 1 < self.lines.len() {
            let next = self.lines.remove(row + 1);
            self.lines[row].push_str(&next);
        }
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_char_len(self.cursor_row);
        }
    }

    fn move_right(&mut self) {
        if self.cursor_col < self.current_line_char_len() {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.line_char_len(self.cursor_row));
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.line_char_len(self.cursor_row));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn new_editor_from_content() {
        let ed = Editor::new("hello\nworld".into());
        assert_eq!(ed.lines, vec!["hello", "world"]);
        assert_eq!(ed.cursor_row, 0);
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn new_editor_empty_content() {
        let ed = Editor::new("".into());
        assert_eq!(ed.lines, vec![""]);
    }

    #[test]
    fn insert_char() {
        let mut ed = Editor::new("hello".into());
        ed.handle_key(key(KeyCode::Char('X')));
        assert_eq!(ed.content(), "Xhello");
        assert_eq!(ed.cursor_col, 1);
    }

    #[test]
    fn insert_newline() {
        let mut ed = Editor::new("hello".into());
        ed.cursor_col = 3;
        ed.handle_key(key(KeyCode::Enter));
        assert_eq!(ed.lines, vec!["hel", "lo"]);
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn backspace_middle_of_line() {
        let mut ed = Editor::new("hello".into());
        ed.cursor_col = 3;
        ed.handle_key(key(KeyCode::Backspace));
        assert_eq!(ed.content(), "helo");
        assert_eq!(ed.cursor_col, 2);
    }

    #[test]
    fn backspace_start_of_line_joins() {
        let mut ed = Editor::new("hello\nworld".into());
        ed.cursor_row = 1;
        ed.cursor_col = 0;
        ed.handle_key(key(KeyCode::Backspace));
        assert_eq!(ed.content(), "helloworld");
        assert_eq!(ed.cursor_row, 0);
        assert_eq!(ed.cursor_col, 5);
    }

    #[test]
    fn delete_forward() {
        let mut ed = Editor::new("hello".into());
        ed.cursor_col = 2;
        ed.handle_key(key(KeyCode::Delete));
        assert_eq!(ed.content(), "helo");
        assert_eq!(ed.cursor_col, 2);
    }

    #[test]
    fn delete_forward_at_end_joins() {
        let mut ed = Editor::new("ab\ncd".into());
        ed.cursor_col = 2;
        ed.handle_key(key(KeyCode::Delete));
        assert_eq!(ed.content(), "abcd");
    }

    #[test]
    fn arrow_navigation() {
        let mut ed = Editor::new("abc\ndef".into());
        ed.handle_key(key(KeyCode::Right));
        assert_eq!(ed.cursor_col, 1);
        ed.handle_key(key(KeyCode::Down));
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 1);
        ed.handle_key(key(KeyCode::Left));
        assert_eq!(ed.cursor_col, 0);
        ed.handle_key(key(KeyCode::Up));
        assert_eq!(ed.cursor_row, 0);
    }

    #[test]
    fn cursor_clamps_to_line_length() {
        let mut ed = Editor::new("long line\nhi".into());
        ed.cursor_col = 9; // end of "long line"
        ed.handle_key(key(KeyCode::Down));
        assert_eq!(ed.cursor_col, 2); // clamped to "hi" length
    }

    #[test]
    fn left_wraps_to_previous_line() {
        let mut ed = Editor::new("abc\ndef".into());
        ed.cursor_row = 1;
        ed.cursor_col = 0;
        ed.handle_key(key(KeyCode::Left));
        assert_eq!(ed.cursor_row, 0);
        assert_eq!(ed.cursor_col, 3);
    }

    #[test]
    fn right_wraps_to_next_line() {
        let mut ed = Editor::new("abc\ndef".into());
        ed.cursor_col = 3;
        ed.handle_key(key(KeyCode::Right));
        assert_eq!(ed.cursor_row, 1);
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn ctrl_a_moves_to_start() {
        let mut ed = Editor::new("hello".into());
        ed.cursor_col = 3;
        ed.handle_key(ctrl_key('a'));
        assert_eq!(ed.cursor_col, 0);
    }

    #[test]
    fn ctrl_e_moves_to_end() {
        let mut ed = Editor::new("hello".into());
        ed.handle_key(ctrl_key('e'));
        assert_eq!(ed.cursor_col, 5);
    }

    #[test]
    fn ctrl_k_kills_to_end() {
        let mut ed = Editor::new("hello world".into());
        ed.cursor_col = 5;
        ed.handle_key(ctrl_key('k'));
        assert_eq!(ed.content(), "hello");
    }

    #[test]
    fn ctrl_k_at_end_joins_next_line() {
        let mut ed = Editor::new("abc\ndef".into());
        ed.cursor_col = 3;
        ed.handle_key(ctrl_key('k'));
        assert_eq!(ed.content(), "abcdef");
    }

    #[test]
    fn ctrl_d_deletes_char() {
        let mut ed = Editor::new("hello".into());
        ed.cursor_col = 1;
        ed.handle_key(ctrl_key('d'));
        assert_eq!(ed.content(), "hllo");
    }

    #[test]
    fn tab_inserts_spaces() {
        let mut ed = Editor::new("".into());
        ed.handle_key(key(KeyCode::Tab));
        assert_eq!(ed.content(), "    ");
        assert_eq!(ed.cursor_col, 4);
    }

    #[test]
    fn content_roundtrip() {
        let original = "# Hello\n\nSome text\n- item";
        let ed = Editor::new(original.into());
        assert_eq!(ed.content(), original);
    }

    #[test]
    fn multibyte_char_navigation() {
        // em dash is 3 bytes but 1 character
        let mut ed = Editor::new("a—b".into());
        assert_eq!(ed.line_char_len(0), 3); // 3 chars: a, —, b
        ed.handle_key(key(KeyCode::Right)); // past 'a'
        assert_eq!(ed.cursor_col, 1);
        ed.handle_key(key(KeyCode::Right)); // past '—'
        assert_eq!(ed.cursor_col, 2);
        ed.handle_key(key(KeyCode::Right)); // past 'b'
        assert_eq!(ed.cursor_col, 3);
    }

    #[test]
    fn multibyte_char_insert() {
        let mut ed = Editor::new("a—b".into());
        ed.cursor_col = 2; // after the em dash
        ed.handle_key(key(KeyCode::Char('X')));
        assert_eq!(ed.content(), "a—Xb");
    }

    #[test]
    fn multibyte_char_backspace() {
        let mut ed = Editor::new("a—b".into());
        ed.cursor_col = 2; // after the em dash
        ed.handle_key(key(KeyCode::Backspace));
        assert_eq!(ed.content(), "ab");
        assert_eq!(ed.cursor_col, 1);
    }

    #[test]
    fn multibyte_char_delete() {
        let mut ed = Editor::new("a—b".into());
        ed.cursor_col = 1; // on the em dash
        ed.handle_key(key(KeyCode::Delete));
        assert_eq!(ed.content(), "ab");
        assert_eq!(ed.cursor_col, 1);
    }

    #[test]
    fn multibyte_ctrl_k() {
        let mut ed = Editor::new("hello — world".into());
        ed.cursor_col = 6; // after "hello "
        ed.handle_key(ctrl_key('k'));
        assert_eq!(ed.content(), "hello ");
    }
}
