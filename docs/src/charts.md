# Chart Types

Every chart is a method on `Axes`. The data-adding methods that return a
`&mut` artist let you chain builder methods (`.color()`, `.label()`, …) to
style the series. All examples assume:

```rust
use plotkit::prelude::*;
let ax = fig.add_subplot(1, 1, 1);
```

## Reference table

| Chart | Method | Returns |
|---|---|---|
| Line | `ax.plot(x, y)` | `&mut LineArtist` |
| Scatter | `ax.scatter(x, y)` | `&mut ScatterArtist` |
| Bar | `ax.bar(cats, heights)` | `&mut BarArtist` |
| Horizontal bar | `ax.barh(cats, widths)` | `&mut BarArtist` |
| Grouped bar | `ax.bar_group(cats, series)` | `Result<()>` |
| Histogram | `ax.hist(data, bins)` | `&mut HistArtist` |
| Fill between | `ax.fill_between(x, y1, y2)` | `&mut FillBetweenArtist` |
| Step | `ax.step(x, y)` | `&mut StepArtist` |
| Stem | `ax.stem(x, y)` | `&mut StemArtist` |
| Box plot | `ax.boxplot(datasets)` | `&mut BoxPlotArtist` |
| Violin | `ax.violin(datasets)` | `&mut ViolinArtist` |
| Error bar | `ax.errorbar(x, y)` | `ErrorBarArtist` (owned) |
| Heatmap | `ax.heatmap(data)` | `&mut HeatmapArtist` |
| Pie | `ax.pie(sizes)` | `&mut PieArtist` |
| Contour | `ax.contour(x, y, z)` | `&mut ContourArtist` |
| Filled contour | `ax.contourf(x, y, z)` | `&mut ContourArtist` |
| Hexbin | `ax.hexbin(x, y)` | `&mut HexbinArtist` |
| Polar line | `ax.polar_plot(theta, r)` | `&mut PolarArtist` |
| Polar fill (radar) | `ax.polar_fill(theta, r)` | `&mut PolarArtist` |
| Waterfall | `ax.waterfall(cats, values)` | `&mut WaterfallArtist` |

Data arguments accept anything implementing `IntoSeries` (slices, `Vec<f64>`,
`Vec<i32>`, arrays, integer ranges, and — with the relevant feature —
`ndarray` and Polars types). Category arguments accept `IntoCategories`
(`&[&str]`, `Vec<String>`, …).

## Line

```rust
ax.plot(&x, &y)?
  .color(Color::TAB_BLUE)
  .width(2.0)
  .style(LineStyle::Dashed)
  .label("series A");
```

## Scatter

```rust
ax.scatter(&x, &y)?
  .marker(Marker::Circle)
  .size(8.0)
  .label("points");
```

Map a third value to color with `.c(values)` and `.cmap(colormap)` — see
[Colormaps](./colormaps.md).

## Bar and horizontal bar

```rust
ax.bar(&["Q1", "Q2", "Q3", "Q4"][..], vec![10.0, 14.0, 9.0, 18.0])?
  .color(Color::TAB_GREEN)
  .label("2024");

ax.barh(&["A", "B", "C"][..], vec![3.0, 7.0, 5.0])?;
```

Stack bars by passing `.bottom(prev_heights)` on a second `bar` call.

## Grouped bar

`bar_group` places several series side-by-side within each category. The
series slice is `&[(&str, Vec<f64>)]`:

```rust
ax.bar_group(
    &["North", "South", "East"][..],
    &[
        ("2023", vec![10.0, 12.0, 9.0]),
        ("2024", vec![14.0, 11.0, 13.0]),
    ],
)?;
ax.legend();
```

## Histogram

```rust
ax.hist(&samples, 30)?.alpha(0.85).label("distribution");
```

## Fill between

```rust
ax.fill_between(&x, &lower, &upper)?
  .color(Color::TAB_BLUE)
  .alpha(0.3)
  .label("confidence band");
```

## Step and stem

```rust
ax.step(&x, &y)?.width(1.5).label("staircase");
ax.stem(&x, &y)?.baseline(0.0).marker_size(6.0);
```

## Box plot and violin

Both take `Vec<Vec<f64>>` — one inner vector per group.

```rust
ax.boxplot(vec![group_a.clone(), group_b.clone()])?
  .show_outliers(true)
  .width(0.5);

ax.violin(vec![group_a, group_b])?
  .show_median(true)
  .show_quartiles(true);
```

## Error bar

`errorbar` returns an owned artist that you configure and then add with
`add_errorbar`:

```rust
let eb = ax.errorbar(&x, &y)?
    .yerr_symmetric(&y_errors)
    .cap_size(4.0)
    .label("measured");
ax.add_errorbar(eb);
```

Asymmetric errors use `.yerr_asymmetric(low, high)` (and `xerr_*` variants).

## Heatmap

`heatmap` takes a `Vec<Vec<f64>>` grid:

```rust
ax.heatmap(matrix)?
  .colormap(Colormap::Viridis)
  .show_values(true)
  .colorbar(true);
```

## Pie

```rust
ax.pie(vec![30.0, 25.0, 20.0, 25.0])?
  .labels(vec!["A", "B", "C", "D"])
  .autopct(true)
  .start_angle(90.0);
```

## Contour and filled contour

Both take `x: &[f64]`, `y: &[f64]`, and `z: Vec<Vec<f64>>`:

```rust
ax.contour(&x, &y, z.clone())?.num_levels(12);
ax.contourf(&x, &y, z)?.colormap(Colormap::Plasma);
```

## Hexbin

```rust
ax.hexbin(&x, &y)?
  .gridsize(25)
  .colormap(Colormap::Inferno)
  .mincnt(1)
  .colorbar(true);
```

## Polar and radar

`theta` is in radians; `polar_fill` closes and fills the path for a radar/area
chart:

```rust
ax.polar_plot(&theta, &r)?.label("orbit");
ax.polar_fill(&theta, &r)?.label("radar");
```

## Waterfall

`waterfall` draws incremental changes from a running total and sets the x-axis
tick labels to the category names automatically:

```rust
ax.waterfall(
    &["Start", "Sales", "Costs", "End"][..],
    vec![100.0, 40.0, -30.0, 0.0],
)?.show_values(true);
```
