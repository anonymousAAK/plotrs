<div align="center">

# plotkit

### Publication-quality plots in 3 lines of Rust

[![Crates.io](https://img.shields.io/crates/v/plotkit.svg)](https://crates.io/crates/plotkit)
[![docs.rs](https://docs.rs/plotkit/badge.svg)](https://docs.rs/plotkit)
[![CI](https://github.com/pyscripter99/plotkit/actions/workflows/ci.yml/badge.svg)](https://github.com/pyscripter99/plotkit/actions)
[![License](https://img.shields.io/crates/l/plotkit.svg)](https://github.com/pyscripter99/plotkit)

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

That's it. Three lines. You get anti-aliased text, a clean grid, and a color palette that doesn't look like it was chosen in 1997.

### Compare that to matplotlib

```python
import matplotlib.pyplot as plt
import numpy as np

x = np.linspace(0, 10, 100)
y = np.sin(x)
plt.plot(x, y)
plt.title("sin(x)")
plt.savefig("plot.png")
```

Same result. More ceremony. And you had to leave Rust to get it.

## Features

| Feature | Status |
|---|---|
| **Line plots** | Supported |
| **Scatter plots** | Supported |
| **Bar charts** | Supported |
| **Histograms** | Supported |
| **fill_between** | Supported |
| **PNG output** | Supported |
| **SVG output** | Supported |
| **Beautiful defaults** | Tableau-10 palette, embedded fonts |
| **Figure/Axes API** | Full control over layout and subplots |
| **Theme system** | Swap styles without touching your plot code |

## Installation

```sh
cargo add plotkit
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
plotkit = "0.0.1"
```

## Figure & Axes API

When you need more control — subplots, shared axes, fine-grained styling — drop down to the `Figure`/`Axes` API:

```rust
use plotkit::{Figure, Axes};

let mut fig = Figure::new();

// 1x2 subplot grid
let ax1 = fig.add_subplot(1, 2, 1);
ax1.plot(&x, &sin_y);
ax1.set_title("sin(x)");
ax1.set_xlabel("x");
ax1.set_ylabel("y");

let ax2 = fig.add_subplot(1, 2, 2);
ax2.scatter(&x, &cos_y);
ax2.set_title("cos(x)");
ax2.set_xlabel("x");

fig.savefig("subplots.png")?;
```

### Styling with Themes

```rust
use plotkit::theme;

// Use a built-in theme
theme::set("dark");

plotkit::plot(&x, &y)?;
plotkit::savefig("dark_plot.svg")?;
```

### Filled Regions

```rust
use plotkit::Axes;

let mut ax = Axes::new();
ax.fill_between(&x, &lower, &upper, Some("Confidence band"));
ax.plot(&x, &mean);
ax.legend();
ax.savefig("confidence.png")?;
```

## Performance

Pure-Rust CPU renderer. No GPU, no system dependencies, no surprises.

plotkit compiles to a single static binary with zero runtime dependencies. It renders plots entirely on the CPU using optimized rasterization. There is no OpenGL context to initialize, no system font lookup to fail, and no shared library to go missing on a CI server at 2 AM.

It's fast enough for batch rendering thousands of figures, and lightweight enough to embed in CLI tools where adding a plotting dependency shouldn't double your compile time.

## Roadmap

What's coming next:

- **Polars integration** — plot directly from DataFrames
- **PDF export** — vector output for print-ready figures
- **WASM support** — render plots in the browser, no backend required
- **Jupyter integration** — inline plots in Jupyter notebooks via Evcxr

## Contributing

Contributions are welcome. Whether it's a bug report, a feature request, or a pull request — we appreciate it.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before getting started.

## License

Licensed under either of

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
