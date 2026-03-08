use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use unicode_width::UnicodeWidthChar;

use crate::app::{App, Mode};
use crate::renderer;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    let status_area = chunks[1];

    // Add left gutter for breathing room
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2), // Left gutter
            Constraint::Min(1),   // Content
        ])
        .split(chunks[0]);
    let content_area = h_chunks[1];

    match app.mode {
        Mode::View => draw_view(frame, app, content_area),
        Mode::Edit | Mode::Conflict => draw_edit(frame, app, content_area),
    }

    draw_status_bar(frame, app, status_area);
}

fn draw_view(frame: &mut Frame, app: &mut App, area: Rect) {
    let content = app.editor.content();
    let rendered = renderer::render_markdown(&content);
    let total_lines = rendered.len();

    // Store for scroll calculations
    app.rendered_line_count = total_lines;
    app.view_height = area.height as usize;

    let text = ratatui::text::Text::from(rendered);

    let paragraph = Paragraph::new(text)
        .scroll((app.scroll_offset as u16, 0))
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default()),
        );

    frame.render_widget(paragraph, area);

    // Scrollbar
    if total_lines > area.height as usize {
        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(app.scroll_offset);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(Color::DarkGray)),
            area,
            &mut scrollbar_state,
        );
    }
}

fn draw_edit(frame: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height as usize;
    let cursor_row = app.editor.cursor_row;

    // Update scroll to keep cursor visible
    app.update_edit_scroll(visible_height);
    let scroll = app.edit_scroll;

    // Build lines with line numbers and syntax highlighting
    let mut lines: Vec<Line<'static>> = Vec::new();
    for (i, line) in app.editor.lines.iter().enumerate() {
        let line_num = format!(" {:>3} │ ", i + 1);
        let num_style = if i == cursor_row {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Syntax-highlighted content spans
        let mut content_spans = app.highlighter.highlight_line(line);

        // Dim highlight on current line (just brighten the line number, leave syntax colors)
        if content_spans.is_empty() {
            content_spans.push(Span::raw(""));
        }

        let mut all_spans = vec![Span::styled(line_num, num_style)];
        all_spans.extend(content_spans);
        lines.push(Line::from(all_spans));
    }

    let text = ratatui::text::Text::from(lines);
    let paragraph = Paragraph::new(text)
        .scroll((scroll as u16, 0))
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default()),
        );

    frame.render_widget(paragraph, area);

    // Position cursor — compute visual width of chars before cursor
    let gutter_width: u16 = 7; // " NNN │ "
    let visual_col: u16 = app.editor.current_line()
        .chars()
        .take(app.editor.cursor_col)
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(1) as u16)
        .sum();
    let cursor_x = area.x + gutter_width + visual_col;
    let cursor_y = area.y + (cursor_row - scroll) as u16;
    if cursor_y < area.y + area.height {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let bar_bg = Style::default().bg(Color::Rgb(40, 40, 55));

    let (mode_str, mode_style) = match app.mode {
        Mode::View => (" VIEW ", Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)),
        Mode::Edit => (" EDIT ", Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)),
        Mode::Conflict => (" CONFLICT ", Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)),
    };

    let file_name = app
        .file_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let position = match app.mode {
        Mode::Edit | Mode::Conflict => format!(
            " Ln {}, Col {} ",
            app.editor.cursor_row + 1,
            app.editor.cursor_col + 1
        ),
        Mode::View => format!(" Line {} ", app.scroll_offset + 1),
    };

    let help = match app.mode {
        Mode::View => {
            if app.file_changed_externally {
                " R:reload  q:quit  e/i:edit  j/k:scroll "
            } else {
                " q:quit  e/i:edit  j/k:scroll  R:reload "
            }
        }
        Mode::Edit => " Esc:view  C-a:home  C-e:end  C-k:kill  C-t:top  C-b:bottom ",
        Mode::Conflict => " [S]ave yours / [R]eload theirs / [Esc] back to edit ",
    };

    let mut spans = vec![
        Span::styled(mode_str, mode_style),
        Span::styled(
            format!(" {} ", file_name),
            Style::default().fg(Color::White).bg(Color::Rgb(40, 40, 55)),
        ),
    ];

    // External change indicator
    if app.file_changed_externally && app.mode != Mode::Conflict {
        spans.push(Span::styled(
            " ! CHANGED ",
            Style::default().fg(Color::Red).bg(Color::Rgb(40, 40, 55)),
        ));
    }

    // Unsaved edits indicator
    if app.has_unsaved_edits {
        spans.push(Span::styled(
            " [modified] ",
            Style::default().fg(Color::Yellow).bg(Color::Rgb(40, 40, 55)),
        ));
    }

    spans.push(Span::styled(
        help,
        Style::default().fg(Color::DarkGray).bg(Color::Rgb(40, 40, 55)),
    ));
    spans.push(Span::styled(
        position,
        Style::default().fg(Color::Cyan).bg(Color::Rgb(40, 40, 55)),
    ));

    let status = Line::from(spans);
    let bar = Paragraph::new(status).style(bar_bg);
    frame.render_widget(bar, area);
}
