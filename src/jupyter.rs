//! Jupyter notebook display support via evcxr.
//!
//! Enable with the `jupyter` feature flag:
//! ```toml
//! plotkit = { version = "0.1", features = ["jupyter"] }
//! ```

use crate::Figure;
use base64::Engine;

/// Renders a figure inline in a Jupyter notebook via evcxr.
///
/// Outputs the figure as a base64-encoded PNG using evcxr's content protocol.
pub fn evcxr_display_figure(fig: &Figure) {
    let renderer = plotkit_render_skia::SkiaRenderer::new(fig.width(), fig.height());
    let bytes = fig.render_to(renderer);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    println!("EVCXR_BEGIN_CONTENT image/png\n{b64}\nEVCXR_END_CONTENT");
}

/// Displays the current pyplot-style figure inline in a Jupyter notebook.
pub fn show() {
    crate::CURRENT_FIGURE.with(|fig| {
        evcxr_display_figure(&fig.borrow());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figure_evcxr_display_does_not_panic() {
        let mut fig = Figure::new();
        let ax = fig.add_subplot(1, 1, 1);
        ax.plot(vec![1.0, 2.0, 3.0], vec![1.0, 4.0, 9.0]).unwrap();
        // Just verify it doesn't panic — output goes to stdout
        evcxr_display_figure(&fig);
    }

    #[test]
    fn png_bytes_produce_valid_base64() {
        let fig = Figure::new();
        let renderer = plotkit_render_skia::SkiaRenderer::new(fig.width(), fig.height());
        let bytes = fig.render_to(renderer);
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&b64)
            .unwrap();
        assert_eq!(bytes, decoded);
    }

    #[test]
    fn show_does_not_panic() {
        crate::clf();
        crate::CURRENT_FIGURE.with(|fig| {
            let mut fig = fig.borrow_mut();
            fig.add_subplot(1, 1, 1);
        });
        show();
    }
}
