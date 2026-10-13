//! Tab ribbon fitting follows the built-in tab bar: the active tab stays,
//! neighbors are added toward the shorter side, and hidden tabs become
//! `← +N` / `+N →`. The line is never wider than the screen, down to 1 column.
//!
//! Column counts use [`unicode_width`](https://docs.rs/unicode-width), the same
//! measure as Zellij's `default-plugins/tab-bar` at v0.45.1.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Theme inks for the bar, the plus button, and the session placard.
/// Defaults match `Styling::from(default_palette())` on Zellij 0.45:
/// bar 16, ribbon base 16, ribbon background 245, emphasis 255, text 255.
///
/// The tab fills themselves stay the classic green 154 and gray 238.
#[derive(Clone, Copy, Debug)]
pub struct Chrome {
    pub bar: Ink,
    pub dark: Ink,
    pub light: Ink,
    pub plus_hover: Ink,
    pub title: Ink,
}

impl Default for Chrome {
    fn default() -> Self {
        Self {
            bar: Ink::Bit(16),
            dark: Ink::Bit(16),
            light: Ink::Bit(245),
            plus_hover: Ink::Bit(255),
            title: Ink::Bit(255),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Ink {
    Bit(u8),
    Rgb(u8, u8, u8),
}

impl Ink {
    pub fn fg(self) -> String {
        match self {
            Ink::Bit(n) => format!("38;5;{n}"),
            Ink::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
        }
    }
    pub fn bg(self) -> String {
        match self {
            Ink::Bit(n) => format!("48;5;{n}"),
            Ink::Rgb(r, g, b) => format!("48;2;{r};{g};{b}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabAction {
    /// The tab's own ribbon. A press switches to this tab.
    Switch(u32),
    /// A `← +N` / `+N →` chip. A press switches to the hidden tab.
    Reveal(u32),
    NewTab,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabHit {
    pub start: usize,
    pub end: usize,
    pub action: TabAction,
}

#[derive(Clone, Debug)]
pub struct TabSpec {
    /// 1-based index. `switch_tab_to` uses this.
    pub index: u32,
    pub name: String,
    pub active: bool,
    pub fullscreen: bool,
    pub sync: bool,
}

#[derive(Clone)]
struct Piece {
    width: usize,
    ansi: String,
    action: Option<TabAction>,
}

pub(crate) fn strip_ansi(rendered: &str) -> String {
    let mut out = String::new();
    let mut chars = rendered.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else if chars.peek() == Some(&']') {
                chars.next();
                for c in chars.by_ref() {
                    if c == '\u{7}' || c == '\\' {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn col_width(text: &str) -> usize {
    text.width()
}

fn truncate_cols(text: &str, cols: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let cw = ch.width().unwrap_or(0);
        if used + cw > cols {
            break;
        }
        out.push(ch);
        used += cw;
    }
    out
}

fn painted_width(rendered: &str) -> usize {
    col_width(&strip_ansi(rendered))
}

fn hits_of(line: &[Piece]) -> Vec<TabHit> {
    let mut hits = Vec::new();
    let mut at = 0;
    for piece in line {
        if let Some(action) = piece.action {
            if piece.width > 0 {
                hits.push(TabHit {
                    start: at,
                    end: at + piece.width,
                    action,
                });
            }
        }
        at += piece.width;
    }
    hits
}

fn clip_line(line: &mut Vec<Piece>, cols: usize) {
    let mut used = 0;
    let mut keep = 0;
    for piece in line.iter() {
        if used + piece.width > cols {
            break;
        }
        used += piece.width;
        keep += 1;
    }
    // A following piece that does not fit whole stays off. The active tab
    // was already shrunk to `cols` before neighbors were added.
    line.truncate(keep);
}

#[derive(Clone, Copy)]
enum Collapse {
    Long,
    Short,
}

fn try_fill(
    before: &[Piece],
    after: &[Piece],
    middle: &[Piece],
    cols: usize,
    style: Collapse,
    chrome: &Chrome,
) -> Option<Vec<Piece>> {
    let mut before = before.to_vec();
    let mut after = after.to_vec();
    let mut shown = middle.to_vec();
    let mut middle_size: usize = shown.iter().map(|p| p.width).sum();
    let mut total_left = 0;
    let mut total_right = 0;

    loop {
        let left_count = before.len();
        let right_count = after.len();
        let collapsed_left = collapse_piece(left_count, Side::Left, style, before.last(), chrome);
        let collapsed_right =
            collapse_piece(right_count, Side::Right, style, after.first(), chrome);
        let total = collapsed_left.width + middle_size + collapsed_right.width;
        if total > cols {
            return None;
        }

        let left_w = before.last().map(|p| p.width).unwrap_or(usize::MAX);
        let right_w = after.first().map(|p| p.width).unwrap_or(usize::MAX);
        let size_left = left_w
            .saturating_add(total)
            .saturating_sub(if left_count == 1 {
                collapsed_left.width
            } else {
                0
            });
        let size_right = right_w
            .saturating_add(total)
            .saturating_sub(if right_count == 1 {
                collapsed_right.width
            } else {
                0
            });
        let left_fits = size_left <= cols;
        let right_fits = size_right <= cols;

        if (total_left <= total_right || !right_fits) && left_fits {
            let tab = before.pop().unwrap();
            middle_size += tab.width;
            total_left += tab.width;
            shown.insert(0, tab);
        } else if right_fits {
            let tab = after.remove(0);
            middle_size += tab.width;
            total_right += tab.width;
            shown.push(tab);
        } else {
            if collapsed_left.width > 0 {
                shown.insert(0, collapsed_left);
            }
            if collapsed_right.width > 0 {
                shown.push(collapsed_right);
            }
            return Some(shown);
        }
    }
}

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

fn collapse_piece(
    count: usize,
    side: Side,
    style: Collapse,
    edge: Option<&Piece>,
    chrome: &Chrome,
) -> Piece {
    if count == 0 {
        return Piece {
            width: 0,
            ansi: String::new(),
            action: None,
        };
    }
    let body = match (style, side) {
        (Collapse::Long, Side::Left) => format!(" ← +{count} "),
        (Collapse::Long, Side::Right) => format!(" +{count} → "),
        (Collapse::Short, _) => format!("+{count}"),
    };
    let action = edge.and_then(|piece| match piece.action {
        Some(TabAction::Switch(index) | TabAction::Reveal(index)) => Some(TabAction::Reveal(index)),
        other => other,
    });
    ribbon(&body, false, action, chrome)
}

const PLUS_WIDTH: usize = 5;

fn plus_button(chrome: &Chrome, hovered: bool) -> Piece {
    let face = if hovered {
        chrome.plus_hover
    } else {
        chrome.light
    };
    let body = " + ";
    let ink = face.fg();
    let bar = chrome.bar.bg();
    let fg = chrome.dark.fg();
    let fill = face.bg();
    let ansi = format!(
        "\u{1b}[0m\u{1b}[{ink}m\u{1b}[{bar}m\u{e0ba}\u{1b}[0m\u{1b}[1;{fg}m\u{1b}[{fill}m{body}\u{1b}[0m\u{1b}[{ink}m\u{1b}[{bar}m\u{e0bc}\u{1b}[0m"
    );
    Piece {
        width: 2 + col_width(body),
        ansi,
        action: Some(TabAction::NewTab),
    }
}

fn label_of(tab: &TabSpec) -> String {
    let mut name = tab.name.clone();
    if tab.fullscreen {
        name.push_str(" [full]");
    }
    if tab.sync {
        name.push_str(" [sync]");
    }
    name
}

fn full_tab(tab: &TabSpec, chrome: &Chrome) -> Piece {
    ribbon(
        &format!(" {} ", label_of(tab)),
        tab.active,
        Some(TabAction::Switch(tab.index)),
        chrome,
    )
}

fn clip_ansi_piece(piece: &Piece, max_width: usize, chrome: &Chrome) -> Piece {
    if max_width == 0 {
        return Piece {
            width: 0,
            ansi: String::new(),
            action: piece.action,
        };
    }
    // The active tab is shrunk through shrink_named when we know the name.
    // This fallback keeps an already-built piece inside the remaining columns.
    let visible = strip_ansi(&piece.ansi);
    let clipped = truncate_cols(&visible, max_width);
    Piece {
        width: col_width(&clipped),
        ansi: paint_raw(&clipped, false, chrome),
        action: piece.action,
    }
}

fn shrink_named(tab: &TabSpec, max_width: usize, chrome: &Chrome) -> Piece {
    let label = label_of(tab);
    let action = Some(TabAction::Switch(tab.index));
    let full = ribbon(&format!(" {label} "), tab.active, action, chrome);
    if full.width <= max_width {
        return full;
    }
    if max_width == 0 {
        return Piece {
            width: 0,
            ansi: String::new(),
            action,
        };
    }
    let body_budget = max_width.saturating_sub(2);
    let body = if body_budget == 0 {
        String::new()
    } else if body_budget < 4 {
        truncate_cols("…", body_budget)
    } else {
        let keep = body_budget - 3; // two spaces and the ellipsis
        let truncated = truncate_cols(&label, keep);
        format!(" {truncated}… ")
    };
    let made = if body.is_empty() && max_width < 2 {
        clip_ansi_piece(&full, max_width, chrome)
    } else if max_width < 2 {
        clip_ansi_piece(
            &ribbon(&body, tab.active, action, chrome),
            max_width,
            chrome,
        )
    } else {
        ribbon(&body, tab.active, action, chrome)
    };
    if made.width <= max_width {
        made
    } else {
        clip_ansi_piece(&made, max_width, chrome)
    }
}

fn ribbon(body: &str, active: bool, action: Option<TabAction>, chrome: &Chrome) -> Piece {
    // 154 and 238 are the classic Zellij ribbon fills (green active, gray inactive).
    let (cap, fill) = if active { (154, 154) } else { (238, 238) };
    let bar = chrome.bar.bg();
    let text = chrome.dark.fg();
    let ansi = format!(
        "\u{1b}[0m\u{1b}[38;5;{cap}m\u{1b}[{bar}m\u{e0ba}\u{1b}[0m\u{1b}[{text}m\u{1b}[48;5;{fill}m{body}\u{1b}[0m\u{1b}[38;5;{cap}m\u{1b}[{bar}m\u{e0bc}\u{1b}[0m"
    );
    Piece {
        width: 2 + col_width(body),
        ansi,
        action,
    }
}

fn paint_raw(text: &str, active: bool, chrome: &Chrome) -> String {
    let fill = if active { 154 } else { 238 };
    let text_fg = chrome.dark.fg();
    format!("\u{1b}[0m\u{1b}[{text_fg}m\u{1b}[48;5;{fill}m{text}\u{1b}[0m")
}

fn prepare(tabs: &[TabSpec], cols: usize, chrome: &Chrome) -> (Vec<Piece>, Vec<Piece>, Vec<Piece>) {
    let active_pos = tabs.iter().position(|tab| tab.active).unwrap_or(0);
    let active = shrink_named(&tabs[active_pos], cols, chrome);
    let before: Vec<Piece> = tabs[..active_pos]
        .iter()
        .map(|tab| full_tab(tab, chrome))
        .collect();
    let after: Vec<Piece> = tabs[active_pos + 1..]
        .iter()
        .map(|tab| full_tab(tab, chrome))
        .collect();
    (before, after, vec![active])
}

pub(crate) fn render_tabs_themed(
    tabs: &[TabSpec],
    cols: usize,
    session_name: Option<&str>,
    hide_session_name: bool,
    chrome: &Chrome,
    plus_hovered: bool,
) -> (String, Vec<TabHit>) {
    if cols == 0 {
        return (String::new(), Vec::new());
    }
    let (prefix, prefix_w) = session_prefix(session_name, hide_session_name, cols, chrome);
    let tab_cols = cols - prefix_w;
    let (body, mut hits) = if tabs.is_empty() || tab_cols == 0 {
        (String::new(), Vec::new())
    } else {
        render_tab_body(tabs, tab_cols, chrome, plus_hovered)
    };
    let body_w = painted_width(&body);
    for hit in &mut hits {
        hit.start += prefix_w;
        hit.end += prefix_w;
    }
    let pad = cols.saturating_sub(prefix_w + body_w);
    let pad_ansi = if pad > 0 {
        let bar = chrome.bar.bg();
        format!("\u{1b}[0m\u{1b}[{bar}m{}", " ".repeat(pad))
    } else {
        String::new()
    };
    (format!("{prefix}{body}{pad_ansi}"), hits)
}

fn session_prefix(
    session_name: Option<&str>,
    hide_session_name: bool,
    cols: usize,
    chrome: &Chrome,
) -> (String, usize) {
    let mut ansi = String::new();
    let mut used = 0;
    let placard = " Zellij ";
    let placard_w = col_width(placard);
    if cols >= placard_w {
        ansi.push_str(&on_bar_bold(placard, chrome));
        used += placard_w;
    }
    if !hide_session_name {
        if let Some(name) = session_name.filter(|name| !name.is_empty()) {
            let part = format!("({name}) ");
            let width = col_width(&part);
            if used + width <= cols {
                ansi.push_str(&on_bar_bold(&part, chrome));
                used += width;
            }
        }
    }
    (ansi, used)
}

fn on_bar_bold(text: &str, chrome: &Chrome) -> String {
    let fg = chrome.title.fg();
    let bg = chrome.bar.bg();
    format!("\u{1b}[0m\u{1b}[1;{fg}m\u{1b}[{bg}m{text}")
}

fn render_tab_body(
    tabs: &[TabSpec],
    cols: usize,
    chrome: &Chrome,
    plus_hovered: bool,
) -> (String, Vec<TabHit>) {
    let (before, after, middle) = prepare(tabs, cols, chrome);
    let mut line = try_fill(&before, &after, &middle, cols, Collapse::Long, chrome)
        .or_else(|| try_fill(&before, &after, &middle, cols, Collapse::Short, chrome))
        .unwrap_or(middle);

    let used = line.iter().map(|p| p.width).sum::<usize>();
    if used + PLUS_WIDTH <= cols {
        line.push(plus_button(chrome, plus_hovered));
    }
    clip_line(&mut line, cols);
    let hits = hits_of(&line);
    let ansi = line.into_iter().map(|p| p.ansi).collect();
    (ansi, hits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tabs(n: usize, active: usize) -> Vec<TabSpec> {
        (0..n)
            .map(|i| TabSpec {
                index: (i + 1) as u32,
                name: format!("Tab #{}", i + 1),
                active: i == active,
                fullscreen: false,
                sync: false,
            })
            .collect()
    }

    fn visible_line(rendered: &str) -> String {
        strip_ansi(rendered)
    }

    fn column_of(text: &str, needle: &str) -> usize {
        let byte = text.find(needle).expect(needle);
        col_width(&text[..byte])
    }

    #[test]
    fn ribbon_glyphs_are_one_column() {
        assert_eq!(col_width("\u{e0ba}"), 1);
        assert_eq!(col_width("\u{e0bc}"), 1);
        assert_eq!(col_width("…"), 1);
    }

    #[test]
    fn never_wider_than_the_screen() {
        for n in 1..8 {
            for active in 0..n {
                for cols in 0..80 {
                    let (line, hits) = render_tabs_themed(
                        &tabs(n, active),
                        cols,
                        None,
                        false,
                        &Chrome::default(),
                        false,
                    );
                    let text = visible_line(&line);
                    assert!(
                        col_width(&text) <= cols,
                        "{n} tabs active {active} at {cols}: {text}"
                    );
                    for hit in hits {
                        assert!(hit.end <= cols);
                        assert!(hit.start < hit.end);
                    }
                }
            }
        }
    }

    #[test]
    fn wide_line_shows_every_name_and_the_plus() {
        let (line, hits) = render_tabs_themed(
            &tabs(3, 1),
            120,
            Some("demo"),
            false,
            &Chrome::default(),
            false,
        );
        let text = visible_line(&line);
        assert!(text.starts_with(" Zellij (demo) "));
        assert!(text.contains("Tab #1"));
        assert!(text.contains("Tab #2"));
        assert!(text.contains("Tab #3"));
        assert!(text.contains('+'));
        assert!(!text.contains('←'));
        assert!(line.contains("48;5;154"));
        assert!(line.contains("48;5;238"));
        assert!(line.contains("48;5;245"));
        let plus_at = column_of(&text, "+");
        assert!(hits.iter().any(|hit| {
            hit.action == TabAction::NewTab && plus_at >= hit.start && plus_at < hit.end
        }));
        let tab2 = column_of(&text, "Tab #2");
        assert!(hits.iter().any(|hit| {
            hit.action == TabAction::Switch(2) && tab2 >= hit.start && tab2 < hit.end
        }));
    }

    #[test]
    fn eighty_columns_match_the_ribbon_picture() {
        let (line, _) = render_tabs_themed(
            &tabs(3, 1),
            80,
            Some("demo"),
            false,
            &Chrome::default(),
            false,
        );
        let text = visible_line(&line);
        assert_eq!(col_width(&text), 80);
        assert!(text.contains("\u{e0ba} Tab #1 \u{e0bc}"));
        assert!(text.contains("\u{e0ba} Tab #2 \u{e0bc}"));
        assert!(text.contains("\u{e0ba} Tab #3 \u{e0bc}"));
        assert!(text.contains("\u{e0ba} + \u{e0bc}"));
        assert!(!text.contains('←'));
    }

    #[test]
    fn plus_hover_uses_the_hover_ink() {
        let (line, _) = render_tabs_themed(
            &tabs(3, 1),
            80,
            Some("demo"),
            false,
            &Chrome::default(),
            true,
        );
        assert!(line.contains("48;5;255"));
        assert!(!line.contains("48;5;245"));
        assert!(line.contains("48;5;154"));
    }

    #[test]
    fn narrow_line_keeps_the_active_tab_and_counts_the_rest() {
        let (line, _) = render_tabs_themed(&tabs(6, 2), 36, None, false, &Chrome::default(), false);
        let text = visible_line(&line);
        assert!(text.contains("Tab #3") || text.contains('…'));
        assert!(text.contains('+') || text.contains('…') || text.contains('←'));
        assert!(col_width(&text) <= 36);
    }

    #[test]
    fn one_column_is_a_single_cell() {
        let (line, _) = render_tabs_themed(
            &tabs(3, 0),
            1,
            Some("demo"),
            false,
            &Chrome::default(),
            false,
        );
        assert_eq!(col_width(&visible_line(&line)), 1);
    }

    #[test]
    fn suffixes_mark_fullscreen_and_sync() {
        let tabs = vec![TabSpec {
            index: 1,
            name: "Main".into(),
            active: true,
            fullscreen: true,
            sync: true,
        }];
        let (line, _) = render_tabs_themed(&tabs, 80, None, false, &Chrome::default(), false);
        let text = visible_line(&line);
        assert!(text.contains("Main [full] [sync]"));
    }

    #[test]
    fn wide_character_names_stay_inside_the_screen() {
        let tabs = vec![TabSpec {
            index: 1,
            name: "画面".into(),
            active: true,
            fullscreen: false,
            sync: false,
        }];
        for cols in 1..40 {
            let (line, hits) =
                render_tabs_themed(&tabs, cols, None, false, &Chrome::default(), false);
            let text = visible_line(&line);
            assert!(col_width(&text) <= cols, "{cols}: {text}");
            for hit in hits {
                assert!(hit.end <= cols);
            }
        }
    }
}
