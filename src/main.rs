use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use plugin::{
    chrome_from, render_tab_hits, render_tab_line, Chrome, TabAction, TabHit, TabSpec, TabState,
};

#[derive(Default)]
struct State {
    tabs: Vec<TabSpec>,
    hits: Vec<TabHit>,
    session_name: Option<String>,
    hide_session_name: bool,
    chrome: Chrome,
    hover_col: Option<usize>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[
            EventType::ModeUpdate,
            EventType::TabUpdate,
            EventType::Mouse,
            EventType::PermissionRequestResult,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(status) => {
                if status == PermissionStatus::Granted {
                    // Chrome, not a pane. Stay selectable until this event so
                    // the y/n permission prompt can receive a key.
                    set_selectable(false);
                }
                true
            }
            Event::ModeUpdate(info) => {
                self.session_name = info.session_name;
                self.hide_session_name = info.style.hide_session_name;
                self.chrome = chrome_from(&info.style);
                true
            }
            Event::TabUpdate(tabs) => {
                self.tabs = tabs
                    .into_iter()
                    .map(|tab| TabSpec {
                        index: (tab.position + 1) as u32,
                        name: tab.name,
                        active: tab.active,
                        fullscreen: tab.is_fullscreen_active,
                        sync: tab.is_sync_panes_active,
                    })
                    .collect();
                true
            }
            // Click-drag reorder is abandoned. Zellij 0.45.1 cannot deliver the
            // gesture to this pane, and nothing in the plugin API reconstructs it.
            //
            // The press arrives as Mouse::LeftClick. Buttonless motion arrives as
            // Mouse::Hover. Mouse::Hold and Mouse::Release are sent only from
            // PluginPane::update_selection and end_selection. The mouse handler
            // calls those only while selecting_with_mouse_in_pane is set, and it
            // sets that latch only when supports_mouse_selection() is already
            // true. With the latch set, those methods paint a grid selection and
            // do not send Hold or Release. The plugin does not run between the
            // two reads of that flag, so set_self_mouse_selection_support during
            // the gesture cannot change which branch this press took.
            //
            // This pane is unselectable, and that path calls start_selection
            // without setting the latch, so update_selection and end_selection
            // never run here. Making the pane selectable focuses it on the first
            // press instead of sending LeftClick. A pane stacked above this one
            // takes the hit and does not forward it. There is no open Zellij
            // issue for the missing Hold and Release.
            //
            // https://github.com/zellij-org/zellij/blob/v0.45.1/zellij-server/src/panes/plugin_pane.rs
            // https://github.com/zellij-org/zellij/blob/v0.45.1/zellij-server/src/tab/mouse_handler.rs
            Event::Mouse(Mouse::LeftClick(_, col)) => self.press(col),
            Event::Mouse(Mouse::Hover(_, col)) => {
                if self.hover_col == Some(col) {
                    false
                } else {
                    self.hover_col = Some(col);
                    true
                }
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let state = TabState {
            tabs: &self.tabs,
            session_name: self.session_name.as_deref(),
            hide_session_name: self.hide_session_name,
            chrome: self.chrome,
            hover_col: self.hover_col,
        };
        self.hits = render_tab_hits(cols, &state);
        print!("{}", render_tab_line(cols, &state));
    }
}

impl State {
    fn press(&mut self, col: usize) -> bool {
        let Some(hit) = self
            .hits
            .iter()
            .find(|hit| col >= hit.start && col < hit.end)
        else {
            return false;
        };
        match hit.action {
            TabAction::Switch(index) | TabAction::Reveal(index) => {
                switch_tab_to(index);
            }
            TabAction::NewTab => {
                new_tab::<&str>(None, None);
            }
        }
        false
    }
}
