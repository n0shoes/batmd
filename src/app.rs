use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;

use crate::editor::Editor;
use crate::ui;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    View,
    Edit,
}

pub struct App {
    pub mode: Mode,
    pub editor: Editor,
    pub file_path: PathBuf,
    pub scroll_offset: usize,
    pub should_quit: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(file_path: PathBuf) -> AppResult<Self> {
        let content = std::fs::read_to_string(&file_path)?;
        let editor = Editor::new(content);

        Ok(Self {
            mode: Mode::View,
            editor,
            file_path,
            scroll_offset: 0,
            should_quit: false,
            status_message: None,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> AppResult<()> {
        loop {
            terminal.draw(|frame| ui::draw(frame, self))?;

            if self.should_quit {
                break;
            }

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl-C or Ctrl-Q always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('q') => {
                    self.should_quit = true;
                    return;
                }
                _ => {}
            }
        }

        match self.mode {
            Mode::View => self.handle_view_key(key),
            Mode::Edit => self.handle_edit_key(key),
        }
    }

    fn handle_view_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('e') | KeyCode::Char('i') => {
                self.mode = Mode::Edit;
                self.status_message = Some("-- EDIT --".into());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_down(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_up(1);
            }
            KeyCode::PageDown => {
                self.scroll_down(20);
            }
            KeyCode::PageUp => {
                self.scroll_up(20);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.scroll_offset = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                let total = self.editor.line_count();
                self.scroll_offset = total.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn handle_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::View;
                self.status_message = None;
                self.save();
            }
            _ => {
                self.editor.handle_key(key);
            }
        }
    }

    fn scroll_down(&mut self, n: usize) {
        let max = self.editor.line_count().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + n).min(max);
    }

    fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    fn save(&mut self) {
        let content = self.editor.content();
        match std::fs::write(&self.file_path, &content) {
            Ok(_) => self.status_message = Some("Saved".into()),
            Err(e) => self.status_message = Some(format!("Save error: {}", e)),
        }
    }
}
