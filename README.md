<div align="center">

# plotkit

### Publication-quality plots in three lines of Rust.

[![Crates.io](https://img.shields.io/crates/v/plotkit.svg?style=flat-square&logo=rust&color=E15759)](https://crates.io/crates/plotkit)
[![docs.rs](https://img.shields.io/docsrs/plotkit?style=flat-square&logo=docs.rs)](https://docs.rs/plotkit)
[![CI](https://img.shields.io/github/actions/workflow/status/anonymousAAK/plotrs/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/anonymousAAK/plotrs/actions)
[![License](https://img.shields.io/crates/l/plotkit.svg?style=flat-square&color=4E79A7)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.75-blue?style=flat-square&logo=rust)](#installation)

**[Documentation](https://docs.rs/plotkit)** · **[User Guide](docs/)** · **[Gallery](examples/)** · **[Benchmarks](BENCHMARKS.md)**

</div>

---

Rust deserves charts that belong in a journal paper — without shelling out to Python,
booting a browser, or hunting for a system font that exists on your laptop but not on CI.

**plotkit** is a pure-Rust plotting library with a matplotlib-shaped API. Add one crate,
write three lines, and get an anti-aliased, professionally themed figure as a PNG, SVG, or
PDF. No GPU. No subprocess. No system dependencies. Identical bytes on Linux, macOS, and Windows.

```rust
plotkit::plot(&x, &y)?;
plotkit::title("sin(x)");
plotkit::savefig("plot.png")?;
```

## Why plotkit

- **🎨 It looks good by default.** A hand-tuned theme, the Tableau-10 palette, the
  Talbot–Lin–Hanrahan tick algorithm, and an automatic layout that never clips a label.
- **⚡ It's fast.** A 10,000-point line renders to PNG in **~11 ms**; a decimated
  1,000,000-point scatter in **~34 ms** — all on the CPU. See [BENCHMARKS.md](BENCHMARKS.md).
- **📦 It's self-contained.** One static binary, zero runtime dependencies. The Inter font
  is embedded, so text renders deterministically everywhere — no font lookup to fail at 2 AM.
- **🐍 It's familiar.** If you know matplotlib, you already know plotkit:
  `Figure`, `Axes`, `ax.plot(...)`, `ax.scatter(...)`, `fig.savefig(...)`.
- **🔌 It's composable.** Plot straight from Polars `DataFrame`s and `ndarray` arrays,
  render inline in Jupyter (evcxr), or draw to an HTML canvas via WebAssembly.

## Installation

```toml
[dependencies]
plotkit = "1.0"
```

<details>
<summary><b>Optional features</b></summary>

```toml
plotkit = { version = "1.0", features = ["svg", "pdf", "jupyter", "ndarray", "polars"] }
```

| Feature   | Description                                       |
|-----------|---------------------------------------------------|
| `png`     | PNG rasterization (**default**)                   |
| `svg`     | SVG vector output (**default**)                   |
| `pdf`     | Print-ready PDF vector output                     |
| `jupyter` | Inline display in Evcxr notebooks                 |
| `ndarray` | Plot directly from `ndarray::Array1`              |
| `polars`  | Plot directly from Polars `Series` / `DataFrame`  |

</details>

## Quick start

The **pyplot facade** is perfect for scripts — it operates on a current figure, just like
`matplotlib.pyplot`:

```rust
use plotkit::prelude::*;

let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

plotkit::plot(&x, &y)?;
plotkit::title("sin(x)");
plotkit::xlabel("x");
plotkit::ylabel("y");
plotkit::savefig("sine.png")?;
```

For full control, drop down to the **Figure / Axes** model:

```rust
use plotkit::prelude::*;

let mut fig = Figure::with_size(1200, 500);

let ax1 = fig.add_subplot(1, 2, 1);
ax1.plot(&x, &sin_y)?.label("sin(x)").color(Color::TAB_BLUE);
ax1.plot(&x, &cos_y)?.label("cos(x)").color(Color::TAB_ORANGE);
ax1.set_title("Trigonometric Functions");
ax1.legend();

let ax2 = fig.add_subplot(1, 2, 2);
ax2.scatter(&x, &y)?.marker(Marker::Circle);
ax2.set_title("Scatter");

fig.tight_layout();
fig.save("subplots.png")?;
```

## Chart types

Sixteen chart types, one familiar API:

| | | | |
|---|---|---|---|
| `plot` — line | `scatter` | `bar` / `barh` | `bar_group` |
| `hist` | `fill_between` | `step` | `stem` |
| `boxplot` | `violin` | `errorbar` | `heatmap` |
| `pie` | `contour` / `contourf` | `hexbin` | `polar` / `waterfall` |

Browse the **[gallery of 55+ runnable examples](examples/)** — each is a complete program:

```sh
cargo run --example 21_multi_line
cargo run --example 47_df_plot --features polars
```

## DataFrames & arrays

Polars and ndarray are first-class inputs. Any `Series`, `Array1`, or slice flows through
the same `IntoSeries` trait — zero glue code:

```rust
use ndarray::Array1;
let x = Array1::linspace(0.0, 10.0, 100);
ax.plot(&x, &x.mapv(f64::sin))?;
```

Polars users get a pandas-style **`df.plot()`** accessor with categorical grouping:

```rust
use plotkit::prelude::*;
use plotkit::plotkit_polars::DataFramePlotExt;

df.plot()
    .color_by("symbol")          // one labelled series per category
    .line("date", "price")?
    .save("stocks.png")?;
```

## Themes, colormaps & scales

```rust
fig.set_theme(Theme::dark());          // default · dark · seaborn · ggplot · publication · nature · solarized
ax.scatter(&x, &y)?.cmap(Colormap::Viridis).c(values);   // 14 perceptual colormaps
ax.set_yscale(Scale::Log10);           // Linear · Log10 · SymLog
```

<details>
<summary><b>Axis control, twin axes, annotations</b></summary>

```rust
ax.set_xlim(0.1, 1000.0);
ax.set_xticks(&[0.1, 1.0, 10.0, 100.0, 1000.0]);
ax.invert_yaxis();
ax.grid(true);

let ax2 = fig.twinx(0);                 // independent secondary y-axis
ax2.plot(&time, &pressure)?;

ax.annotate("peak", (3.0, 0.5), (4.5, 0.8));
ax.text(2.0, 0.8, "note");
```

</details>

## Output formats

| Format | Method | Notes |
|--------|--------|-------|
| PNG | `fig.save("out.png")` | CPU-rasterized, deterministic |
| SVG | `fig.save("out.svg")` | Scalable vector (`svg` feature) |
| PDF | `fig.save("out.pdf")` | Print-ready vector (`pdf` feature) |
| Bytes | `fig.to_png_bytes()` / `fig.to_pdf_bytes()` | Embed or stream |
| String | `fig.to_svg_string()` | Web / templating |

Format is selected from the file extension — one method, every backend.

## Performance

Pure-CPU rendering, measured with Criterion (medians, AMD Ryzen AI 7 350):

| Workload | Measured |
|---|---|
| 10k-point line → PNG | **10.9 ms** |
| 100k-point line → PNG (auto LTTB) | **13.7 ms** |
| 1M-point scatter → PNG (decimated) | **33.7 ms** |
| Figure creation (no render) | **434 ns** |

Automatic [LTTB](https://github.com/sveinn-steinarsson/flot-downsample) decimation keeps the
rasterizer working on a screen-sized point count no matter how large the input. Full
methodology and TRD targets in **[BENCHMARKS.md](BENCHMARKS.md)**.

## Render anywhere — including the browser

The `Renderer` trait is the only seam between plot logic and pixels, so the same figure code
targets every backend. `plotkit-render-wasm` draws to an HTML5 canvas via WebAssembly:

```sh
wasm-pack build crates/plotkit-render-wasm --target web --out-dir ../../web/pkg
```

A ready-to-serve demo lives in [`web/`](web/).

## Architecture

plotkit is a focused multi-crate workspace:

| Crate | Purpose |
|-------|---------|
| `plotkit` | Umbrella crate — pyplot facade + `save` |
| `plotkit-core` | `Figure`, `Axes`, artists, the `Renderer` trait |
| `plotkit-render-skia` | PNG backend (tiny-skia + cosmic-text) |
| `plotkit-render-svg` | SVG backend |
| `plotkit-render-pdf` | PDF backend (printpdf) |
| `plotkit-render-wasm` | WASM Canvas2D backend |
| `plotkit-ndarray` · `plotkit-polars` | DataFrame / array integration |

Core logic never depends on a concrete backend — adding a new output format means
implementing one trait.

## Status & roadmap

**v1.0 — stable.** The public API is semver-stable; signatures shipped in 1.0 only grow,
never break. Shipping today: 16 chart types, PNG/SVG/PDF, 7 themes, 14 colormaps,
log/symlog scales, twin axes, subplots, Polars + ndarray, Jupyter inline rendering, and a
WASM backend.

| Version | Focus |
|---------|-------|
| v1.1 | Animation / frame sequences |
| v1.2 | 3D surface plots |
| v1.x | Interactive widgets; optional GPU (Vello) backend behind the `Renderer` trait |

## Contributing

Bug reports, feature requests, and pull requests are all welcome — start with
[CONTRIBUTING.md](CONTRIBUTING.md). Every visual change needs an `insta` golden-image
snapshot; run `cargo insta review` before committing.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
