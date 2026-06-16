# Colormaps & Colorbars

A `Colormap` maps a continuous scalar value to a color. Colormaps drive
heatmaps, filled contours, hexbins, and color-mapped scatter plots, and pair
with a colorbar that shows the value-to-color legend.

## Named colormaps

`Colormap` provides 14 well-known scientific and perceptual colormaps:

| Category | Variants |
|---|---|
| Perceptually uniform sequential | `Viridis`, `Plasma`, `Inferno`, `Magma`, `Cividis` |
| Rainbow-like | `Turbo` |
| Diverging | `Coolwarm`, `Spectral`, `RdBu` |
| Single-hue sequential | `Blues`, `Reds`, `Greens`, `Greys` |
| Multi-hue sequential | `YlOrRd` |

You can also map values directly: `Colormap::Viridis.map(0.5)` returns a
`Color` for a `[0, 1]` input, and `map_value(v, vmin, vmax)` normalises first.

## Colorbars

Many chart builders expose a `.colorbar(true)` flag that attaches a colorbar
automatically, sized from the chart's own value range:

```rust
use plotkit::prelude::*;

ax.heatmap(matrix)?
  .colormap(Colormap::Viridis)
  .colorbar(true);
```

For full control, attach an explicit colorbar to the axes with
`ax.colorbar(cmap, vmin, vmax)`, which returns a `&mut Colorbar` for builder
configuration:

```rust
ax.colorbar(Colormap::Plasma, 0.0, 100.0)
  .set_label("Temperature (C)")
  .set_orientation(ColorbarOrientation::Vertical)
  .set_num_steps(128);
```

## Color-mapped scatter

For a scatter plot, pass a third array of values with `.c(values)` and choose
the colormap with `.cmap(colormap)` (note: scatter uses `.cmap`, while
heatmap, contour, and hexbin use `.colormap`):

```rust
use plotkit::prelude::*;

ax.scatter(&x, &y)?
  .c(magnitudes)          // Vec<f64>, one value per point
  .cmap(Colormap::Plasma)
  .size(8.0);
```

## Other colormap-driven charts

```rust
ax.contourf(&x, &y, z)?.colormap(Colormap::Inferno);
ax.hexbin(&x, &y)?.colormap(Colormap::Turbo).colorbar(true);
```
