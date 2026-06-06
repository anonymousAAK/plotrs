//! Text annotations and arrow annotations for axes.
//!
//! This module defines the types used by [`Axes::text`] and [`Axes::annotate`]
//! to place text labels and annotated callouts on a plot. Both types support
//! builder-style configuration for font size, color, alignment, and (for
//! annotations) arrow styling.

use crate::primitives::Color;

// ---------------------------------------------------------------------------
// ArrowStyle
// ---------------------------------------------------------------------------

/// Arrow style for annotations connecting text to a data point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowStyle {
    /// No arrow is drawn; only the text is shown.
    None,
    /// A simple line arrow with a small triangular head.
    Simple,
    /// A wider, more prominent arrowhead.
    Fancy,
}

// ---------------------------------------------------------------------------
// HAlign / VAlign (re-exports for annotation API convenience)
// ---------------------------------------------------------------------------

// We re-use HAlign and VAlign from primitives. The annotation structs store
// them directly so that the builder methods can set them.

use crate::primitives::{HAlign, VAlign};

// ---------------------------------------------------------------------------
// TextAnnotation
// ---------------------------------------------------------------------------

/// A text label placed at a data-space coordinate.
///
/// Created by [`Axes::text`]. Supports builder-style chaining to customise
/// font size, color, alignment, and rotation.
#[derive(Debug, Clone)]
pub struct TextAnnotation {
    /// The text string to render.
    pub text: String,
    /// X position in data coordinates.
    pub x: f64,
    /// Y position in data coordinates.
    pub y: f64,
    /// Optional font size override (in points). `None` uses the theme default.
    pub fontsize: Option<f64>,
    /// Optional text color override. `None` uses the theme text color.
    pub color: Option<Color>,
    /// Horizontal alignment of the text relative to `(x, y)`.
    pub ha: HAlign,
    /// Vertical alignment of the text relative to `(x, y)`.
    pub va: VAlign,
    /// Rotation angle in degrees (counter-clockwise).
    pub rotation: f64,
}

impl TextAnnotation {
    /// Sets the font size (in points).
    pub fn fontsize(&mut self, size: f64) -> &mut Self {
        self.fontsize = Some(size);
        self
    }

    /// Sets the text color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Sets the horizontal alignment.
    pub fn ha(&mut self, ha: HAlign) -> &mut Self {
        self.ha = ha;
        self
    }

    /// Sets the vertical alignment.
    pub fn va(&mut self, va: VAlign) -> &mut Self {
        self.va = va;
        self
    }

    /// Sets the rotation angle in degrees (counter-clockwise).
    pub fn rotation(&mut self, degrees: f64) -> &mut Self {
        self.rotation = degrees;
        self
    }
}

// ---------------------------------------------------------------------------
// Annotation
// ---------------------------------------------------------------------------

/// An annotation with optional arrow from a text position to a data point.
///
/// Created by [`Axes::annotate`]. The text is drawn at `xytext` and, when an
/// arrow style other than [`ArrowStyle::None`] is set, an arrow is drawn from
/// `xytext` to `xy`.
#[derive(Debug, Clone)]
pub struct Annotation {
    /// The annotation text string.
    pub text: String,
    /// The data-space point being annotated.
    pub xy: (f64, f64),
    /// The data-space position where the text is placed.
    pub xytext: (f64, f64),
    /// Optional font size override (in points). `None` uses the theme default.
    pub fontsize: Option<f64>,
    /// Optional text color override. `None` uses the theme text color.
    pub color: Option<Color>,
    /// Horizontal alignment of the text relative to `xytext`.
    pub ha: HAlign,
    /// Vertical alignment of the text relative to `xytext`.
    pub va: VAlign,
    /// The style of arrow drawn from `xytext` to `xy`.
    pub arrowstyle: ArrowStyle,
    /// Optional arrow color override. `None` uses the text color.
    pub arrow_color: Option<Color>,
}

impl Annotation {
    /// Sets the font size (in points).
    pub fn fontsize(&mut self, size: f64) -> &mut Self {
        self.fontsize = Some(size);
        self
    }

    /// Sets the text color.
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Sets the horizontal alignment.
    pub fn ha(&mut self, ha: HAlign) -> &mut Self {
        self.ha = ha;
        self
    }

    /// Sets the vertical alignment.
    pub fn va(&mut self, va: VAlign) -> &mut Self {
        self.va = va;
        self
    }

    /// Sets the arrow style.
    pub fn arrowstyle(&mut self, style: ArrowStyle) -> &mut Self {
        self.arrowstyle = style;
        self
    }

    /// Sets the arrow color.
    pub fn arrow_color(&mut self, color: Color) -> &mut Self {
        self.arrow_color = Some(color);
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_annotation_defaults() {
        let mut t = TextAnnotation {
            text: "hello".to_string(),
            x: 1.0,
            y: 2.0,
            fontsize: None,
            color: None,
            ha: HAlign::Left,
            va: VAlign::Baseline,
            rotation: 0.0,
        };
        assert_eq!(t.text, "hello");
        assert_eq!(t.x, 1.0);
        assert_eq!(t.y, 2.0);
        assert!(t.fontsize.is_none());
        assert!(t.color.is_none());
        assert_eq!(t.ha, HAlign::Left);
        assert_eq!(t.va, VAlign::Baseline);
        assert!((t.rotation - 0.0).abs() < f64::EPSILON);

        // Builder chaining.
        t.fontsize(14.0).color(Color::TAB_RED).ha(HAlign::Center).va(VAlign::Top).rotation(45.0);
        assert_eq!(t.fontsize, Some(14.0));
        assert_eq!(t.color, Some(Color::TAB_RED));
        assert_eq!(t.ha, HAlign::Center);
        assert_eq!(t.va, VAlign::Top);
        assert!((t.rotation - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn annotation_defaults() {
        let mut a = Annotation {
            text: "peak".to_string(),
            xy: (1.0, 2.0),
            xytext: (3.0, 4.0),
            fontsize: None,
            color: None,
            ha: HAlign::Center,
            va: VAlign::Bottom,
            arrowstyle: ArrowStyle::None,
            arrow_color: None,
        };
        assert_eq!(a.text, "peak");
        assert_eq!(a.xy, (1.0, 2.0));
        assert_eq!(a.xytext, (3.0, 4.0));
        assert_eq!(a.arrowstyle, ArrowStyle::None);

        // Builder chaining.
        a.arrowstyle(ArrowStyle::Simple).arrow_color(Color::TAB_BLUE).fontsize(12.0);
        assert_eq!(a.arrowstyle, ArrowStyle::Simple);
        assert_eq!(a.arrow_color, Some(Color::TAB_BLUE));
        assert_eq!(a.fontsize, Some(12.0));
    }

    #[test]
    fn arrow_style_equality() {
        assert_eq!(ArrowStyle::None, ArrowStyle::None);
        assert_eq!(ArrowStyle::Simple, ArrowStyle::Simple);
        assert_eq!(ArrowStyle::Fancy, ArrowStyle::Fancy);
        assert_ne!(ArrowStyle::None, ArrowStyle::Simple);
        assert_ne!(ArrowStyle::Simple, ArrowStyle::Fancy);
    }
}
