use ratatui::style::Style;
use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(crate) fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if UnicodeWidthStr::width(current_line.as_str())
                + 1
                + UnicodeWidthStr::width(word)
                <= width
            {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

pub(crate) fn wrap_styled_lines(lines: Vec<Line<'static>>, width: usize) -> Vec<Line<'static>> {
    lines
        .into_iter()
        .flat_map(|line| wrap_styled_line(line, width))
        .collect()
}

pub(crate) fn wrap_styled_line(line: Line<'static>, width: usize) -> Vec<Line<'static>> {
    if width == 0 || line.spans.is_empty() {
        return vec![line];
    }

    let line_style = line.style;
    let tokens = line
        .spans
        .into_iter()
        .flat_map(|span| tokenize_styled_text(span.content.to_string(), span.style))
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        return vec![Line::default().style(line_style)];
    }

    let mut wrapped = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0usize;

    for (token_text, token_style) in tokens {
        let token_width = UnicodeWidthStr::width(token_text.as_str());

        if token_width == 0 {
            current_spans.push(Span::styled(token_text, token_style));
            continue;
        }

        if current_width > 0 && current_width + token_width > width {
            wrapped.push(Line::from(std::mem::take(&mut current_spans)).style(line_style));
            current_width = 0;
            if token_text.chars().all(char::is_whitespace) {
                continue;
            }
        }

        if token_width <= width {
            current_width += token_width;
            current_spans.push(Span::styled(token_text, token_style));
            continue;
        }

        for chunk in split_text_by_width(&token_text, width) {
            let chunk_width = UnicodeWidthStr::width(chunk.as_str());
            if current_width > 0 {
                wrapped.push(Line::from(std::mem::take(&mut current_spans)).style(line_style));
            }

            current_spans.push(Span::styled(chunk, token_style));
            current_width = chunk_width;
            if current_width >= width {
                wrapped.push(Line::from(std::mem::take(&mut current_spans)).style(line_style));
                current_width = 0;
            }
        }
    }

    if !current_spans.is_empty() {
        wrapped.push(Line::from(current_spans).style(line_style));
    }

    if wrapped.is_empty() {
        wrapped.push(Line::default().style(line_style));
    }

    wrapped
}

pub(crate) fn tokenize_styled_text(text: String, style: Style) -> Vec<(String, Style)> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut tokens = Vec::new();
    let mut start = 0usize;
    let mut chars = text.char_indices();
    let Some((_, first)) = chars.next() else {
        return tokens;
    };
    let mut in_whitespace = first.is_whitespace();

    for (idx, ch) in chars {
        if ch.is_whitespace() != in_whitespace {
            tokens.push((text[start..idx].to_string(), style));
            start = idx;
            in_whitespace = ch.is_whitespace();
        }
    }

    tokens.push((text[start..].to_string(), style));
    tokens
}

pub(crate) fn split_text_by_width(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;

    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width > 0 && current_width + ch_width > width {
            chunks.push(std::mem::take(&mut current));
            current_width = 0;
        }
        current.push(ch);
        current_width += ch_width;
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    if chunks.is_empty() {
        chunks.push(String::new());
    }

    chunks
}
