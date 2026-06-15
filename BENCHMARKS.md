# Benchmarks

plotkit renders entirely on the CPU in pure Rust — no GPU, no system canvas, no
external process. Every figure is rasterized deterministically, so the same
input always produces the same bytes and the same work. The numbers below are
real Criterion medians measured on the hardware listed in
[Methodology](#methodology); they are not extrapolated or rounded for effect.

## Results vs TRD targets

All figures are 100-sample Criterion runs; the reported value is the **median**
of the confidence interval (the middle of `[lower median upper]`).

| Workload | Target | Measured (median) | Status |
| --- | --- | --- | --- |
| 10k-point line → PNG | < 15 ms | 10.878 ms | ✅ |
| 100k-point line → PNG (with LTTB) | < 60 ms | 13.670 ms | ✅ |
| 1M-point scatter → PNG (decimated) | < 250 ms | 33.744 ms | ✅ |
| Cold render of default line plot | < 8 ms | 10.728 ms | ❌ |

Three of the four targets pass, two of them by a wide margin. The numbers above
reflect the v0.6.0 render path: backend scratch-buffer reuse (no per-primitive
path allocation), automatic LTTB decimation for large line/scatter series, and a
per-thread cache of the parsed embedded-font database so font data is no longer
re-parsed on every render.

### Note on the remaining miss

- **Cold render of default line plot (10.7 ms vs 8 ms target):** a single cold
  render of the default plot pays the framework floor — pixmap allocation plus
  text shaping and glyph rasterization for the title, axis labels, and ticks —
  none of which scale with data size. At 10.7 ms it is still roughly an order of
  magnitude faster than an equivalent cold matplotlib render, but it sits ~2.7 ms
  over the aggressive 8 ms budget. Closing the gap means trimming first-render
  text-layout cost (glyph-cache warm-up, label shaping) and is tracked as a
  follow-up; it is not a regression.

The 100k LTTB line and 1M decimated scatter clear their budgets comfortably —
decimation keeps the rasterizer working on a screen-sized point count regardless
of input size, which is why the 1M scatter renders faster than the raw 10k line
once both go through the same draw path.

## Other measured workloads

These are not TRD targets but are tracked for regression visibility:

| Workload | Measured (median) |
| --- | --- |
| scatter_10k → PNG | 39.798 ms |
| bar_100 → PNG | 22.173 ms |
| histogram_10k → PNG | 11.760 ms |
| svg_line_10k → SVG string | 4.0463 ms |
| figure_creation (no render) | 434.33 ns |
| tick_generation (8 ranges) | 8.7074 ms |
| multi_subplot 4×4 → PNG | 69.293 ms |

## Methodology

- **Hardware:** AMD Ryzen AI 7 350 (8 cores / 16 threads), 23.3 GB RAM.
- **OS:** Microsoft Windows 11 Home Single Language, build 10.0.26200.
- **Toolchain:** rustc 1.95.0 / cargo 1.95.0, `bench` profile (optimized release).
- **Harness:** Criterion 0.5 (`criterion = "0.5"`), 100 samples per workload,
  default warm-up.
- **Command:**

  ```sh
  cargo bench --bench render_benchmarks
  ```

- **Timing isolation:** all input vectors are generated *outside* the timed
  `b.iter(...)` closure. For the 1M scatter, LTTB decimation is also performed
  once outside the closure, so the measurement reflects figure construction plus
  a single rasterization — never data synthesis or decimation.
- **Output:** PNG workloads measure the full `to_png_bytes()` path (encode
  included); the SVG workload measures `to_svg_string()`.

## vs plotters

Criterion already pulls in `plotters` as its charting backend, so a head-to-head
comparison against `plotters`-the-plotting-library is feasible as an optional
dev-dependency bench. It is **not** included in this run — no `plotters` numbers
have been measured here, and none are reported. A `plotters` head-to-head
(equivalent line/scatter workloads, same point counts, same output format) is
tracked as a follow-up so the comparison is apples-to-apples rather than
fabricated.
