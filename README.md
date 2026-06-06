<div align="center">

# plotkit

### Publication-quality plots in 3 lines of Rust

[![Crates.io](https://img.shields.io/crates/v/plotkit.svg)](https://crates.io/crates/plotkit)
[![docs.rs](https://docs.rs/plotkit/badge.svg)](https://docs.rs/plotkit)
[![CI](https://github.com/anonymousAAK/plotrs/actions/workflows/ci.yml/badge.svg)](https://github.com/anonymousAAK/plotrs/actions)
[![License](https://img.shields.io/crates/l/plotkit.svg)](https://github.com/anonymousAAK/plotrs)

</div>

---

**Rust finally has a plotting library that looks as good as matplotlib — and it's faster.**

No Python subprocess. No JavaScript runtime. No system dependencies. Just add `plotkit` to your `Cargo.toml` and start making figures that belong in a journal paper.

## Quick Start

```rust
plotkit::plot(&x, &y)?;
plotkit::title("sin(x)");
plotkit::savefig("plot.png")?;
```

Three lines. Anti-aliased text, clean axes, and a Tableau-10 color palette out of the box.

## Installation

```toml
[dependencies]
plotkit = "0.4"
```

Optional features:

```toml
plotkit = { version = "0.4", features = ["svg", "jupyter", "ndarray", "polars"] }
```

| Feature    | Description                                |
|------------|--------------------------------------------|
| `png`      | PNG rasterization (enabled by default)     |
| `svg`      | SVG vector output                          |
| `jupyter`  | Inline display in Evcxr notebooks          |
| `ndarray`  | Plot directly from `ndarray::Array1`       |
| `polars`   | Plot directly from Polars `Series`/`DataFrame` |

## Chart Types

plotkit supports 16 chart types with a matplotlib-familiar API:

| Chart Type | Method | Description |
|---|---|---|
| Line | `ax.plot(x, y)` | Connected data points with optional markers |
| Scatter | `ax.scatter(x, y)` | Individual data points with colormap support |
| Bar | `ax.bar(cats, vals)` | Vertical bars with stacking and grouping |
| Horizontal Bar | `ax.barh(cats, vals)` | Horizontal bars |
| Grouped Bar | `ax.bar_group(cats, series)` | Side-by-side bars for multiple series |
| Histogram | `ax.hist(data, bins)` | Frequency distribution |
| Fill Between | `ax.fill_between(x, y1, y2)` | Shaded region between curves |
| Step | `ax.step(x, y)` | Staircase function |
| Stem | `ax.stem(x, y)` | Lollipop chart |
| Box Plot | `ax.boxplot(datasets)` | Statistical box-and-whisker |
| Violin | `ax.violin(datasets)` | Kernel density distribution |
| Error Bar | `ax.errorbar(x, y)` | Data with uncertainty ranges |
| Heatmap | `ax.heatmap(data)` | 2D color matrix with colorbar |
| Pie | `ax.pie(sizes)` | Proportional wedge chart |
| Contour | `ax.contour(x, y, z)` | Iso-line contour plot |
| Filled Contour | `ax.contourf(x, y, z)` | Filled contour regions |

## Two APIs

### pyplot-style (quick scripts)

```rust
use plotkit::prelude::*;

let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();

plotkit::plot(&x, &y)?;
plotkit::title("sin(x)");
plotkit::xlabel("x");
plotkit::ylabel("y");
plotkit::savefig("sine.png")?;
```

### Figure/Axes (full control)

```rust
use plotkit::prelude::*;
use plotkit::FigureExt;

let mut fig = Figure::with_size(1200, 500);

let ax1 = fig.add_subplot(1, 2, 1);
ax1.plot(&x, &sin_y)?.label("sin(x)");
ax1.plot(&x, &cos_y)?.label("cos(x)");
ax1.set_title("Trigonometric Functions");
ax1.set_xlabel("x");
ax1.set_ylabel("y");
ax1.legend();

let ax2 = fig.add_subplot(1, 2, 2);
ax2.scatter(&x, &y)?;
ax2.set_title("Scatter Plot");

fig.save("subplots.png")?;
```

## Twin Axes

Overlay two datasets with independent y-scales:

```rust
let mut fig = Figure::with_size(800, 600);
let ax1 = fig.add_subplot(1, 1, 1);
ax1.plot(&time, &temperature)?.label("Temperature");
ax1.set_ylabel("Temperature (°C)");

let ax2 = fig.twinx(0);
ax2.plot(&time, &pressure)?.label("Pressure");
ax2.set_ylabel("Pressure (hPa)");

ax1.legend();
```

## Colormaps & Colorbar

14 built-in colormaps: Viridis, Plasma, Inferno, Magma, Cividis, Turbo, Coolwarm, Spectral, RdYlBu, RdYlGn, PiYG, BrBG, PRGn, RdBu.

```rust
ax.heatmap(data)?.colormap(Colormap::Viridis).colorbar(true);
ax.scatter(&x, &y)?.colormap(Colormap::Plasma).c(&values);
```

## Scales & Axis Control

```rust
ax.set_xscale(Scale::Log10);
ax.set_yscale(Scale::SymLog { linthresh: 1.0 });
ax.set_xlim(0.1, 1000.0);
ax.set_xticks(&[0.1, 1.0, 10.0, 100.0, 1000.0]);
ax.invert_yaxis();
ax.grid(true);
```

## Annotations

```rust
ax.text(2.0, 0.8, "Peak", None);
ax.annotate(
    "Important point",
    (3.0, 0.5),   // data point
    (4.5, 0.8),   // text position
    ArrowStyle::Arrow,
);
```

## Themes

```rust
fig.set_theme(Theme::dark());   // Dark background, light text
fig.set_theme(Theme::default()); // Clean white background
```

Custom themes:

```rust
let theme = Theme {
    figure_background: Color::rgb(0xF5, 0xF5, 0xF5),
    font_family: Some("Helvetica".to_string()),
    ..Theme::default()
};
fig.set_theme(theme);
```

## DataFrame Integration

### Polars

```rust
use plotkit::prelude::*;

let df = polars::df! {
    "x" => [1.0, 2.0, 3.0, 4.0],
    "y" => [1.0, 4.0, 9.0, 16.0],
}?;

ax.plot(df.column("x")?, df.column("y")?)?;
```

### ndarray

```rust
use ndarray::Array1;

let x = Array1::linspace(0.0, 10.0, 100);
let y = x.mapv(f64::sin);
ax.plot(&x, &y)?;
```

## Output Formats

| Format | Method | Notes |
|--------|--------|-------|
| PNG | `fig.save("out.png")` | CPU-rendered, deterministic |
| SVG | `fig.save("out.svg")` | Scalable vector, feature `svg` |
| Bytes | `fig.to_png_bytes()` | For embedding / streaming |
| String | `fig.to_svg_string()` | For web / templating |

## Architecture

plotkit is a multi-crate workspace:

| Crate | Purpose |
|-------|---------|
| `plotkit` | Umbrella crate with pyplot API |
| `plotkit-core` | Figure, Axes, Artists, Renderer trait |
| `plotkit-render-skia` | PNG backend (tiny-skia + cosmic-text) |
| `plotkit-render-svg` | SVG backend |
| `plotkit-ndarray` | ndarray integration |
| `plotkit-polars` | Polars integration |

The `Renderer` trait is the abstraction boundary — core logic never depends on a specific backend. Adding a new output format means implementing one trait.

## Performance

Pure-Rust CPU renderer. No GPU, no system dependencies, no surprises.

plotkit compiles to a single static binary with zero runtime dependencies. It renders plots entirely on the CPU using optimized rasterization. There is no OpenGL context to initialize, no system font lookup to fail, and no shared library to go missing on a CI server at 2 AM.

Embedded Inter font (Regular + Bold) ensures deterministic, cross-platform text rendering — the same plot looks identical on Linux, macOS, and Windows.

## Roadmap

| Version | Focus |
|---------|-------|
| v0.5 | WASM support — render in the browser |
| v0.6 | 3D surface plots |
| v0.7 | Animation / frame sequences |
| v1.0 | Stable API, full documentation |

## Contributing

Contributions are welcome. Whether it's a bug report, a feature request, or a pull request — we appreciate it.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before getting started.

## License

Licensed under either of

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
