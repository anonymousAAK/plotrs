//! Colorbar component for displaying color-to-value mappings.
//!
//! A colorbar is a narrow strip showing the gradient of a colormap alongside
//! tick marks and labels indicating the corresponding data values. It is the
//! essential companion to heatmaps and colormap-based scatter plots.
//!
//! # Examples
//!
//! ```
//! use plotkit_core::colorbar::{Colorbar, ColorbarOrientation};
//! use plotkit_core::colormap::Colormap;
//!
//! let cb = Colorbar::new(Colormap::Viridis, 0.0, 100.0)
//!     .label("Temperature (C)")
//!     .orientation(ColorbarOrientation::Vertical)
//!     .num_steps(128);
//! ```

use crate::colormap::Colormap;
use crate::primitives::*;
use crate::renderer::Renderer;
use crate::ticks;
use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default number of discrete color steps in the gradient strip.
const DEFAULT_NUM_STEPS: usize = 256;

/// Number of ticks to aim for on the colorbar axis.
const COLORBAR_TICK_COUNT: usize = 5;

/// Fraction of the colorbar rect reserved for the gradient strip (the
/// remainder is used for tick labels and the optional label).
const GRADIENT_FRACTION: f64 = 0.35;

/// Padding between the gradient strip and the tick labels (in pixels).
const TICK_LABEL_GAP: f64 = 4.0;

/// Padding between the tick labels and the colorbar label (in pixels).
const LABEL_GAP: f64 = 6.0;

/// Length of each tick mark (in pixels).
const TICK_LENGTH: f64 = 4.0;

/// Fraction of the axes width reserved for the colorbar when one is present.
pub(crate) const COLORBAR_WIDTH_FRACTION: f64 = 0.12;

/// Gap between the plot area and the colorbar (in pixels).
pub(crate) const COLORBAR_GAP: f64 = 10.0;

// ---------------------------------------------------------------------------
// ColorbarOrientation
// ---------------------------------------------------------------------------

/// Orientation of the colorbar gradient strip.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ColorbarOrientation {
    /// A vertical strip with values increasing from bottom to top.
    #[default]
    Vertical,
    /// A horizontal strip with values increasing from left to right.
    Horizontal,
}

// ---------------------------------------------------------------------------
// Colorbar
// ---------------------------------------------------------------------------

/// Configuration for a colorbar that maps colors to data values.
///
/// A colorbar renders as a narrow gradient strip with tick marks and labels
/// showing the value-to-color mapping used by a heatmap or scatter plot.
///
/// # Builder API
///
/// Use [`Colorbar::new`] to create a colorbar, then chain builder methods
/// (`.label()`, `.orientation()`, `.num_steps()`) to customise it.
#[derive(Debug, Clone)]
pub struct Colorbar {
    /// The colormap to display.
    pub cmap: Colormap,
    /// Minimum value of the data range.
    pub vmin: f64,
    /// Maximum value of the data range.
    pub vmax: f64,
    /// Optional label text displayed alongside the gradient.
    pub label: Option<String>,
    /// Orientation of the gradient strip.
    pub orientation: ColorbarOrientation,
    /// Number of discrete color steps used to render the gradient.
    pub num_steps: usize,
}

impl Colorbar {
    /// Creates a new colorbar with the given colormap and value range.
    ///
    /// Defaults to vertical orientation with 256 color steps and no label.
    pub fn new(cmap: Colormap, vmin: f64, vmax: f64) -> Self {
        Self {
            cmap,
            vmin,
            vmax,
            label: None,
            orientation: ColorbarOrientation::default(),
            num_steps: DEFAULT_NUM_STEPS,
        }
    }

    /// Sets the label text displayed alongside the colorbar gradient.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the orientation of the colorbar.
    pub fn orientation(mut self, orientation: ColorbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the number of discrete color steps in the gradient.
    ///
    /// Higher values produce smoother gradients. The default is 256.
    pub fn num_steps(mut self, n: usize) -> Self {
        self.num_steps = n.max(2);
        self
    }

    /// Sets the label text (mutable-borrow variant for in-place building).
    pub fn set_label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the orientation (mutable-borrow variant for in-place building).
    pub fn set_orientation(&mut self, orientation: ColorbarOrientation) -> &mut Self {
        self.orientation = orientation;
        self
    }

    /// Sets the number of color steps (mutable-borrow variant for in-place building).
    pub fn set_num_steps(&mut self, n: usize) -> &mut Self {
        self.num_steps = n.max(2);
        self
    }
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

/// Draws a colorbar into the given rectangle.
///
/// The function divides `rect` into a gradient area and a label/tick area,
/// draws the colormap gradient as a series of thin filled rectangles, then
/// overlays tick marks and labels showing the corresponding data values.
///
/// For a vertical colorbar, values increase from bottom to top. For a
/// horizontal colorbar, values increase from left to right.
pub fn draw_colorbar(
    renderer: &mut impl Renderer,
    colorbar: &Colorbar,
    rect: &Rect,
    theme: &Theme,
) {
    match colorbar.orientation {
        ColorbarOrientation::Vertical => draw_vertical(renderer, colorbar, rect, theme),
        ColorbarOrientation::Horizontal => draw_horizontal(renderer, colorbar, rect, theme),
    }
}

/// Draws a vertical colorbar (gradient runs bottom-to-top).
fn draw_vertical(
    renderer: &mut impl Renderer,
    colorbar: &Colorbar,
    rect: &Rect,
    theme: &Theme,
) {
    // Divide the rect: gradient on the left, ticks/label on the right.
    let gradient_width = (rect.width * GRADIENT_FRACTION).max(8.0);
    let gradient_rect = Rect::new(rect.x, rect.y, gradient_width, rect.height);

    // --- Draw gradient strip ---
    let n = colorbar.num_steps;
    let step_height = rect.height / n as f64;

    for i in 0..n {
        // t goes from 1.0 (top) to 0.0 (bottom) so that values increase upward.
        let t = 1.0 - (i as f64 + 0.5) / n as f64;
        let color = colorbar.cmap.map(t);

        let cell_y = rect.y + i as f64 * step_height;
        let cell_rect = Rect::new(rect.x, cell_y, gradient_width, step_height + 0.5);
        let cell_path = Path::rect(cell_rect);
        renderer.fill_path(&cell_path, &Paint::new(color), Affine::IDENTITY);
    }

    // --- Draw border around gradient ---
    let border_path = Path::rect(gradient_rect);
    let border_paint = Paint::new(theme.spine_color);
    let border_stroke = Stroke::new(theme.spine_width);
    renderer.stroke_path(&border_path, &border_paint, &border_stroke, Affine::IDENTITY);

    // --- Generate ticks ---
    let tick_data = ticks::generate_ticks(
        colorbar.vmin,
        colorbar.vmax,
        COLORBAR_TICK_COUNT,
        &crate::scale::Scale::Linear,
    );

    let tick_x_start = rect.x + gradient_width;
    let tick_x_end = tick_x_start + TICK_LENGTH;
    let label_x = tick_x_end + TICK_LABEL_GAP;

    let tick_paint = Paint::new(theme.tick_color);
    let tick_stroke = Stroke::new(theme.spine_width);

    let label_style = TextStyle {
        size: theme.tick_label_size,
        color: theme.text_color,
        weight: FontWeight::Normal,
        family: theme.font_family.clone(),
        halign: HAlign::Left,
        valign: VAlign::Middle,
    };

    let range = colorbar.vmax - colorbar.vmin;

    for tick in &tick_data {
        // Map tick value to y pixel (vmax at top, vmin at bottom).
        let t = if range.abs() < f64::EPSILON {
            0.5
        } else {
            (tick.value - colorbar.vmin) / range
        };
        let py = rect.y + rect.height * (1.0 - t);

        // Tick mark.
        let mut tick_path = Path::new();
        tick_path.move_to(tick_x_start, py);
        tick_path.line_to(tick_x_end, py);
        renderer.stroke_path(&tick_path, &tick_paint, &tick_stroke, Affine::IDENTITY);

        // Tick label.
        renderer.draw_text(
            &tick.label,
            Point::new(label_x, py),
            &label_style,
            Affine::IDENTITY,
        );
    }

    // --- Draw optional label ---
    if let Some(ref label_text) = colorbar.label {
        let cb_label_style = TextStyle {
            size: theme.axis_label_size,
            color: theme.text_color,
            weight: FontWeight::Normal,
            family: theme.font_family.clone(),
            halign: HAlign::Center,
            valign: VAlign::Top,
        };

        // Place label below the gradient, centered horizontally.
        let label_x_pos = rect.x + gradient_width / 2.0;
        let label_y_pos = rect.y + rect.height + LABEL_GAP;
        renderer.draw_text(
            label_text,
            Point::new(label_x_pos, label_y_pos),
            &cb_label_style,
            Affine::IDENTITY,
        );
    }
}

/// Draws a horizontal colorbar (gradient runs left-to-right).
fn draw_horizontal(
    renderer: &mut impl Renderer,
    colorbar: &Colorbar,
    rect: &Rect,
    theme: &Theme,
) {
    // Divide the rect: gradient on top, ticks/label below.
    let gradient_height = (rect.height * GRADIENT_FRACTION).max(8.0);
    let gradient_rect = Rect::new(rect.x, rect.y, rect.width, gradient_height);

    // --- Draw gradient strip ---
    let n = colorbar.num_steps;
    let step_width = rect.width / n as f64;

    for i in 0..n {
        let t = (i as f64 + 0.5) / n as f64;
        let color = colorbar.cmap.map(t);

        let cell_x = rect.x + i as f64 * step_width;
        let cell_rect = Rect::new(cell_x, rect.y, step_width + 0.5, gradient_height);
        let cell_path = Path::rect(cell_rect);
        renderer.fill_path(&cell_path, &Paint::new(color), Affine::IDENTITY);
    }

    // --- Draw border around gradient ---
    let border_path = Path::rect(gradient_rect);
    let border_paint = Paint::new(theme.spine_color);
    let border_stroke = Stroke::new(theme.spine_width);
    renderer.stroke_path(&border_path, &border_paint, &border_stroke, Affine::IDENTITY);

    // --- Generate ticks ---
    let tick_data = ticks::generate_ticks(
        colorbar.vmin,
        colorbar.vmax,
        COLORBAR_TICK_COUNT,
        &crate::scale::Scale::Linear,
    );

    let tick_y_start = rect.y + gradient_height;
    let tick_y_end = tick_y_start + TICK_LENGTH;
    let label_y = tick_y_end + TICK_LABEL_GAP;

    let tick_paint = Paint::new(theme.tick_color);
    let tick_stroke = Stroke::new(theme.spine_width);

    let label_style = TextStyle {
        size: theme.tick_label_size,
        color: theme.text_color,
        weight: FontWeight::Normal,
        family: theme.font_family.clone(),
        halign: HAlign::Center,
        valign: VAlign::Top,
    };

    let range = colorbar.vmax - colorbar.vmin;

    for tick in &tick_data {
        let t = if range.abs() < f64::EPSILON {
            0.5
        } else {
            (tick.value - colorbar.vmin) / range
        };
        let px = rect.x + rect.width * t;

        // Tick mark.
        let mut tick_path = Path::new();
        tick_path.move_to(px, tick_y_start);
        tick_path.line_to(px, tick_y_end);
        renderer.stroke_path(&tick_path, &tick_paint, &tick_stroke, Affine::IDENTITY);

        // Tick label.
        renderer.draw_text(
            &tick.label,
            Point::new(px, label_y),
            &label_style,
            Affine::IDENTITY,
        );
    }

    // --- Draw optional label ---
    if let Some(ref label_text) = colorbar.label {
        let cb_label_style = TextStyle {
            size: theme.axis_label_size,
            color: theme.text_color,
            weight: FontWeight::Normal,
            family: theme.font_family.clone(),
            halign: HAlign::Center,
            valign: VAlign::Top,
        };

        let label_x_pos = rect.x + rect.width / 2.0;
        let label_y_pos = label_y + theme.tick_label_size + LABEL_GAP;
        renderer.draw_text(
            label_text,
            Point::new(label_x_pos, label_y_pos),
            &cb_label_style,
            Affine::IDENTITY,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorbar_default_settings() {
        let cb = Colorbar::new(Colormap::Viridis, 0.0, 1.0);
        assert_eq!(cb.cmap, Colormap::Viridis);
        assert!((cb.vmin - 0.0).abs() < f64::EPSILON);
        assert!((cb.vmax - 1.0).abs() < f64::EPSILON);
        assert!(cb.label.is_none());
        assert_eq!(cb.orientation, ColorbarOrientation::Vertical);
        assert_eq!(cb.num_steps, DEFAULT_NUM_STEPS);
    }

    #[test]
    fn colorbar_with_label() {
        let cb = Colorbar::new(Colormap::Plasma, 0.0, 100.0)
            .label("Temperature");
        assert_eq!(cb.label.as_deref(), Some("Temperature"));
    }

    #[test]
    fn colorbar_set_label_mut() {
        let mut cb = Colorbar::new(Colormap::Plasma, 0.0, 100.0);
        cb.set_label("Pressure");
        assert_eq!(cb.label.as_deref(), Some("Pressure"));
    }

    #[test]
    fn colorbar_vertical_orientation() {
        let cb = Colorbar::new(Colormap::Viridis, 0.0, 1.0)
            .orientation(ColorbarOrientation::Vertical);
        assert_eq!(cb.orientation, ColorbarOrientation::Vertical);
    }

    #[test]
    fn colorbar_horizontal_orientation() {
        let cb = Colorbar::new(Colormap::Viridis, 0.0, 1.0)
            .orientation(ColorbarOrientation::Horizontal);
        assert_eq!(cb.orientation, ColorbarOrientation::Horizontal);
    }

    #[test]
    fn colorbar_custom_num_steps() {
        let cb = Colorbar::new(Colormap::Inferno, -1.0, 1.0)
            .num_steps(128);
        assert_eq!(cb.num_steps, 128);
    }

    #[test]
    fn colorbar_num_steps_clamped_to_min_2() {
        let cb = Colorbar::new(Colormap::Inferno, 0.0, 1.0)
            .num_steps(0);
        assert_eq!(cb.num_steps, 2);
    }

    #[test]
    fn colorbar_vmin_vmax_respected() {
        let cb = Colorbar::new(Colormap::Coolwarm, -50.0, 50.0);
        assert!((cb.vmin - (-50.0)).abs() < f64::EPSILON);
        assert!((cb.vmax - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn colorbar_builder_chaining() {
        let cb = Colorbar::new(Colormap::Turbo, 0.0, 10.0)
            .label("Speed")
            .orientation(ColorbarOrientation::Horizontal)
            .num_steps(64);
        assert_eq!(cb.cmap, Colormap::Turbo);
        assert_eq!(cb.label.as_deref(), Some("Speed"));
        assert_eq!(cb.orientation, ColorbarOrientation::Horizontal);
        assert_eq!(cb.num_steps, 64);
    }

    #[test]
    fn colorbar_different_colormaps() {
        let colormaps = [
            Colormap::Viridis,
            Colormap::Plasma,
            Colormap::Inferno,
            Colormap::Magma,
            Colormap::Cividis,
            Colormap::Turbo,
            Colormap::Coolwarm,
            Colormap::Blues,
        ];
        for &cmap in &colormaps {
            let cb = Colorbar::new(cmap, 0.0, 1.0);
            assert_eq!(cb.cmap, cmap);
        }
    }

    #[test]
    fn orientation_default_is_vertical() {
        let orientation = ColorbarOrientation::default();
        assert_eq!(orientation, ColorbarOrientation::Vertical);
    }

    #[test]
    fn colorbar_equal_vmin_vmax() {
        // Edge case: zero-width range should not panic.
        let cb = Colorbar::new(Colormap::Viridis, 5.0, 5.0);
        assert!((cb.vmin - cb.vmax).abs() < f64::EPSILON);
    }
}
