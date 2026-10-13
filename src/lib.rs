mod from_palette;
mod render;
mod tab_ribbon;

pub use from_palette::chrome_from;
pub use render::{render_tab_hits, render_tab_line, strip_ansi, visible_width, TabState};
pub use tab_ribbon::{Chrome, TabAction, TabHit, TabSpec};
