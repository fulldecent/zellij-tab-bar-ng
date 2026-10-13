use zellij_tile::prelude::{PaletteColor, Style};

use crate::tab_ribbon::{Chrome, Ink};

pub fn chrome_from(style: &Style) -> Chrome {
    let colors = &style.colors;
    Chrome {
        bar: ink(colors.text_unselected.background),
        dark: ink(colors.ribbon_unselected.base),
        light: ink(colors.ribbon_unselected.background),
        plus_hover: ink(colors.ribbon_unselected.emphasis_1),
        title: ink(colors.text_unselected.base),
    }
}

fn ink(color: PaletteColor) -> Ink {
    match color {
        PaletteColor::EightBit(n) => Ink::Bit(n),
        PaletteColor::Rgb((r, g, b)) => Ink::Rgb(r, g, b),
    }
}
