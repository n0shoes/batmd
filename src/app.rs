use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;

use crate::editor::Editor;
use crate::highlight::Highlighter;
use crate::ui;
use crate::watcher::FileWatcher;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    View,
    Edit,
    /// File changed externally while user has edits. Prompt to resolve.
    Conflict,
}

pub struct App {
    pub mode: Mode,
    pub editor: Editor,
    pub highlighter: Highlighter,
    pub file_path: PathBuf,
    pub scroll_offset: usize,
    pub edit_scroll: usize,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub rendered_line_count: usize,
    pub view_height: usize,
    pub file_changed_externally: bool,
    pub has_unsaved_edits: bool,
    last_known_modified: Option<SystemTime>,
    watcher: Option<FileWatcher>,
}

impl App {
    pub fn new(file_path: PathBuf) -> AppResult<Self> {
        let content = std::fs::read_to_string(&file_path)?;
        let editor = Editor::new(content);
        let highlighter = Highlighter::new();
        let modified = std::fs::metadata(&file_path)?.modified().ok();

        let watcher = FileWatcher::new(&file_path).ok();

        Ok(Self {
            mode: Mode::View,
            editor,
            highlighter,
            file_path,
            scroll_offset: 0,
            edit_scroll: 0,
            should_quit: false,
            status_message: None,
            rendered_line_count: 0,
            view_height: 0,
            file_changed_externally: false,
            has_unsaved_edits: false,
            last_known_modified: modified,
            watcher,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> AppResult<()> {
        loop {
            terminal.draw(|frame| ui::draw(frame, &mut *self))?;

            if self.should_quit {
                break;
            }

            // Check for external file changes
            self.check_file_changes();

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }

    fn check_file_changes(&mut self) {
        let changed = self.watcher.as_ref().is_some_and(|w| w.poll_change());
        if !changed {
            return;
        }

        // Verify it's a real content change by checking mtime
        if let Ok(meta) = std::fs::metadata(&self.file_path) {
            if let Ok(modified) = meta.modified() {
                if self.last_known_modified.is_some_and(|m| modified > m) {
                    self.file_changed_externally = true;
                    if self.mode == Mode::View && !self.has_unsaved_edits {
                        // Auto-reload in view mode if no edits pending
                        self.reload();
                    }
                }
            }
        }
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
            Mode::Conflict => self.handle_conflict_key(key),
        }
    }

    fn handle_view_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('t') => { self.scroll_to_top(); return; }
                KeyCode::Char('b') => { self.scroll_to_bottom(); return; }
                _ => {}
            }
        }
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('e') | KeyCode::Char('i') => {
                // Approximate source line from view scroll position
                if self.rendered_line_count > 0 {
                    let ratio = self.scroll_offset as f64 / self.rendered_line_count as f64;
                    let target_row = (ratio * self.editor.line_count() as f64) as usize;
                    self.editor.cursor_row = target_row.min(self.editor.line_count().saturating_sub(1));
                    self.editor.cursor_col = 0;
                }
                self.mode = Mode::Edit;
                self.status_message = Some("-- EDIT --".into());
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.reload();
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
                self.scroll_to_top();
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll_to_bottom();
            }
            _ => {}
        }
    }

    fn handle_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                if self.file_changed_externally && self.has_unsaved_edits {
                    self.mode = Mode::Conflict;
                    self.status_message = Some(
                        "File changed externally! [S]ave yours / [R]eload theirs / [Esc] cancel".into()
                    );
                } else {
                    self.mode = Mode::View;
                    if self.has_unsaved_edits {
                        self.save();
                    } else {
                        self.status_message = None;
                    }
                }
            }
            _ => {
                if self.editor.handle_key(key) {
                    self.has_unsaved_edits = true;
                }
            }
        }
    }

    fn handle_conflict_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Force save user's version
                self.force_save();
                self.mode = Mode::View;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Reload external version, discard edits
                self.reload();
                self.mode = Mode::View;
            }
            KeyCode::Esc => {
                // Go back to edit mode
                self.mode = Mode::Edit;
                self.status_message = Some("-- EDIT --".into());
            }
            _ => {}
        }
    }

    /// Update edit scroll to keep cursor visible with context lines.
    pub fn update_edit_scroll(&mut self, visible_height: usize) {
        let cursor_row = self.editor.cursor_row;
        let total_lines = self.editor.line_count();
        let context = 3.min(visible_height / 4);

        // Scroll down if cursor is below viewport
        if cursor_row >= self.edit_scroll + visible_height.saturating_sub(context) {
            self.edit_scroll = cursor_row.saturating_sub(visible_height.saturating_sub(context + 1));
        }
        // Scroll up if cursor is above viewport
        if cursor_row < self.edit_scroll + context {
            self.edit_scroll = cursor_row.saturating_sub(context);
        }
        // Never scroll past the content — last line should stay at bottom of viewport
        let max_scroll = total_lines.saturating_sub(visible_height);
        self.edit_scroll = self.edit_scroll.min(max_scroll);
    }

    fn scroll_down(&mut self, n: usize) {
        let max = self.max_view_scroll();
        self.scroll_offset = (self.scroll_offset + n).min(max);
    }

    fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    fn max_view_scroll(&self) -> usize {
        self.rendered_line_count.saturating_sub(self.view_height)
    }

    fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_view_scroll();
    }

    fn save(&mut self) {
        let content = self.editor.content();
        match std::fs::write(&self.file_path, &content) {
            Ok(_) => {
                self.status_message = Some("Saved".into());
                self.has_unsaved_edits = false;
                self.file_changed_externally = false;
                self.update_known_modified();
            }
            Err(e) => self.status_message = Some(format!("Save error: {}", e)),
        }
    }

    fn force_save(&mut self) {
        let content = self.editor.content();
        match std::fs::write(&self.file_path, &content) {
            Ok(_) => {
                self.status_message = Some("Saved (overwrote external changes)".into());
                self.has_unsaved_edits = false;
                self.file_changed_externally = false;
                self.update_known_modified();
            }
            Err(e) => self.status_message = Some(format!("Save error: {}", e)),
        }
    }

    fn reload(&mut self) {
        match std::fs::read_to_string(&self.file_path) {
            Ok(content) => {
                self.editor = Editor::new(content);
                self.has_unsaved_edits = false;
                self.file_changed_externally = false;
                self.scroll_offset = 0;
                self.edit_scroll = 0;
                self.update_known_modified();
                self.status_message = Some("Reloaded".into());
            }
            Err(e) => self.status_message = Some(format!("Reload error: {}", e)),
        }
    }

    fn update_known_modified(&mut self) {
        if let Ok(meta) = std::fs::metadata(&self.file_path) {
            self.last_known_modified = meta.modified().ok();
        }
    }
}
