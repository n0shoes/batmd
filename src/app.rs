use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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
    /// Typing a search query in view mode.
    Search,
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
    pub read_only: bool,
    last_known_modified: Option<SystemTime>,
    watcher: Option<FileWatcher>,
    // Search state
    pub search_query: String,
    pub search_input: String,
    /// (line_index, byte_start, byte_end) in rendered lines
    pub search_matches: Vec<(usize, usize, usize)>,
    pub search_match_idx: Option<usize>,
}

impl App {
    pub fn new(file_path: PathBuf) -> AppResult<Self> {
        let content = std::fs::read_to_string(&file_path)?;
        let editor = Editor::new(content);
        let highlighter = Highlighter::new();
        let meta = std::fs::metadata(&file_path)?;
        let modified = meta.modified().ok();
        let read_only = meta.permissions().readonly();

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
            read_only,
            last_known_modified: modified,
            watcher,
            search_query: String::new(),
            search_input: String::new(),
            search_matches: Vec::new(),
            search_match_idx: None,
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
        // Only handle key press events (ignore release/repeat for Kitty protocol terminals)
        if key.kind != KeyEventKind::Press {
            return;
        }

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
            Mode::Search => self.handle_search_key(key),
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
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.clear_search();
                    self.status_message = None;
                }
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('e') | KeyCode::Char('i') => {
                if self.read_only {
                    self.status_message = Some("Read-only file".into());
                    return;
                }
                // Approximate source line from view scroll position
                if self.rendered_line_count > 0 {
                    let ratio = self.scroll_offset as f64 / self.rendered_line_count as f64;
                    let target_row = (ratio * self.editor.line_count() as f64) as usize;
                    self.editor.cursor_row = target_row.min(self.editor.line_count().saturating_sub(1));
                    self.editor.cursor_col = 0;
                }
                self.mode = Mode::Edit;
                self.clear_search();
                self.status_message = Some("-- EDIT --".into());
            }
            KeyCode::Char('/') => {
                self.search_input.clear();
                self.mode = Mode::Search;
                self.status_message = Some("/ ".into());
            }
            KeyCode::Char('n') => {
                self.search_next();
            }
            KeyCode::Char('N') => {
                self.search_prev();
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

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.search_query = self.search_input.clone();
                self.mode = Mode::View;
                // Matches will be computed after next render (needs rendered lines)
                // Trigger a find on current rendered content
                self.find_matches();
                if self.search_matches.is_empty() {
                    self.status_message = Some(format!("Pattern not found: {}", self.search_query));
                } else {
                    self.search_next();
                }
            }
            KeyCode::Esc => {
                self.mode = Mode::View;
                self.status_message = None;
            }
            KeyCode::Backspace => {
                self.search_input.pop();
                self.status_message = Some(format!("/ {}", self.search_input));
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
                self.status_message = Some(format!("/ {}", self.search_input));
            }
            _ => {}
        }
    }

    fn find_matches(&mut self) {
        self.search_matches.clear();
        self.search_match_idx = None;
        if self.search_query.is_empty() {
            return;
        }
        let query_lower = self.search_query.to_lowercase();
        let content = self.editor.content();
        let rendered = crate::renderer::render_markdown(&content);
        for (line_idx, line) in rendered.iter().enumerate() {
            let plain: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            let plain_lower = plain.to_lowercase();
            let mut start = 0;
            while let Some(pos) = plain_lower[start..].find(&query_lower) {
                let abs_pos = start + pos;
                self.search_matches.push((line_idx, abs_pos, abs_pos + query_lower.len()));
                start = abs_pos + 1;
            }
        }
    }

    fn search_next(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        let idx = match self.search_match_idx {
            Some(i) => (i + 1) % self.search_matches.len(),
            None => {
                // Find first match at or after current scroll position
                self.search_matches.iter()
                    .position(|(line, _, _)| *line >= self.scroll_offset)
                    .unwrap_or(0)
            }
        };
        self.search_match_idx = Some(idx);
        self.scroll_to_match(idx);
    }

    fn search_prev(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        let idx = match self.search_match_idx {
            Some(0) => self.search_matches.len() - 1,
            Some(i) => i - 1,
            None => {
                self.search_matches.iter()
                    .rposition(|(line, _, _)| *line <= self.scroll_offset)
                    .unwrap_or(self.search_matches.len() - 1)
            }
        };
        self.search_match_idx = Some(idx);
        self.scroll_to_match(idx);
    }

    fn scroll_to_match(&mut self, idx: usize) {
        let (line, _, _) = self.search_matches[idx];
        let total = self.search_matches.len();
        // Center the match in the viewport
        let half = self.view_height / 2;
        self.scroll_offset = line.saturating_sub(half);
        let max = self.max_view_scroll();
        self.scroll_offset = self.scroll_offset.min(max);
        self.status_message = Some(format!(
            "[{}/{}] {}",
            idx + 1,
            total,
            self.search_query,
        ));
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_matches.clear();
        self.search_match_idx = None;
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
