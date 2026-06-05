//! Tests for SVG rendering in plotkit.
//!
//! Validates the FigureExt::to_svg_string() pipeline and the SvgRenderer
//! backend directly.

use plotkit::prelude::*;
use plotkit::FigureExt;

// ===========================================================================
// 1. End-to-end: FigureExt::to_svg_string()
// ===========================================================================

#[test]
fn svg_output_is_valid_svg_wrapper() {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y).unwrap();

    let svg = fig.to_svg_string().expect("to_svg_string failed");

    // Must start with <svg and end with </svg>
    let trimmed = svg.trim();
    assert!(
        trimmed.starts_with("<svg"),
        "SVG output must start with <svg, got: {}",
        &trimmed[..trimmed.len().min(80)]
    );
    assert!(
        trimmed.ends_with("</svg>"),
        "SVG output must end with </svg>, got: {}",
        &trimmed[trimmed.len().saturating_sub(80)..]
    );
}

#[test]
fn svg_contains_xmlns_and_viewbox() {
    let mut fig = Figure::with_size(640, 480);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&[0.0, 1.0, 2.0], &[0.0, 1.0, 0.5]).unwrap();

    let svg = fig.to_svg_string().unwrap();

    assert!(
        svg.contains("xmlns=\"http://www.w3.org/2000/svg\""),
        "SVG must contain xmlns declaration"
    );
    assert!(
        svg.contains("viewBox=\"0 0 640 480\""),
        "SVG must contain correct viewBox"
    );
    assert!(
        svg.contains("width=\"640\""),
        "SVG must contain width attribute"
    );
    assert!(
        svg.contains("height=\"480\""),
        "SVG must contain height attribute"
    );
}

#[test]
fn svg_contains_path_elements_for_plot_data() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&v| v * v).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y).unwrap();

    let svg = fig.to_svg_string().unwrap();

    // The rendered SVG should contain <path elements for the plot lines
    assert!(
        svg.contains("<path"),
        "SVG must contain <path elements for the plotted data"
    );

    // Should contain SVG path data commands (M for move, L for line)
    assert!(
        svg.contains(" d=\"M"),
        "SVG path elements must contain 'd' attribute with path data starting with M (moveto)"
    );
}

#[test]
fn svg_contains_text_elements_for_labels() {
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&[1.0, 2.0, 3.0], &[1.0, 4.0, 9.0]).unwrap();
    ax.set_title("Test Title");
    ax.set_xlabel("X Axis");
    ax.set_ylabel("Y Axis");

    let svg = fig.to_svg_string().unwrap();

    assert!(
        svg.contains("<text"),
        "SVG must contain <text elements for labels"
    );
    assert!(
        svg.contains("Test Title"),
        "SVG must contain the title text"
    );
    assert!(
        svg.contains("X Axis"),
        "SVG must contain the xlabel text"
    );
    assert!(
        svg.contains("Y Axis"),
        "SVG must contain the ylabel text"
    );
}

#[test]
fn svg_scatter_contains_circles_or_paths() {
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![2.0, 4.0, 1.0, 3.0, 5.0];

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.scatter(&x, &y).unwrap();

    let svg = fig.to_svg_string().unwrap();

    // Scatter points are rendered as circle paths (via Path::circle -> CurveTo elements)
    // so we expect multiple <path elements with curve commands (C)
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 5,
        "Scatter plot with 5 points should generate at least 5 path elements (got {})",
        path_count
    );
}

#[test]
fn svg_bar_chart_generates_filled_paths() {
    let cats = vec!["A", "B", "C"];
    let vals = vec![10.0, 20.0, 15.0];

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.bar(cats.as_slice(), &vals).unwrap();

    let svg = fig.to_svg_string().unwrap();

    // Bar charts produce filled rectangles (fill_path calls)
    // Check for filled paths (no stroke, have fill attribute)
    assert!(
        svg.contains("fill=\""),
        "Bar chart SVG must contain filled paths"
    );
}

#[test]
fn svg_multi_series_with_legend() {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.1).collect();
    let y1: Vec<f64> = x.iter().map(|&v| v.sin()).collect();
    let y2: Vec<f64> = x.iter().map(|&v| v.cos()).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y1).unwrap().label("sin(x)");
    ax.plot(&x, &y2).unwrap().label("cos(x)");
    ax.legend();

    let svg = fig.to_svg_string().unwrap();

    // Legend text should appear in the SVG
    assert!(svg.contains("sin(x)"), "SVG must contain legend label sin(x)");
    assert!(svg.contains("cos(x)"), "SVG must contain legend label cos(x)");
}

#[test]
fn svg_suptitle_appears_in_output() {
    let mut fig = Figure::with_size(800, 600);
    fig.suptitle("Overall Title");
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&[0.0, 1.0], &[0.0, 1.0]).unwrap();

    let svg = fig.to_svg_string().unwrap();

    assert!(
        svg.contains("Overall Title"),
        "SVG must contain the suptitle text"
    );
}

#[test]
fn svg_dark_theme_uses_dark_background() {
    let mut fig = Figure::with_size(800, 600);
    fig.set_theme(Theme::dark());
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&[0.0, 1.0], &[0.0, 1.0]).unwrap();

    let svg = fig.to_svg_string().unwrap();

    // Dark theme background is Color::rgb(0x1C, 0x1C, 0x1C) = #1c1c1c
    assert!(
        svg.contains("#1c1c1c"),
        "Dark theme SVG should contain the dark background color #1c1c1c"
    );
}

#[test]
fn svg_empty_axes_still_produces_valid_svg() {
    let mut fig = Figure::with_size(400, 300);
    let _ax = fig.add_subplot(1, 1, 1);
    // No data plotted

    let svg = fig.to_svg_string().unwrap();

    let trimmed = svg.trim();
    assert!(trimmed.starts_with("<svg"), "Empty axes SVG must start with <svg");
    assert!(trimmed.ends_with("</svg>"), "Empty axes SVG must end with </svg>");
}

#[test]
fn svg_fill_between_generates_filled_region() {
    let x: Vec<f64> = (0..30).map(|i| i as f64 * 0.1).collect();
    let y1: Vec<f64> = x.iter().map(|&v| v.sin() - 0.2).collect();
    let y2: Vec<f64> = x.iter().map(|&v| v.sin() + 0.2).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.fill_between(&x, &y1, &y2).unwrap();

    let svg = fig.to_svg_string().unwrap();

    // fill_between should produce a filled path
    assert!(
        svg.contains("<path") && svg.contains("fill=\""),
        "fill_between SVG must contain a filled path element"
    );
}

// ===========================================================================
// 2. SvgRenderer unit-level tests (via the public API)
// ===========================================================================

#[test]
fn svg_renderer_clip_path_elements() {
    // Clipping is used during axes rendering, so any plot should produce clip paths
    let mut fig = Figure::with_size(400, 300);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&[0.0, 1.0, 2.0], &[0.0, 1.0, 0.5]).unwrap();

    let svg = fig.to_svg_string().unwrap();

    // The renderer uses clipPath + <g clip-path="url(#clipN)"> for clipping
    assert!(
        svg.contains("<clipPath"),
        "SVG should contain <clipPath elements for axes clipping"
    );
    assert!(
        svg.contains("clip-path=\"url(#clip"),
        "SVG should contain clip-path references"
    );
}

#[test]
fn svg_stroke_attributes_present() {
    let mut fig = Figure::with_size(400, 300);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&[0.0, 1.0, 2.0, 3.0], &[0.0, 1.0, 0.5, 2.0]).unwrap();

    let svg = fig.to_svg_string().unwrap();

    // Line plots produce stroked paths
    assert!(
        svg.contains("stroke=\""),
        "Line plot SVG must contain stroke attributes"
    );
    assert!(
        svg.contains("stroke-width=\""),
        "Line plot SVG must contain stroke-width attributes"
    );
    assert!(
        svg.contains("stroke-linecap=\""),
        "Line plot SVG must contain stroke-linecap attributes"
    );
    assert!(
        svg.contains("stroke-linejoin=\""),
        "Line plot SVG must contain stroke-linejoin attributes"
    );
}

#[test]
fn svg_text_html_escaping() {
    // Test that special characters in titles/labels are properly escaped
    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&[0.0, 1.0], &[0.0, 1.0]).unwrap();
    ax.set_title("x < y & z > w");
    ax.set_xlabel("\"quotes\" & 'apostrophes'");

    let svg = fig.to_svg_string().unwrap();

    // The html_escape function should convert these:
    // & -> &amp;   < -> &lt;   > -> &gt;   " -> &quot;   ' -> &#x27;
    assert!(
        svg.contains("&lt;"),
        "SVG must escape < to &lt;"
    );
    assert!(
        svg.contains("&amp;"),
        "SVG must escape & to &amp;"
    );
    assert!(
        svg.contains("&gt;"),
        "SVG must escape > to &gt;"
    );
    assert!(
        svg.contains("&quot;"),
        "SVG must escape \" to &quot;"
    );
    assert!(
        svg.contains("&#x27;"),
        "SVG must escape ' to &#x27;"
    );

    // Make sure the raw unescaped characters do NOT appear in text content
    // (they will appear in SVG attribute syntax though, so we check specifically
    // within text content)
    assert!(
        !svg.contains(">x < y"),
        "Raw < in text content should be escaped"
    );
}

#[test]
fn svg_font_family_in_text_style() {
    // When a theme specifies a font family, it should appear in text elements
    let mut fig = Figure::with_size(800, 600);
    let mut theme = Theme::default();
    theme.font_family = Some("Arial".to_string());
    fig.set_theme(theme);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&[0.0, 1.0], &[0.0, 1.0]).unwrap();
    ax.set_title("Font Test");

    let svg = fig.to_svg_string().unwrap();

    assert!(
        svg.contains("font-family=\"Arial\""),
        "SVG text elements should include the font-family attribute when set"
    );
}

#[test]
fn svg_output_is_valid_utf8() {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y).unwrap();
    ax.set_title("Unicode test: degree sign");

    // to_svg_string() internally does String::from_utf8 -- if it succeeds, the
    // output is valid UTF-8
    let result = fig.to_svg_string();
    assert!(result.is_ok(), "SVG output must be valid UTF-8");
}

#[test]
fn svg_histogram_renders() {
    let data: Vec<f64> = (0..100).map(|i| (i as f64 * 0.31).sin() * 2.0 + 3.0).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.hist(&data, 10).unwrap();
    ax.set_title("Histogram SVG");

    let svg = fig.to_svg_string().unwrap();

    // Histogram bins are rendered as filled rectangles
    let trimmed = svg.trim();
    assert!(trimmed.starts_with("<svg"));
    assert!(trimmed.ends_with("</svg>"));
    // Should have multiple filled paths for the bins
    let fill_count = svg.matches("fill=\"#").count();
    assert!(
        fill_count >= 10,
        "Histogram with 10 bins should produce at least 10 filled elements (got {})",
        fill_count
    );
}

#[test]
fn svg_multiple_subplots() {
    let mut fig = Figure::with_size(800, 600);

    let ax1 = fig.add_subplot(1, 2, 1);
    ax1.plot(&[0.0, 1.0], &[0.0, 1.0]).unwrap();
    ax1.set_title("Left");

    let ax2 = fig.add_subplot(1, 2, 2);
    ax2.plot(&[0.0, 1.0], &[1.0, 0.0]).unwrap();
    ax2.set_title("Right");

    let svg = fig.to_svg_string().unwrap();

    assert!(svg.contains("Left"), "SVG must contain first subplot title");
    assert!(svg.contains("Right"), "SVG must contain second subplot title");

    // Two subplots should produce at least 2 clip regions
    let clip_count = svg.matches("<clipPath").count();
    assert!(
        clip_count >= 2,
        "Two subplots should produce at least 2 clipPath elements (got {})",
        clip_count
    );
}

#[test]
fn svg_output_is_reasonably_sized() {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.plot(&x, &y).unwrap();

    let svg = fig.to_svg_string().unwrap();

    // SVG should be non-trivial in size (more than just the wrapper)
    assert!(
        svg.len() > 500,
        "SVG output should be substantial (got {} bytes)",
        svg.len()
    );
    // But also shouldn't be absurdly large for a simple plot
    assert!(
        svg.len() < 1_000_000,
        "SVG output should not be excessively large (got {} bytes)",
        svg.len()
    );
}
