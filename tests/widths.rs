use plugin::{
    render_tab_hits, render_tab_line, visible_width, Chrome, TabAction, TabSpec, TabState,
};

fn sample(n: usize, active: usize) -> Vec<TabSpec> {
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

#[test]
fn ribbon_tab_bar_renders_every_width_1_to_100() {
    for n in 1..=6 {
        for active in 0..n {
            let tabs = sample(n, active);
            let state = TabState {
                tabs: &tabs,
                session_name: Some("demo"),
                hide_session_name: false,
                chrome: Chrome::default(),
                hover_col: None,
            };
            for cols in 1..=100 {
                let line = render_tab_line(cols, &state);
                let width = visible_width(&line);
                assert!(
                    width <= cols,
                    "{n} tabs active {active} at {cols} painted {width}"
                );
                assert_eq!(
                    width, cols,
                    "{n} tabs active {active} at {cols} did not fill the row"
                );
            }
        }
    }
}

#[test]
fn hover_on_the_plus_uses_the_hover_ink() {
    let tabs = sample(3, 1);
    let base = TabState {
        tabs: &tabs,
        session_name: Some("demo"),
        hide_session_name: false,
        chrome: Chrome::default(),
        hover_col: None,
    };
    let plain = render_tab_line(80, &base);
    let plus = render_tab_hits(80, &base)
        .into_iter()
        .find(|hit| hit.action == TabAction::NewTab)
        .expect("plus button");
    let hovered = TabState {
        hover_col: Some(plus.start),
        ..base
    };
    let line = render_tab_line(80, &hovered);
    assert_ne!(line, plain);
    assert!(line.contains("48;5;255"));
    assert!(!line.contains("48;5;245"));
}

#[test]
fn empty_tab_list_fills_the_row() {
    let state = TabState {
        tabs: &[],
        session_name: None,
        hide_session_name: true,
        chrome: Chrome::default(),
        hover_col: None,
    };
    for cols in 1..=40 {
        assert_eq!(visible_width(&render_tab_line(cols, &state)), cols);
    }
}
