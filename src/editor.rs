use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct Editor {
    pub lines: Vec<String>,
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

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                // Ctrl-A: move to beginning of line
                KeyCode::Char('a') => {
                    self.cursor_col = 0;
                }
                // Ctrl-E: move to end of line
                KeyCode::Char('e') => {
                    self.cursor_col = self.current_line().len();
                }
                // Ctrl-K: kill from cursor to end of line
                KeyCode::Char('k') => {
                    let row = self.cursor_row;
                    let col = self.cursor_col;
                    if col < self.lines[row].len() {
                        self.lines[row].truncate(col);
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
            KeyCode::End => self.cursor_col = self.current_line().len(),
            KeyCode::Tab => {
                // Insert 4 spaces
                for _ in 0..4 {
                    self.insert_char(' ');
                }
            }
            _ => {}
        }
    }

    fn insert_char(&mut self, c: char) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        self.lines[row].insert(col, c);
        self.cursor_col += 1;
    }

    fn insert_newline(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let remainder = self.lines[row][col..].to_string();
        self.lines[row].truncate(col);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.lines.insert(self.cursor_row, remainder);
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            self.lines[self.cursor_row].remove(self.cursor_col);
        } else if self.cursor_row > 0 {
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current_line);
        }
    }

    fn delete_forward(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        if col < self.lines[row].len() {
            self.lines[row].remove(col);
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
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    fn move_right(&mut self) {
        if self.cursor_col < self.current_line().len() {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.current_line().len());
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.current_line().len());
        }
    }
}
