//! Core primitive types for plotkit rendering.
//!
//! This module defines the fundamental drawing primitives used throughout the
//! plotkit rendering pipeline: geometric shapes, colors, strokes, text styles,
//! paths, and images. These types form the interface between chart logic and
//! backend renderers — no backend-specific types appear here.

pub use kurbo::Affine;

// ---------------------------------------------------------------------------
// Point
// ---------------------------------------------------------------------------

/// A 2D point in device-independent coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// The x (horizontal) coordinate.
    pub x: f64,
    /// The y (vertical) coordinate.
    pub y: f64,
}

impl Point {
    /// Creates a new point at `(x, y)`.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

// ---------------------------------------------------------------------------
// Rect
// ---------------------------------------------------------------------------

/// An axis-aligned rectangle defined by its top-left corner and dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// The x coordinate of the left edge.
    pub x: f64,
    /// The y coordinate of the top edge.
    pub y: f64,
    /// The width of the rectangle (must be non-negative for well-formed rects).
    pub width: f64,
    /// The height of the rectangle (must be non-negative for well-formed rects).
    pub height: f64,
}

impl Rect {
    /// Creates a new rectangle from a top-left corner and dimensions.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    /// Creates the smallest axis-aligned rectangle that contains both points.
    ///
    /// The two points may be any pair of opposite corners; the result is always
    /// a rectangle with non-negative width and height.
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p1.x - p2.x).abs();
        let height = (p1.y - p2.y).abs();
        Self { x, y, width, height }
    }

    /// Returns `true` if `p` lies inside or on the boundary of this rectangle.
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x
            && p.x <= self.x + self.width
            && p.y >= self.y
            && p.y <= self.y + self.height
    }

    /// Returns the center point of the rectangle.
    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Returns the x coordinate of the right edge (`x + width`).
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// Returns the y coordinate of the bottom edge (`y + height`).
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }
}

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

/// An RGBA color with 8 bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red channel (0–255).
    pub r: u8,
    /// Green channel (0–255).
    pub g: u8,
    /// Blue channel (0–255).
    pub b: u8,
    /// Alpha channel (0 = fully transparent, 255 = fully opaque).
    pub a: u8,
}

impl Color {
    /// Creates a new color from individual RGBA components.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a fully opaque color from RGB components (alpha = 255).
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Returns a copy of this color with the alpha channel set to `a`.
    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Parses a hex color string such as `"#4E79A7"` or `"4E79A7"`.
    ///
    /// Returns `None` if the string is not a valid 6-digit hex color.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self::rgb(r, g, b))
    }

    // -- Tableau-10 categorical palette (exact hex values) ------------------

    /// Tableau-10 blue (`#4E79A7`).
    pub const TAB_BLUE: Self = Self::rgb(0x4E, 0x79, 0xA7);
    /// Tableau-10 orange (`#F28E2B`).
    pub const TAB_ORANGE: Self = Self::rgb(0xF2, 0x8E, 0x2B);
    /// Tableau-10 green (`#59A14F`).
    pub const TAB_GREEN: Self = Self::rgb(0x59, 0xA1, 0x4F);
    /// Tableau-10 red (`#E15759`).
    pub const TAB_RED: Self = Self::rgb(0xE1, 0x57, 0x59);
    /// Tableau-10 purple (`#B07AA1`).
    pub const TAB_PURPLE: Self = Self::rgb(0xB0, 0x7A, 0xA1);
    /// Tableau-10 brown (`#9C755F`).
    pub const TAB_BROWN: Self = Self::rgb(0x9C, 0x75, 0x5F);
    /// Tableau-10 pink (`#FF9DA7`).
    pub const TAB_PINK: Self = Self::rgb(0xFF, 0x9D, 0xA7);
    /// Tableau-10 grey (`#BAB0AC`).
    pub const TAB_GREY: Self = Self::rgb(0xBA, 0xB0, 0xAC);
    /// Tableau-10 olive / yellow (`#EDC948`).
    pub const TAB_OLIVE: Self = Self::rgb(0xED, 0xC9, 0x48);
    /// Tableau-10 cyan / teal (`#76B7B2`).
    pub const TAB_CYAN: Self = Self::rgb(0x76, 0xB7, 0xB2);

    /// Pure white (`#FFFFFF`).
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    /// Pure black (`#000000`).
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    /// The complete Tableau-10 categorical palette, in canonical order.
    pub const TABLEAU_10: [Self; 10] = [
        Self::TAB_BLUE,
        Self::TAB_ORANGE,
        Self::TAB_GREEN,
        Self::TAB_RED,
        Self::TAB_PURPLE,
        Self::TAB_BROWN,
        Self::TAB_PINK,
        Self::TAB_GREY,
        Self::TAB_OLIVE,
        Self::TAB_CYAN,
    ];
}

// ---------------------------------------------------------------------------
// Paint
// ---------------------------------------------------------------------------

/// Describes how a filled region should be painted.
#[derive(Debug, Clone)]
pub struct Paint {
    /// The fill color.
    pub color: Color,
    /// Whether anti-aliasing is enabled for this fill.
    pub anti_alias: bool,
}

impl Paint {
    /// Creates a new paint with the given color and anti-aliasing enabled.
    pub fn new(color: Color) -> Self {
        Self {
            color,
            anti_alias: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Stroke
// ---------------------------------------------------------------------------

/// Describes the visual style of a stroked path.
#[derive(Debug, Clone)]
pub struct Stroke {
    /// The width of the stroke in device-independent units.
    pub width: f64,
    /// The shape used at the endpoints of open sub-paths.
    pub cap: StrokeCap,
    /// The shape used at corners where two path segments meet.
    pub join: StrokeJoin,
    /// An optional dash pattern; `None` means a solid stroke.
    pub dash: Option<DashPattern>,
}

/// The shape applied at the endpoints of an open sub-path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeCap {
    /// The stroke ends exactly at the endpoint with no extension.
    Butt,
    /// The stroke is extended by a half-circle at each endpoint.
    Round,
    /// The stroke is extended by a half-square at each endpoint.
    Square,
}

/// The shape applied at the join between two path segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeJoin {
    /// A sharp corner is drawn (subject to the miter limit).
    Miter,
    /// A circular arc is drawn at the join.
    Round,
    /// A flat diagonal is drawn across the join.
    Bevel,
}

/// A repeating dash pattern for stroked paths.
#[derive(Debug, Clone)]
pub struct DashPattern {
    /// Alternating lengths of painted and unpainted segments.
    pub dashes: Vec<f64>,
    /// Offset into the dash pattern at which the stroke begins.
    pub offset: f64,
}

impl Stroke {
    /// Creates a solid stroke with the given width.
    ///
    /// Defaults to [`StrokeCap::Butt`], [`StrokeJoin::Miter`], and no dash
    /// pattern.
    pub fn new(width: f64) -> Self {
        Self {
            width,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            dash: None,
        }
    }

    /// Sets the dash pattern on this stroke (builder-style).
    pub fn with_dash(mut self, pattern: DashPattern) -> Self {
        self.dash = Some(pattern);
        self
    }
}

// ---------------------------------------------------------------------------
// TextStyle
// ---------------------------------------------------------------------------

/// Controls how text is rendered: size, color, weight, font, and alignment.
#[derive(Debug, Clone)]
pub struct TextStyle {
    /// Font size in device-independent units (points).
    pub size: f64,
    /// The color used to render the glyphs.
    pub color: Color,
    /// Font weight (normal or bold).
    pub weight: FontWeight,
    /// Optional font family name (e.g. `"Helvetica"`). `None` uses the
    /// renderer's default.
    pub family: Option<String>,
    /// Horizontal alignment relative to the anchor point.
    pub halign: HAlign,
    /// Vertical alignment relative to the anchor point.
    pub valign: VAlign,
}

/// Font weight selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    /// Normal (regular) weight.
    Normal,
    /// Bold weight.
    Bold,
}

/// Horizontal text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    /// Align the left edge of the text to the anchor point.
    Left,
    /// Center the text horizontally on the anchor point.
    Center,
    /// Align the right edge of the text to the anchor point.
    Right,
}

/// Vertical text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    /// Align the top of the text bounding box to the anchor point.
    Top,
    /// Center the text vertically on the anchor point.
    Middle,
    /// Align the bottom of the text bounding box to the anchor point.
    Bottom,
    /// Align the text baseline to the anchor point.
    Baseline,
}

impl TextStyle {
    /// Creates a new text style with the given font size.
    ///
    /// Defaults: color [`Color::BLACK`], weight [`FontWeight::Normal`], no
    /// explicit font family, horizontal alignment [`HAlign::Left`], vertical
    /// alignment [`VAlign::Baseline`].
    pub fn new(size: f64) -> Self {
        Self {
            size,
            color: Color::BLACK,
            weight: FontWeight::Normal,
            family: None,
            halign: HAlign::Left,
            valign: VAlign::Baseline,
        }
    }
}

// ---------------------------------------------------------------------------
// Image
// ---------------------------------------------------------------------------

/// A raster image stored as raw RGBA pixel data.
#[derive(Debug, Clone)]
pub struct Image {
    /// Raw pixel data in RGBA order, row-major, with `4 * width * height`
    /// bytes.
    pub data: Vec<u8>,
    /// Width of the image in pixels.
    pub width: u32,
    /// Height of the image in pixels.
    pub height: u32,
}

// ---------------------------------------------------------------------------
// Path / PathEl
// ---------------------------------------------------------------------------

/// A vector path composed of a sequence of [`PathEl`] elements.
///
/// Paths are the primary geometric primitive passed to renderers. Use the
/// builder methods ([`move_to`](Path::move_to), [`line_to`](Path::line_to),
/// etc.) to construct paths incrementally, or the convenience constructors
/// [`Path::rect`] and [`Path::circle`] for common shapes.
#[derive(Debug, Clone, Default)]
pub struct Path {
    /// The ordered sequence of path elements.
    pub elements: Vec<PathEl>,
}

/// A single element within a [`Path`].
#[derive(Debug, Clone, Copy)]
pub enum PathEl {
    /// Begins a new sub-path at the given point.
    MoveTo(Point),
    /// Draws a straight line from the current point to the given point.
    LineTo(Point),
    /// Draws a quadratic Bezier curve with one control point and an endpoint.
    QuadTo(Point, Point),
    /// Draws a cubic Bezier curve with two control points and an endpoint.
    CurveTo(Point, Point, Point),
    /// Closes the current sub-path by drawing a straight line back to its
    /// starting point.
    ClosePath,
}

impl Path {
    /// Creates a new, empty path.
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Begins a new sub-path at `(x, y)`.
    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.elements.push(PathEl::MoveTo(Point::new(x, y)));
        self
    }

    /// Appends a straight line from the current point to `(x, y)`.
    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.elements.push(PathEl::LineTo(Point::new(x, y)));
        self
    }

    /// Appends a quadratic Bezier curve through control point `(x1, y1)` to
    /// endpoint `(x, y)`.
    pub fn quad_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) -> &mut Self {
        self.elements.push(PathEl::QuadTo(
            Point::new(x1, y1),
            Point::new(x, y),
        ));
        self
    }

    /// Appends a cubic Bezier curve through control points `(x1, y1)` and
    /// `(x2, y2)` to endpoint `(x, y)`.
    pub fn curve_to(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x: f64,
        y: f64,
    ) -> &mut Self {
        self.elements.push(PathEl::CurveTo(
            Point::new(x1, y1),
            Point::new(x2, y2),
            Point::new(x, y),
        ));
        self
    }

    /// Closes the current sub-path.
    pub fn close(&mut self) -> &mut Self {
        self.elements.push(PathEl::ClosePath);
        self
    }

    /// Creates a closed rectangular path from the given [`Rect`].
    pub fn rect(r: Rect) -> Self {
        let mut p = Self::new();
        p.move_to(r.x, r.y)
            .line_to(r.right(), r.y)
            .line_to(r.right(), r.bottom())
            .line_to(r.x, r.bottom())
            .close();
        p
    }

    /// Creates a closed circular path centered at `center` with the given
    /// `radius`, approximated by four cubic Bezier curves.
    ///
    /// The approximation uses the standard constant `kappa ≈ 0.5522847498`,
    /// which gives a maximum radial error of about 0.027%.
    pub fn circle(center: Point, radius: f64) -> Self {
        // Magic number for a 4-segment cubic Bezier circle approximation.
        const KAPPA: f64 = 0.552_284_749_8;
        let k = radius * KAPPA;
        let cx = center.x;
        let cy = center.y;

        let mut p = Self::new();
        // Start at the rightmost point and go counter-clockwise.
        p.move_to(cx + radius, cy);
        // Top-right quarter arc.
        p.curve_to(cx + radius, cy - k, cx + k, cy - radius, cx, cy - radius);
        // Top-left quarter arc.
        p.curve_to(cx - k, cy - radius, cx - radius, cy - k, cx - radius, cy);
        // Bottom-left quarter arc.
        p.curve_to(cx - radius, cy + k, cx - k, cy + radius, cx, cy + radius);
        // Bottom-right quarter arc.
        p.curve_to(cx + k, cy + radius, cx + radius, cy + k, cx + radius, cy);
        p.close();
        p
    }

    /// Returns `true` if the path contains no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_new() {
        let p = Point::new(1.0, 2.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
    }

    #[test]
    fn rect_basics() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.right(), 110.0);
        assert_eq!(r.bottom(), 70.0);
        assert_eq!(r.center(), Point::new(60.0, 45.0));
        assert!(r.contains(Point::new(60.0, 45.0)));
        assert!(!r.contains(Point::new(0.0, 0.0)));
    }

    #[test]
    fn rect_from_points() {
        let r = Rect::from_points(Point::new(10.0, 20.0), Point::new(5.0, 30.0));
        assert_eq!(r.x, 5.0);
        assert_eq!(r.y, 20.0);
        assert_eq!(r.width, 5.0);
        assert_eq!(r.height, 10.0);
    }

    #[test]
    fn color_hex_parsing() {
        assert_eq!(Color::from_hex("#4E79A7"), Some(Color::TAB_BLUE));
        assert_eq!(Color::from_hex("4E79A7"), Some(Color::TAB_BLUE));
        assert_eq!(Color::from_hex("invalid"), None);
        assert_eq!(Color::from_hex("#FFF"), None);
    }

    #[test]
    fn color_with_alpha() {
        let c = Color::TAB_BLUE.with_alpha(128);
        assert_eq!(c.r, 0x4E);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn tableau_10_length() {
        assert_eq!(Color::TABLEAU_10.len(), 10);
        assert_eq!(Color::TABLEAU_10[0], Color::TAB_BLUE);
        assert_eq!(Color::TABLEAU_10[9], Color::TAB_CYAN);
    }

    #[test]
    fn stroke_defaults() {
        let s = Stroke::new(2.0);
        assert_eq!(s.width, 2.0);
        assert_eq!(s.cap, StrokeCap::Butt);
        assert_eq!(s.join, StrokeJoin::Miter);
        assert!(s.dash.is_none());
    }

    #[test]
    fn stroke_with_dash() {
        let s = Stroke::new(1.0).with_dash(DashPattern {
            dashes: vec![5.0, 3.0],
            offset: 0.0,
        });
        assert!(s.dash.is_some());
        assert_eq!(s.dash.as_ref().unwrap().dashes, vec![5.0, 3.0]);
    }

    #[test]
    fn text_style_defaults() {
        let ts = TextStyle::new(12.0);
        assert_eq!(ts.size, 12.0);
        assert_eq!(ts.color, Color::BLACK);
        assert_eq!(ts.weight, FontWeight::Normal);
        assert!(ts.family.is_none());
        assert_eq!(ts.halign, HAlign::Left);
        assert_eq!(ts.valign, VAlign::Baseline);
    }

    #[test]
    fn path_rect() {
        let p = Path::rect(Rect::new(0.0, 0.0, 10.0, 10.0));
        // MoveTo + 3 LineTo + ClosePath = 5 elements
        assert_eq!(p.elements.len(), 5);
        assert!(!p.is_empty());
    }

    #[test]
    fn path_circle() {
        let p = Path::circle(Point::new(0.0, 0.0), 50.0);
        // MoveTo + 4 CurveTo + ClosePath = 6 elements
        assert_eq!(p.elements.len(), 6);
    }

    #[test]
    fn path_builder() {
        let mut p = Path::new();
        assert!(p.is_empty());
        p.move_to(0.0, 0.0)
            .line_to(10.0, 0.0)
            .quad_to(15.0, 5.0, 10.0, 10.0)
            .curve_to(5.0, 15.0, -5.0, 15.0, -10.0, 10.0)
            .close();
        assert_eq!(p.elements.len(), 5);
    }

    #[test]
    fn paint_defaults() {
        let p = Paint::new(Color::BLACK);
        assert!(p.anti_alias);
        assert_eq!(p.color, Color::BLACK);
    }
}
