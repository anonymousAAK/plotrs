//! Core types and rendering logic for the plotkit plotting library.
//!
//! This crate provides the fundamental types (`Figure`, `Axes`, `Artist`),
//! the `Renderer` trait, and all chart rendering logic. It is backend-agnostic —
//! concrete renderers live in separate crates.
//!
//! Most users should use the `plotkit` umbrella crate instead of depending on
//! this crate directly.

#![deny(missing_docs)]

pub mod annotations;
pub mod artist;
pub mod axes;
pub mod charts;
pub mod colorbar;
pub mod colormap;
pub mod decimate;
pub mod error;
pub mod figure;
pub mod layout;
pub mod legend;
pub mod primitives;
pub mod renderer;
pub mod scale;
pub mod series;
pub mod text;
pub mod theme;
pub mod ticks;

/// The plotkit-core prelude — import common types with a single `use` statement.
pub mod prelude {
    pub use crate::annotations::{Annotation, ArrowStyle, TextAnnotation};
    pub use crate::axes::{Axes, TwinSide};
    pub use crate::colorbar::{Colorbar, ColorbarOrientation};
    pub use crate::colormap::Colormap;
    pub use crate::decimate::{DecimateMethod, DecimateMode};
    pub use crate::error::{PlotError, Result};
    pub use crate::figure::Figure;
    pub use crate::primitives::Color;
    pub use crate::primitives::{HAlign, VAlign};
    pub use crate::scale::Scale;
    pub use crate::series::{IntoCategories, IntoSeries};
    pub use crate::theme::{GridAxis, LineStyle, Loc, Marker, Theme};
}
