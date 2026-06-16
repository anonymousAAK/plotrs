# Installation

plotkit targets a recent stable Rust toolchain (edition 2021, Rust 1.75+).

Add it with `cargo add`:

```bash
cargo add plotkit
```

or edit `Cargo.toml` directly:

```toml
[dependencies]
plotkit = "1"
```

By default the `png` and `svg` features are enabled, so PNG rasterization and
SVG vector output work with no extra configuration.

## Optional features

Enable additional backends and integrations through Cargo features:

```bash
cargo add plotkit --features pdf,jupyter,ndarray,polars
```

```toml
[dependencies]
plotkit = { version = "1", features = ["pdf", "jupyter", "ndarray", "polars"] }
```

| Feature   | Default | Description                                              |
|-----------|:-------:|----------------------------------------------------------|
| `png`     |   yes   | PNG rasterization (the default raster backend)           |
| `svg`     |   yes   | SVG vector output                                        |
| `pdf`     |   no    | PDF vector output and `to_pdf_bytes`                     |
| `jupyter` |   no    | Inline display in Evcxr (Rust Jupyter) notebooks         |
| `ndarray` |   no    | Plot directly from `ndarray::Array1` / `ArrayView1`      |
| `polars`  |   no    | Plot directly from Polars `Series` and `DataFrame`       |

To opt out of the defaults (for example, an SVG-only build) disable default
features and re-enable just what you need:

```toml
[dependencies]
plotkit = { version = "1", default-features = false, features = ["svg"] }
```

## Importing

Most code pulls in the prelude, which re-exports the common types
(`Figure`, `Axes`, `Theme`, `Colormap`, `Scale`, `Color`, `Marker`, …) and the
`pyplot`-style free functions:

```rust
use plotkit::prelude::*;
```

The `FigureExt` trait (providing `fig.save`, `fig.to_png_bytes`,
`fig.to_svg_string`, and `fig.to_pdf_bytes`) is included in the prelude as
well.
