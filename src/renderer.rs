use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, HeadingLevel};
use ratatui::prelude::*;
use ratatui::text::{Line, Span};

/// Render markdown content into styled ratatui Lines for view mode.
pub fn render_markdown(content: &str) -> Vec<Line<'static>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();

    let mut in_heading: Option<HeadingLevel> = None;
    let mut in_bold = false;
    let mut in_italic = false;
    let mut in_code_block = false;
    let mut in_link = false;
    let mut link_url: Option<String> = None;
    let mut in_image = false;
    let mut image_url: Option<String> = None;
    let mut list_stack: Vec<ListType> = Vec::new();
    let mut in_block_quote = false;
    let mut code_block_lines: Vec<String> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    in_heading = Some(level);
                    // Add heading prefix
                    let prefix = match level {
                        HeadingLevel::H1 => "  ",
                        HeadingLevel::H2 => "  ",
                        HeadingLevel::H3 => "  ",
                        _ => "  ",
                    };
                    current_spans.push(Span::styled(
                        prefix.to_string(),
                        heading_style(level),
                    ));
                }
                Tag::Strong => in_bold = true,
                Tag::Emphasis => in_italic = true,
                Tag::CodeBlock(_) => {
                    in_code_block = true;
                    code_block_lines.clear();
                }
                Tag::Link { dest_url, .. } => {
                    in_link = true;
                    link_url = Some(dest_url.to_string());
                }
                Tag::Image { dest_url, .. } => {
                    in_image = true;
                    image_url = Some(dest_url.to_string());
                }
                Tag::List(start_num) => {
                    if let Some(start) = start_num {
                        list_stack.push(ListType::Ordered(start as usize));
                    } else {
                        list_stack.push(ListType::Unordered);
                    }
                }
                Tag::Item => {
                    let depth = list_stack.len();
                    let indent = "  ".repeat(depth.saturating_sub(1));
                    let marker = match list_stack.last_mut() {
                        Some(ListType::Ordered(n)) => {
                            let s = format!("{}{}. ", indent, n);
                            *n += 1;
                            s
                        }
                        Some(ListType::Unordered) => {
                            let bullet = if depth % 2 == 1 { "●" } else { "○" };
                            format!("{}{} ", indent, bullet)
                        }
                        None => "  ".to_string(),
                    };
                    current_spans.push(Span::styled(
                        marker,
                        Style::default().fg(Color::Cyan),
                    ));
                }
                Tag::BlockQuote(_) => {
                    in_block_quote = true;
                }
                Tag::Paragraph => {}
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    let level = in_heading.unwrap_or(HeadingLevel::H1);
                    let style = heading_style(level);
                    let styled_spans: Vec<Span<'static>> = current_spans
                        .drain(..)
                        .map(|s| Span::styled(s.content.to_string(), style))
                        .collect();
                    let heading_text_len: usize = styled_spans
                        .iter()
                        .map(|s| s.content.len())
                        .sum();
                    lines.push(Line::from(styled_spans));
                    // Underline for h1/h2
                    match level {
                        HeadingLevel::H1 => {
                            lines.push(Line::from(Span::styled(
                                "─".repeat(heading_text_len.max(30)),
                                heading_style(HeadingLevel::H1),
                            )));
                        }
                        HeadingLevel::H2 => {
                            lines.push(Line::from(Span::styled(
                                "─".repeat(heading_text_len.max(30)),
                                heading_style(HeadingLevel::H2),
                            )));
                        }
                        _ => {}
                    }
                    lines.push(Line::from(""));
                    in_heading = None;
                }
                TagEnd::Strong => in_bold = false,
                TagEnd::Emphasis => in_italic = false,
                TagEnd::CodeBlock => {
                    for (i, code_line) in code_block_lines.drain(..).enumerate() {
                        let line_num = format!("{:>3} ", i + 1);
                        lines.push(Line::from(vec![
                            Span::styled(
                                "  │ ".to_string(),
                                Style::default().fg(Color::Cyan),
                            ),
                            Span::styled(
                                line_num,
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                code_line,
                                Style::default().fg(Color::Green),
                            ),
                        ]));
                    }
                    lines.push(Line::from(""));
                    in_code_block = false;
                }
                TagEnd::Link => {
                    in_link = false;
                    if let Some(url) = link_url.take() {
                        current_spans.push(Span::styled(
                            format!(" ({})", url),
                            Style::default().fg(Color::Rgb(80, 140, 80)),
                        ));
                    }
                }
                TagEnd::Image => {
                    in_image = false;
                    if let Some(url) = image_url.take() {
                        current_spans.push(Span::styled(
                            format!(" [img: {}]", url),
                            Style::default().fg(Color::Rgb(140, 110, 180)),
                        ));
                    }
                }
                TagEnd::List(_) => {
                    list_stack.pop();
                    if list_stack.is_empty() {
                        lines.push(Line::from(""));
                    }
                }
                TagEnd::Item => {
                    flush_spans(&mut current_spans, &mut lines);
                }
                TagEnd::BlockQuote(_) => {
                    in_block_quote = false;
                }
                TagEnd::Paragraph => {
                    if in_block_quote {
                        let mut bq_spans = vec![Span::styled(
                            "  ▐ ".to_string(),
                            Style::default().fg(Color::Blue),
                        )];
                        bq_spans.extend(current_spans.drain(..));
                        lines.push(Line::from(bq_spans));
                    } else {
                        flush_spans(&mut current_spans, &mut lines);
                    }
                    lines.push(Line::from(""));
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    for line in text.lines() {
                        code_block_lines.push(line.to_string());
                    }
                } else {
                    let style = text_style(in_heading, in_bold, in_italic, in_link, in_image, in_block_quote);
                    current_spans.push(Span::styled(text.to_string(), style));
                }
            }
            Event::Code(code) => {
                current_spans.push(Span::styled(
                    format!("`{}`", code),
                    Style::default()
                        .fg(Color::Rgb(255, 180, 80)),
                ));
            }
            Event::SoftBreak => {
                // Soft break = space between lines in a paragraph
                current_spans.push(Span::raw(" "));
            }
            Event::HardBreak => {
                flush_spans(&mut current_spans, &mut lines);
            }
            Event::Rule => {
                lines.push(Line::from(Span::styled(
                    "  ──────────────────────────────────────────────────────────",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
            }
            _ => {}
        }
    }

    flush_spans(&mut current_spans, &mut lines);

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty file)",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )));
    }

    lines
}

enum ListType {
    Ordered(usize),
    Unordered,
}

fn flush_spans(spans: &mut Vec<Span<'static>>, lines: &mut Vec<Line<'static>>) {
    if !spans.is_empty() {
        // Add left padding
        let mut padded = vec![Span::raw("  ")];
        padded.extend(spans.drain(..));
        lines.push(Line::from(padded));
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => Style::default()
            .fg(Color::Rgb(210, 180, 230)),
        HeadingLevel::H2 => Style::default()
            .fg(Color::Rgb(170, 190, 220)),
        HeadingLevel::H3 => Style::default()
            .fg(Color::Rgb(160, 200, 200)),
        HeadingLevel::H4 => Style::default()
            .fg(Color::Rgb(170, 200, 170)),
        _ => Style::default()
            .fg(Color::Rgb(190, 190, 190)),
    }
}

fn text_style(
    in_heading: Option<HeadingLevel>,
    in_bold: bool,
    in_italic: bool,
    in_link: bool,
    in_image: bool,
    in_block_quote: bool,
) -> Style {
    if let Some(level) = in_heading {
        return heading_style(level);
    }

    let mut style = Style::default().fg(Color::Rgb(220, 220, 230)).bg(Color::Reset);

    if in_bold {
        style = style.fg(Color::Yellow);
    }
    if in_italic {
        style = style.fg(Color::Rgb(180, 200, 255));
    }
    if in_link {
        style = style.fg(Color::Green);
    }
    if in_image {
        style = style.fg(Color::Magenta);
    }
    if in_block_quote {
        style = style.fg(Color::Blue);
    }

    style
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: extract plain text from rendered lines (strips styling)
    fn plain_text(lines: &[Line]) -> Vec<String> {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect()
    }

    /// Helper: check if any line contains the given substring
    fn any_line_contains(lines: &[Line], needle: &str) -> bool {
        plain_text(lines).iter().any(|l| l.contains(needle))
    }

    #[test]
    fn empty_content_shows_placeholder() {
        let lines = render_markdown("");
        assert!(any_line_contains(&lines, "(empty file)"));
    }

    #[test]
    fn h1_heading_renders_with_underline() {
        let lines = render_markdown("# Hello World");
        let text = plain_text(&lines);
        assert!(text.iter().any(|l| l.contains("Hello World")), "H1 text missing");
        assert!(text.iter().any(|l| l.contains("─")), "H1 underline missing");
    }

    #[test]
    fn h2_heading_renders_with_underline() {
        let lines = render_markdown("## Section");
        let text = plain_text(&lines);
        assert!(text.iter().any(|l| l.contains("Section")), "H2 text missing");
        assert!(text.iter().any(|l| l.contains("─")), "H2 underline missing");
    }

    #[test]
    fn h3_heading_renders_without_underline() {
        let lines = render_markdown("### Subsection");
        let text = plain_text(&lines);
        assert!(text.iter().any(|l| l.contains("Subsection")), "H3 text missing");
        assert!(!text.iter().any(|l| l.contains("─")), "H3 should not have underline");
    }

    #[test]
    fn unordered_list_has_bullets() {
        let lines = render_markdown("- Item one\n- Item two\n- Item three");
        let text = plain_text(&lines);
        let bullet_count = text.iter().filter(|l| l.contains("●")).count();
        assert_eq!(bullet_count, 3, "Expected 3 bullet markers, got {}", bullet_count);
    }

    #[test]
    fn ordered_list_has_numbers() {
        let lines = render_markdown("1. First\n2. Second\n3. Third");
        let text = plain_text(&lines);
        assert!(text.iter().any(|l| l.contains("1.")), "Missing number 1");
        assert!(text.iter().any(|l| l.contains("2.")), "Missing number 2");
        assert!(text.iter().any(|l| l.contains("3.")), "Missing number 3");
    }

    #[test]
    fn code_block_renders_with_line_numbers() {
        let lines = render_markdown("```\nfn main() {\n    println!(\"hi\");\n}\n```");
        let text = plain_text(&lines);
        assert!(text.iter().any(|l| l.contains("fn main()")), "Code content missing");
        assert!(text.iter().any(|l| l.contains("│")), "Line number gutter missing");
    }

    #[test]
    fn inline_code_renders() {
        let lines = render_markdown("Use `cargo build` to compile");
        assert!(any_line_contains(&lines, "cargo build"));
    }

    #[test]
    fn bold_text_renders() {
        let lines = render_markdown("This is **bold** text");
        assert!(any_line_contains(&lines, "bold"));
        // Check that bold span has BOLD modifier
        let bold_span = lines.iter()
            .flat_map(|l| l.spans.iter())
            .find(|s| s.content.contains("bold"));
        assert!(bold_span.is_some(), "Bold span not found");
        assert_eq!(
            bold_span.unwrap().style.fg,
            Some(Color::Yellow),
            "Bold text should have yellow color"
        );
    }

    #[test]
    fn italic_text_renders() {
        let lines = render_markdown("This is *italic* text");
        let italic_span = lines.iter()
            .flat_map(|l| l.spans.iter())
            .find(|s| s.content.contains("italic"));
        assert!(italic_span.is_some(), "Italic span not found");
        assert_eq!(
            italic_span.unwrap().style.fg,
            Some(Color::Rgb(180, 200, 255)),
            "Italic text should have light blue color"
        );
    }

    #[test]
    fn link_renders_with_url() {
        let lines = render_markdown("Check [Rust](https://rust-lang.org) out");
        let text = plain_text(&lines);
        assert!(text.iter().any(|l| l.contains("Rust")), "Link text missing");
        assert!(text.iter().any(|l| l.contains("rust-lang.org")), "Link URL missing");
    }

    #[test]
    fn image_renders_as_text() {
        let lines = render_markdown("![Logo](https://example.com/logo.png)");
        assert!(any_line_contains(&lines, "img:"));
        assert!(any_line_contains(&lines, "logo.png"));
    }

    #[test]
    fn blockquote_has_marker() {
        let lines = render_markdown("> This is a quote");
        let text = plain_text(&lines);
        assert!(text.iter().any(|l| l.contains("▐")), "Blockquote marker missing");
        assert!(any_line_contains(&lines, "This is a quote"));
    }

    #[test]
    fn horizontal_rule_renders() {
        let lines = render_markdown("above\n\n---\n\nbelow");
        let text = plain_text(&lines);
        assert!(text.iter().any(|l| l.contains("───")), "Horizontal rule missing");
    }

    #[test]
    fn headings_use_distinct_colors() {
        let h1 = render_markdown("# H1");
        let h2 = render_markdown("## H2");
        let h3 = render_markdown("### H3");

        let h1_color = h1.first().and_then(|l| l.spans.first()).map(|s| s.style.fg);
        let h2_color = h2.first().and_then(|l| l.spans.first()).map(|s| s.style.fg);
        let h3_color = h3.first().and_then(|l| l.spans.first()).map(|s| s.style.fg);

        assert_ne!(h1_color, h2_color, "H1 and H2 should have different colors");
        assert_ne!(h2_color, h3_color, "H2 and H3 should have different colors");
    }
}
