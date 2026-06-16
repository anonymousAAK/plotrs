# Output Formats

plotkit selects the output backend by file extension. The same figure renders
to PNG, SVG, or PDF without changing any drawing code.

## Saving by extension

With the `pyplot` facade, `savefig` saves the current figure:

```rust
plotkit::savefig("plot.png")?;
plotkit::savefig("plot.svg")?;   // requires the `svg` feature
```

With the object model, bring the `FigureExt` trait into scope (it is in the
prelude) and call `fig.save`:

```rust
use plotkit::prelude::*;

fig.save("plot.png")?;   // PNG raster (default backend)
fig.save("plot.svg")?;   // SVG vector  (feature `svg`)
fig.save("plot.pdf")?;   // PDF vector  (feature `pdf`)
```

| Extension | Backend | Feature |
|---|---|---|
| `.png` | tiny-skia raster | `png` (default) |
| `.svg` | SVG vector | `svg` (default) |
| `.pdf` | PDF vector | `pdf` |

Saving to an unsupported or disabled extension returns
`PlotError::UnsupportedFormat`.

## Rendering to bytes and strings

For embedding, streaming, or templating, render without touching the
filesystem:

```rust
use plotkit::prelude::*;

let png: Vec<u8> = fig.to_png_bytes()?;     // always available
let svg: String  = fig.to_svg_string()?;    // requires `svg`
let pdf: Vec<u8> = fig.to_pdf_bytes()?;      // requires `pdf`
```

`to_png_bytes` is handy for serving a chart over HTTP, `to_svg_string` for
inlining vector graphics into HTML, and `to_pdf_bytes` for assembling reports.

## Figure size

Output dimensions come from the figure. PNG output is in pixels; the SVG and
PDF backends use the same width/height as their canvas size:

```rust
let fig = Figure::with_size(1600, 900);
```

PNG output is deterministic and cross-platform: the embedded font guarantees
identical text rendering on Linux, macOS, and Windows.
