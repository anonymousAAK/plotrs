//! Chart builder methods for configuring visual properties.
//!
//! Each chart type (line, scatter, bar, histogram, fill_between, step, stem,
//! boxplot, errorbar, heatmap, pie, violin, contour, polar, hexbin, waterfall)
//! has builder methods implemented directly on its artist type.

pub mod bar;
pub mod boxplot;
pub mod contour;
pub mod errorbar;
pub mod fill_between;
pub mod heatmap;
pub mod hexbin;
pub mod histogram;
pub mod line;
pub mod pie;
pub mod polar;
pub mod scatter;
pub mod stem;
pub mod step;
pub mod violin;
pub mod waterfall;
