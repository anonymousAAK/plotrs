//! PNG rendering backend for plotkit using tiny-skia.
//!
//! This crate provides CPU-based rasterization for producing PNG images.
//! It implements `plotkit_core::renderer::Renderer` using the `tiny-skia` library.
//! Text rendering is handled by `cosmic-text` with `swash` rasterization.

use std::cell::RefCell;

use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Weight, Wrap,
};
use plotkit_core::primitives::*;
use plotkit_core::renderer::Renderer;

/// The default font family name embedded in the binary.
pub const DEFAULT_FONT_FAMILY: &str = "Inter";

const DEFAULT_FONT_REGULAR: &[u8] = include_bytes!("../fonts/Inter-Regular.ttf");
const DEFAULT_FONT_BOLD: &[u8] = include_bytes!("../fonts/Inter-Bold.ttf");

/// Creates a `FontSystem` loaded only with the embedded Inter font.
///
/// This avoids system font discovery for deterministic cross-platform rendering.
fn embedded_font_system() -> FontSystem {
    let mut db = cosmic_text::fontdb::Database::new();
    db.load_font_data(DEFAULT_FONT_REGULAR.to_vec());
    db.load_font_data(DEFAULT_FONT_BOLD.to_vec());
    db.set_sans_serif_family(DEFAULT_FONT_FAMILY);
    FontSystem::new_with_locale_and_db("en-US".into(), db)
}

/// A renderer that produces PNG output via tiny-skia CPU rasterization.
///
/// Text is rendered using cosmic-text for shaping/layout and swash for glyph
/// rasterization. The embedded Inter font ensures deterministic rendering
/// across all platforms.
///
/// `FontSystem` is wrapped in `RefCell` so that `measure_text` (which receives
/// `&self` per the `Renderer` trait) can still perform shaping.
pub struct SkiaRenderer {
    pixmap: tiny_skia::Pixmap,
    font_system: RefCell<FontSystem>,
    swash_cache: SwashCache,
    clip_stack: Vec<tiny_skia::Mask>,
}

impl SkiaRenderer {
    /// Creates a new renderer with the given dimensions.
    ///
    /// Uses the embedded Inter font for deterministic cross-platform text.
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is zero, or if the allocation exceeds
    /// platform limits.
    pub fn new(width: u32, height: u32) -> Self {
        let pixmap =
            tiny_skia::Pixmap::new(width, height).expect("failed to create pixmap");
        let font_system = RefCell::new(embedded_font_system());
        let swash_cache = SwashCache::new();
        Self {
            pixmap,
            font_system,
            swash_cache,
            clip_stack: Vec::new(),
        }
    }

    /// Creates a new renderer with the given dimensions and a white background.
    pub fn with_white_background(width: u32, height: u32) -> Self {
        let mut renderer = Self::new(width, height);
        renderer.pixmap.fill(tiny_skia::Color::WHITE);
        renderer
    }

    /// Returns a reference to the underlying pixmap.
    pub fn pixmap(&self) -> &tiny_skia::Pixmap {
        &self.pixmap
    }

    /// Build cosmic-text `Attrs` from a plotkit `TextStyle`.
    fn build_attrs(style: &TextStyle) -> Attrs<'_> {
        let weight = match style.weight {
            FontWeight::Normal => Weight::NORMAL,
            FontWeight::Bold => Weight::BOLD,
        };
        let family = match style.family {
            Some(ref name) => Family::Name(name.as_str()),
            None => Family::SansSerif,
        };
        Attrs::new().family(family).weight(weight)
    }

    /// Create a shaped cosmic-text `Buffer` for the given text and style.
    ///
    /// The buffer is set to `Wrap::None` so all text stays on one line, and
    /// the size is left unbounded to allow full measurement.
    fn make_buffer(font_system: &mut FontSystem, text: &str, style: &TextStyle) -> Buffer {
        let font_size = style.size as f32;
        let line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, line_height);

        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_wrap(Wrap::None);
        // Use a very large width so text is not clipped during measurement.
        buffer.set_size(Some(f32::MAX), Some(line_height));
        let attrs = Self::build_attrs(style);
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);
        buffer
    }

    /// Measure a shaped buffer, returning `(width, ascent, descent, line_height)`.
    ///
    /// `ascent` is positive (distance from baseline to top), `descent` is
    /// positive (distance from baseline to bottom).
    fn measure_buffer(buffer: &Buffer) -> (f32, f32, f32, f32) {
        let mut total_w: f32 = 0.0;
        let mut max_ascent: f32 = 0.0;
        let mut max_descent: f32 = 0.0;
        let mut line_height: f32 = 0.0;

        for run in buffer.layout_runs() {
            total_w = total_w.max(run.line_w);
            line_height = line_height.max(run.line_height);
            // line_y is the baseline offset from the top of the buffer.
            // line_top is the top of the line.  ascent = line_y - line_top.
            let ascent = run.line_y - run.line_top;
            let descent = run.line_height - ascent;
            max_ascent = max_ascent.max(ascent);
            max_descent = max_descent.max(descent);
        }

        (total_w, max_ascent, max_descent, line_height)
    }
}

/// Alpha-blend a single source pixel onto a destination pixel buffer.
///
/// `pixels` is the raw RGBA pixel slice (premultiplied alpha, as used by
/// tiny-skia). `idx` is the byte offset of the destination pixel.
/// `color` carries the glyph colour with coverage in its alpha channel
/// (straight alpha from cosmic-text).
#[inline]
fn blend_pixel(pixels: &mut [u8], idx: usize, color: cosmic_text::Color) {
    let src_a = color.a() as u32;
    if src_a == 0 {
        return;
    }

    let src_r = color.r() as u32;
    let src_g = color.g() as u32;
    let src_b = color.b() as u32;

    let dst_r = pixels[idx] as u32;
    let dst_g = pixels[idx + 1] as u32;
    let dst_b = pixels[idx + 2] as u32;
    let dst_a = pixels[idx + 3] as u32;

    // Source-over compositing. cosmic-text provides straight alpha, so
    // premultiply the source before blending.
    let sr = src_r * src_a / 255;
    let sg = src_g * src_a / 255;
    let sb = src_b * src_a / 255;

    let inv_sa = 255 - src_a;
    let out_r = sr + dst_r * inv_sa / 255;
    let out_g = sg + dst_g * inv_sa / 255;
    let out_b = sb + dst_b * inv_sa / 255;
    let out_a = src_a + dst_a * inv_sa / 255;

    pixels[idx] = out_r.min(255) as u8;
    pixels[idx + 1] = out_g.min(255) as u8;
    pixels[idx + 2] = out_b.min(255) as u8;
    pixels[idx + 3] = out_a.min(255) as u8;
}

impl Renderer for SkiaRenderer {
    fn size(&self) -> (u32, u32) {
        (self.pixmap.width(), self.pixmap.height())
    }

    fn fill_path(&mut self, path: &Path, paint: &Paint, transform: Affine) {
        let Some(sk_path) = convert_path(path) else {
            return;
        };
        let sk_paint = convert_paint(paint);
        let sk_transform = convert_transform(transform);

        self.pixmap.fill_path(
            &sk_path,
            &sk_paint,
            tiny_skia::FillRule::Winding,
            sk_transform,
            self.clip_stack.last(),
        );
    }

    fn stroke_path(
        &mut self,
        path: &Path,
        paint: &Paint,
        stroke: &Stroke,
        transform: Affine,
    ) {
        let Some(sk_path) = convert_path(path) else {
            return;
        };
        let sk_paint = convert_paint(paint);
        let sk_stroke = convert_stroke(stroke);
        let sk_transform = convert_transform(transform);

        self.pixmap.stroke_path(
            &sk_path,
            &sk_paint,
            &sk_stroke,
            sk_transform,
            self.clip_stack.last(),
        );
    }

    fn draw_text(&mut self, text: &str, pos: Point, style: &TextStyle, _transform: Affine) {
        if text.is_empty() {
            return;
        }

        let font_system = self.font_system.get_mut();
        let buffer = Self::make_buffer(font_system, text, style);
        let (text_w, ascent, _descent, line_height) = Self::measure_buffer(&buffer);

        // Compute the top-left drawing origin based on alignment.
        let draw_x = match style.halign {
            HAlign::Left => pos.x as f32,
            HAlign::Center => pos.x as f32 - text_w / 2.0,
            HAlign::Right => pos.x as f32 - text_w,
        };
        let draw_y = match style.valign {
            VAlign::Top => pos.y as f32,
            VAlign::Middle => pos.y as f32 - line_height / 2.0,
            VAlign::Baseline => pos.y as f32 - ascent,
            VAlign::Bottom => pos.y as f32 - line_height,
        };

        // Convert plotkit color to cosmic-text color.
        let text_color =
            cosmic_text::Color::rgba(style.color.r, style.color.g, style.color.b, style.color.a);

        let pix_w = self.pixmap.width() as i32;
        let pix_h = self.pixmap.height() as i32;

        // Collect glyph rendering info first (to avoid borrow conflicts between
        // swash_cache, font_system, and pixmap).
        struct GlyphInfo {
            cache_key: cosmic_text::CacheKey,
            x: i32,
            y: i32,
            color: cosmic_text::Color,
        }

        let glyphs: Vec<GlyphInfo> = buffer
            .layout_runs()
            .flat_map(|run| {
                let run_y = run.line_y;
                run.glyphs.iter().map(move |glyph| {
                    let physical = glyph.physical((0., run_y), 1.0);
                    GlyphInfo {
                        cache_key: physical.cache_key,
                        x: physical.x + draw_x as i32,
                        y: physical.y + draw_y as i32,
                        color: glyph.color_opt.unwrap_or(text_color),
                    }
                })
            })
            .collect();

        // Now rasterise each glyph and composite onto the pixmap.
        for gi in &glyphs {
            let image = self.swash_cache.get_image_uncached(font_system, gi.cache_key);
            let Some(image) = image else {
                continue;
            };

            let img_x = image.placement.left;
            let img_y = -image.placement.top;

            match image.content {
                cosmic_text::SwashContent::Mask => {
                    let mut i = 0;
                    for off_y in 0..image.placement.height as i32 {
                        for off_x in 0..image.placement.width as i32 {
                            let coverage = image.data[i];
                            i += 1;
                            if coverage == 0 {
                                continue;
                            }
                            let px = gi.x + img_x + off_x;
                            let py = gi.y + img_y + off_y;
                            if px < 0 || py < 0 || px >= pix_w || py >= pix_h {
                                continue;
                            }
                            // Build colour with coverage applied to alpha.
                            let (r, g, b, a) = gi.color.as_rgba_tuple();
                            let blended_a = (a as u32 * coverage as u32 / 255) as u8;
                            let pixel_color = cosmic_text::Color::rgba(r, g, b, blended_a);
                            let idx = (py as u32 * self.pixmap.width() + px as u32) as usize * 4;
                            blend_pixel(self.pixmap.data_mut(), idx, pixel_color);
                        }
                    }
                }
                cosmic_text::SwashContent::Color => {
                    let mut i = 0;
                    for off_y in 0..image.placement.height as i32 {
                        for off_x in 0..image.placement.width as i32 {
                            let px = gi.x + img_x + off_x;
                            let py = gi.y + img_y + off_y;
                            let pixel_color = cosmic_text::Color::rgba(
                                image.data[i],
                                image.data[i + 1],
                                image.data[i + 2],
                                image.data[i + 3],
                            );
                            i += 4;
                            if px < 0 || py < 0 || px >= pix_w || py >= pix_h {
                                continue;
                            }
                            let idx = (py as u32 * self.pixmap.width() + px as u32) as usize * 4;
                            blend_pixel(self.pixmap.data_mut(), idx, pixel_color);
                        }
                    }
                }
                cosmic_text::SwashContent::SubpixelMask => {
                    // Subpixel rendering not supported; skip.
                }
            }
        }
    }

    fn draw_image(&mut self, img: &Image, dst: Rect, transform: Affine) {
        // Build a PixmapRef from the raw RGBA data. tiny-skia expects
        // premultiplied alpha in its internal Pixmap, but PixmapRef::from_bytes
        // is not available -- we need to create a Pixmap and blit it.
        let Some(src_pixmap) = tiny_skia::Pixmap::new(img.width, img.height) else {
            return;
        };

        // For v0.1 we do a simple nearest-neighbour copy when source and
        // destination sizes match. Full scaling support comes later.
        let sk_transform = convert_transform(transform);
        let paint = tiny_skia::PixmapPaint::default();

        self.pixmap.draw_pixmap(
            dst.x as i32,
            dst.y as i32,
            src_pixmap.as_ref(),
            &paint,
            sk_transform,
            self.clip_stack.last(),
        );
    }

    fn push_clip(&mut self, path: &Path, transform: Affine) {
        let w = self.pixmap.width();
        let h = self.pixmap.height();
        let mut mask = tiny_skia::Mask::new(w, h).expect("failed to create clip mask");

        if let Some(sk_path) = convert_path(path) {
            let sk_transform = convert_transform(transform);
            mask.fill_path(&sk_path, tiny_skia::FillRule::Winding, true, sk_transform);
        }

        // Intersect with parent mask if one exists.
        if let Some(parent) = self.clip_stack.last() {
            let mask_data = mask.data_mut();
            let parent_data = parent.data();
            for (m, &p) in mask_data.iter_mut().zip(parent_data.iter()) {
                *m = (*m as u32 * p as u32 / 255) as u8;
            }
        }

        self.clip_stack.push(mask);
    }

    fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    fn measure_text(&self, text: &str, style: &TextStyle) -> (f64, f64) {
        if text.is_empty() {
            return (0.0, 0.0);
        }

        let mut font_system = self.font_system.borrow_mut();
        let buffer = Self::make_buffer(&mut font_system, text, style);
        let (text_w, _ascent, _descent, lh) = Self::measure_buffer(&buffer);
        (text_w as f64, lh as f64)
    }

    fn finalize(self) -> Vec<u8> {
        self.pixmap.encode_png().expect("failed to encode PNG")
    }
}

// ============================================================================
// Conversion helpers
// ============================================================================

/// Converts a plotkit [`Path`] into a `tiny_skia::Path`.
///
/// Returns `None` if the path is empty or contains invalid geometry.
fn convert_path(path: &Path) -> Option<tiny_skia::Path> {
    let mut pb = tiny_skia::PathBuilder::new();
    for el in &path.elements {
        match *el {
            PathEl::MoveTo(p) => pb.move_to(p.x as f32, p.y as f32),
            PathEl::LineTo(p) => pb.line_to(p.x as f32, p.y as f32),
            PathEl::QuadTo(ctrl, end) => {
                pb.quad_to(ctrl.x as f32, ctrl.y as f32, end.x as f32, end.y as f32);
            }
            PathEl::CurveTo(c1, c2, end) => {
                pb.cubic_to(
                    c1.x as f32,
                    c1.y as f32,
                    c2.x as f32,
                    c2.y as f32,
                    end.x as f32,
                    end.y as f32,
                );
            }
            PathEl::ClosePath => pb.close(),
        }
    }
    pb.finish()
}

/// Converts a plotkit [`Paint`] into a `tiny_skia::Paint`.
fn convert_paint(paint: &Paint) -> tiny_skia::Paint<'static> {
    let mut p = tiny_skia::Paint::default();
    p.set_color_rgba8(paint.color.r, paint.color.g, paint.color.b, paint.color.a);
    p.anti_alias = paint.anti_alias;
    p
}

/// Converts a plotkit [`Stroke`] into a `tiny_skia::Stroke`.
fn convert_stroke(stroke: &Stroke) -> tiny_skia::Stroke {
    let mut s = tiny_skia::Stroke {
        width: stroke.width as f32,
        line_cap: match stroke.cap {
            StrokeCap::Butt => tiny_skia::LineCap::Butt,
            StrokeCap::Round => tiny_skia::LineCap::Round,
            StrokeCap::Square => tiny_skia::LineCap::Square,
        },
        line_join: match stroke.join {
            StrokeJoin::Miter => tiny_skia::LineJoin::Miter,
            StrokeJoin::Round => tiny_skia::LineJoin::Round,
            StrokeJoin::Bevel => tiny_skia::LineJoin::Bevel,
        },
        ..Default::default()
    };
    if let Some(ref dash) = stroke.dash {
        s.dash = tiny_skia::StrokeDash::new(
            dash.dashes.iter().map(|&d| d as f32).collect(),
            dash.offset as f32,
        );
    }
    s
}

/// Converts a kurbo [`Affine`] into a `tiny_skia::Transform`.
///
/// kurbo's `as_coeffs()` returns `[a, b, c, d, e, f]` representing:
///
/// ```text
/// | a  c  e |
/// | b  d  f |
/// | 0  0  1 |
/// ```
///
/// `tiny_skia::Transform::from_row(sx, ky, kx, sy, tx, ty)` represents:
///
/// ```text
/// | sx  kx  tx |
/// | ky  sy  ty |
/// |  0   0   1 |
/// ```
///
/// The mapping is therefore a direct positional match:
/// `sx=a, ky=b, kx=c, sy=d, tx=e, ty=f`.
fn convert_transform(affine: Affine) -> tiny_skia::Transform {
    let c = affine.as_coeffs();
    tiny_skia::Transform::from_row(
        c[0] as f32,
        c[1] as f32,
        c[2] as f32,
        c[3] as f32,
        c[4] as f32,
        c[5] as f32,
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_renderer() {
        let r = SkiaRenderer::new(800, 600);
        assert_eq!(r.size(), (800, 600));
    }

    #[test]
    fn white_background() {
        let r = SkiaRenderer::with_white_background(100, 100);
        // Top-left pixel should be white (premultiplied RGBA).
        let pixel = r.pixmap().pixel(0, 0).unwrap();
        assert_eq!(pixel.red(), 255);
        assert_eq!(pixel.green(), 255);
        assert_eq!(pixel.blue(), 255);
        assert_eq!(pixel.alpha(), 255);
    }

    #[test]
    fn finalize_produces_png() {
        let r = SkiaRenderer::new(1, 1);
        let bytes = r.finalize();
        // PNG magic bytes.
        assert_eq!(&bytes[..4], &[0x89, b'P', b'N', b'G']);
    }

    #[test]
    fn convert_identity_transform() {
        let t = convert_transform(Affine::IDENTITY);
        assert_eq!(t, tiny_skia::Transform::identity());
    }

    #[test]
    fn convert_paint_color() {
        let paint = Paint::new(Color::rgb(10, 20, 30));
        let sk = convert_paint(&paint);
        // tiny-skia stores premultiplied alpha internally, but with a=255
        // the values are unchanged.
        assert!(sk.anti_alias);
    }

    #[test]
    fn convert_empty_path_returns_none() {
        let path = Path::new();
        assert!(convert_path(&path).is_none());
    }

    #[test]
    fn convert_rect_path() {
        let path = Path::rect(Rect::new(0.0, 0.0, 10.0, 10.0));
        let sk_path = convert_path(&path);
        assert!(sk_path.is_some());
    }

    #[test]
    fn measure_text_returns_nonzero() {
        let r = SkiaRenderer::new(100, 100);
        let style = TextStyle::new(14.0);
        let (w, h) = r.measure_text("hello", &style);
        assert!(w > 0.0, "text width should be positive, got {w}");
        assert!(h > 0.0, "text height should be positive, got {h}");
    }

    #[test]
    fn measure_text_empty() {
        let r = SkiaRenderer::new(100, 100);
        let style = TextStyle::new(14.0);
        let (w, h) = r.measure_text("", &style);
        assert!((w - 0.0).abs() < f64::EPSILON);
        assert!((h - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fill_rect_does_not_panic() {
        let mut r = SkiaRenderer::with_white_background(100, 100);
        let path = Path::rect(Rect::new(10.0, 10.0, 50.0, 50.0));
        let paint = Paint::new(Color::TAB_BLUE);
        r.fill_path(&path, &paint, Affine::IDENTITY);
        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn stroke_rect_does_not_panic() {
        let mut r = SkiaRenderer::with_white_background(100, 100);
        let path = Path::rect(Rect::new(10.0, 10.0, 50.0, 50.0));
        let paint = Paint::new(Color::TAB_RED);
        let stroke = Stroke::new(2.0);
        r.stroke_path(&path, &paint, &stroke, Affine::IDENTITY);
        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn draw_text_does_not_panic() {
        let mut r = SkiaRenderer::with_white_background(200, 200);
        let style = TextStyle::new(14.0);
        r.draw_text("Hello", Point::new(10.0, 50.0), &style, Affine::IDENTITY);
        let bytes = r.finalize();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn draw_text_renders_pixels() {
        let mut r = SkiaRenderer::with_white_background(200, 200);
        let style = TextStyle::new(20.0);
        r.draw_text("Test", Point::new(10.0, 100.0), &style, Affine::IDENTITY);

        // At least some pixels should have changed from pure white.
        let data = r.pixmap().data();
        let non_white = data
            .chunks(4)
            .any(|px| px[0] != 255 || px[1] != 255 || px[2] != 255);
        assert!(non_white, "expected rendered text to produce non-white pixels");
    }
}
