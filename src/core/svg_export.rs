use crate::core::color::Color;
use crate::core::document::Document;
use crate::core::element::{
    BlendMode, Element, FillLayer, FillStyle, Gradient, GradientType, PathElement, PathNode, PatternType,
    StrokeStyle, TextAlign,
};
use crate::core::geometry::Rect;
use crate::core::page::{Page, PageId};
use crate::core::ruler::Guide;
use std::fmt::Write;

/// Metadata structure stored inside the SVG <metadata> for 100% lossless project reload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentProjectData {
    pub format_version: String,
    pub elements: Vec<Element>,
    pub pages: Vec<Page>,
    pub active_page_id: Option<PageId>,
    pub guides: Vec<Guide>,
}

impl DocumentProjectData {
    pub fn from_document(doc: &Document) -> Self {
        Self {
            format_version: "1.0".to_string(),
            elements: doc.elements.clone(),
            pages: doc.pages.clone(),
            active_page_id: doc.active_page_id,
            guides: doc.guides.clone(),
        }
    }

    pub fn to_document(self) -> Document {
        let mut doc = Document::default();
        doc.elements = self.elements;
        doc.pages = if self.pages.is_empty() {
            vec![Page::default_a4()]
        } else {
            self.pages
        };
        doc.active_page_id = self
            .active_page_id
            .or_else(|| doc.pages.first().map(|p| p.id));
        doc.guides = self.guides;
        doc
    }
}

/// Simple standard Base64 encoder for embedding raster images in SVG
pub fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        result.push(CHARSET[(b0 >> 2) as usize] as char);
        result.push(CHARSET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARSET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARSET[(b2 & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Escapes XML characters in text content
pub fn xml_escape(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            _ => output.push(c),
        }
    }
    output
}

/// Formats a Color into CSS hex or rgba string
pub fn color_to_svg(color: Color) -> String {
    if color.a >= 0.999 {
        color.to_hex()
    } else {
        format!(
            "rgba({}, {}, {}, {:.3})",
            (color.r * 255.0).round() as u8,
            (color.g * 255.0).round() as u8,
            (color.b * 255.0).round() as u8,
            color.a
        )
    }
}

/// Generates a full SVG document representing the GNOME Paths work
pub fn export_document_to_svg(doc: &Document) -> String {
    let page_rect = doc
        .active_page()
        .map(|p| p.rect.normalize())
        .unwrap_or_else(|| Rect::new(0.0, 0.0, 794.0, 1123.0));

    let width = page_rect.width;
    let height = page_rect.height;
    let view_box_x = page_rect.x;
    let view_box_y = page_rect.y;

    let mut svg = String::new();
    let _ = writeln!(
        svg,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
     xmlns:gnomepaths="https://gitlab.com/lewisHeart/gnome-paths"
     width="{:.2}" height="{:.2}" viewBox="{:.2} {:.2} {:.2} {:.2}" version="1.1">"#,
        width, height, view_box_x, view_box_y, width, height
    );

    // 1. Lossless Project Metadata
    let project_data = DocumentProjectData::from_document(doc);
    if let Ok(json_str) = serde_json::to_string(&project_data) {
        let _ = writeln!(svg, "  <metadata id=\"gnome-paths-metadata\">");
        let _ = writeln!(
            svg,
            "    <gnomepaths:project version=\"1.0\"><![CDATA[{}]]></gnomepaths:project>",
            json_str
        );
        let _ = writeln!(svg, "  </metadata>");
    }

    // 2. Collect and emit <defs> (gradients, patterns, filters)
    let mut defs = String::new();
    let mut def_count = 0;
    collect_defs_for_elements(&doc.elements, &mut defs, &mut def_count);

    if !defs.is_empty() {
        let _ = writeln!(svg, "  <defs>");
        svg.push_str(&defs);
        let _ = writeln!(svg, "  </defs>");
    }

    // 3. Render all visible elements
    for element in &doc.elements {
        if element.visible() {
            render_element_to_svg(element, &mut svg, 1);
        }
    }

    let _ = writeln!(svg, "</svg>");
    svg
}

fn collect_defs_for_elements(elements: &[Element], defs: &mut String, count: &mut usize) {
    for elem in elements {
        match elem {
            Element::Rect(r) => {
                if let Some(grad) = &r.gradient {
                    *count += 1;
                    render_gradient_def(grad, &format!("grad_{}", r.id.0), defs);
                }
                if r.blur > 0.001 {
                    render_blur_filter_def(r.blur, &format!("blur_{}", r.id.0), defs);
                }
                for (idx, fill) in r.fills.iter().enumerate() {
                    if fill.style == FillStyle::Pattern {
                        render_pattern_def(fill, &format!("pat_{}_{}", r.id.0, idx), defs);
                    }
                }
            }
            Element::Path(p) => {
                if let Some(grad) = &p.gradient {
                    *count += 1;
                    render_gradient_def(grad, &format!("grad_{}", p.id.0), defs);
                }
                if p.blur > 0.001 {
                    render_blur_filter_def(p.blur, &format!("blur_{}", p.id.0), defs);
                }
                for (idx, fill) in p.fills.iter().enumerate() {
                    if fill.style == FillStyle::Pattern {
                        render_pattern_def(fill, &format!("pat_{}_{}", p.id.0, idx), defs);
                    }
                }
            }
            Element::Brush(b) => {
                if b.blur > 0.001 {
                    render_blur_filter_def(b.blur, &format!("blur_{}", b.id.0), defs);
                }
            }
            Element::Text(t) => {
                if t.blur > 0.001 {
                    render_blur_filter_def(t.blur, &format!("blur_{}", t.id.0), defs);
                }
            }
            Element::Group(g) => {
                if g.blur > 0.001 {
                    render_blur_filter_def(g.blur, &format!("blur_{}", g.id.0), defs);
                }
                if let Some(clip) = &g.clip_element {
                    render_clip_path_def(clip, &format!("clip_{}", g.id.0), defs);
                }
                collect_defs_for_elements(&g.children, defs, count);
            }
            Element::Image(i) => {
                if i.blur > 0.001 {
                    render_blur_filter_def(i.blur, &format!("blur_{}", i.id.0), defs);
                }
            }
            Element::Clone(c) => {
                if c.blur_radius > 0.001 {
                    render_blur_filter_def(c.blur_radius, &format!("blur_{}", c.id.0), defs);
                }
            }
        }
    }
}

fn render_gradient_def(grad: &Gradient, id: &str, defs: &mut String) {
    match grad.kind {
        GradientType::Linear => {
            let _ = writeln!(
                defs,
                "    <linearGradient id=\"{}\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" gradientUnits=\"userSpaceOnUse\">",
                id, grad.start.x, grad.start.y, grad.end.x, grad.end.y
            );
            for stop in &grad.stops {
                let _ = writeln!(
                    defs,
                    "      <stop offset=\"{:.3}\" stop-color=\"{}\" stop-opacity=\"{:.3}\"/>",
                    stop.offset,
                    stop.color.to_hex(),
                    stop.color.a
                );
            }
            let _ = writeln!(defs, "    </linearGradient>");
        }
        GradientType::Radial => {
            let r = grad.start.distance_to(grad.end).max(1.0);
            let _ = writeln!(
                defs,
                "    <radialGradient id=\"{}\" cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" gradientUnits=\"userSpaceOnUse\">",
                id, grad.start.x, grad.start.y, r
            );
            for stop in &grad.stops {
                let _ = writeln!(
                    defs,
                    "      <stop offset=\"{:.3}\" stop-color=\"{}\" stop-opacity=\"{:.3}\"/>",
                    stop.offset,
                    stop.color.to_hex(),
                    stop.color.a
                );
            }
            let _ = writeln!(defs, "    </radialGradient>");
        }
    }
}

fn render_blur_filter_def(blur: f32, id: &str, defs: &mut String) {
    let std_dev = blur * 40.0;
    let _ = writeln!(
        defs,
        "    <filter id=\"{}\" x=\"-50%\" y=\"-50%\" width=\"200%\" height=\"200%\">\n      <feGaussianBlur stdDeviation=\"{:.2}\"/>\n    </filter>",
        id, std_dev
    );
}

fn render_pattern_def(fill: &FillLayer, id: &str, defs: &mut String) {
    let sz = fill.pattern_scale.clamp(4.0, 256.0);
    let (tile_w, tile_h) = match fill.pattern_type {
        PatternType::Hexagon => (sz, (sz * 1.7320508).max(4.0)),
        PatternType::Brick | PatternType::Scales => (sz, (sz * 0.5).max(4.0)),
            _ => (sz, sz),
    };
    let c1 = color_to_svg(fill.color);
    let c2 = color_to_svg(fill.secondary_color);

    let mut transform_attrs = String::new();
    let has_trans = fill.pattern_offset.x.abs() > 0.001 || fill.pattern_offset.y.abs() > 0.001;
    let has_rot = fill.angle.abs() > 0.01;
    if has_trans && has_rot {
        transform_attrs = format!(" patternTransform=\"translate({:.2} {:.2}) rotate({:.2})\"", fill.pattern_offset.x, fill.pattern_offset.y, fill.angle);
    } else if has_trans {
        transform_attrs = format!(" patternTransform=\"translate({:.2} {:.2})\"", fill.pattern_offset.x, fill.pattern_offset.y);
    } else if has_rot {
        transform_attrs = format!(" patternTransform=\"rotate({:.2})\"", fill.angle);
    }

    let _ = writeln!(
        defs,
        "    <pattern id=\"{}\" width=\"{:.2}\" height=\"{:.2}\" patternUnits=\"userSpaceOnUse\"{}>",
        id, tile_w, tile_h, transform_attrs
    );
    let _ = writeln!(
        defs,
        "      <rect width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>",
        tile_w, tile_h, c1
    );

    match fill.pattern_type {
        PatternType::Checkerboard => {
            let half = sz * 0.5;
            let _ = writeln!(
                defs,
                "      <rect x=\"0\" y=\"0\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>",
                half, half, c2
            );
            let _ = writeln!(
                defs,
                "      <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>",
                half, half, half, half, c2
            );
        }
        PatternType::Dots => {
            let r = sz * 0.22;
            let _ = writeln!(
                defs,
                "      <circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{}\"/>",
                sz * 0.5,
                sz * 0.5,
                r,
                c2
            );
        }
        PatternType::Stripes => {
            let w = (sz * 0.35).max(1.0);
            let _ = writeln!(
                defs,
                "      <line x1=\"0\" y1=\"0\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n      <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n      <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                sz, sz, c2, w,
                -sz * 0.5, sz * 0.5, sz * 0.5, sz * 1.5, c2, w,
                sz * 0.5, -sz * 0.5, sz * 1.5, sz * 0.5, c2, w
            );
        }
        PatternType::Grid => {
            let w = (sz * 0.12).max(1.0);
            let _ = writeln!(
                defs,
                "      <rect x=\"0\" y=\"0\" width=\"{:.2}\" height=\"{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                sz, sz, c2, w
            );
        }
        PatternType::Hexagon => {
            let w = tile_w;
            let h = tile_h;
            let half_w = w * 0.5;
            let h_6 = h / 6.0;
            let h_2 = h * 0.5;
            let h_23 = h * (2.0 / 3.0);
            let stroke_w = (sz * 0.08).max(1.0);
            let _ = writeln!(
                defs,
                "      <path d=\"M 0 0 L {:.2} {:.2} L {:.2} {:.2} L 0 {:.2} L 0 {:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                half_w, h_6, half_w, h_2, h_23, h, c2, stroke_w
            );
            let _ = writeln!(
                defs,
                "      <path d=\"M {:.2} 0 L {:.2} {:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                w, half_w, h_6, c2, stroke_w
            );
            let _ = writeln!(
                defs,
                "      <path d=\"M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                half_w, h_2, w, h_23, w, h, c2, stroke_w
            );
        }
        PatternType::Brick => {
            let w = tile_w;
            let h = tile_h;
            let half_h = h * 0.5;
            let half_w = w * 0.5;
            let stroke_w = (sz * 0.08).max(1.0);
            let _ = writeln!(
                defs,
                "      <line x1=\"0\" y1=\"0\" x2=\"{:.2}\" y2=\"0\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n      <line x1=\"0\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                w, c2, stroke_w,
                half_h, w, half_h, c2, stroke_w
            );
            let _ = writeln!(
                defs,
                "      <line x1=\"0\" y1=\"0\" x2=\"0\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n      <line x1=\"{:.2}\" y1=\"0\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                half_h, c2, stroke_w,
                w, w, half_h, c2, stroke_w
            );
            let _ = writeln!(
                defs,
                "      <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                half_w, half_h, half_w, h, c2, stroke_w
            );
        }
        PatternType::Crosshatch => {
            let w = (sz * 0.12).max(1.0);
            let _ = writeln!(
                defs,
                "      <line x1=\"0\" y1=\"0\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>\n      <line x1=\"0\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"0\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                sz, sz, c2, w,
                sz, sz, c2, w
            );
        }
        PatternType::Scales => {
            let w = tile_w;
            let h = tile_h;
            let stroke_w = (sz * 0.07).max(1.0);
            let r_base = w * 0.5;
            let radii = [r_base, r_base * 0.70, r_base * 0.40];
            for &r in &radii {
                let _ = writeln!(
                    defs,
                    "      <path d=\"M {:.2} {:.2} A {:.2} {:.2} 0 0 1 {:.2} {:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                    (w * 0.5) - r, h, r, r, (w * 0.5) + r, h, c2, stroke_w
                );
                let _ = writeln!(
                    defs,
                    "      <path d=\"M {:.2} 0 A {:.2} {:.2} 0 0 1 {:.2} 0\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                    -r, r, r, r, c2, stroke_w
                );
                let _ = writeln!(
                    defs,
                    "      <path d=\"M {:.2} 0 A {:.2} {:.2} 0 0 1 {:.2} 0\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                    w - r, r, r, w + r, c2, stroke_w
                );
            }
        }
        PatternType::Houndstooth => {
            let half = sz * 0.5;
            let _ = writeln!(
                defs,
                "      <polygon points=\"0,0 {:.2},0 {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} 0,{:.2}\" fill=\"{}\"/>",
                half, sz, half, half, half, half, sz, half, c2
            );
        }
        PatternType::Basketweave => {
            let half = sz * 0.5;
            let w = (sz * 0.15).max(1.0);
            let _ = writeln!(
                defs,
                "      <line x1=\"0\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/><line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/><line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/><line x1=\"{:.2}\" y1=\"0\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                half * 0.5, half, half * 0.5, c2, w, half * 0.5, half, half * 0.5, sz, c2, w, half, sz * 0.75, sz, sz * 0.75, c2, w, sz * 0.75, sz * 0.75, half, c2, w
            );
        }
        PatternType::Custom => {
            let mut custom_drawn = false;
            if let Some(ref path_str) = fill.custom_pattern_path {
                let p = std::path::Path::new(path_str);
                if p.exists() {
                    let ext = p.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()).unwrap_or_default();
                    if ext == "svg" {
                        if let Ok(svg_content) = std::fs::read_to_string(p) {
                            if let Ok(imported) = crate::core::svg_import::parse_svg(&svg_content) {
                                for elem in &imported.elements {
                                    match elem {
                                        crate::core::Element::Path(pe) => {
                                            let d = path_nodes_to_svg_d(&pe.nodes, pe.is_closed);
                                            let col = pe.fill_color.map(color_to_svg).unwrap_or_else(|| c2.clone());
                                            let _ = writeln!(defs, "      <path d=\"{}\" fill=\"{}\"/>", d, col);
                                        }
                                        _ => {}
                                    }
                                }
                                custom_drawn = true;
                            }
                        }
                    }
                }
            }
            if !custom_drawn {
                let half = sz * 0.5;
                let stroke_w = (sz * 0.1).max(1.0);
                let _ = writeln!(
                    defs,
                    "      <polygon points=\"{:.2},0 {:.2},{:.2} {:.2},{:.2} 0,{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\"/>",
                    half, sz, half, half, sz, half, c2, stroke_w
                );
            }
        }
    }
    let _ = writeln!(defs, "    </pattern>");
}

fn render_clip_path_def(clip: &Element, id: &str, defs: &mut String) {
    let _ = writeln!(defs, "    <clipPath id=\"{}\">", id);
    match clip {
        Element::Rect(r) => {
            let radii = r.effective_radii();
            if r.corner_style != crate::core::CornerStyle::Round || !radii.is_uniform() {
                let p = r.to_path_element();
                let d = path_nodes_to_svg_d(&p.nodes, p.is_closed);
                let _ = writeln!(defs, "      <path d=\"{}\"/>", d);
            } else {
                let norm = r.rect.normalize();
                let rad = radii.max_radius();
                let _ = writeln!(
                    defs,
                    "      <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\"/>",
                    norm.x, norm.y, norm.width, norm.height, rad, rad
                );
            }
        }
        Element::Path(p) => {
            let d = path_element_to_svg_d(p);
            let _ = writeln!(defs, "      <path d=\"{}\"/>", d);
        }
        _ => {}
    }
    let _ = writeln!(defs, "    </clipPath>");
}

fn render_element_to_svg(elem: &Element, svg: &mut String, indent_level: usize) {
    let indent = "  ".repeat(indent_level);

    match elem {
        Element::Rect(r) => {
            let radii = r.effective_radii();
            if r.corner_style != crate::core::CornerStyle::Round || !radii.is_uniform() {
                let p = r.to_path_element();
                render_element_to_svg(&Element::Path(p), svg, indent_level);
                return;
            }

            let norm = r.rect.normalize();
            let rad = radii.max_radius();
            let mut fill_str = "none".to_string();
            if let Some(_grad) = &r.gradient {
                fill_str = format!("url(#grad_{})", r.id.0);
            } else if let Some(first_fill) = r.fills.iter().find(|f| f.enabled) {
                if first_fill.style == FillStyle::Pattern {
                    fill_str = format!("url(#pat_{}_0)", r.id.0);
                } else {
                    fill_str = color_to_svg(first_fill.color);
                }
            } else if let Some(c) = r.fill_color {
                fill_str = color_to_svg(c);
            }

            let mut stroke_str = "none".to_string();
            let mut stroke_w = 0.0f32;
            let mut stroke_dash = "";
            if let Some(first_stroke) = r.strokes.iter().find(|s| s.enabled) {
                stroke_str = color_to_svg(first_stroke.color);
                stroke_w = first_stroke.width;
                stroke_dash = match first_stroke.style {
                    StrokeStyle::Solid => "",
                    StrokeStyle::Dashed => "stroke-dasharray=\"6,6\" ",
                    StrokeStyle::Dotted => "stroke-dasharray=\"2,4\" ",
                };
            } else if let Some(c) = r.stroke_color {
                stroke_str = color_to_svg(c);
                stroke_w = r.stroke_width;
            }

            let filter_attr = if r.blur > 0.001 {
                format!("filter=\"url(#blur_{})\" ", r.id.0)
            } else {
                String::new()
            };

            let blend_attr = blend_mode_to_svg(r.blend_mode);
            let id_attr = r
                .name
                .as_ref()
                .map(|n| format!("id=\"{}\" ", xml_escape(n)))
                .unwrap_or_else(|| format!("id=\"elem_{}\" ", r.id.0));

            let _ = writeln!(
                svg,
                "{indent}<rect {id_attr}x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{:.2}\" {stroke_dash}opacity=\"{:.3}\" {filter_attr}{blend_attr}/>",
                norm.x, norm.y, norm.width, norm.height, rad, rad, fill_str, stroke_str, stroke_w, r.opacity
            );
        }
        Element::Path(p) => {
            let d = path_element_to_svg_d(p);
            let mut fill_str = "none".to_string();
            if let Some(_grad) = &p.gradient {
                fill_str = format!("url(#grad_{})", p.id.0);
            } else if let Some(first_fill) = p.fills.iter().find(|f| f.enabled) {
                if first_fill.style == FillStyle::Pattern {
                    fill_str = format!("url(#pat_{}_0)", p.id.0);
                } else {
                    fill_str = color_to_svg(first_fill.color);
                }
            } else if let Some(c) = p.fill_color {
                fill_str = color_to_svg(c);
            }

            let mut stroke_str = "none".to_string();
            let mut stroke_w = 0.0f32;
            let mut stroke_dash = "";
            if let Some(first_stroke) = p.strokes.iter().find(|s| s.enabled) {
                stroke_str = color_to_svg(first_stroke.color);
                stroke_w = first_stroke.width;
                stroke_dash = match first_stroke.style {
                    StrokeStyle::Solid => "",
                    StrokeStyle::Dashed => "stroke-dasharray=\"6,6\" ",
                    StrokeStyle::Dotted => "stroke-dasharray=\"2,4\" ",
                };
            } else if let Some(c) = p.stroke_color {
                stroke_str = color_to_svg(c);
                stroke_w = p.stroke_width;
            }

            let filter_attr = if p.blur > 0.001 {
                format!("filter=\"url(#blur_{})\" ", p.id.0)
            } else {
                String::new()
            };

            let blend_attr = blend_mode_to_svg(p.blend_mode);
            let id_attr = p
                .name
                .as_ref()
                .map(|n| format!("id=\"{}\" ", xml_escape(n)))
                .unwrap_or_else(|| format!("id=\"elem_{}\" ", p.id.0));

            let _ = writeln!(
                svg,
                "{indent}<path {id_attr}d=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{:.2}\" {stroke_dash}opacity=\"{:.3}\" {filter_attr}{blend_attr}/>",
                d, fill_str, stroke_str, stroke_w, p.opacity
            );
        }
        Element::Brush(b) => {
            if b.points.is_empty() {
                return;
            }
            let mut d = String::new();
            let _ = write!(d, "M {:.2} {:.2}", b.points[0].x, b.points[0].y);
            for pt in &b.points[1..] {
                let _ = write!(d, " L {:.2} {:.2}", pt.x, pt.y);
            }

            let stroke_str = color_to_svg(b.color);
            let filter_attr = if b.blur > 0.001 {
                format!("filter=\"url(#blur_{})\" ", b.id.0)
            } else {
                String::new()
            };
            let blend_attr = blend_mode_to_svg(b.blend_mode);
            let id_attr = b
                .name
                .as_ref()
                .map(|n| format!("id=\"{}\" ", xml_escape(n)))
                .unwrap_or_else(|| format!("id=\"elem_{}\" ", b.id.0));

            let _ = writeln!(
                svg,
                "{indent}<path {id_attr}d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" opacity=\"{:.3}\" {filter_attr}{blend_attr}/>",
                d, stroke_str, b.width, b.opacity
            );
        }
        Element::Text(t) => {
            let fill_str = color_to_svg(t.color);
            let anchor = match t.alignment {
                TextAlign::Left => "start",
                TextAlign::Center => "middle",
                TextAlign::Right | TextAlign::Justify => "end",
            };
            let filter_attr = if t.blur > 0.001 {
                format!("filter=\"url(#blur_{})\" ", t.id.0)
            } else {
                String::new()
            };
            let blend_attr = blend_mode_to_svg(t.blend_mode);
            let id_attr = t
                .name
                .as_ref()
                .map(|n| format!("id=\"{}\" ", xml_escape(n)))
                .unwrap_or_else(|| format!("id=\"elem_{}\" ", t.id.0));

            let _ = writeln!(
                svg,
                "{indent}<text {id_attr}x=\"{:.2}\" y=\"{:.2}\" font-family=\"{}\" font-size=\"{:.2}\" font-weight=\"{}\" fill=\"{}\" text-anchor=\"{}\" opacity=\"{:.3}\" {filter_attr}{blend_attr}>{}</text>",
                t.position.x, t.position.y + t.font_size, xml_escape(&t.font_family), t.font_size, t.font_weight, fill_str, anchor, t.opacity, xml_escape(&t.text)
            );
        }
        Element::Image(i) => {
            let norm = i.rect.normalize();
            let b64 = base64_encode(&i.image_data);
            let filter_attr = if i.blur > 0.001 {
                format!("filter=\"url(#blur_{})\" ", i.id.0)
            } else {
                String::new()
            };
            let blend_attr = blend_mode_to_svg(i.blend_mode);
            let id_attr = i
                .name
                .as_ref()
                .map(|n| format!("id=\"{}\" ", xml_escape(n)))
                .unwrap_or_else(|| format!("id=\"elem_{}\" ", i.id.0));

            let _ = writeln!(
                svg,
                "{indent}<image {id_attr}x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" href=\"data:image/png;base64,{}\" opacity=\"{:.3}\" {filter_attr}{blend_attr}/>",
                norm.x, norm.y, norm.width, norm.height, b64, i.opacity
            );
        }
        Element::Group(g) => {
            let clip_attr = if g.clip_element.is_some() {
                format!("clip-path=\"url(#clip_{})\" ", g.id.0)
            } else {
                String::new()
            };
            let filter_attr = if g.blur > 0.001 {
                format!("filter=\"url(#blur_{})\" ", g.id.0)
            } else {
                String::new()
            };
            let blend_attr = blend_mode_to_svg(g.blend_mode);
            let id_attr = g
                .name
                .as_ref()
                .map(|n| format!("id=\"{}\" ", xml_escape(n)))
                .unwrap_or_else(|| format!("id=\"elem_{}\" ", g.id.0));

            let _ = writeln!(
                svg,
                "{indent}<g {id_attr}opacity=\"{:.3}\" {clip_attr}{filter_attr}{blend_attr}>",
                g.opacity
            );
            for child in &g.children {
                if child.visible() {
                    render_element_to_svg(child, svg, indent_level + 1);
                }
            }
            let _ = writeln!(svg, "{indent}</g>");
        }
        Element::Clone(c) => {
            let filter_attr = if c.blur_radius > 0.001 {
                format!("filter=\"url(#blur_{})\" ", c.id.0)
            } else {
                String::new()
            };
            let blend_attr = blend_mode_to_svg(c.blend_mode);
            let id_attr = c
                .name
                .as_ref()
                .map(|n| format!("id=\"{}\" ", xml_escape(n)))
                .unwrap_or_else(|| format!("id=\"elem_{}\" ", c.id.0));

            let mut transform_parts = Vec::new();
            if c.offset.x.abs() > 0.001 || c.offset.y.abs() > 0.001 {
                transform_parts.push(format!("translate({:.2} {:.2})", c.offset.x, c.offset.y));
            }
            if c.rotation.abs() > 0.001 {
                transform_parts.push(format!("rotate({:.2})", c.rotation.to_degrees()));
            }
            if (c.scale.x - 1.0).abs() > 0.001 || (c.scale.y - 1.0).abs() > 0.001 {
                transform_parts.push(format!("scale({:.3} {:.3})", c.scale.x, c.scale.y));
            }
            let transform_attr = if !transform_parts.is_empty() {
                format!("transform=\"{}\" ", transform_parts.join(" "))
            } else {
                String::new()
            };

            let _ = writeln!(
                svg,
                "{indent}<use {id_attr}href=\"#elem_{}\" {transform_attr}opacity=\"{:.3}\" {filter_attr}{blend_attr}/>",
                c.source_id.0, c.opacity
            );
        }
    }
}

pub fn path_nodes_to_svg_d(nodes: &[PathNode], is_closed: bool) -> String {
    if nodes.is_empty() {
        return String::new();
    }

    let mut d = String::new();
    let _ = write!(d, "M {:.2} {:.2}", nodes[0].point.x, nodes[0].point.y);

    for i in 1..nodes.len() {
        let prev = &nodes[i - 1];
        let curr = &nodes[i];

        if prev.handle_out.is_some() || curr.handle_in.is_some() {
            let cp1 = prev.handle_out.unwrap_or(prev.point);
            let cp2 = curr.handle_in.unwrap_or(curr.point);
            let _ = write!(
                d,
                " C {:.2} {:.2}, {:.2} {:.2}, {:.2} {:.2}",
                cp1.x, cp1.y, cp2.x, cp2.y, curr.point.x, curr.point.y
            );
        } else {
            let _ = write!(d, " L {:.2} {:.2}", curr.point.x, curr.point.y);
        }
    }

    if is_closed && nodes.len() > 1 {
        let prev = &nodes[nodes.len() - 1];
        let first = &nodes[0];
        if prev.handle_out.is_some() || first.handle_in.is_some() {
            let cp1 = prev.handle_out.unwrap_or(prev.point);
            let cp2 = first.handle_in.unwrap_or(first.point);
            let _ = write!(
                d,
                " C {:.2} {:.2}, {:.2} {:.2}, {:.2} {:.2} Z",
                cp1.x, cp1.y, cp2.x, cp2.y, first.point.x, first.point.y
            );
        } else {
            d.push_str(" Z");
        }
    }

    d
}

pub fn path_element_to_svg_d(p: &PathElement) -> String {
    if !p.subpath_lengths.is_empty() {
        let mut parts = Vec::new();
        let mut offset = 0;
        for &len in &p.subpath_lengths {
            if len == 0 || offset >= p.nodes.len() {
                continue;
            }
            let end = (offset + len).min(p.nodes.len());
            let sub = &p.nodes[offset..end];
            offset = end;
            parts.push(path_nodes_to_svg_d(sub, p.is_closed));
        }
        parts.join(" ")
    } else {
        path_nodes_to_svg_d(&p.nodes, p.is_closed)
    }
}

fn blend_mode_to_svg(mode: BlendMode) -> String {
    match mode {
        BlendMode::Normal => String::new(),
        BlendMode::Multiply => "style=\"mix-blend-mode: multiply;\" ".to_string(),
        BlendMode::Screen => "style=\"mix-blend-mode: screen;\" ".to_string(),
        BlendMode::Overlay => "style=\"mix-blend-mode: overlay;\" ".to_string(),
        BlendMode::Darken => "style=\"mix-blend-mode: darken;\" ".to_string(),
        BlendMode::Lighten => "style=\"mix-blend-mode: lighten;\" ".to_string(),
        BlendMode::ColorDodge => "style=\"mix-blend-mode: color-dodge;\" ".to_string(),
        BlendMode::ColorBurn => "style=\"mix-blend-mode: color-burn;\" ".to_string(),
        BlendMode::HardLight => "style=\"mix-blend-mode: hard-light;\" ".to_string(),
        BlendMode::SoftLight => "style=\"mix-blend-mode: soft-light;\" ".to_string(),
        BlendMode::Difference => "style=\"mix-blend-mode: difference;\" ".to_string(),
        BlendMode::Exclusion => "style=\"mix-blend-mode: exclusion;\" ".to_string(),
        BlendMode::Hue => "style=\"mix-blend-mode: hue;\" ".to_string(),
        BlendMode::Saturation => "style=\"mix-blend-mode: saturation;\" ".to_string(),
        BlendMode::Color => "style=\"mix-blend-mode: color;\" ".to_string(),
        BlendMode::Luminosity => "style=\"mix-blend-mode: luminosity;\" ".to_string(),
    }
}
