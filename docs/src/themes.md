# Themes

A `Theme` controls every visual default: backgrounds, grid, spines, ticks,
font sizes, the categorical color cycle, and more. A plot rendered with zero
custom styling still looks professional because the default theme follows a
deliberate visual design brief.

## Applying a theme

Set the theme on the whole figure, or override it for a single axes:

```rust
use plotkit::prelude::*;

fig.set_theme(Theme::dark());        // whole figure
ax.set_theme(Theme::publication());  // this axes only
```

## Built-in themes

| Theme | Constructor | Look |
|---|---|---|
| Default | `Theme::default()` | White background, light grid, despined axes, Tableau-10 palette |
| Dark | `Theme::dark()` | Near-black background, bright neon palette, light text |
| Seaborn | `Theme::seaborn()` | Tinted blue-grey axes face with white grid, muted palette |
| ggplot | `Theme::ggplot()` | Grey panel, white grid, four-sided border, ggplot2 hues |
| Publication | `Theme::publication()` | White, no grid, thin four-sided spines, serif font, inward ticks |
| Nature | `Theme::nature()` | Ultra-compact, sans-serif, high-contrast print-safe palette |
| Solarized | `Theme::solarized()` | Solarized dark scheme, low-contrast eye-friendly tones |

```rust
fig.set_theme(Theme::seaborn());
fig.set_theme(Theme::ggplot());
fig.set_theme(Theme::nature());
fig.set_theme(Theme::solarized());
```

## Custom themes

`Theme` is a plain struct, so build one with struct-update syntax over a base
theme and override just the fields you care about:

```rust
use plotkit::prelude::*;

let theme = Theme {
    figure_background: Color::rgb(0xF5, 0xF5, 0xF5),
    font_family: Some("Helvetica".to_string()),
    grid_width: 0.5,
    ..Theme::default()
};
fig.set_theme(theme);
```

Some of the most useful fields:

| Field | Type | Meaning |
|---|---|---|
| `figure_background` / `axes_background` | `Color` | Outer and plot-area fills |
| `grid_color` / `grid_width` / `show_grid` | `Color` / `f64` / `bool` | Grid styling and default visibility |
| `spine_color` / `spine_width` | `Color` / `f64` | Axis spine styling |
| `show_top_spine` … `show_left_spine` | `bool` | Which of the four spines are drawn |
| `tick_color` / `tick_length` / `tick_direction` | — | Tick styling (`TickDirection::Outward` / `Inward`) |
| `tick_label_size` / `axis_label_size` / `title_size` | `f64` | Font sizes in points |
| `title_weight` | `FontWeight` | `Normal` or `Bold` |
| `text_color` | `Color` | Color of all text |
| `line_width` / `marker_size` / `marker_alpha` | `f64` | Data-element defaults |
| `color_cycle` | `Vec<Color>` | Categorical palette assigned to successive series |
| `font_family` | `Option<String>` | `None` uses the renderer's built-in font |

## Colors

Build colors with `Color::rgb(r, g, b)`, `Color::from_hex("#4E79A7")`, or the
Tableau-10 constants (`Color::TAB_BLUE`, `Color::TAB_ORANGE`, …). Adjust
opacity with `color.with_alpha(a)`.

```rust
let cycle = vec![
    Color::TAB_BLUE,
    Color::TAB_ORANGE,
    Color::from_hex("#59A14F").unwrap(),
];
let theme = Theme { color_cycle: cycle, ..Theme::default() };
```
