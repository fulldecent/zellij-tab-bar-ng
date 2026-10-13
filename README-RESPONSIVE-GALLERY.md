# Responsive gallery

Pictures are SVG from [Zellij Plugin Snapshot](https://github.com/fulldecent/zellij-plugin-snapshot). Every glyph is one cell, except a wide character in a tab name, which takes the columns Unicode assigns it. Widths 1 through 100 are asserted in `tests/widths.rs`.

## Three tabs

The session placard, `Tab #1`, active `Tab #2`, `Tab #3`, and `+` when they fit. Default theme colors. The ribbon fills are the classic green and gray.

### 20

![20 columns](shots/w20.svg)

### 40

![40 columns](shots/w40.svg)

### 80

![80 columns](screenshot.svg)

### 100

![100 columns](shots/w100.svg)

## Overflow

Eight tabs at 80 columns. Five names fit, including the active tab. The other three collapse to `+3 →`, and the `+` button stays.

![Eight tabs at 80 columns](shots/overflow.svg)

## Mouse over

Hovering `+` paints that button in the theme hover color. The button is columns 45 through 49.

![Mouse over the plus button at 80 columns](shots/hover.svg)
