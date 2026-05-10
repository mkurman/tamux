use crate::theme::ThemeTokens;
use ratatui::text::{Line, Span};
pub(crate) fn render_about_tab(theme: &ThemeTokens) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled("zorai", theme.fg_active)),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Version:   ", theme.fg_dim),
            Span::raw(env!("CARGO_PKG_VERSION")),
        ]),
        Line::from(vec![
            Span::styled("Author:    ", theme.fg_dim),
            Span::raw("Mariusz Kurman"),
        ]),
        Line::from(vec![
            Span::styled("GitHub:    ", theme.fg_dim),
            Span::raw("mkurman/zorai"),
        ]),
        Line::from(vec![
            Span::styled("Homepage:  ", theme.fg_dim),
            Span::raw("zorai.app"),
        ]),
    ]
}
