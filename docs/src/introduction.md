# Introduction

**plotkit** is a publication-quality plotting library for Rust with a
matplotlib-familiar API. It renders charts entirely on the CPU in pure Rust —
no Python subprocess, no JavaScript runtime, no GPU context, and no system
dependencies. Add one crate to your `Cargo.toml` and you can produce figures
that belong in a journal paper.

```rust
plotkit::plot(&x, &y)?;
plotkit::title("sin(x)");
plotkit::savefig("plot.png")?;
```

Three lines: anti-aliased text, clean despined axes, and a Tableau-10 color
palette out of the box.

## Why plotkit?

- **Looks good by default.** The default theme follows a deliberate visual
  design brief — white background, light grid behind data, despined axes, and
  a curated categorical palette — so a zero-styling plot still looks
  professional.
- **Zero external runtime.** plotkit compiles to a single static binary. There
  is no OpenGL context to initialize, no system font lookup to fail, and no
  shared library to go missing on a CI server at 2 AM. An embedded font ensures
  the same plot renders identically on Linux, macOS, and Windows.
- **Two APIs.** A `pyplot`-style facade of free functions for quick scripts,
  and a `Figure`/`Axes` object model for full control over subplots, twin axes,
  themes, and layout.
- **Backend-agnostic core.** A `Renderer` trait is the abstraction boundary;
  PNG, SVG, and PDF backends all implement the same trait, so output format is
  just a matter of file extension.

## How this guide is organised

Start with [Installation](./installation.md) and [Quickstart](./quickstart.md)
to make your first plot, then read [Figure & Axes](./figure-axes.md) for the
object model. [Chart Types](./charts.md) is the reference for every chart, and
the remaining chapters cover [themes](./themes.md),
[colormaps](./colormaps.md), [axis scales](./scales-axes.md),
[dataframe integration](./dataframes.md), and
[output formats](./output-formats.md).
