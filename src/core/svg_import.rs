use skia_safe as skia;

use crate::core::color::Color;
use crate::core::element::{CloneElement, Element, ElementId, PathElement, PathNode, RectElement};
use crate::core::geometry::{Point, Rect};

/// Result of parsing an SVG file
pub struct SvgImportResult {
    pub elements: Vec<Element>,
    pub width: f32,
    pub height: f32,
}

/// Parses SVG XML content into native GnomePaths vector elements
pub fn parse_svg(content: &str) -> Result<SvgImportResult, String> {
    let mut elements = Vec::new();
    let (svg_w, svg_h, view_box) = extract_svg_dimensions(content);

    // Simple robust XML tag tokenizer for SVG elements
    let mut pos = 0;
    let bytes = content.as_bytes();
    let len = bytes.len();

    while pos < len {
        if bytes[pos] == b'<' {
            let start = pos;
            while pos < len && bytes[pos] != b'>' {
                pos += 1;
            }
            if pos < len {
                pos += 1; // include '>'
                let tag_str = &content[start..pos];
                elements.extend(parse_svg_tag_elements(tag_str));
            }
        } else {
            pos += 1;
        }
    }

    if elements.is_empty() {
        return Err("No supported vector elements found in SVG".to_string());
    }

    let width = svg_w
        .or_else(|| view_box.map(|vb| vb.width))
        .unwrap_or(800.0);
    let height = svg_h
        .or_else(|| view_box.map(|vb| vb.height))
        .unwrap_or(600.0);

    Ok(SvgImportResult {
        elements,
        width,
        height,
    })
}

fn extract_svg_dimensions(content: &str) -> (Option<f32>, Option<f32>, Option<Rect>) {
    let mut width = None;
    let mut height = None;
    let mut view_box = None;

    if let Some(svg_start) = content.find("<svg") {
        if let Some(svg_end) = content[svg_start..].find('>') {
            let svg_tag = &content[svg_start..svg_start + svg_end + 1];
            if let Some(w_str) = get_attribute(svg_tag, "width") {
                width = parse_dimension(&w_str);
            }
            if let Some(h_str) = get_attribute(svg_tag, "height") {
                height = parse_dimension(&h_str);
            }
            if let Some(vb_str) =
                get_attribute(svg_tag, "viewBox").or_else(|| get_attribute(svg_tag, "viewbox"))
            {
                let nums: Vec<f32> = vb_str
                    .split(|c: char| c.is_whitespace() || c == ',')
                    .filter_map(|s| s.trim().parse::<f32>().ok())
                    .collect();
                if nums.len() >= 4 {
                    view_box = Some(Rect::new(nums[0], nums[1], nums[2], nums[3]));
                }
            }
        }
    }

    (width, height, view_box)
}

fn parse_dimension(s: &str) -> Option<f32> {
    let s = s.trim().trim_end_matches("px").trim_end_matches("pt");
    s.parse::<f32>().ok()
}

fn parse_svg_tag_elements(tag: &str) -> Vec<Element> {
    let trimmed = tag
        .trim_start_matches('<')
        .trim_end_matches('>')
        .trim_end_matches('/');
    let Some(tag_name) = trimmed.split_whitespace().next() else {
        return Vec::new();
    };

    match tag_name {
        "path" => parse_path_tags(tag),
        "rect" => parse_rect_tag(tag).into_iter().collect(),
        "circle" => parse_circle_tag(tag).into_iter().collect(),
        "ellipse" => parse_ellipse_tag(tag).into_iter().collect(),
        "line" => parse_line_tag(tag).into_iter().collect(),
        "polygon" => parse_polygon_tag(tag, true).into_iter().collect(),
        "polyline" => parse_polygon_tag(tag, false).into_iter().collect(),
        "use" => parse_use_tag(tag).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn parse_path_tags(tag: &str) -> Vec<Element> {
    let Some(d) = get_attribute(tag, "d") else {
        return Vec::new();
    };
    let (fill_col, stroke_col, stroke_w, opacity) = extract_style(tag);
    let id_str = get_attribute(tag, "id");
    let mut elements = parse_svg_path_to_elements(&d, fill_col, stroke_col, stroke_w);
    for el in &mut elements {
        el.opacity = opacity;
        if let Some(ref name) = id_str {
            el.name = Some(name.clone());
        }
    }
    elements.into_iter().map(Element::Path).collect()
}

fn parse_use_tag(tag: &str) -> Option<Element> {
    let href = get_attribute(tag, "href").or_else(|| get_attribute(tag, "xlink:href"))?;
    let target_id_str = href.trim_start_matches('#');
    let source_id = if let Some(num_str) = target_id_str.strip_prefix("elem_") {
        num_str.parse::<u64>().map(ElementId).unwrap_or(ElementId(1))
    } else {
        target_id_str.parse::<u64>().map(ElementId).unwrap_or(ElementId(1))
    };
    let x = get_attribute(tag, "x").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let y = get_attribute(tag, "y").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let (_fill_col, _stroke_col, _stroke_w, opacity) = extract_style(tag);

    let mut clone_elem = CloneElement::new(source_id, Point::new(x, y));
    clone_elem.opacity = opacity;
    if let Some(id_str) = get_attribute(tag, "id") {
        clone_elem.name = Some(id_str);
    }
    if let Some(transform_str) = get_attribute(tag, "transform") {
        if let Some(start) = transform_str.find("translate(") {
            let inner = &transform_str[start + 10..];
            if let Some(end) = inner.find(')') {
                let coords: Vec<f32> = inner[..end]
                    .split(|c: char| c.is_whitespace() || c == ',')
                    .filter_map(|s| s.trim().parse::<f32>().ok())
                    .collect();
                if coords.len() >= 2 {
                    clone_elem.offset.x += coords[0];
                    clone_elem.offset.y += coords[1];
                } else if coords.len() == 1 {
                    clone_elem.offset.x += coords[0];
                }
            }
        }
        if let Some(start) = transform_str.find("rotate(") {
            let inner = &transform_str[start + 7..];
            if let Some(end) = inner.find(')') {
                if let Ok(deg) = inner[..end].trim().parse::<f32>() {
                    clone_elem.rotation = deg.to_radians();
                }
            }
        }
        if let Some(start) = transform_str.find("scale(") {
            let inner = &transform_str[start + 6..];
            if let Some(end) = inner.find(')') {
                let scales: Vec<f32> = inner[..end]
                    .split(|c: char| c.is_whitespace() || c == ',')
                    .filter_map(|s| s.trim().parse::<f32>().ok())
                    .collect();
                if scales.len() >= 2 {
                    clone_elem.scale.x *= scales[0];
                    clone_elem.scale.y *= scales[1];
                } else if scales.len() == 1 {
                    clone_elem.scale.x *= scales[0];
                    clone_elem.scale.y *= scales[0];
                }
            }
        }
    }
    Some(Element::Clone(clone_elem))
}

fn parse_rect_tag(tag: &str) -> Option<Element> {
    let x = get_attribute(tag, "x")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let y = get_attribute(tag, "y")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let w = get_attribute(tag, "width").and_then(|s| s.parse::<f32>().ok())?;
    let h = get_attribute(tag, "height").and_then(|s| s.parse::<f32>().ok())?;
    let (fill_col, stroke_col, stroke_w, opacity) = extract_style(tag);

    let mut rect_elem = RectElement::new(Rect::new(x, y, w, h), fill_col, stroke_col);
    rect_elem.stroke_width = stroke_w;
    rect_elem.opacity = opacity;
    if let Some(id_str) = get_attribute(tag, "id") {
        rect_elem.name = Some(id_str);
    }
    Some(Element::Rect(rect_elem))
}

fn parse_circle_tag(tag: &str) -> Option<Element> {
    let cx = get_attribute(tag, "cx")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let cy = get_attribute(tag, "cy")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let r = get_attribute(tag, "r").and_then(|s| s.parse::<f32>().ok())?;
    let (fill_col, stroke_col, stroke_w, opacity) = extract_style(tag);

    let k = 0.5522847498 * r;
    let nodes = vec![
        PathNode::with_handles(
            Point::new(cx, cy - r),
            Some(Point::new(cx - k, cy - r)),
            Some(Point::new(cx + k, cy - r)),
        ),
        PathNode::with_handles(
            Point::new(cx + r, cy),
            Some(Point::new(cx + r, cy - k)),
            Some(Point::new(cx + r, cy + k)),
        ),
        PathNode::with_handles(
            Point::new(cx, cy + r),
            Some(Point::new(cx + k, cy + r)),
            Some(Point::new(cx - k, cy + r)),
        ),
        PathNode::with_handles(
            Point::new(cx - r, cy),
            Some(Point::new(cx - r, cy + k)),
            Some(Point::new(cx - r, cy - k)),
        ),
    ];

    let mut path_elem = PathElement::new(nodes, true, fill_col, stroke_col, stroke_w);
    path_elem.opacity = opacity;
    if let Some(id_str) = get_attribute(tag, "id") {
        path_elem.name = Some(id_str);
    }
    Some(Element::Path(path_elem))
}

fn parse_ellipse_tag(tag: &str) -> Option<Element> {
    let cx = get_attribute(tag, "cx")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let cy = get_attribute(tag, "cy")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let rx = get_attribute(tag, "rx").and_then(|s| s.parse::<f32>().ok())?;
    let ry = get_attribute(tag, "ry").and_then(|s| s.parse::<f32>().ok())?;
    let (fill_col, stroke_col, stroke_w, opacity) = extract_style(tag);

    let kx = 0.5522847498 * rx;
    let ky = 0.5522847498 * ry;
    let nodes = vec![
        PathNode::with_handles(
            Point::new(cx, cy - ry),
            Some(Point::new(cx - kx, cy - ry)),
            Some(Point::new(cx + kx, cy - ry)),
        ),
        PathNode::with_handles(
            Point::new(cx + rx, cy),
            Some(Point::new(cx + rx, cy - ky)),
            Some(Point::new(cx + rx, cy + ky)),
        ),
        PathNode::with_handles(
            Point::new(cx, cy + ry),
            Some(Point::new(cx + kx, cy + ry)),
            Some(Point::new(cx - kx, cy + ry)),
        ),
        PathNode::with_handles(
            Point::new(cx - rx, cy),
            Some(Point::new(cx - rx, cy + ky)),
            Some(Point::new(cx - rx, cy - ky)),
        ),
    ];

    let mut path_elem = PathElement::new(nodes, true, fill_col, stroke_col, stroke_w);
    path_elem.opacity = opacity;
    if let Some(id_str) = get_attribute(tag, "id") {
        path_elem.name = Some(id_str);
    }
    Some(Element::Path(path_elem))
}

fn parse_line_tag(tag: &str) -> Option<Element> {
    let x1 = get_attribute(tag, "x1")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let y1 = get_attribute(tag, "y1")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let x2 = get_attribute(tag, "x2")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let y2 = get_attribute(tag, "y2")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let (_, stroke_col, stroke_w, opacity) = extract_style(tag);
    let stroke_color = stroke_col.unwrap_or(Color::BLACK);

    let nodes = vec![
        PathNode::new(Point::new(x1, y1)),
        PathNode::new(Point::new(x2, y2)),
    ];

    let mut path_elem = PathElement::new(nodes, false, None, Some(stroke_color), stroke_w);
    path_elem.opacity = opacity;
    Some(Element::Path(path_elem))
}

fn parse_polygon_tag(tag: &str, is_closed: bool) -> Option<Element> {
    let pts_str = get_attribute(tag, "points")?;
    let nums: Vec<f32> = pts_str
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
        .collect();

    if nums.len() < 4 {
        return None;
    }

    let mut nodes = Vec::new();
    for chunk in nums.chunks(2) {
        if chunk.len() == 2 {
            nodes.push(PathNode::new(Point::new(chunk[0], chunk[1])));
        }
    }

    let (fill_col, stroke_col, stroke_w, opacity) = extract_style(tag);
    let mut path_elem = PathElement::new(nodes, is_closed, fill_col, stroke_col, stroke_w);
    path_elem.opacity = opacity;
    Some(Element::Path(path_elem))
}

fn get_attribute(tag: &str, attr: &str) -> Option<String> {
    let key = format!("{}=\"", attr);
    if let Some(start) = tag.find(&key) {
        let val_start = start + key.len();
        if let Some(end) = tag[val_start..].find('"') {
            return Some(tag[val_start..val_start + end].to_string());
        }
    }
    let key_single = format!("{}='", attr);
    if let Some(start) = tag.find(&key_single) {
        let val_start = start + key_single.len();
        if let Some(end) = tag[val_start..].find('\'') {
            return Some(tag[val_start..val_start + end].to_string());
        }
    }
    None
}

fn extract_style(tag: &str) -> (Option<Color>, Option<Color>, f32, f32) {
    let mut fill = None;
    let mut stroke = None;
    let mut stroke_w = 1.0f32;
    let mut opacity = 1.0f32;

    if let Some(f_str) = get_attribute(tag, "fill") {
        fill = parse_color(&f_str);
    }
    if let Some(s_str) = get_attribute(tag, "stroke") {
        stroke = parse_color(&s_str);
    }
    if let Some(sw_str) = get_attribute(tag, "stroke-width") {
        if let Ok(w) = sw_str.trim().trim_end_matches("px").parse::<f32>() {
            stroke_w = w;
        }
    }
    if let Some(op_str) = get_attribute(tag, "opacity") {
        if let Ok(op) = op_str.trim().parse::<f32>() {
            opacity = op.clamp(0.0, 1.0);
        }
    }

    if let Some(style_str) = get_attribute(tag, "style") {
        for rule in style_str.split(';') {
            let mut parts = rule.split(':');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                let k = k.trim();
                let v = v.trim();
                match k {
                    "fill" => fill = parse_color(v),
                    "stroke" => stroke = parse_color(v),
                    "stroke-width" => {
                        if let Ok(w) = v.trim_end_matches("px").parse::<f32>() {
                            stroke_w = w;
                        }
                    }
                    "opacity" => {
                        if let Ok(op) = v.parse::<f32>() {
                            opacity = op.clamp(0.0, 1.0);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if fill.is_none() && stroke.is_none() {
        fill = Some(Color::BLACK);
    }

    (fill, stroke, stroke_w, opacity)
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_ascii_lowercase();
    if s == "none" || s == "transparent" {
        return None;
    }
    if s.starts_with('#') {
        let hex = &s[1..];
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                return Some(Color::from_rgba_u8(r, g, b, 255));
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color::from_rgba_u8(r, g, b, 255));
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                return Some(Color::from_rgba_u8(r, g, b, a));
            }
            _ => return None,
        }
    }
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len() - 1];
        let parts: Vec<u8> = inner
            .split(',')
            .filter_map(|p| p.trim().parse::<u8>().ok())
            .collect();
        if parts.len() >= 3 {
            return Some(Color::from_rgba_u8(parts[0], parts[1], parts[2], 255));
        }
    }
    if s.starts_with("rgba(") && s.ends_with(')') {
        let inner = &s[5..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() >= 4 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            let a = (parts[3].parse::<f32>().ok()?.clamp(0.0, 1.0) * 255.0) as u8;
            return Some(Color::from_rgba_u8(r, g, b, a));
        }
    }

    match s.as_str() {
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::from_rgba_u8(255, 0, 0, 255)),
        "green" => Some(Color::from_rgba_u8(0, 128, 0, 255)),
        "blue" => Some(Color::from_rgba_u8(0, 0, 255, 255)),
        "yellow" => Some(Color::from_rgba_u8(255, 255, 0, 255)),
        "cyan" => Some(Color::from_rgba_u8(0, 255, 255, 255)),
        "magenta" => Some(Color::from_rgba_u8(255, 0, 255, 255)),
        "gray" | "grey" => Some(Color::from_rgba_u8(128, 128, 128, 255)),
        "orange" => Some(Color::from_rgba_u8(255, 165, 0, 255)),
        "purple" => Some(Color::from_rgba_u8(128, 0, 128, 255)),
        _ => None,
    }
}

pub fn parse_svg_path_to_elements(
    d: &str,
    fill_color: Option<Color>,
    stroke_color: Option<Color>,
    stroke_width: f32,
) -> Vec<PathElement> {
    if let Some(sk_path) = skia::Path::from_svg(d) {
        let elements = PathElement::from_skia_path(&sk_path, fill_color, stroke_color, stroke_width);
        if !elements.is_empty() {
            return elements;
        }
    }

    let subpaths = parse_svg_path_data_subpaths(d);
    let is_closed = d.to_ascii_lowercase().contains('z');
    if subpaths.is_empty() {
        return Vec::new();
    }
    if subpaths.len() == 1 {
        vec![PathElement::new(
            subpaths.into_iter().next().unwrap(),
            is_closed,
            fill_color,
            stroke_color,
            stroke_width,
        )]
    } else {
        let subpath_lengths = subpaths.iter().map(|s| s.len()).collect();
        let nodes = subpaths.into_iter().flatten().collect();
        vec![PathElement::new_compound(
            nodes,
            subpath_lengths,
            is_closed,
            fill_color,
            stroke_color,
            stroke_width,
        )]
    }
}

#[allow(dead_code)]
pub fn parse_svg_path_data(d: &str) -> Vec<PathNode> {
    parse_svg_path_data_subpaths(d).into_iter().flatten().collect()
}

pub fn parse_svg_path_data_subpaths(d: &str) -> Vec<Vec<PathNode>> {
    let mut subpaths = Vec::new();
    let mut current_nodes = Vec::new();
    let mut current_point = Point::new(0.0, 0.0);
    let mut start_point = Point::new(0.0, 0.0);

    let tokens = tokenize_path_data(d);
    let mut i = 0;
    let mut last_cmd = 'M';

    while i < tokens.len() {
        let start_idx = i;
        let cmd = match &tokens[i] {
            PathToken::Command(c) => {
                last_cmd = *c;
                i += 1;
                *c
            }
            PathToken::Number(_) => {
                if last_cmd == 'M' {
                    last_cmd = 'L';
                } else if last_cmd == 'm' {
                    last_cmd = 'l';
                }
                last_cmd
            }
        };

        match cmd {
            'M' => {
                if !current_nodes.is_empty() {
                    subpaths.push(std::mem::take(&mut current_nodes));
                }
                if let (Some(x), Some(y)) = (get_num(&tokens, &mut i), get_num(&tokens, &mut i)) {
                    current_point = Point::new(x, y);
                    start_point = current_point;
                    current_nodes.push(PathNode::new(current_point));
                }
            }
            'm' => {
                if !current_nodes.is_empty() {
                    subpaths.push(std::mem::take(&mut current_nodes));
                }
                if let (Some(dx), Some(dy)) = (get_num(&tokens, &mut i), get_num(&tokens, &mut i)) {
                    current_point = Point::new(current_point.x + dx, current_point.y + dy);
                    start_point = current_point;
                    current_nodes.push(PathNode::new(current_point));
                }
            }
            'L' => {
                if let (Some(x), Some(y)) = (get_num(&tokens, &mut i), get_num(&tokens, &mut i)) {
                    current_point = Point::new(x, y);
                    current_nodes.push(PathNode::new(current_point));
                }
            }
            'l' => {
                if let (Some(dx), Some(dy)) = (get_num(&tokens, &mut i), get_num(&tokens, &mut i)) {
                    current_point = Point::new(current_point.x + dx, current_point.y + dy);
                    current_nodes.push(PathNode::new(current_point));
                }
            }
            'H' => {
                if let Some(x) = get_num(&tokens, &mut i) {
                    current_point = Point::new(x, current_point.y);
                    current_nodes.push(PathNode::new(current_point));
                }
            }
            'h' => {
                if let Some(dx) = get_num(&tokens, &mut i) {
                    current_point = Point::new(current_point.x + dx, current_point.y);
                    current_nodes.push(PathNode::new(current_point));
                }
            }
            'V' => {
                if let Some(y) = get_num(&tokens, &mut i) {
                    current_point = Point::new(current_point.x, y);
                    current_nodes.push(PathNode::new(current_point));
                }
            }
            'v' => {
                if let Some(dy) = get_num(&tokens, &mut i) {
                    current_point = Point::new(current_point.x, current_point.y + dy);
                    current_nodes.push(PathNode::new(current_point));
                }
            }
            'C' => {
                if let (Some(x1), Some(y1), Some(x2), Some(y2), Some(x), Some(y)) = (
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                ) {
                    if let Some(last_node) = current_nodes.last_mut() {
                        last_node.handle_out = Some(Point::new(x1, y1));
                    }
                    current_point = Point::new(x, y);
                    let node =
                        PathNode::with_handles(current_point, Some(Point::new(x2, y2)), None);
                    current_nodes.push(node);
                }
            }
            'c' => {
                if let (Some(dx1), Some(dy1), Some(dx2), Some(dy2), Some(dx), Some(dy)) = (
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                ) {
                    let cp1 = Point::new(current_point.x + dx1, current_point.y + dy1);
                    let cp2 = Point::new(current_point.x + dx2, current_point.y + dy2);
                    let end = Point::new(current_point.x + dx, current_point.y + dy);

                    if let Some(last_node) = current_nodes.last_mut() {
                        last_node.handle_out = Some(cp1);
                    }
                    current_point = end;
                    let node = PathNode::with_handles(current_point, Some(cp2), None);
                    current_nodes.push(node);
                }
            }
            'S' | 's' => {
                if let (Some(x2_raw), Some(y2_raw), Some(x_raw), Some(y_raw)) = (
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                ) {
                    let is_rel = cmd == 's';
                    let end = if is_rel {
                        Point::new(current_point.x + x_raw, current_point.y + y_raw)
                    } else {
                        Point::new(x_raw, y_raw)
                    };
                    let cp2 = if is_rel {
                        Point::new(current_point.x + x2_raw, current_point.y + y2_raw)
                    } else {
                        Point::new(x2_raw, y2_raw)
                    };
                    let cp1 = if let Some(last) = current_nodes.last() {
                        if let Some(h_out) = last.handle_out {
                            Point::new(
                                2.0 * current_point.x - h_out.x,
                                2.0 * current_point.y - h_out.y,
                            )
                        } else {
                            current_point
                        }
                    } else {
                        current_point
                    };

                    if let Some(last_node) = current_nodes.last_mut() {
                        last_node.handle_out = Some(cp1);
                    }
                    current_point = end;
                    current_nodes.push(PathNode::with_handles(current_point, Some(cp2), None));
                }
            }
            'Q' | 'q' => {
                if let (Some(x1_raw), Some(y1_raw), Some(x_raw), Some(y_raw)) = (
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                ) {
                    let is_rel = cmd == 'q';
                    let qp = if is_rel {
                        Point::new(current_point.x + x1_raw, current_point.y + y1_raw)
                    } else {
                        Point::new(x1_raw, y1_raw)
                    };
                    let end = if is_rel {
                        Point::new(current_point.x + x_raw, current_point.y + y_raw)
                    } else {
                        Point::new(x_raw, y_raw)
                    };

                    let cp1 = Point::new(
                        current_point.x + (2.0 / 3.0) * (qp.x - current_point.x),
                        current_point.y + (2.0 / 3.0) * (qp.y - current_point.y),
                    );
                    let cp2 = Point::new(
                        end.x + (2.0 / 3.0) * (qp.x - end.x),
                        end.y + (2.0 / 3.0) * (qp.y - end.y),
                    );

                    if let Some(last_node) = current_nodes.last_mut() {
                        last_node.handle_out = Some(cp1);
                    }
                    current_point = end;
                    current_nodes.push(PathNode::with_handles(current_point, Some(cp2), None));
                }
            }
            'A' | 'a' => {
                if let (Some(_rx), Some(_ry), Some(_rot), Some(_large_arc), Some(_sweep), Some(x_raw), Some(y_raw)) = (
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                    get_num(&tokens, &mut i),
                ) {
                    let end = if cmd == 'a' {
                        Point::new(current_point.x + x_raw, current_point.y + y_raw)
                    } else {
                        Point::new(x_raw, y_raw)
                    };
                    current_point = end;
                    current_nodes.push(PathNode::new(current_point));
                }
            }
            'Z' | 'z' => {
                current_point = start_point;
                if !current_nodes.is_empty() {
                    subpaths.push(std::mem::take(&mut current_nodes));
                }
            }
            _ => {}
        }

        if i <= start_idx {
            i += 1;
        }
    }

    if !current_nodes.is_empty() {
        subpaths.push(current_nodes);
    }

    subpaths
}

fn get_num(tokens: &[PathToken], idx: &mut usize) -> Option<f32> {
    if *idx < tokens.len() {
        if let PathToken::Number(n) = tokens[*idx] {
            *idx += 1;
            return Some(n);
        }
    }
    None
}

#[derive(Debug, PartialEq)]
enum PathToken {
    Command(char),
    Number(f32),
}

fn tokenize_path_data(d: &str) -> Vec<PathToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = d.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() || c == ',' {
            i += 1;
            continue;
        }

        if c.is_ascii_alphabetic() {
            tokens.push(PathToken::Command(c));
            i += 1;
            continue;
        }

        if c == '-' || c == '+' || c == '.' || c.is_ascii_digit() {
            let start = i;
            if chars[i] == '-' || chars[i] == '+' {
                i += 1;
            }
            let mut has_dot = false;
            while i < chars.len() {
                let ch = chars[i];
                if ch.is_ascii_digit() {
                    i += 1;
                } else if ch == '.' && !has_dot {
                    has_dot = true;
                    i += 1;
                } else if ch == 'e' || ch == 'E' {
                    i += 1;
                    if i < chars.len() && (chars[i] == '-' || chars[i] == '+') {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            let num_str: String = chars[start..i].iter().collect();
            if let Ok(val) = num_str.parse::<f32>() {
                tokens.push(PathToken::Number(val));
            }
            continue;
        }

        i += 1;
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_svg_rect() {
        let svg = r##"<svg width="200" height="100"><rect x="10" y="20" width="80" height="50" fill="#ff0000" stroke="#000000" stroke-width="2"/></svg>"##;
        let res = parse_svg(svg).expect("Failed to parse svg");
        assert_eq!(res.elements.len(), 1);
        if let Element::Rect(r) = &res.elements[0] {
            assert_eq!(r.rect.x, 10.0);
            assert_eq!(r.rect.y, 20.0);
            assert_eq!(r.rect.width, 80.0);
            assert_eq!(r.rect.height, 50.0);
        } else {
            panic!("Expected Rect element");
        }
    }

    #[test]
    fn test_parse_svg_path_data() {
        let d = "M 10 10 L 50 10 L 50 50 Z";
        let nodes = parse_svg_path_data(d);
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].point.x, 10.0);
        assert_eq!(nodes[1].point.x, 50.0);
    }

    #[test]
    fn test_parse_svg_circle_and_polygon() {
        let svg = r##"<svg><circle cx="50" cy="50" r="40" fill="blue"/><polygon points="0,0 100,0 50,100" stroke="green" stroke-width="3"/></svg>"##;
        let res = parse_svg(svg).expect("Failed to parse svg");
        assert_eq!(res.elements.len(), 2);
    }

    #[test]
    fn test_parse_svg_cubic_curve_path() {
        let svg = r##"<svg><path d="M 10 80 C 40 10, 65 10, 95 80 S 150 150, 180 80" fill="none" stroke="black"/></svg>"##;
        let res = parse_svg(svg).expect("Failed to parse svg");
        assert_eq!(res.elements.len(), 1);
        if let Element::Path(p) = &res.elements[0] {
            assert!(p.nodes.len() >= 3);
        } else {
            panic!("Expected Path element");
        }
    }

    #[test]
    fn test_parse_compound_svg_path_with_subpaths() {
        let compound_d = "M 0 0 L 10 0 L 10 10 Z M 20 20 L 30 20 L 30 30 Z";
        let elements = parse_svg_path_to_elements(compound_d, Some(Color::BLACK), None, 1.0);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].subpath_lengths, vec![3, 3]);
        assert_eq!(elements[0].nodes.len(), 6);
        assert_eq!(elements[0].nodes[0].point, Point::new(0.0, 0.0));
        assert_eq!(elements[0].nodes[3].point, Point::new(20.0, 20.0));
    }
}
