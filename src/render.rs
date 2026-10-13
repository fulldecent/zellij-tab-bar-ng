use unicode_width::UnicodeWidthStr;

use crate::tab_ribbon::{
    render_tabs_themed, strip_ansi as strip_ribbon_ansi, Chrome, TabAction, TabHit, TabSpec,
};

/// Snapshot the tab plugin would paint. No Zellij I/O.
#[derive(Clone, Copy, Debug)]
pub struct TabState<'a> {
    pub tabs: &'a [TabSpec],
    pub session_name: Option<&'a str>,
    pub hide_session_name: bool,
    pub chrome: Chrome,
    /// Cell column under the pointer. Resolved while painting, so the first
    /// frame after a hover already uses the plus-button color.
    pub hover_col: Option<usize>,
}

fn painted(cols: usize, state: &TabState<'_>) -> (String, Vec<TabHit>) {
    let (line, hits) = render_tabs_themed(
        state.tabs,
        cols,
        state.session_name,
        state.hide_session_name,
        &state.chrome,
        false,
    );
    let plus_hovered = state.hover_col.is_some_and(|col| {
        hits.iter()
            .any(|hit| matches!(hit.action, TabAction::NewTab) && col >= hit.start && col < hit.end)
    });
    if plus_hovered {
        render_tabs_themed(
            state.tabs,
            cols,
            state.session_name,
            state.hide_session_name,
            &state.chrome,
            true,
        )
    } else {
        (line, hits)
    }
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
