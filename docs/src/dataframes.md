# DataFrames & Arrays

plotkit's chart methods are generic over the `IntoSeries` trait, so beyond the
built-in `Vec<f64>`, `Vec<i32>`, slices, arrays, and integer ranges, you can
plot `ndarray` arrays and Polars columns directly when the matching feature is
enabled. Categorical inputs (bar/pie labels) go through the `IntoCategories`
trait, which Polars string columns also implement.

## ndarray

Enable the `ndarray` feature:

```toml
[dependencies]
plotkit = { version = "1", features = ["ndarray"] }
ndarray = "0.16"
```

`Array1<f64>`, `&Array1<f64>`, and `ArrayView1<f64>` all implement
`IntoSeries` (with zero-copy where the memory layout allows). `f32`, `i32`,
and `i64` arrays are accepted too and cast to `f64`.

```rust
use plotkit::prelude::*;
use ndarray::Array1;

let x = Array1::linspace(0.0, 10.0, 200);
let y = x.mapv(f64::sin);

let mut fig = Figure::with_size(800, 600);
let ax = fig.add_subplot(1, 1, 1);
ax.plot(&x, &y)?;
fig.save("ndarray.png")?;
```

## Polars

Enable the `polars` feature:

```toml
[dependencies]
plotkit = { version = "1", features = ["polars"] }
polars = "0.43"
```

Both `polars::Series` and `&polars::Series` implement `IntoSeries`. Numeric
columns are cast to `f64` (null entries become `NaN`); string columns
implement `IntoCategories` for use as bar/pie labels (nulls become `"null"`).

```rust
use plotkit::prelude::*;
use polars::prelude::*;

let df = df! {
    "x" => [1.0, 2.0, 3.0, 4.0],
    "y" => [1.0, 4.0, 9.0, 16.0],
}?;

let mut fig = Figure::with_size(800, 600);
let ax = fig.add_subplot(1, 1, 1);
ax.plot(df.column("x")?, df.column("y")?)?;
fig.save("polars.png")?;
```

A string column drives categorical bars:

```rust
let df = df! {
    "label" => ["A", "B", "C"],
    "value" => [10.0, 14.0, 9.0],
}?;
ax.bar(df.column("label")?, df.column("value")?)?;
```

Because every chart method takes `impl IntoSeries` / `impl IntoCategories`,
the same code works whether the data comes from a `Vec`, an `ndarray` array,
or a Polars column — no manual conversion required.
