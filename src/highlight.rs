use ratatui::prelude::*;
use ratatui::text::Span;

pub struct Highlighter;

impl Highlighter {
    pub fn new() -> Self {
        Self
    }

    /// Highlight a single line of raw markdown using simple pattern matching.
    /// Returns styled spans using ANSI colors for terminal compatibility.
    pub fn highlight_line(&self, line: &str) -> Vec<Span<'static>> {
        let trimmed = line.trim_start();

        // Headings
        if trimmed.starts_with('#') {
            let hashes = trimmed.chars().take_while(|c| *c == '#').count();
            if hashes <= 6 && trimmed.get(hashes..hashes + 1) == Some(" ") {
                let color = match hashes {
                    1 => Color::Magenta,
                    2 => Color::Blue,
                    3 => Color::Cyan,
                    _ => Color::Green,
                };
                return vec![Span::styled(line.to_string(), Style::default().fg(color))];
            }
        }

        // Code fence
        if trimmed.starts_with("```") {
            return vec![Span::styled(line.to_string(), Style::default().fg(Color::DarkGray))];
        }

        // Horizontal rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            return vec![Span::styled(line.to_string(), Style::default().fg(Color::DarkGray))];
        }

        // Blockquote
        if trimmed.starts_with('>') {
            return vec![Span::styled(line.to_string(), Style::default().fg(Color::Blue))];
        }

        // List items (unordered)
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            return highlight_inline(line, Some(Color::Cyan));
        }

        // List items (ordered)
        if trimmed.len() > 2 {
            let dot_pos = trimmed.find(". ");
            if let Some(pos) = dot_pos {
                if pos <= 3 && trimmed[..pos].chars().all(|c| c.is_ascii_digit()) {
                    return highlight_inline(line, Some(Color::Cyan));
                }
            }
        }

        // Regular line with inline highlighting
        highlight_inline(line, None)
    }
}

/// Highlight inline markdown elements: **bold**, *italic*, `code`, [links]
fn highlight_inline(line: &str, prefix_color: Option<Color>) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut buf = String::new();
    let default_fg = Color::White;
    let base_color = prefix_color.unwrap_or(default_fg);
    let has_prefix = prefix_color.is_some();

    while i < len {
        // Backtick inline code
        if chars[i] == '`' {
            let color = if has_prefix && spans.is_empty() { base_color } else { default_fg };
            flush_buf(&mut buf, &mut spans, color);
            let mut code = String::from('`');
            i += 1;
            while i < len && chars[i] != '`' {
                code.push(chars[i]);
                i += 1;
            }
            if i < len {
                code.push('`');
                i += 1;
            }
            spans.push(Span::styled(code, Style::default().fg(Color::Yellow)));
            continue;
        }

        // Bold **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            let color = if has_prefix && spans.is_empty() { base_color } else { default_fg };
            flush_buf(&mut buf, &mut spans, color);
            let mut bold = String::from("**");
            i += 2;
            while i < len && !(i + 1 < len && chars[i] == '*' && chars[i + 1] == '*') {
                bold.push(chars[i]);
                i += 1;
            }
            if i + 1 < len {
                bold.push_str("**");
                i += 2;
            }
            spans.push(Span::styled(bold, Style::default().fg(Color::Yellow)));
            continue;
        }

        // Italic *text*
        if chars[i] == '*' && (i + 1 < len && chars[i + 1] != ' ') {
            let color = if has_prefix && spans.is_empty() { base_color } else { default_fg };
            flush_buf(&mut buf, &mut spans, color);
            let mut italic = String::from('*');
            i += 1;
            while i < len && chars[i] != '*' {
                italic.push(chars[i]);
                i += 1;
            }
            if i < len {
                italic.push('*');
                i += 1;
            }
            spans.push(Span::styled(italic, Style::default().fg(Color::Cyan)));
            continue;
        }

        // Link [text](url)
        if chars[i] == '[' {
            let color = if has_prefix && spans.is_empty() { base_color } else { default_fg };
            flush_buf(&mut buf, &mut spans, color);
            let mut link = String::from('[');
            i += 1;
            while i < len && chars[i] != ']' {
                link.push(chars[i]);
                i += 1;
            }
            if i < len {
                link.push(']');
                i += 1;
            }
            if i < len && chars[i] == '(' {
                link.push('(');
                i += 1;
                while i < len && chars[i] != ')' {
                    link.push(chars[i]);
                    i += 1;
                }
                if i < len {
                    link.push(')');
                    i += 1;
                }
            }
            spans.push(Span::styled(link, Style::default().fg(Color::Green)));
            continue;
        }

        buf.push(chars[i]);
        i += 1;
    }

    let color = if has_prefix && spans.is_empty() { base_color } else { default_fg };
    flush_buf(&mut buf, &mut spans, color);

    if spans.is_empty() {
        spans.push(Span::styled(line.to_string(), Style::default().fg(default_fg)));
    }

    spans
}

fn flush_buf(buf: &mut String, spans: &mut Vec<Span<'static>>, color: Color) {
    if !buf.is_empty() {
        spans.push(Span::styled(buf.clone(), Style::default().fg(color)));
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_text(spans: &[Span]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn has_color(spans: &[Span], color: Color) -> bool {
        spans.iter().any(|s| s.style.fg == Some(color))
    }

    #[test]
    fn heading_h1() {
        let h = Highlighter::new();
        let spans = h.highlight_line("# Hello");
        assert_eq!(plain_text(&spans), "# Hello");
        assert!(has_color(&spans, Color::Magenta));
    }

    #[test]
    fn heading_h2() {
        let h = Highlighter::new();
        let spans = h.highlight_line("## World");
        assert!(has_color(&spans, Color::Blue));
    }

    #[test]
    fn code_fence() {
        let h = Highlighter::new();
        let spans = h.highlight_line("```rust");
        assert!(has_color(&spans, Color::DarkGray));
    }

    #[test]
    fn inline_code() {
        let h = Highlighter::new();
        let spans = h.highlight_line("use `cargo build` here");
        assert!(has_color(&spans, Color::Yellow));
        assert_eq!(plain_text(&spans), "use `cargo build` here");
    }

    #[test]
    fn bold_text() {
        let h = Highlighter::new();
        let spans = h.highlight_line("some **bold** text");
        assert!(has_color(&spans, Color::Yellow));
        assert_eq!(plain_text(&spans), "some **bold** text");
    }

    #[test]
    fn italic_text() {
        let h = Highlighter::new();
        let spans = h.highlight_line("some *italic* text");
        assert!(has_color(&spans, Color::Cyan));
    }

    #[test]
    fn link() {
        let h = Highlighter::new();
        let spans = h.highlight_line("see [Rust](https://rust-lang.org)");
        assert!(has_color(&spans, Color::Green));
    }

    #[test]
    fn blockquote() {
        let h = Highlighter::new();
        let spans = h.highlight_line("> some quote");
        assert!(has_color(&spans, Color::Blue));
    }

    #[test]
    fn unordered_list() {
        let h = Highlighter::new();
        let spans = h.highlight_line("- list item");
        assert!(has_color(&spans, Color::Cyan));
    }

    #[test]
    fn ordered_list() {
        let h = Highlighter::new();
        let spans = h.highlight_line("1. first item");
        assert!(has_color(&spans, Color::Cyan));
    }

    #[test]
    fn plain_line() {
        let h = Highlighter::new();
        let spans = h.highlight_line("just normal text");
        assert_eq!(plain_text(&spans), "just normal text");
        assert!(has_color(&spans, Color::White));
    }
}
