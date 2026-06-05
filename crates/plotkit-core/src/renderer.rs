//! The renderer trait — the single seam between chart logic and output backends.

use crate::primitives::{Affine, Image, Paint, Path, Point, Rect, Stroke, TextStyle};

/// A rendering backend that can produce raster or vector output.
///
/// Implementations translate plotkit's core primitive types into backend-specific
/// draw calls. PNG, SVG, PDF, and WASM are all implementations of this trait.
///
/// # Contract
///
/// - All coordinates and transforms use plotkit's coordinate system (origin top-left, y-down).
/// - The renderer must handle the full `Paint`, `Stroke`, and `TextStyle` types faithfully.
/// - `finalize` consumes the renderer and returns the encoded output bytes.
pub trait Renderer {
    /// Returns the output dimensions in pixels.
    fn size(&self) -> (u32, u32);

    /// Fills a path with the given paint, under the given transform.
    fn fill_path(&mut self, path: &Path, paint: &Paint, transform: Affine);

    /// Strokes a path with the given paint and stroke style, under the given transform.
    fn stroke_path(&mut self, path: &Path, paint: &Paint, stroke: &Stroke, transform: Affine);

    /// Draws text at the given position with the given style, under the given transform.
    fn draw_text(&mut self, text: &str, pos: Point, style: &TextStyle, transform: Affine);

    /// Draws a raster image into the destination rectangle, under the given transform.
    fn draw_image(&mut self, img: &Image, dst: Rect, transform: Affine);

    /// Pushes a clipping path. All subsequent draws are clipped to this path.
    fn push_clip(&mut self, path: &Path, transform: Affine);

    /// Pops the most recent clipping path.
    fn pop_clip(&mut self);

    /// Measures the bounding box of the given text with the given style.
    /// Returns the width and height in pixels.
    fn measure_text(&self, text: &str, style: &TextStyle) -> (f64, f64);

    /// Consumes the renderer and returns the encoded output (PNG bytes, SVG string as bytes, etc.).
    fn finalize(self) -> Vec<u8>;
}
