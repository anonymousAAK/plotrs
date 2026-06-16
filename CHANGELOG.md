# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0]

First stable release. The public API is now semver-stable: signatures shipped in
1.0 will not change within the 1.x series (additions only).

### Added

- **PDF output wired into the umbrella crate**: `fig.save("out.pdf")` and
  `Figure::to_pdf_bytes()` behind the new `pdf` feature. Previously the PDF
  backend existed but was unreachable from `save`.
- **WASM backend fixed and demoed**: `plotkit-render-wasm` now compiles for
  `wasm32-unknown-unknown` (it was missing the `web-sys` `TextMetrics`
  feature), exports `render_demo`, and ships a browser demo under `web/`.
- **Gallery**: expanded to 50+ runnable examples covering every chart type,
  theme, colormap, and scale.
- **mdBook user guide** under `docs/`, deployed to GitHub Pages.
- **Release hardening**: `cargo-semver-checks` enforced in release-plz and CI;
  `cargo-deny` and a `wasm32` build added to CI.

### Changed

- All crates bumped to `1.0.0`.
- README corrected and expanded: accurate colormap list, `scatter().cmap()`
  usage, seven themes, a PDF section, and a post-1.0 roadmap.

### Stability

- **Stable:** the pyplot facade, `Figure` / `Axes` construction and the core
  chart methods, `IntoSeries`, and the output API (`save` / `to_*_bytes`).
- **Evolving (non-exhaustive enums; variants may be added):** `Theme`,
  `Marker`, `LineStyle`, `Loc`, `Scale`, `Colormap`, `PlotError`.
