use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::app::{App, Mode};
use crate::renderer;

pub fn draw(frame: &mut Frame, app: &App) {
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
        Mode::Edit => draw_edit(frame, app, content_area),
    }

    draw_status_bar(frame, app, status_area);
}

fn draw_view(frame: &mut Frame, app: &App, area: Rect) {
    let content = app.editor.content();
    let rendered = renderer::render_markdown(&content);
    let total_lines = rendered.len();

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

fn draw_edit(frame: &mut Frame, app: &App, area: Rect) {
    let visible_height = area.height as usize;
    let cursor_row = app.editor.cursor_row;

    // Calculate scroll to keep cursor visible
    let scroll = if cursor_row >= visible_height {
        cursor_row - visible_height + 1
    } else {
        0
    };

    // Build lines with line numbers (raw markdown, syntax-highlighted later)
    let mut lines: Vec<Line<'static>> = Vec::new();
    for (i, line) in app.editor.lines.iter().enumerate() {
        let line_num = format!(" {:>3} │ ", i + 1);
        let num_style = if i == cursor_row {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let content_style = if i == cursor_row {
            Style::default().fg(Color::White).bg(Color::Rgb(35, 35, 50))
        } else {
            Style::default().fg(Color::Rgb(200, 200, 200))
        };

        lines.push(Line::from(vec![
            Span::styled(line_num, num_style),
            Span::styled(line.clone(), content_style),
        ]));
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

    // Position cursor
    let gutter_width = 7; // " NNN │ "
    let cursor_x = area.x + gutter_width + app.editor.cursor_col as u16;
    let cursor_y = area.y + (cursor_row - scroll) as u16;
    if cursor_y < area.y + area.height {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mode_str = match app.mode {
        Mode::View => " VIEW ",
        Mode::Edit => " EDIT ",
    };

    let mode_style = match app.mode {
        Mode::View => Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        Mode::Edit => Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    };

    let file_name = app
        .file_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let position = match app.mode {
        Mode::Edit => format!(
            " Ln {}, Col {} ",
            app.editor.cursor_row + 1,
            app.editor.cursor_col + 1
        ),
        Mode::View => format!(" Line {} ", app.scroll_offset + 1),
    };

    let help = match app.mode {
        Mode::View => " q:quit  e/i:edit  j/k:scroll ",
        Mode::Edit => " Esc:view  C-a:home  C-e:end  C-k:kill ",
    };

    let status = Line::from(vec![
        Span::styled(mode_str, mode_style),
        Span::styled(
            format!(" {} ", file_name),
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(40, 40, 55)),
        ),
        Span::styled(
            help,
            Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Rgb(40, 40, 55)),
        ),
        Span::styled(
            position,
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Rgb(40, 40, 55)),
        ),
    ]);

    let bar = Paragraph::new(status).style(
        Style::default().bg(Color::Rgb(40, 40, 55)),
    );

    frame.render_widget(bar, area);
}
