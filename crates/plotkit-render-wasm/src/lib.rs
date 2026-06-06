//! WASM Canvas2D rendering backend for plotkit.
//!
//! This crate provides `WasmRenderer`, a rendering backend that translates
//! plotkit drawing primitives into HTML5 Canvas2D API calls via `web-sys`.
//! It is designed to run in WebAssembly environments inside a browser.
//!
//! # Architecture
//!
//! The renderer wraps a `CanvasRenderingContext2d` obtained from an
//! `HtmlCanvasElement`. Each plotkit primitive (path fill, path stroke,
//! text draw, clipping, image blit) maps directly to the corresponding
//! Canvas2D method calls.
//!
//! Helper functions for color conversion, font string construction, and
//! affine transform decomposition are exposed as pure functions so they
//! can be unit-tested without a browser environment.
//!
//! # Example (browser-side)
//!
//! ```ignore
//! use plotkit_render_wasm::WasmRenderer;
//! use web_sys::CanvasRenderingContext2d;
//!
//! let ctx: CanvasRenderingContext2d = /* obtain from canvas element */;
//! let mut renderer = WasmRenderer::new(ctx, 800, 600);
//! // ... use renderer via the Renderer trait ...
//! let _ = renderer.finalize();
//! ```

#![deny(missing_docs)]

// Re-export core types for downstream convenience.
pub use plotkit_core::primitives::*;
pub use plotkit_core::renderer::Renderer;

// ---------------------------------------------------------------------------
// Pure helper functions (testable without a browser)
// ---------------------------------------------------------------------------

/// Converts a plotkit [`Color`] to a CSS `rgba(...)` string.
///
/// The alpha channel is normalized from the 0-255 range to a 0.0-1.0 float
/// with four decimal places of precision, matching CSS color level 4 syntax.
///
/// # Examples
///
/// ```
/// use plotkit_render_wasm::{color_to_css, Color};
///
/// assert_eq!(color_to_css(&Color::rgb(255, 0, 0)), "rgba(255,0,0,1)");
/// assert_eq!(color_to_css(&Color::new(0, 128, 255, 128)), "rgba(0,128,255,0.5020)");
/// ```
pub fn color_to_css(c: &Color) -> String {
    if c.a == 255 {
        format!("rgba({},{},{},1)", c.r, c.g, c.b)
    } else {
        format!(
            "rgba({},{},{},{:.4})",
            c.r,
            c.g,
            c.b,
            c.a as f64 / 255.0
        )
    }
}

/// Builds a CSS font shorthand string from a [`TextStyle`].
///
/// The resulting string follows the CSS font shorthand syntax:
/// `"[weight] [size]px [family]"`. If no family is specified, `"sans-serif"`
/// is used as a sensible default that works across all browsers.
///
/// # Examples
///
/// ```
/// use plotkit_render_wasm::{build_font_string, TextStyle, FontWeight};
///
/// let style = TextStyle::new(14.0);
/// assert_eq!(build_font_string(&style), "14px sans-serif");
///
/// let mut bold = TextStyle::new(16.0);
/// bold.weight = FontWeight::Bold;
/// bold.family = Some("Helvetica".to_string());
/// assert_eq!(build_font_string(&bold), "bold 16px Helvetica");
/// ```
pub fn build_font_string(style: &TextStyle) -> String {
    let weight = match style.weight {
        FontWeight::Normal => "",
        FontWeight::Bold => "bold ",
    };
    let family = style
        .family
        .as_deref()
        .unwrap_or("sans-serif");
    format!("{}{:.0}px {}", weight, style.size, family)
}

/// Returns the Canvas2D `textAlign` property value for a given [`HAlign`].
///
/// Maps plotkit's horizontal alignment enum to the string values expected
/// by `CanvasRenderingContext2d.textAlign`.
pub fn halign_to_canvas(align: HAlign) -> &'static str {
    match align {
        HAlign::Left => "left",
        HAlign::Center => "center",
        HAlign::Right => "right",
    }
}

/// Returns the Canvas2D `textBaseline` property value for a given [`VAlign`].
///
/// Maps plotkit's vertical alignment enum to the string values expected
/// by `CanvasRenderingContext2d.textBaseline`.
pub fn valign_to_canvas(align: VAlign) -> &'static str {
    match align {
        VAlign::Top => "top",
        VAlign::Middle => "middle",
        VAlign::Bottom => "bottom",
        VAlign::Baseline => "alphabetic",
    }
}

/// Returns the Canvas2D `lineCap` property value for a given [`StrokeCap`].
pub fn stroke_cap_to_canvas(cap: StrokeCap) -> &'static str {
    match cap {
        StrokeCap::Butt => "butt",
        StrokeCap::Round => "round",
        StrokeCap::Square => "square",
    }
}

/// Returns the Canvas2D `lineJoin` property value for a given [`StrokeJoin`].
pub fn stroke_join_to_canvas(join: StrokeJoin) -> &'static str {
    match join {
        StrokeJoin::Miter => "miter",
        StrokeJoin::Round => "round",
        StrokeJoin::Bevel => "bevel",
    }
}

/// Counts the number of each path element type in a [`Path`].
///
/// Returns a tuple of `(move_to, line_to, quad_to, curve_to, close)` counts.
/// Useful for diagnostics, debugging, and testing path construction.
pub fn count_path_elements(path: &Path) -> (usize, usize, usize, usize, usize) {
    let mut m = 0;
    let mut l = 0;
    let mut q = 0;
    let mut c = 0;
    let mut z = 0;
    for el in &path.elements {
        match el {
            PathEl::MoveTo(_) => m += 1,
            PathEl::LineTo(_) => l += 1,
            PathEl::QuadTo(_, _) => q += 1,
            PathEl::CurveTo(_, _, _) => c += 1,
            PathEl::ClosePath => z += 1,
        }
    }
    (m, l, q, c, z)
}

/// Decomposes a plotkit [`Affine`] transform into the six parameters
/// expected by `CanvasRenderingContext2d.setTransform(a, b, c, d, e, f)`.
///
/// The kurbo `Affine` stores its coefficients as `[a, b, c, d, e, f]` where
/// the matrix is:
///
/// ```text
/// | a  c  e |
/// | b  d  f |
/// | 0  0  1 |
/// ```
///
/// Canvas2D `setTransform` expects `(a, b, c, d, e, f)` with the same layout.
pub fn affine_to_canvas_params(affine: Affine) -> [f64; 6] {
    affine.as_coeffs()
}

/// Estimates the width of a text string for a given [`TextStyle`].
///
/// This uses a heuristic based on average character width ratios for common
/// proportional fonts. The estimate is `char_count * size * 0.6` for normal
/// weight and `char_count * size * 0.65` for bold, which provides reasonable
/// approximations when the Canvas2D `measureText` API is not available
/// (e.g., during testing or server-side pre-layout).
pub fn estimate_text_width(text: &str, style: &TextStyle) -> f64 {
    let factor = match style.weight {
        FontWeight::Normal => 0.6,
        FontWeight::Bold => 0.65,
    };
    text.len() as f64 * style.size * factor
}

/// Computes a dash pattern array string suitable for Canvas2D `setLineDash`.
///
/// Returns the dash lengths as a `Vec<f64>`. If the stroke has no dash
/// pattern, returns an empty vector (which clears any active dash on the
/// canvas context).
pub fn dash_pattern_values(stroke: &Stroke) -> Vec<f64> {
    match &stroke.dash {
        Some(pattern) => pattern.dashes.clone(),
        None => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// WasmRenderer (only available on wasm32 targets)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    //! Browser-targeted renderer implementation using `web-sys`.

    use super::*;
    use js_sys::Array;
    use wasm_bindgen::prelude::*;
    use web_sys::CanvasRenderingContext2d;

    /// A plotkit renderer that draws to an HTML5 Canvas2D context.
    ///
    /// This struct wraps a `CanvasRenderingContext2d` and implements the full
    /// [`Renderer`] trait. All drawing operations are executed immediately on
    /// the canvas — the `finalize` method returns an empty `Vec<u8>` since
    /// the output is already visible on screen.
    ///
    /// # Clipping
    ///
    /// Clipping is implemented using the canvas `save()`/`restore()` stack.
    /// Each call to [`push_clip`](Renderer::push_clip) saves the context state,
    /// applies a clip path, and the matching [`pop_clip`](Renderer::pop_clip)
    /// restores it.
    pub struct WasmRenderer {
        ctx: CanvasRenderingContext2d,
        width: u32,
        height: u32,
    }

    impl WasmRenderer {
        /// Creates a new WASM renderer targeting the given canvas context.
        ///
        /// The `width` and `height` parameters should match the canvas element's
        /// dimensions in CSS pixels. They are used for `Renderer::size()` and
        /// do not modify the canvas element itself.
        pub fn new(ctx: CanvasRenderingContext2d, width: u32, height: u32) -> Self {
            Self { ctx, width, height }
        }

        /// Returns a reference to the underlying `CanvasRenderingContext2d`.
        pub fn context(&self) -> &CanvasRenderingContext2d {
            &self.ctx
        }

        /// Traces a plotkit [`Path`] onto the current canvas path.
        ///
        /// This begins a new path on the context and issues the appropriate
        /// `moveTo`, `lineTo`, `quadraticCurveTo`, and `bezierCurveTo` calls
        /// for each path element.
        fn trace_path(&self, path: &Path) {
            self.ctx.begin_path();
            for el in &path.elements {
                match *el {
                    PathEl::MoveTo(p) => {
                        self.ctx.move_to(p.x, p.y);
                    }
                    PathEl::LineTo(p) => {
                        self.ctx.line_to(p.x, p.y);
                    }
                    PathEl::QuadTo(cp, end) => {
                        self.ctx.quadratic_curve_to(cp.x, cp.y, end.x, end.y);
                    }
                    PathEl::CurveTo(cp1, cp2, end) => {
                        self.ctx.bezier_curve_to(
                            cp1.x, cp1.y, cp2.x, cp2.y, end.x, end.y,
                        );
                    }
                    PathEl::ClosePath => {
                        self.ctx.close_path();
                    }
                }
            }
        }

        /// Applies a plotkit [`Affine`] transform to the canvas context.
        fn apply_transform(&self, transform: Affine) {
            let [a, b, c, d, e, f] = affine_to_canvas_params(transform);
            let _ = self.ctx.set_transform(a, b, c, d, e, f);
        }

        /// Resets the canvas transform to the identity matrix.
        fn reset_transform(&self) {
            let _ = self.ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        }

        /// Configures the canvas stroke style from plotkit types.
        fn configure_stroke(&self, paint: &Paint, stroke: &Stroke) {
            let color = color_to_css(&paint.color);
            self.ctx.set_stroke_style_str(&color);
            self.ctx.set_line_width(stroke.width);
            self.ctx.set_line_cap(stroke_cap_to_canvas(stroke.cap));
            self.ctx.set_line_join(stroke_join_to_canvas(stroke.join));

            let dash_values = dash_pattern_values(stroke);
            let js_array = Array::new();
            for &v in &dash_values {
                js_array.push(&JsValue::from_f64(v));
            }
            let _ = self.ctx.set_line_dash(&js_array);

            if let Some(ref pattern) = stroke.dash {
                self.ctx.set_line_dash_offset(pattern.offset);
            } else {
                self.ctx.set_line_dash_offset(0.0);
            }
        }
    }

    impl Renderer for WasmRenderer {
        fn size(&self) -> (u32, u32) {
            (self.width, self.height)
        }

        fn fill_path(&mut self, path: &Path, paint: &Paint, transform: Affine) {
            self.ctx.save();
            self.apply_transform(transform);

            let color = color_to_css(&paint.color);
            self.ctx.set_fill_style_str(&color);

            self.trace_path(path);
            self.ctx.fill();

            self.ctx.restore();
        }

        fn stroke_path(
            &mut self,
            path: &Path,
            paint: &Paint,
            stroke: &Stroke,
            transform: Affine,
        ) {
            self.ctx.save();
            self.apply_transform(transform);
            self.configure_stroke(paint, stroke);

            self.trace_path(path);
            self.ctx.stroke();

            self.ctx.restore();
        }

        fn draw_text(&mut self, text: &str, pos: Point, style: &TextStyle, transform: Affine) {
            self.ctx.save();
            self.apply_transform(transform);

            let font = build_font_string(style);
            self.ctx.set_font(&font);

            let color = color_to_css(&style.color);
            self.ctx.set_fill_style_str(&color);

            self.ctx.set_text_align(halign_to_canvas(style.halign));
            self.ctx
                .set_text_baseline(valign_to_canvas(style.valign));

            let _ = self.ctx.fill_text(text, pos.x, pos.y);

            self.ctx.restore();
        }

        fn draw_image(&mut self, img: &Image, dst: Rect, transform: Affine) {
            self.ctx.save();
            self.apply_transform(transform);

            // Create ImageData from raw RGBA pixels and draw it scaled into the
            // destination rectangle. We use a temporary canvas for proper scaling.
            if let Ok(clamped) =
                wasm_bindgen::Clamped(img.data.as_slice()).try_into()
            {
                if let Ok(image_data) =
                    web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                        clamped,
                        img.width,
                        img.height,
                    )
                {
                    // Use createImageBitmap or putImageData with scaling.
                    // The simplest approach: put image data at origin, then
                    // use drawImage to scale. For proper scaling we create
                    // a temporary offscreen canvas.
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Ok(temp_canvas) = document.create_element("canvas") {
                                let temp_canvas: web_sys::HtmlCanvasElement =
                                    temp_canvas.unchecked_into();
                                temp_canvas.set_width(img.width);
                                temp_canvas.set_height(img.height);
                                if let Ok(Some(temp_ctx)) =
                                    temp_canvas.get_context("2d")
                                {
                                    let temp_ctx: CanvasRenderingContext2d =
                                        temp_ctx.unchecked_into();
                                    let _ = temp_ctx.put_image_data(&image_data, 0.0, 0.0);
                                    let _ = self.ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                                        &temp_canvas,
                                        dst.x,
                                        dst.y,
                                        dst.width,
                                        dst.height,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            self.ctx.restore();
        }

        fn push_clip(&mut self, path: &Path, transform: Affine) {
            self.ctx.save();
            self.apply_transform(transform);
            self.trace_path(path);
            self.ctx.clip();
            // Reset transform after clipping so subsequent draws are not
            // double-transformed. The clip region remains in the saved state.
            self.reset_transform();
        }

        fn pop_clip(&mut self) {
            self.ctx.restore();
        }

        fn measure_text(&self, text: &str, style: &TextStyle) -> (f64, f64) {
            let font = build_font_string(style);
            self.ctx.set_font(&font);

            if let Ok(metrics) = self.ctx.measure_text(text) {
                let width = metrics.width();
                // Canvas2D measureText gives width natively. For height,
                // use the font metrics if available, otherwise fall back
                // to the font size as a reasonable approximation.
                let height = style.size;
                (width, height)
            } else {
                // Fallback to heuristic estimate if measureText fails.
                (estimate_text_width(text, style), style.size)
            }
        }

        fn finalize(self) -> Vec<u8> {
            // Canvas renders immediately — there is no serialized output.
            // Return an empty vector as per the contract for immediate-mode
            // rendering backends.
            Vec::new()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_impl::WasmRenderer;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Color conversion tests --------------------------------------------

    #[test]
    fn color_opaque_to_css() {
        let c = Color::rgb(255, 0, 0);
        assert_eq!(color_to_css(&c), "rgba(255,0,0,1)");
    }

    #[test]
    fn color_transparent_to_css() {
        let c = Color::TRANSPARENT;
        assert_eq!(color_to_css(&c), "rgba(0,0,0,0.0000)");
    }

    #[test]
    fn color_semi_transparent_to_css() {
        let c = Color::new(0, 128, 255, 128);
        let css = color_to_css(&c);
        assert_eq!(css, "rgba(0,128,255,0.5020)");
    }

    #[test]
    fn color_white_to_css() {
        let c = Color::WHITE;
        assert_eq!(color_to_css(&c), "rgba(255,255,255,1)");
    }

    #[test]
    fn color_black_to_css() {
        let c = Color::BLACK;
        assert_eq!(color_to_css(&c), "rgba(0,0,0,1)");
    }

    #[test]
    fn color_with_alpha_one_to_css() {
        // Alpha = 1 (nearly transparent, but not zero)
        let c = Color::new(100, 200, 50, 1);
        let css = color_to_css(&c);
        assert!(css.contains("0.0039"), "expected '0.0039' in {}", css);
    }

    #[test]
    fn color_tableau_blue_to_css() {
        let c = Color::TAB_BLUE; // 0x4E, 0x79, 0xA7
        assert_eq!(color_to_css(&c), "rgba(78,121,167,1)");
    }

    // -- Font string building tests ----------------------------------------

    #[test]
    fn font_string_default() {
        let style = TextStyle::new(14.0);
        assert_eq!(build_font_string(&style), "14px sans-serif");
    }

    #[test]
    fn font_string_bold() {
        let mut style = TextStyle::new(20.0);
        style.weight = FontWeight::Bold;
        assert_eq!(build_font_string(&style), "bold 20px sans-serif");
    }

    #[test]
    fn font_string_custom_family() {
        let mut style = TextStyle::new(12.0);
        style.family = Some("Helvetica Neue".to_string());
        assert_eq!(build_font_string(&style), "12px Helvetica Neue");
    }

    #[test]
    fn font_string_bold_custom_family() {
        let mut style = TextStyle::new(16.0);
        style.weight = FontWeight::Bold;
        style.family = Some("Georgia".to_string());
        assert_eq!(build_font_string(&style), "bold 16px Georgia");
    }

    #[test]
    fn font_string_fractional_size() {
        let style = TextStyle::new(10.5);
        // The format spec is {:.0} so it should round
        assert_eq!(build_font_string(&style), "10px sans-serif");
    }

    // -- Alignment mapping tests -------------------------------------------

    #[test]
    fn halign_mapping() {
        assert_eq!(halign_to_canvas(HAlign::Left), "left");
        assert_eq!(halign_to_canvas(HAlign::Center), "center");
        assert_eq!(halign_to_canvas(HAlign::Right), "right");
    }

    #[test]
    fn valign_mapping() {
        assert_eq!(valign_to_canvas(VAlign::Top), "top");
        assert_eq!(valign_to_canvas(VAlign::Middle), "middle");
        assert_eq!(valign_to_canvas(VAlign::Bottom), "bottom");
        assert_eq!(valign_to_canvas(VAlign::Baseline), "alphabetic");
    }

    // -- Stroke style mapping tests ----------------------------------------

    #[test]
    fn stroke_cap_mapping() {
        assert_eq!(stroke_cap_to_canvas(StrokeCap::Butt), "butt");
        assert_eq!(stroke_cap_to_canvas(StrokeCap::Round), "round");
        assert_eq!(stroke_cap_to_canvas(StrokeCap::Square), "square");
    }

    #[test]
    fn stroke_join_mapping() {
        assert_eq!(stroke_join_to_canvas(StrokeJoin::Miter), "miter");
        assert_eq!(stroke_join_to_canvas(StrokeJoin::Round), "round");
        assert_eq!(stroke_join_to_canvas(StrokeJoin::Bevel), "bevel");
    }

    // -- Path element counting tests ---------------------------------------

    #[test]
    fn count_empty_path() {
        let path = Path::new();
        assert_eq!(count_path_elements(&path), (0, 0, 0, 0, 0));
    }

    #[test]
    fn count_rect_path() {
        let path = Path::rect(Rect::new(0.0, 0.0, 100.0, 50.0));
        let (m, l, q, c, z) = count_path_elements(&path);
        assert_eq!(m, 1, "rect should have 1 MoveTo");
        assert_eq!(l, 3, "rect should have 3 LineTo");
        assert_eq!(q, 0, "rect should have 0 QuadTo");
        assert_eq!(c, 0, "rect should have 0 CurveTo");
        assert_eq!(z, 1, "rect should have 1 ClosePath");
    }

    #[test]
    fn count_circle_path() {
        let path = Path::circle(Point::new(50.0, 50.0), 25.0);
        let (m, l, q, c, z) = count_path_elements(&path);
        assert_eq!(m, 1, "circle should have 1 MoveTo");
        assert_eq!(l, 0, "circle should have 0 LineTo");
        assert_eq!(q, 0, "circle should have 0 QuadTo");
        assert_eq!(c, 4, "circle should have 4 CurveTo");
        assert_eq!(z, 1, "circle should have 1 ClosePath");
    }

    #[test]
    fn count_mixed_path() {
        let mut path = Path::new();
        path.move_to(0.0, 0.0)
            .line_to(10.0, 0.0)
            .quad_to(15.0, 5.0, 10.0, 10.0)
            .curve_to(5.0, 15.0, -5.0, 15.0, -10.0, 10.0)
            .close();
        let (m, l, q, c, z) = count_path_elements(&path);
        assert_eq!(m, 1);
        assert_eq!(l, 1);
        assert_eq!(q, 1);
        assert_eq!(c, 1);
        assert_eq!(z, 1);
    }

    // -- Affine transform tests -------------------------------------------

    #[test]
    fn identity_affine_params() {
        let params = affine_to_canvas_params(Affine::IDENTITY);
        assert_eq!(params, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn translate_affine_params() {
        let t = Affine::translate((100.0, 200.0));
        let [a, b, c, d, e, f] = affine_to_canvas_params(t);
        assert_eq!(a, 1.0);
        assert_eq!(b, 0.0);
        assert_eq!(c, 0.0);
        assert_eq!(d, 1.0);
        assert_eq!(e, 100.0);
        assert_eq!(f, 200.0);
    }

    #[test]
    fn scale_affine_params() {
        let s = Affine::scale_non_uniform(2.0, 3.0);
        let [a, b, c, d, e, f] = affine_to_canvas_params(s);
        assert_eq!(a, 2.0);
        assert_eq!(d, 3.0);
        assert_eq!(e, 0.0);
        assert_eq!(f, 0.0);
        assert_eq!(b, 0.0);
        assert_eq!(c, 0.0);
    }

    // -- Text estimation tests --------------------------------------------

    #[test]
    fn estimate_text_width_normal() {
        let style = TextStyle::new(10.0);
        let width = estimate_text_width("hello", &style);
        // 5 chars * 10.0 * 0.6 = 30.0
        assert!((width - 30.0).abs() < 1e-10);
    }

    #[test]
    fn estimate_text_width_bold() {
        let mut style = TextStyle::new(10.0);
        style.weight = FontWeight::Bold;
        let width = estimate_text_width("hello", &style);
        // 5 chars * 10.0 * 0.65 = 32.5
        assert!((width - 32.5).abs() < 1e-10);
    }

    #[test]
    fn estimate_text_width_empty() {
        let style = TextStyle::new(16.0);
        let width = estimate_text_width("", &style);
        assert_eq!(width, 0.0);
    }

    // -- Dash pattern tests -----------------------------------------------

    #[test]
    fn dash_pattern_solid_stroke() {
        let stroke = Stroke::new(2.0);
        let dashes = dash_pattern_values(&stroke);
        assert!(dashes.is_empty());
    }

    #[test]
    fn dash_pattern_dashed_stroke() {
        let stroke = Stroke::new(1.5).with_dash(DashPattern {
            dashes: vec![5.0, 3.0, 1.0],
            offset: 2.0,
        });
        let dashes = dash_pattern_values(&stroke);
        assert_eq!(dashes, vec![5.0, 3.0, 1.0]);
    }

    // -- Font size rounding edge cases ------------------------------------

    #[test]
    fn font_string_large_size() {
        let style = TextStyle::new(72.0);
        assert_eq!(build_font_string(&style), "72px sans-serif");
    }

    #[test]
    fn font_string_small_size() {
        let style = TextStyle::new(6.0);
        assert_eq!(build_font_string(&style), "6px sans-serif");
    }
}
