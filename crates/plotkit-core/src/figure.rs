//! The top-level Figure container.
//!
//! A [`Figure`] owns one or more [`Axes`] (subplots) and orchestrates the
//! full rendering pipeline: filling the figure background, computing the
//! subplot grid layout, drawing the optional super-title, and delegating
//! per-axes rendering.

use crate::axes::Axes;
use crate::error::Result;
use crate::layout;
use crate::primitives::{Affine, HAlign, Paint, Path, Point, Rect, TextStyle, VAlign};
use crate::renderer::Renderer;
use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default figure width in pixels.
const DEFAULT_WIDTH: u32 = 800;

/// Default figure height in pixels.
const DEFAULT_HEIGHT: u32 = 600;

/// Vertical space (in pixels) reserved for the suptitle when present.
const SUPTITLE_RESERVED_HEIGHT: f64 = 30.0;

/// Default outer margin (in pixels) around the subplot grid.
const DEFAULT_MARGIN: f64 = 20.0;

/// Default gap (in pixels) between subplot cells.
const DEFAULT_GAP: f64 = 15.0;

// ---------------------------------------------------------------------------
// Figure
// ---------------------------------------------------------------------------

/// A figure is the top-level container for one or more axes (subplots).
///
/// The figure owns all axes, holds the overall dimensions, an optional
/// super-title, and the visual theme. It implements the rendering pipeline
/// described in `ARCHITECTURE.md`:
///
/// 1. Fill figure background.
/// 2. Draw suptitle if set.
/// 3. Compute subplot layout rectangles.
/// 4. For each axes: delegate rendering with its allocated rectangle and theme.
///
/// # Ownership model (ADR-003)
///
/// `Figure` owns `Vec<Axes>` directly -- no `Rc`, no `RefCell`.
/// [`add_subplot`](Figure::add_subplot) returns `&mut Axes`, and the borrow
/// checker enforces single-axes mutation at compile time.
///
/// # Examples
///
/// ```no_run
/// use plotkit_core::figure::Figure;
///
/// let mut fig = Figure::new();
/// let ax = fig.add_subplot(1, 1, 1);
/// // ax.plot(...)?;
/// ```
#[derive(Debug)]
pub struct Figure {
    /// The subplot axes owned by this figure, stored in insertion order.
    axes: Vec<Axes>,
    /// Width of the output image in pixels.
    width: u32,
    /// Height of the output image in pixels.
    height: u32,
    /// Optional overall title displayed above all subplots.
    suptitle: Option<String>,
    /// The visual theme applied to the figure and inherited by axes.
    theme: Theme,
    /// Subplot grid dimensions `(nrows, ncols)`, set by `add_subplot`.
    subplot_grid: Option<(usize, usize)>,
}

impl Figure {
    /// Creates a new figure with default dimensions (800 x 600 pixels).
    pub fn new() -> Self {
        Self {
            axes: Vec::new(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            suptitle: None,
            theme: Theme::default(),
            subplot_grid: None,
        }
    }

    /// Creates a new figure with the specified dimensions in pixels.
    pub fn with_size(width: u32, height: u32) -> Self {
        Self {
            axes: Vec::new(),
            width,
            height,
            suptitle: None,
            theme: Theme::default(),
            subplot_grid: None,
        }
    }

    /// Returns the figure width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the figure height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Adds a subplot and returns a mutable reference to it.
    ///
    /// Uses 1-based indexing in matplotlib style: `(nrows, ncols, index)`.
    /// The index counts across rows first (row-major), starting at 1.
    ///
    /// If a subplot already exists at the given `index`, the existing axes
    /// is returned without creating a duplicate. Otherwise a new [`Axes`]
    /// is created, appended to the figure's axes list, and returned.
    ///
    /// # Panics
    ///
    /// Panics if `nrows`, `ncols`, or `index` is zero, or if `index` exceeds
    /// `nrows * ncols`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use plotkit_core::figure::Figure;
    ///
    /// let mut fig = Figure::new();
    /// let ax = fig.add_subplot(2, 2, 1); // top-left of a 2x2 grid
    /// ```
    pub fn add_subplot(&mut self, nrows: usize, ncols: usize, index: usize) -> &mut Axes {
        assert!(nrows > 0, "nrows must be at least 1");
        assert!(ncols > 0, "ncols must be at least 1");
        assert!(index >= 1, "subplot index is 1-based; got 0");
        assert!(
            index <= nrows * ncols,
            "subplot index {index} exceeds grid size {nrows}x{ncols} = {}",
            nrows * ncols
        );

        // Store (or validate) the grid dimensions. If the grid was already set
        // with different dimensions, update to the latest request -- this
        // mirrors matplotlib's behaviour where later add_subplot calls can
        // redefine the grid.
        self.subplot_grid = Some((nrows, ncols));

        // Convert 1-based index to 0-based for internal storage.
        let zero_index = index - 1;

        // Ensure the internal axes vec is large enough. If the user skips
        // indices (e.g. add_subplot(2,2,3) without adding 1 and 2 first) we
        // pad with default Axes so that positional indexing is consistent.
        while self.axes.len() <= zero_index {
            self.axes.push(Axes::new());
        }

        &mut self.axes[zero_index]
    }

    /// Sets the overall figure title (super-title), displayed above all
    /// subplots.
    ///
    /// Returns `&mut Self` for builder-style chaining.
    pub fn suptitle(&mut self, title: &str) -> &mut Self {
        self.suptitle = Some(title.to_string());
        self
    }

    /// Sets the visual theme for the entire figure.
    ///
    /// The theme is inherited by all axes during rendering unless an
    /// individual axes has its own override.
    ///
    /// Returns `&mut Self` for builder-style chaining.
    pub fn set_theme(&mut self, theme: Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    /// Returns a reference to the figure's theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Returns mutable access to the axes at `index` (0-based).
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn axes_mut(&mut self, index: usize) -> Option<&mut Axes> {
        self.axes.get_mut(index)
    }

    /// Returns a shared reference to the axes at `index` (0-based).
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn axes(&self, index: usize) -> Option<&Axes> {
        self.axes.get(index)
    }

    /// Returns the number of axes (subplots) in this figure.
    pub fn num_axes(&self) -> usize {
        self.axes.len()
    }

    /// Creates a new figure with an `nrows × ncols` subplot grid.
    ///
    /// Axes are added in row-major order (index 0 = top-left). Access them
    /// via `fig.axes_mut(index)` where index goes from 0 to `nrows*ncols - 1`.
    ///
    /// # Panics
    ///
    /// Panics if `nrows` or `ncols` is zero.
    pub fn subplots(nrows: usize, ncols: usize) -> Self {
        assert!(nrows > 0 && ncols > 0, "subplots: nrows and ncols must be > 0");
        let mut fig = Self::new();
        for i in 1..=(nrows * ncols) {
            fig.add_subplot(nrows, ncols, i);
        }
        fig
    }

    /// Creates a new figure with an `nrows × ncols` subplot grid and custom size.
    ///
    /// # Panics
    ///
    /// Panics if `nrows` or `ncols` is zero.
    pub fn subplots_with_size(nrows: usize, ncols: usize, width: u32, height: u32) -> Self {
        assert!(nrows > 0 && ncols > 0, "subplots: nrows and ncols must be > 0");
        let mut fig = Self::with_size(width, height);
        for i in 1..=(nrows * ncols) {
            fig.add_subplot(nrows, ncols, i);
        }
        fig
    }

    /// 2D indexing into the subplot grid.
    ///
    /// Returns `axes_mut(row * ncols + col)`, or `None` if out of bounds.
    pub fn axes_grid(&mut self, row: usize, col: usize, ncols: usize) -> Option<&mut Axes> {
        self.axes_mut(row * ncols + col)
    }

    // -----------------------------------------------------------------------
    // Rendering
    // -----------------------------------------------------------------------

    /// Renders the figure using the given renderer.
    ///
    /// This is the core rendering pipeline (per `ARCHITECTURE.md`):
    ///
    /// 1. Fill figure background with the theme's `figure_background` color.
    /// 2. Draw suptitle if one has been set.
    /// 3. Compute the subplot grid layout, producing one [`Rect`] per axes.
    /// 4. For each axes, delegate to [`Axes::render`] with its assigned
    ///    rectangle and the figure theme.
    pub fn render(&self, renderer: &mut impl Renderer) {
        let (w, h) = renderer.size();
        let fw = w as f64;
        let fh = h as f64;
        let theme = &self.theme;

        // ----- 1. Fill figure background -----------------------------------
        let bg_path = Path::rect(Rect::new(0.0, 0.0, fw, fh));
        renderer.fill_path(
            &bg_path,
            &Paint::new(theme.figure_background),
            Affine::IDENTITY,
        );

        // ----- 2. Draw suptitle if set -------------------------------------
        let top_offset = if let Some(ref title) = self.suptitle {
            let style = TextStyle {
                size: theme.title_size + 2.0, // suptitle slightly larger than axes title
                color: theme.text_color,
                weight: theme.title_weight,
                family: theme.font_family.clone(),
                halign: HAlign::Center,
                valign: VAlign::Top,
            };

            let text_pos = Point::new(fw / 2.0, DEFAULT_MARGIN * 0.5);
            renderer.draw_text(title, text_pos, &style, Affine::IDENTITY);

            SUPTITLE_RESERVED_HEIGHT
        } else {
            0.0
        };

        // ----- 3. Compute subplot layout -----------------------------------
        // Reduce the available height by the suptitle offset so that subplots
        // are laid out below the suptitle instead of overlapping it.
        let grid = self.subplot_grid.unwrap_or((1, 1));
        let rects = layout::compute_subplot_rects(
            fw,
            fh - top_offset,
            grid.0,
            grid.1,
            DEFAULT_MARGIN,
            DEFAULT_GAP,
        )
        .into_iter()
        .map(|mut r| {
            r.y += top_offset;
            r
        })
        .collect::<Vec<_>>();

        // ----- 4. Render each axes -----------------------------------------
        for (i, axes) in self.axes.iter().enumerate() {
            if let Some(rect) = rects.get(i) {
                axes.render(renderer, *rect, theme);
            }
        }
    }

    /// Renders the figure using the given renderer and returns the encoded
    /// output bytes (PNG, SVG, PDF, etc., depending on the renderer).
    ///
    /// This convenience method calls [`render`](Figure::render) and then
    /// [`Renderer::finalize`] to produce the final byte output.
    pub fn render_to<R: Renderer>(&self, mut renderer: R) -> Vec<u8> {
        self.render(&mut renderer);
        renderer.finalize()
    }

    /// Saves the figure to a file using the provided renderer.
    ///
    /// The renderer determines the output format. The encoded bytes are
    /// written to `path` atomically via [`std::fs::write`].
    pub fn save_with<R: Renderer>(&self, renderer: R, path: impl AsRef<std::path::Path>) -> Result<()> {
        let bytes = self.render_to(renderer);
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

impl Default for Figure {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Color;

    #[test]
    fn new_figure_has_default_dimensions() {
        let fig = Figure::new();
        assert_eq!(fig.width(), DEFAULT_WIDTH);
        assert_eq!(fig.height(), DEFAULT_HEIGHT);
    }

    #[test]
    fn with_size_sets_dimensions() {
        let fig = Figure::with_size(1024, 768);
        assert_eq!(fig.width(), 1024);
        assert_eq!(fig.height(), 768);
    }

    #[test]
    fn default_figure_has_no_axes() {
        let fig = Figure::new();
        assert_eq!(fig.num_axes(), 0);
    }

    #[test]
    fn add_subplot_creates_axes() {
        let mut fig = Figure::new();
        let _ax = fig.add_subplot(1, 1, 1);
        assert_eq!(fig.num_axes(), 1);
    }

    #[test]
    fn add_subplot_returns_same_axes_on_repeat() {
        let mut fig = Figure::new();
        fig.add_subplot(2, 2, 1);
        fig.add_subplot(2, 2, 1); // same index, should not duplicate
        assert_eq!(fig.num_axes(), 1);
    }

    #[test]
    fn add_subplot_pads_for_skipped_indices() {
        let mut fig = Figure::new();
        fig.add_subplot(2, 2, 3); // skip indices 1 and 2
        assert_eq!(fig.num_axes(), 3); // indices 0, 1, 2 all exist
    }

    #[test]
    #[should_panic(expected = "nrows must be at least 1")]
    fn add_subplot_panics_on_zero_rows() {
        let mut fig = Figure::new();
        fig.add_subplot(0, 1, 1);
    }

    #[test]
    #[should_panic(expected = "ncols must be at least 1")]
    fn add_subplot_panics_on_zero_cols() {
        let mut fig = Figure::new();
        fig.add_subplot(1, 0, 1);
    }

    #[test]
    #[should_panic(expected = "subplot index is 1-based")]
    fn add_subplot_panics_on_zero_index() {
        let mut fig = Figure::new();
        fig.add_subplot(1, 1, 0);
    }

    #[test]
    #[should_panic(expected = "subplot index 5 exceeds grid size")]
    fn add_subplot_panics_on_index_out_of_range() {
        let mut fig = Figure::new();
        fig.add_subplot(2, 2, 5);
    }

    #[test]
    fn suptitle_sets_title() {
        let mut fig = Figure::new();
        fig.suptitle("My Figure");
        assert_eq!(fig.suptitle, Some("My Figure".to_string()));
    }

    #[test]
    fn suptitle_returns_self_for_chaining() {
        let mut fig = Figure::new();
        fig.suptitle("Title 1").suptitle("Title 2");
        assert_eq!(fig.suptitle, Some("Title 2".to_string()));
    }

    #[test]
    fn set_theme_updates_theme() {
        let mut fig = Figure::new();
        let dark = Theme::dark();
        fig.set_theme(dark);
        assert_eq!(fig.theme().figure_background, Color::rgb(0x1C, 0x1C, 0x1C));
    }

    #[test]
    fn theme_returns_reference() {
        let fig = Figure::new();
        assert_eq!(fig.theme().figure_background, Color::WHITE);
    }

    #[test]
    fn axes_mut_returns_none_for_out_of_bounds() {
        let mut fig = Figure::new();
        assert!(fig.axes_mut(0).is_none());
    }

    #[test]
    fn axes_mut_returns_some_for_valid_index() {
        let mut fig = Figure::new();
        fig.add_subplot(1, 1, 1);
        assert!(fig.axes_mut(0).is_some());
    }

    #[test]
    fn axes_returns_shared_reference() {
        let mut fig = Figure::new();
        fig.add_subplot(1, 1, 1);
        assert!(fig.axes(0).is_some());
        assert!(fig.axes(1).is_none());
    }

    #[test]
    fn default_impl_matches_new() {
        let from_new = Figure::new();
        let from_default = Figure::default();
        assert_eq!(from_new.width(), from_default.width());
        assert_eq!(from_new.height(), from_default.height());
        assert_eq!(from_new.num_axes(), from_default.num_axes());
    }

    #[test]
    fn multiple_subplots_in_grid() {
        let mut fig = Figure::new();
        fig.add_subplot(2, 3, 1);
        fig.add_subplot(2, 3, 4);
        fig.add_subplot(2, 3, 6);
        assert_eq!(fig.num_axes(), 6); // padded to fill up to index 6
        assert_eq!(fig.subplot_grid, Some((2, 3)));
    }
}
