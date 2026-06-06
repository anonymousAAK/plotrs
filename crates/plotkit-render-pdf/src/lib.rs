//! PDF rendering backend for plotkit.
//!
//! Produces PDF output by translating plotkit primitives into PDF drawing
//! operations using the `printpdf` crate. Text is rendered with built-in
//! PDF fonts (Helvetica family).

#![deny(missing_docs)]

use plotkit_core::primitives::{
    Affine, Color, DashPattern, FontWeight, HAlign, Image, Paint, Path, PathEl, Point, Rect,
    Stroke, StrokeCap, StrokeJoin, TextStyle, VAlign,
};
use plotkit_core::renderer::Renderer;

use printpdf::path::{PaintMode, WindingOrder};
use printpdf::{
    BuiltinFont, IndirectFontRef, LineCapStyle, LineDashPattern, LineJoinStyle, Mm, PdfDocument,
    PdfDocumentReference, PdfLayerIndex, PdfLayerReference, PdfPageIndex, Polygon, Rgb,
};

/// Pixels-to-millimeters conversion factor at 72 DPI.
///
/// 1 inch = 25.4 mm, 1 inch = 72 pixels => 1 pixel = 25.4 / 72 mm.
const PX_TO_MM: f64 = 25.4 / 72.0;

/// Converts a pixel value to printpdf `Mm`.
#[inline]
fn px_to_mm(px: f64) -> Mm {
    Mm((px * PX_TO_MM) as f32)
}

/// A renderer that produces PDF output.
///
/// Uses printpdf's `PdfDocument` / `PdfPage` / `PdfLayer` model.
/// All plotkit coordinates (origin top-left, y-down) are converted to PDF
/// coordinates (origin bottom-left, y-up) during rendering.
pub struct PdfRenderer {
    width: u32,
    height: u32,
    doc: PdfDocumentReference,
    page_idx: PdfPageIndex,
    layer_idx: PdfLayerIndex,
    /// Stack depth for save/restore graphics state (clip simulation).
    clip_depth: usize,
}

impl PdfRenderer {
    /// Creates a new PDF renderer with the given dimensions in pixels.
    ///
    /// A single PDF page is created with dimensions matching the pixel size
    /// at 72 DPI.
    pub fn new(width: u32, height: u32) -> Self {
        let w_mm = px_to_mm(width as f64);
        let h_mm = px_to_mm(height as f64);

        let (doc, page_idx, layer_idx) = PdfDocument::new("plotkit", w_mm, h_mm, "Layer 1");

        Self {
            width,
            height,
            doc,
            page_idx,
            layer_idx,
            clip_depth: 0,
        }
    }

    /// Returns a reference to the current PDF layer for drawing operations.
    fn current_layer(&self) -> PdfLayerReference {
        let page = self.doc.get_page(self.page_idx);
        page.get_layer(self.layer_idx)
    }

    /// Converts a plotkit y-coordinate (top-left origin, y-down) to PDF
    /// y-coordinate (bottom-left origin, y-up).
    #[inline]
    fn flip_y(&self, y: f64) -> f64 {
        self.height as f64 - y
    }

    /// Applies the given affine transform to a point and flips y for PDF.
    #[inline]
    fn transform_point(&self, p: Point, transform: Affine) -> (Mm, Mm) {
        let coeffs = transform.as_coeffs();
        let tx = coeffs[0] * p.x + coeffs[2] * p.y + coeffs[4];
        let ty = coeffs[1] * p.x + coeffs[3] * p.y + coeffs[5];
        (px_to_mm(tx), px_to_mm(self.flip_y(ty)))
    }

    /// Converts a plotkit `Path` into a vector of printpdf polygon rings,
    /// applying the transform and y-flip.
    ///
    /// Each sub-path becomes a separate ring (a `Vec<(printpdf::Point, bool)>`).
    /// The `bool` flag marks bezier control points according to printpdf's
    /// convention: two consecutive `true` flags on adjacent points signal
    /// that the next three points form a cubic bezier curve.
    fn convert_path_to_rings(
        &self,
        path: &Path,
        transform: Affine,
    ) -> Vec<Vec<(printpdf::Point, bool)>> {
        let mut rings: Vec<Vec<(printpdf::Point, bool)>> = Vec::new();
        let mut current_ring: Vec<(printpdf::Point, bool)> = Vec::new();

        for el in &path.elements {
            match *el {
                PathEl::MoveTo(p) => {
                    if !current_ring.is_empty() {
                        rings.push(std::mem::take(&mut current_ring));
                    }
                    let (mx, my) = self.transform_point(p, transform);
                    current_ring.push((printpdf::Point::new(mx, my), false));
                }
                PathEl::LineTo(p) => {
                    let (lx, ly) = self.transform_point(p, transform);
                    current_ring.push((printpdf::Point::new(lx, ly), false));
                }
                PathEl::QuadTo(ctrl, end) => {
                    // Elevate quadratic to cubic bezier.
                    if let Some(last) = current_ring.last_mut() {
                        let p0x = last.0.x.0;
                        let p0y = last.0.y.0;
                        // Mark previous point to start bezier sequence.
                        last.1 = true;

                        let (cx_mm, cy_mm) = self.transform_point(ctrl, transform);
                        let (ex_mm, ey_mm) = self.transform_point(end, transform);

                        // Cubic control points from quadratic:
                        // CP1 = P0 + 2/3 * (Q - P0)
                        // CP2 = P  + 2/3 * (Q - P)
                        let cp1x = p0x + 2.0 / 3.0 * (cx_mm.0 - p0x);
                        let cp1y = p0y + 2.0 / 3.0 * (cy_mm.0 - p0y);
                        let cp2x = ex_mm.0 + 2.0 / 3.0 * (cx_mm.0 - ex_mm.0);
                        let cp2y = ey_mm.0 + 2.0 / 3.0 * (cy_mm.0 - ey_mm.0);

                        current_ring
                            .push((printpdf::Point::new(Mm(cp1x), Mm(cp1y)), true));
                        current_ring
                            .push((printpdf::Point::new(Mm(cp2x), Mm(cp2y)), false));
                        current_ring.push((printpdf::Point::new(ex_mm, ey_mm), false));
                    }
                }
                PathEl::CurveTo(c1, c2, end) => {
                    // Mark previous point to start bezier sequence.
                    if let Some(last) = current_ring.last_mut() {
                        last.1 = true;
                    }
                    let (c1x, c1y) = self.transform_point(c1, transform);
                    let (c2x, c2y) = self.transform_point(c2, transform);
                    let (ex, ey) = self.transform_point(end, transform);

                    current_ring.push((printpdf::Point::new(c1x, c1y), true));
                    current_ring.push((printpdf::Point::new(c2x, c2y), false));
                    current_ring.push((printpdf::Point::new(ex, ey), false));
                }
                PathEl::ClosePath => {
                    // Close the sub-path by pushing the ring.
                    if !current_ring.is_empty() {
                        rings.push(std::mem::take(&mut current_ring));
                    }
                }
            }
        }

        if !current_ring.is_empty() {
            rings.push(current_ring);
        }

        rings
    }

    /// Builds a printpdf `Color` from a plotkit `Color` (RGB only).
    ///
    /// Alpha is not directly supported by PDF color operators; semi-transparent
    /// drawing would require an extended graphics state, which is not yet
    /// exposed by printpdf's public API for arbitrary alpha values.
    fn convert_color(c: &Color) -> printpdf::Color {
        printpdf::Color::Rgb(Rgb::new(
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            None,
        ))
    }

    /// Returns the built-in PDF font matching the requested style.
    fn builtin_font(&self, style: &TextStyle) -> IndirectFontRef {
        let font_name = match style.weight {
            FontWeight::Bold => BuiltinFont::HelveticaBold,
            FontWeight::Normal => BuiltinFont::Helvetica,
        };
        self.doc
            .add_builtin_font(font_name)
            .expect("built-in font")
    }

    /// Converts a plotkit `DashPattern` to a printpdf `LineDashPattern`.
    fn convert_dash(dp: &DashPattern) -> LineDashPattern {
        let dashes_mm: Vec<i64> = dp
            .dashes
            .iter()
            .map(|&d| (d * PX_TO_MM * 1000.0) as i64)
            .collect();
        let offset = (dp.offset * PX_TO_MM * 1000.0) as i64;
        match dashes_mm.len() {
            0 => LineDashPattern::default(),
            1 => LineDashPattern {
                dash_1: Some(dashes_mm[0]),
                gap_1: Some(dashes_mm[0]),
                offset,
                ..Default::default()
            },
            2 => LineDashPattern {
                dash_1: Some(dashes_mm[0]),
                gap_1: Some(dashes_mm[1]),
                offset,
                ..Default::default()
            },
            3 => LineDashPattern {
                dash_1: Some(dashes_mm[0]),
                gap_1: Some(dashes_mm[1]),
                dash_2: Some(dashes_mm[2]),
                offset,
                ..Default::default()
            },
            _ => LineDashPattern {
                dash_1: Some(dashes_mm[0]),
                gap_1: Some(dashes_mm[1]),
                dash_2: Some(dashes_mm[2]),
                gap_2: Some(dashes_mm[3]),
                offset,
                ..Default::default()
            },
        }
    }
}

impl Renderer for PdfRenderer {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn fill_path(&mut self, path: &Path, paint: &Paint, transform: Affine) {
        let rings = self.convert_path_to_rings(path, transform);
        if rings.is_empty() {
            return;
        }

        let layer = self.current_layer();
        let fill_color = Self::convert_color(&paint.color);
        layer.set_fill_color(fill_color);

        let poly = Polygon {
            rings,
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        };
        layer.add_polygon(poly);
    }

    fn stroke_path(
        &mut self,
        path: &Path,
        paint: &Paint,
        stroke: &Stroke,
        transform: Affine,
    ) {
        let rings = self.convert_path_to_rings(path, transform);
        if rings.is_empty() {
            return;
        }

        let layer = self.current_layer();
        let stroke_color = Self::convert_color(&paint.color);
        let width_mm = (stroke.width * PX_TO_MM) as f32;

        let line_cap = match stroke.cap {
            StrokeCap::Butt => LineCapStyle::Butt,
            StrokeCap::Round => LineCapStyle::Round,
            StrokeCap::Square => LineCapStyle::ProjectingSquare,
        };

        let line_join = match stroke.join {
            StrokeJoin::Miter => LineJoinStyle::Miter,
            StrokeJoin::Round => LineJoinStyle::Round,
            StrokeJoin::Bevel => LineJoinStyle::Limit,
        };

        let dash_pattern = match stroke.dash {
            Some(ref dp) => Self::convert_dash(dp),
            None => LineDashPattern::default(),
        };

        layer.set_outline_color(stroke_color);
        layer.set_outline_thickness(width_mm);
        layer.set_line_cap_style(line_cap);
        layer.set_line_join_style(line_join);
        layer.set_line_dash_pattern(dash_pattern);

        let poly = Polygon {
            rings,
            mode: PaintMode::Stroke,
            winding_order: WindingOrder::NonZero,
        };
        layer.add_polygon(poly);
    }

    fn draw_text(&mut self, text: &str, pos: Point, style: &TextStyle, transform: Affine) {
        if text.is_empty() {
            return;
        }

        let font = self.builtin_font(style);
        let font_size_pt = style.size;

        // Measure text for alignment.
        let (text_w, text_h) = self.measure_text(text, style);

        // Compute adjusted position based on alignment.
        let adjusted_x = match style.halign {
            HAlign::Left => pos.x,
            HAlign::Center => pos.x - text_w / 2.0,
            HAlign::Right => pos.x - text_w,
        };

        // PDF text is positioned at the baseline.
        // Approximate ascent as ~0.75 * font_size, descent as ~0.25 * font_size.
        let ascent = style.size * 0.75;
        let adjusted_y = match style.valign {
            VAlign::Top => pos.y + ascent,
            VAlign::Middle => pos.y + ascent - text_h / 2.0,
            VAlign::Baseline => pos.y,
            VAlign::Bottom => pos.y - (text_h - ascent),
        };

        // Apply transform.
        let coeffs = transform.as_coeffs();
        let tx = coeffs[0] * adjusted_x + coeffs[2] * adjusted_y + coeffs[4];
        let ty = coeffs[1] * adjusted_x + coeffs[3] * adjusted_y + coeffs[5];

        let pdf_x = px_to_mm(tx);
        let pdf_y = px_to_mm(self.flip_y(ty));

        let layer = self.current_layer();
        let text_color = Self::convert_color(&style.color);
        layer.set_fill_color(text_color);
        layer.use_text(text, font_size_pt as f32, pdf_x, pdf_y, &font);
    }

    fn draw_image(&mut self, _img: &Image, _dst: Rect, _transform: Affine) {
        // Image embedding in PDF is not yet implemented.
    }

    fn push_clip(&mut self, path: &Path, transform: Affine) {
        let layer = self.current_layer();
        layer.save_graphics_state();
        self.clip_depth += 1;

        // Build and apply a clipping polygon.
        let rings = self.convert_path_to_rings(path, transform);
        if !rings.is_empty() {
            let poly = Polygon {
                rings,
                mode: PaintMode::Clip,
                winding_order: WindingOrder::NonZero,
            };
            layer.add_polygon(poly);
        }
    }

    fn pop_clip(&mut self) {
        if self.clip_depth > 0 {
            let layer = self.current_layer();
            layer.restore_graphics_state();
            self.clip_depth -= 1;
        }
    }

    fn measure_text(&self, text: &str, style: &TextStyle) -> (f64, f64) {
        if text.is_empty() {
            return (0.0, 0.0);
        }
        // Approximate measurement: average character width is roughly 0.6 * font size
        // for Helvetica. This matches the SVG renderer approach.
        let width = text.len() as f64 * style.size * 0.6;
        let height = style.size;
        (width, height)
    }

    fn finalize(self) -> Vec<u8> {
        self.doc
            .save_to_bytes()
            .expect("failed to save PDF to bytes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_renderer() {
        let r = PdfRenderer::new(800, 600);
        assert_eq!(r.size(), (800, 600));
    }

    #[test]
    fn finalize_produces_pdf() {
        let r = PdfRenderer::new(100, 100);
        let bytes = r.finalize();
        // PDF magic bytes: %PDF
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[..5], b"%PDF-");
    }

    #[test]
    fn fill_rect_does_not_panic() {
        let mut r = PdfRenderer::new(200, 200);
        let path = Path::rect(Rect::new(10.0, 10.0, 50.0, 50.0));
        let paint = Paint::new(Color::TAB_BLUE);
        r.fill_path(&path, &paint, Affine::IDENTITY);
        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn stroke_rect_does_not_panic() {
        let mut r = PdfRenderer::new(200, 200);
        let path = Path::rect(Rect::new(10.0, 10.0, 50.0, 50.0));
        let paint = Paint::new(Color::TAB_RED);
        let stroke = Stroke::new(2.0);
        r.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn draw_text_does_not_panic() {
        let mut r = PdfRenderer::new(200, 200);
        let style = TextStyle::new(14.0);
        r.draw_text(
            "Hello PDF",
            Point::new(10.0, 50.0),
            &style,
            Affine::IDENTITY,
        );
        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn measure_text_returns_nonzero() {
        let r = PdfRenderer::new(100, 100);
        let style = TextStyle::new(14.0);
        let (w, h) = r.measure_text("hello", &style);
        assert!(w > 0.0, "text width should be positive, got {w}");
        assert!(h > 0.0, "text height should be positive, got {h}");
    }

    #[test]
    fn measure_text_empty() {
        let r = PdfRenderer::new(100, 100);
        let style = TextStyle::new(14.0);
        let (w, h) = r.measure_text("", &style);
        assert!((w - 0.0).abs() < f64::EPSILON);
        assert!((h - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clip_push_pop_does_not_panic() {
        let mut r = PdfRenderer::new(200, 200);
        let clip = Path::rect(Rect::new(0.0, 0.0, 100.0, 100.0));
        r.push_clip(&clip, Affine::IDENTITY);
        let path = Path::rect(Rect::new(10.0, 10.0, 50.0, 50.0));
        let paint = Paint::new(Color::TAB_GREEN);
        r.fill_path(&path, &paint, Affine::IDENTITY);
        r.pop_clip();
        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn circle_path_does_not_panic() {
        let mut r = PdfRenderer::new(200, 200);
        let path = Path::circle(Point::new(100.0, 100.0), 40.0);
        let paint = Paint::new(Color::TAB_ORANGE);
        r.fill_path(&path, &paint, Affine::IDENTITY);
        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn stroke_with_dash_does_not_panic() {
        let mut r = PdfRenderer::new(200, 200);
        let path = Path::rect(Rect::new(10.0, 10.0, 100.0, 100.0));
        let paint = Paint::new(Color::BLACK);
        let stroke = Stroke::new(1.5).with_dash(DashPattern {
            dashes: vec![5.0, 3.0],
            offset: 0.0,
        });
        r.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn px_to_mm_conversion() {
        // 72 pixels = 1 inch = 25.4 mm
        let mm = px_to_mm(72.0);
        assert!(
            (mm.0 - 25.4).abs() < 0.01,
            "72px should be 25.4mm, got {}",
            mm.0
        );
    }

    #[test]
    fn multiple_fills_produce_valid_pdf() {
        let mut r = PdfRenderer::new(400, 400);
        // Fill a white background.
        let bg = Path::rect(Rect::new(0.0, 0.0, 400.0, 400.0));
        r.fill_path(&bg, &Paint::new(Color::WHITE), Affine::IDENTITY);
        // Fill a colored rectangle.
        let rect = Path::rect(Rect::new(50.0, 50.0, 100.0, 100.0));
        r.fill_path(&rect, &Paint::new(Color::TAB_BLUE), Affine::IDENTITY);
        // Stroke a line.
        let mut line = Path::new();
        line.move_to(10.0, 10.0);
        line.line_to(390.0, 390.0);
        r.stroke_path(&line, &Paint::new(Color::TAB_RED), &Stroke::new(2.0), Affine::IDENTITY);
        let bytes = r.finalize();
        assert_eq!(&bytes[..5], b"%PDF-");
    }

    #[test]
    fn text_alignment_does_not_panic() {
        let mut r = PdfRenderer::new(300, 300);
        let mut style = TextStyle::new(16.0);

        style.halign = HAlign::Left;
        style.valign = VAlign::Top;
        r.draw_text("Top-Left", Point::new(150.0, 50.0), &style, Affine::IDENTITY);

        style.halign = HAlign::Center;
        style.valign = VAlign::Middle;
        r.draw_text("Center", Point::new(150.0, 150.0), &style, Affine::IDENTITY);

        style.halign = HAlign::Right;
        style.valign = VAlign::Bottom;
        r.draw_text("Bottom-Right", Point::new(150.0, 250.0), &style, Affine::IDENTITY);

        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }
}
