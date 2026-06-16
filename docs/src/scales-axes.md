# Scales & Axis Control

## Scales

A `Scale` transforms data values into axis space. plotkit supports three:

| Scale | Constructor | Use |
|---|---|---|
| Linear | `Scale::Linear` | The default, evenly spaced |
| Log base-10 | `Scale::Log10` | Positive data spanning orders of magnitude |
| Symmetric log | `Scale::SymLog { linthresh }` | Data crossing zero; linear within `±linthresh`, logarithmic beyond |
| Time | `Scale::Time` | Timestamps (seconds since the Unix epoch); ticks render as dates |

Set them per axis:

```rust
use plotkit::prelude::*;

ax.set_xscale(Scale::Log10);
ax.set_yscale(Scale::SymLog { linthresh: 1.0 });
```

`Log10` requires strictly positive data. On a log x-axis, minor tick marks are
generated automatically between the decade gridlines.

### Time axis

`Scale::Time` treats values as **seconds since the Unix epoch** and labels ticks
as calendar dates, choosing the granularity (`YYYY`, `YYYY-MM`, `MM-DD`,
`HH:MM`, `HH:MM:SS`) from the visible span:

```rust
let secs: Vec<f64> = /* Unix timestamps */;
ax.plot(&secs, &values)?;
ax.set_xscale(Scale::Time);
```

## Super- and subscripts

Titles, axis labels, and annotations accept `^`/`_` markup, resolved to Unicode:

```rust
ax.set_title("Quadratic: y = x^2");   // → y = x²
ax.set_ylabel("CO_2 (ppm)");          // → CO₂ (ppm)
ax.text(1.0, 2.0, "a^{n-1}");         // braces group multiple characters
```

Use `\^`, `\_`, or `\\` for a literal `^`, `_`, or `\`. Characters without a
Unicode super/subscript form are left unchanged.

## Limits

Without explicit limits, each axis auto-scales to the data (with a small
padding). Override with `set_xlim` / `set_ylim`:

```rust
ax.set_xlim(0.1, 1000.0);
ax.set_ylim(-5.0, 5.0);
```

## Ticks

Replace the auto-generated ticks with explicit positions, and optionally give
them custom labels:

```rust
ax.set_xticks(&[0.1, 1.0, 10.0, 100.0, 1000.0]);
ax.set_xticklabels(&["0.1", "1", "10", "100", "1k"]);
```

Rotate crowded tick labels:

```rust
ax.tick_params_x_rotation(45.0);
ax.tick_params_y_rotation(0.0);
```

## Inverting axes

```rust
ax.invert_xaxis();  // largest value on the left
ax.invert_yaxis();  // largest value at the bottom
```

## Grid

Toggle the grid and tune which axes show it, plus opacity and style:

```rust
ax.grid(true);
ax.grid_axis("y");                 // "x", "y", or "both"
ax.grid_alpha(0.4);
ax.grid_style(LineStyle::Dotted);
```

## Putting it together

```rust
use plotkit::prelude::*;

let ax = fig.add_subplot(1, 1, 1);
ax.plot(&x, &y)?;
ax.set_xscale(Scale::Log10);
ax.set_xlim(1.0, 1.0e6);
ax.set_xticks(&[1.0, 100.0, 10_000.0, 1_000_000.0]);
ax.invert_yaxis();
ax.grid(true);
```

These same operations are available on the current figure through the
`pyplot` facade: `xscale`, `yscale`, `xlim`, `ylim`, `xticks`, `yticks`, and
`grid`.
