//! # plotkit
//!
//! Publication-quality plots in 3 lines of Rust.
//!
//! ```no_run
//! let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
//! let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
//! plotkit::plot(&x, &y).unwrap();
//! plotkit::title("sin(x)");
//! plotkit::savefig("plot.png").unwrap();
//! ```
//!
//! ## Two APIs
//!
//! **pyplot-style** (quick scripts):
//! ```no_run
//! let x = vec![0.0_f64, 1.0, 2.0];
//! let y = vec![0.0_f64, 1.0, 4.0];
//! plotkit::plot(&x, &y).unwrap();
//! plotkit::savefig("out.png").unwrap();
//! ```
//!
//! **Figure/Axes** (full control):
//! ```no_run
//! use plotkit::prelude::*;
//! let mut fig = Figure::with_size(800, 600);
//! let ax = fig.add_subplot(1, 1, 1);
//! // ax.plot(&x, &y).unwrap().label("sin(x)");
//! // ax.legend();
//! ```

#![deny(missing_docs)]

#[cfg(feature = "jupyter")]
pub mod jupyter;

pub use plotkit_core::{primitives, renderer, error, series, scale, ticks, theme, layout, artist, axes, figure, legend, charts, colormap};
pub use plotkit_core::figure::Figure;
pub use plotkit_core::axes::Axes;
pub use plotkit_core::primitives::Color;
pub use plotkit_core::theme::{Theme, LineStyle, Marker, Loc};
pub use plotkit_core::scale::Scale;
pub use plotkit_core::series::{IntoSeries, IntoCategories};
pub use plotkit_core::error::{PlotError, Result};

use std::cell::RefCell;
use std::path::Path;

thread_local! {
    static CURRENT_FIGURE: RefCell<Figure> = RefCell::new(Figure::new());
}

/// The plotkit prelude — import this for convenient access to all common types.
pub mod prelude {
    pub use plotkit_core::prelude::*;
    pub use crate::{plot, scatter, bar, hist, title, xlabel, ylabel, legend, savefig, clf, subplots};
    pub use crate::FigureExt;
}

// === pyplot-style free functions ===

/// Plots a line on the current figure.
pub fn plot<X: IntoSeries, Y: IntoSeries>(x: X, y: Y) -> Result<()> {
    CURRENT_FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        if fig.num_axes() == 0 {
            fig.add_subplot(1, 1, 1);
        }
        fig.axes_mut(0).expect("axes[0] exists after add_subplot").plot(x, y)?;
        Ok(())
    })
}

/// Creates a scatter plot on the current figure.
pub fn scatter<X: IntoSeries, Y: IntoSeries>(x: X, y: Y) -> Result<()> {
    CURRENT_FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        if fig.num_axes() == 0 {
            fig.add_subplot(1, 1, 1);
        }
        fig.axes_mut(0).expect("axes[0] exists after add_subplot").scatter(x, y)?;
        Ok(())
    })
}

/// Creates a bar chart on the current figure.
pub fn bar<C: IntoCategories, H: IntoSeries>(categories: C, heights: H) -> Result<()> {
    CURRENT_FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        if fig.num_axes() == 0 {
            fig.add_subplot(1, 1, 1);
        }
        fig.axes_mut(0).expect("axes[0] exists after add_subplot").bar(categories, heights)?;
        Ok(())
    })
}

/// Creates a histogram on the current figure.
pub fn hist<D: IntoSeries>(data: D, bins: usize) -> Result<()> {
    CURRENT_FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        if fig.num_axes() == 0 {
            fig.add_subplot(1, 1, 1);
        }
        fig.axes_mut(0).expect("axes[0] exists after add_subplot").hist(data, bins)?;
        Ok(())
    })
}

/// Sets the title of the current axes.
pub fn title(s: &str) {
    CURRENT_FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        if fig.num_axes() == 0 {
            fig.add_subplot(1, 1, 1);
        }
        fig.axes_mut(0).expect("axes[0] exists after add_subplot").set_title(s);
    })
}

/// Sets the x-axis label.
pub fn xlabel(s: &str) {
    CURRENT_FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        if fig.num_axes() == 0 {
            fig.add_subplot(1, 1, 1);
        }
        fig.axes_mut(0).expect("axes[0] exists after add_subplot").set_xlabel(s);
    })
}

/// Sets the y-axis label.
pub fn ylabel(s: &str) {
    CURRENT_FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        if fig.num_axes() == 0 {
            fig.add_subplot(1, 1, 1);
        }
        fig.axes_mut(0).expect("axes[0] exists after add_subplot").set_ylabel(s);
    })
}

/// Shows the legend on the current axes.
pub fn legend() {
    CURRENT_FIGURE.with(|fig| {
        let mut fig = fig.borrow_mut();
        if fig.num_axes() == 0 {
            fig.add_subplot(1, 1, 1);
        }
        fig.axes_mut(0).expect("axes[0] exists after add_subplot").legend();
    })
}

/// Saves the current figure to a file. Format is determined by extension (.png, .svg).
pub fn savefig(path: impl AsRef<Path>) -> Result<()> {
    CURRENT_FIGURE.with(|fig| {
        let fig = fig.borrow();
        save_figure(&fig, path.as_ref())
    })
}

/// Replaces the current figure with an `nrows × ncols` subplot grid.
pub fn subplots(nrows: usize, ncols: usize) {
    CURRENT_FIGURE.with(|fig| {
        *fig.borrow_mut() = Figure::subplots(nrows, ncols);
    })
}

/// Clears the current figure.
pub fn clf() {
    CURRENT_FIGURE.with(|fig| {
        *fig.borrow_mut() = Figure::new();
    })
}

/// Saves a figure to a file, selecting the renderer by file extension.
pub fn save_figure(fig: &Figure, path: &Path) -> Result<()> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let bytes = match ext.as_str() {
        "png" => {
            let renderer = plotkit_render_skia::SkiaRenderer::new(fig.width(), fig.height());
            fig.render_to(renderer)
        }
        #[cfg(feature = "svg")]
        "svg" => {
            let renderer = plotkit_render_svg::SvgRenderer::new(fig.width(), fig.height());
            fig.render_to(renderer)
        }
        other => return Err(PlotError::UnsupportedFormat(other.to_string())),
    };

    std::fs::write(path, bytes)?;
    Ok(())
}

/// Extension trait adding save methods to Figure.
pub trait FigureExt {
    /// Saves the figure to a file. Format determined by extension.
    fn save(&self, path: impl AsRef<Path>) -> Result<()>;
    /// Renders to PNG bytes.
    fn to_png_bytes(&self) -> Result<Vec<u8>>;
    /// Renders to SVG string.
    fn to_svg_string(&self) -> Result<String>;
}

impl FigureExt for Figure {
    fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        save_figure(self, path.as_ref())
    }

    fn to_png_bytes(&self) -> Result<Vec<u8>> {
        let renderer = plotkit_render_skia::SkiaRenderer::new(self.width(), self.height());
        Ok(self.render_to(renderer))
    }

    fn to_svg_string(&self) -> Result<String> {
        #[cfg(feature = "svg")]
        {
            let renderer = plotkit_render_svg::SvgRenderer::new(self.width(), self.height());
            let bytes = self.render_to(renderer);
            String::from_utf8(bytes).map_err(|_| PlotError::UnsupportedFormat("svg encoding error".into()))
        }
        #[cfg(not(feature = "svg"))]
        Err(PlotError::UnsupportedFormat("svg feature not enabled".into()))
    }
}
