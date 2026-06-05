//! SVG rendering backend for plotkit.
//!
//! Produces SVG markup by translating plotkit primitives to SVG elements.

use plotkit_core::primitives::*;
use plotkit_core::renderer::Renderer;
use std::fmt::Write;

/// A renderer that produces SVG output as a string.
pub struct SvgRenderer {
    width: u32,
    height: u32,
    content: String,
    clip_id: usize,
}

impl SvgRenderer {
    /// Creates a new SVG renderer with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            content: String::with_capacity(4096),
            clip_id: 0,
        }
    }

    fn color_to_css(c: &Color) -> String {
        if c.a == 255 {
            format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
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

    fn path_to_svg_d(path: &Path) -> String {
        let mut d = String::new();
        for el in &path.elements {
            match *el {
                PathEl::MoveTo(p) => write!(d, "M{:.2} {:.2} ", p.x, p.y).unwrap(),
                PathEl::LineTo(p) => write!(d, "L{:.2} {:.2} ", p.x, p.y).unwrap(),
                PathEl::QuadTo(p1, p) => {
                    write!(d, "Q{:.2} {:.2} {:.2} {:.2} ", p1.x, p1.y, p.x, p.y).unwrap()
                }
                PathEl::CurveTo(p1, p2, p) => {
                    write!(
                        d,
                        "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} ",
                        p1.x, p1.y, p2.x, p2.y, p.x, p.y
                    )
                    .unwrap()
                }
                PathEl::ClosePath => d.push_str("Z "),
            }
        }
        d
    }
}

impl Renderer for SvgRenderer {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn fill_path(&mut self, path: &Path, paint: &Paint, _transform: Affine) {
        let d = Self::path_to_svg_d(path);
        let color = Self::color_to_css(&paint.color);
        writeln!(
            self.content,
            "<path d=\"{}\" fill=\"{}\"/>",
            d.trim_end(),
            color
        )
        .unwrap();
    }

    fn stroke_path(&mut self, path: &Path, paint: &Paint, stroke: &Stroke, _transform: Affine) {
        let d = Self::path_to_svg_d(path);
        let color = Self::color_to_css(&paint.color);

        let cap = match stroke.cap {
            StrokeCap::Butt => "butt",
            StrokeCap::Round => "round",
            StrokeCap::Square => "square",
        };

        let join = match stroke.join {
            StrokeJoin::Miter => "miter",
            StrokeJoin::Round => "round",
            StrokeJoin::Bevel => "bevel",
        };

        let mut attrs = format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"",
            d.trim_end(),
            color,
            stroke.width,
            cap,
            join,
        );

        if let Some(ref dash) = stroke.dash {
            let dash_str: Vec<String> =
                dash.dashes.iter().map(|v| format!("{:.2}", v)).collect();
            write!(attrs, " stroke-dasharray=\"{}\"", dash_str.join(",")).unwrap();
            if dash.offset != 0.0 {
                write!(attrs, " stroke-dashoffset=\"{:.2}\"", dash.offset).unwrap();
            }
        }

        attrs.push_str("/>\n");
        self.content.push_str(&attrs);
    }

    fn draw_text(&mut self, text: &str, pos: Point, style: &TextStyle, _transform: Affine) {
        let color = Self::color_to_css(&style.color);

        let weight = match style.weight {
            FontWeight::Normal => "normal",
            FontWeight::Bold => "bold",
        };

        let anchor = match style.halign {
            HAlign::Left => "start",
            HAlign::Center => "middle",
            HAlign::Right => "end",
        };

        let baseline = match style.valign {
            VAlign::Top => "text-before-edge",
            VAlign::Middle => "central",
            VAlign::Bottom => "text-after-edge",
            VAlign::Baseline => "auto",
        };

        let mut tag = format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" fill=\"{}\" font-size=\"{:.1}\" font-weight=\"{}\" text-anchor=\"{}\" dominant-baseline=\"{}\"",
            pos.x,
            pos.y,
            color,
            style.size,
            weight,
            anchor,
            baseline,
        );

        if let Some(ref family) = style.family {
            write!(tag, " font-family=\"{}\"", html_escape(family)).unwrap();
        }

        writeln!(tag, ">{}</text>", html_escape(text)).unwrap();
        self.content.push_str(&tag);
    }

    fn draw_image(&mut self, _img: &Image, _dst: Rect, _transform: Affine) {
        // Image embedding (e.g. base64 data URI) is not yet implemented.
    }

    fn push_clip(&mut self, path: &Path, _transform: Affine) {
        let id = self.clip_id;
        self.clip_id += 1;
        let d = Self::path_to_svg_d(path);
        write!(
            self.content,
            "<defs><clipPath id=\"clip{}\"><path d=\"{}\"/></clipPath></defs>\n<g clip-path=\"url(#clip{})\">\n",
            id,
            d.trim_end(),
            id,
        )
        .unwrap();
    }

    fn pop_clip(&mut self) {
        self.content.push_str("</g>\n");
    }

    fn measure_text(&self, text: &str, style: &TextStyle) -> (f64, f64) {
        // Approximate measurement: average character width is roughly 0.6 * font size.
        let width = text.len() as f64 * style.size * 0.6;
        let height = style.size;
        (width, height)
    }

    fn finalize(self) -> Vec<u8> {
        let mut svg = String::with_capacity(self.content.len() + 256);
        writeln!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
            self.width, self.height, self.width, self.height,
        )
        .unwrap();
        svg.push_str(&self.content);
        svg.push_str("</svg>\n");
        svg.into_bytes()
    }
}

/// Escapes special XML/HTML characters in a string so it can be safely
/// embedded in SVG element content or attribute values.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
