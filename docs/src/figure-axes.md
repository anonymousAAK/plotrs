# Figure & Axes

The object model has two layers:

- A **`Figure`** is the top-level container. It owns one or more `Axes`
  (subplots), holds the output dimensions, an optional super-title, and the
  visual theme.
- An **`Axes`** is a single subplot. Chart methods (`plot`, `scatter`, `bar`,
  …) add data to it, and configuration methods (`set_title`, `set_xlabel`,
  `set_xlim`, `legend`, …) control its appearance.

`Figure` owns its axes directly (no `Rc`/`RefCell`); `add_subplot` returns
`&mut Axes`, so the borrow checker enforces single-axes mutation.

## Creating a figure

```rust
use plotkit::prelude::*;

let mut fig = Figure::new();                 // 800 x 600 by default
let mut fig = Figure::with_size(1200, 500);  // custom pixel size
```

## Subplots

`add_subplot` uses matplotlib-style 1-based `(nrows, ncols, index)` indexing,
counting across rows first:

```rust
use plotkit::prelude::*;

let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
let sin_y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
let cos_y: Vec<f64> = x.iter().map(|v| v.cos()).collect();

let mut fig = Figure::with_size(1200, 500);

let ax1 = fig.add_subplot(1, 2, 1);
ax1.plot(&x, &sin_y)?.label("sin(x)");
ax1.plot(&x, &cos_y)?.label("cos(x)");
ax1.set_title("Trigonometric Functions");
ax1.set_xlabel("x");
ax1.set_ylabel("y");
ax1.legend();

let ax2 = fig.add_subplot(1, 2, 2);
ax2.scatter(&x, &sin_y)?;
ax2.set_title("Scatter Plot");

fig.save("subplots.png")?;
```

You can also build the whole grid up front with `Figure::subplots(nrows,
ncols)` (or `subplots_with_size`) and reach each cell by 0-based index via
`fig.axes_mut(i)`.

## Super-title

`suptitle` draws an overall title above all subplots:

```rust
fig.suptitle("Quarterly Report");
```

## Layout

plotkit reserves margins automatically — each axes measures its labels, ticks,
and title and claims exactly the space it needs, so nothing clips by default.
`tight_layout()` is provided for matplotlib parity and to make the intent
explicit; the layout is already tight:

```rust
fig.suptitle("Quarterly Report");
fig.tight_layout();
fig.save("report.png")?;
```

## Twin axes

Twin axes overlay a second dataset with an independent secondary axis on the
same plot area. `twinx(parent_index)` shares the x-axis and draws an
independent y-axis on the right; `twiny(parent_index)` shares the y-axis and
draws an independent x-axis on top. Each takes the 0-based index of the parent
axes.

```rust
use plotkit::prelude::*;

let mut fig = Figure::with_size(800, 600);

let ax1 = fig.add_subplot(1, 1, 1);
ax1.plot(&time, &temperature)?.label("Temperature");
ax1.set_ylabel("Temperature (C)");
ax1.legend();

let ax2 = fig.twinx(0);
ax2.plot(&time, &pressure)?.label("Pressure");
ax2.set_ylabel("Pressure (hPa)");
```

The twin inherits the parent's color cycle so the two series get distinct
colors automatically, and the figure draws a single combined legend gathering
entries from both axes.

## Common Axes configuration

Configuration methods return `&mut Self`, so they chain:

```rust
ax.set_title("Demand")
  .set_xlabel("Week")
  .set_ylabel("Units")
  .grid(true)
  .legend();
```

| Method | Effect |
|---|---|
| `set_title(&str)` | Title above the plot area |
| `set_xlabel` / `set_ylabel` | Axis labels |
| `set_xlim(min, max)` / `set_ylim` | Explicit limits (otherwise auto-scaled) |
| `grid(bool)` | Toggle grid lines |
| `legend()` | Enable the legend |
| `set_legend_loc(Loc)` | Place the legend (`Loc::Best`, `Loc::UpperRight`, …) |
| `set_theme(Theme)` | Override the figure theme for this axes only |

See [Scales & Axis Control](./scales-axes.md) for ticks, scales, and axis
inversion, and [Themes](./themes.md) for styling.
