# Quickstart

For quick scripts, plotkit offers a `pyplot`-style facade: a set of free
functions that operate on a thread-local "current figure". This mirrors
matplotlib's `pyplot` module and is the fastest way to get a chart on disk.

```rust
use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();

    plotkit::plot(&x, &y)?;
    plotkit::title("sin(x)");
    plotkit::xlabel("x");
    plotkit::ylabel("y");
    plotkit::savefig("sine.png")?;
    Ok(())
}
```

The first drawing call (`plot`, `scatter`, `bar`, `hist`) lazily creates a
single subplot on the current figure if none exists yet, so you can jump
straight to plotting.

## Available facade functions

| Function | Purpose |
|---|---|
| `plot(x, y)` | Line plot |
| `scatter(x, y)` | Scatter plot |
| `bar(categories, heights)` | Vertical bar chart |
| `hist(data, bins)` | Histogram |
| `title(s)` | Set the axes title |
| `xlabel(s)` / `ylabel(s)` | Set axis labels |
| `xlim(min, max)` / `ylim(min, max)` | Set axis limits |
| `xticks(&[..])` / `yticks(&[..])` | Set custom tick positions |
| `xscale(s)` / `yscale(s)` | Set axis scale (see [Scales](./scales-axes.md)) |
| `grid(visible)` | Toggle grid lines |
| `legend()` | Show the legend |
| `subplots(nrows, ncols)` | Replace the current figure with a grid |
| `clf()` | Clear the current figure |
| `savefig(path)` | Save by file extension (`.png`, `.svg`, …) |

## When to switch to Figure/Axes

The facade always targets axes index 0 of the current figure, which is perfect
for one-off plots but limiting for multi-panel layouts, twin axes, per-axes
themes, or rendering to bytes. For anything beyond a single chart, use the
[`Figure` and `Axes`](./figure-axes.md) object model directly.
