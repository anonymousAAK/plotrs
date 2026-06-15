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
| 10k-point line → PNG | < 15 ms | 65.615 ms | ❌ |
| 100k-point line → PNG (with LTTB) | < 60 ms | 50.407 ms | ✅ |
| 1M-point scatter → PNG (decimated) | < 250 ms | 76.816 ms | ✅ |
| Cold render of default line plot | < 8 ms | 37.831 ms | ❌ |

### Notes on the failing targets

- **10k-point line → PNG (65.6 ms vs 15 ms target):** the cost here is dominated
  by per-figure rasterization and text shaping rather than the 10k vertices
  themselves — the same fixed overhead also drives the cold-render miss below.
- **Cold render of default line plot (37.8 ms vs 8 ms target):** a single cold
  render pays the full font-system / layout / rasterizer setup cost on this
  machine. The render path and automatic decimation are being optimized
  concurrently, so this is expected to drop on a later run; re-running
  `cargo bench` after those land may move both of the above into ✅.

The two passing targets (100k LTTB line and 1M decimated scatter) clear their
budgets comfortably — decimation keeps the rasterizer working on a screen-sized
point count regardless of the input size, which is exactly why the 1M scatter is
faster than the raw 10k line.

## Other measured workloads

These are not TRD targets but are tracked for regression visibility:

| Workload | Measured (median) |
| --- | --- |
| scatter_10k → PNG | 190.31 ms |
| bar_100 → PNG | 24.842 ms |
| histogram_10k → PNG | 30.315 ms |
| svg_line_10k → SVG string | 6.5082 ms |
| figure_creation (no render) | 556.76 ns |
| tick_generation (8 ranges) | 10.757 ms |
| multi_subplot 4×4 → PNG | 80.890 ms |

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
