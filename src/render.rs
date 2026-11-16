use unicode_width::UnicodeWidthStr;

use crate::tab_ribbon::{
    render_tabs_themed, strip_ansi as strip_ribbon_ansi, Chrome, TabHit, TabSpec,
};

/// Snapshot the tab plugin would paint. No Zellij I/O.
#[derive(Clone, Copy, Debug)]
pub struct TabState<'a> {
    pub tabs: &'a [TabSpec],
    pub session_name: Option<&'a str>,
    pub hide_session_name: bool,
    pub chrome: Chrome,
    /// Cell column under the pointer. Resolved while painting, so the first
    /// frame after a hover already uses the hover color.
    pub hover_col: Option<usize>,
}

fn painted(cols: usize, state: &TabState<'_>) -> (String, Vec<TabHit>) {
    render_tabs_themed(
        state.tabs,
        cols,
        state.session_name,
        state.hide_session_name,
        &state.chrome,
        state.hover_col,
    )
}

pub fn render_tab_line(cols: usize, state: &TabState<'_>) -> String {
    painted(cols, state).0
}

pub fn render_tab_hits(cols: usize, state: &TabState<'_>) -> Vec<TabHit> {
    painted(cols, state).1
}

pub fn visible_width(rendered: &str) -> usize {
    strip_ansi(rendered).width()
}

pub fn strip_ansi(rendered: &str) -> String {
    strip_ribbon_ansi(rendered)
}
