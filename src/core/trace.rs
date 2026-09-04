use skia_safe as skia;

use super::color::Color;
use super::element::{Element, GroupElement, ImageElement, PathElement, PathNode};
use super::geometry::{Point, Rect};

/// Operational mode for bitmap vectorization / tracing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceMode {
    /// Monochromatic brightness cutoff (silhouette / black & white)
    BrightnessCutoff,
    /// Color quantization into multiple stacked/grouped colored paths
    ColorQuantization,
    /// Edge and outline contour detection using gradient magnitude
    EdgeDetection,
}

impl Default for TraceMode {
    fn default() -> Self {
        Self::BrightnessCutoff
    }
}

/// Configuration parameters for tracing a raster bitmap into vector paths
#[derive(Debug, Clone)]
pub struct TraceConfig {
    pub mode: TraceMode,
    /// Luminance threshold for cutoff (0.0 .. 1.0)
    pub threshold: f32,
    /// Number of color clusters for ColorQuantization (2 .. 16)
    pub num_colors: usize,
    /// Minimum contour area in pixels to ignore noise/speckles
    pub despeckle: usize,
    /// Ramer-Douglas-Peucker simplification tolerance in pixels (0.2 .. 5.0)
    pub detail: f32,
    /// Smoothness factor for Bezier handle tangent generation (0.0 .. 1.0)
    pub smoothness: f32,
    /// Minimum corner angle in radians below which a node is kept as a sharp corner
    pub corner_threshold: f32,
    /// Invert binary cutoff logic
    pub invert: bool,
    /// Keep original image and place vector paths on top
    pub keep_original: bool,
    /// Target fill color for monochrome tracing (default: black)
    pub fill_color: Option<Color>,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            mode: TraceMode::BrightnessCutoff,
            threshold: 0.50,
            num_colors: 4,
            despeckle: 8,
            detail: 1.0,
            smoothness: 0.65,
            corner_threshold: 1.25, // ~72 degrees
            invert: false,
            keep_original: false,
            fill_color: Some(Color::new(0.12, 0.12, 0.14, 1.0)),
        }
    }
}

/// 2D Binary pixel grid
#[derive(Debug, Clone)]
pub struct BinaryGrid {
    pub width: usize,
    pub height: usize,
    pub data: Vec<bool>,
}

impl BinaryGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![false; width * height],
        }
    }

    pub fn with_padding(&self, pad: usize) -> Self {
        let new_w = self.width + pad * 2;
        let new_h = self.height + pad * 2;
        let mut padded = Self::new(new_w, new_h);
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) {
                    padded.set(x + pad, y + pad, true);
                }
            }
        }
        padded
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.data[y * self.width + x]
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, val: bool) {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = val;
        }
    }
}

/// Extracts pixel buffer (RGBA8888) from an `ImageElement` using Skia
pub fn extract_image_pixels(
    image: &ImageElement,
    max_dim: u32,
) -> Option<(u32, u32, Vec<u8>)> {
    if image.image_data.is_empty() {
        return None;
    }
    let sk_img = skia::Image::from_encoded(skia::Data::new_copy(&image.image_data))?;
    let orig_w = sk_img.width() as u32;
    let orig_h = sk_img.height() as u32;
    if orig_w == 0 || orig_h == 0 {
        return None;
    }

    // Scale down if image is larger than max_dim for speed and responsiveness
    let scale = if orig_w > max_dim || orig_h > max_dim {
        (max_dim as f32) / (orig_w.max(orig_h) as f32)
    } else {
        1.0
    };
    let w = ((orig_w as f32 * scale).round() as u32).max(1);
    let h = ((orig_h as f32 * scale).round() as u32).max(1);

    let mut surface = skia::surfaces::raster_n32_premul((w as i32, h as i32))?;
    let canvas = surface.canvas();
    canvas.clear(skia::Color4f::new(0.0, 0.0, 0.0, 0.0));

    let mut paint = skia::Paint::default();
    paint.set_anti_alias(true);

    if image.has_active_adjustments() {
        let matrix = image.color_matrix();
        let mut cm = skia::ColorMatrix::default();
        cm.set_row_major(&matrix);
        let cf = skia::color_filters::matrix(&cm, None);
        paint.set_color_filter(cf);
    }

    let src_rect = skia::Rect::from_xywh(0.0, 0.0, orig_w as f32, orig_h as f32);
    let dst_rect = skia::Rect::from_xywh(0.0, 0.0, w as f32, h as f32);
    canvas.draw_image_rect(&sk_img, Some((&src_rect, skia::canvas::SrcRectConstraint::Strict)), dst_rect, &paint);

    let info = skia::ImageInfo::new(
        (w as i32, h as i32),
        skia::ColorType::RGBA8888,
        skia::AlphaType::Unpremul,
        None,
    );
    let row_bytes = (w * 4) as usize;
    let mut pixels = vec![0u8; row_bytes * h as usize];
    if surface.read_pixels(&info, &mut pixels, row_bytes, (0, 0)) {
        Some((w, h, pixels))
    } else {
        None
    }
}

/// Converts RGBA pixels to a binary grid using brightness cutoff
pub fn binarize_brightness(
    w: usize,
    h: usize,
    rgba: &[u8],
    threshold: f32,
    invert: bool,
) -> BinaryGrid {
    let mut grid = BinaryGrid::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            let a = rgba[idx + 3] as f32 / 255.0;
            if a < 0.25 {
                grid.set(x, y, false);
                continue;
            }
            let r = rgba[idx] as f32 / 255.0;
            let g = rgba[idx + 1] as f32 / 255.0;
            let b = rgba[idx + 2] as f32 / 255.0;
            let lum = 0.299 * r + 0.587 * g + 0.114 * b;
            let val = if invert {
                lum >= threshold
            } else {
                lum < threshold
            };
            grid.set(x, y, val);
        }
    }
    grid
}

/// Detects edges using a Sobel filter and produces a binary grid
pub fn binarize_edges(
    w: usize,
    h: usize,
    rgba: &[u8],
    threshold: f32,
) -> BinaryGrid {
    let mut gray = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            let a = rgba[idx + 3] as f32 / 255.0;
            if a < 0.25 {
                gray[y * w + x] = 1.0;
            } else {
                let r = rgba[idx] as f32 / 255.0;
                let g = rgba[idx + 1] as f32 / 255.0;
                let b = rgba[idx + 2] as f32 / 255.0;
                gray[y * w + x] = 0.299 * r + 0.587 * g + 0.114 * b;
            }
        }
    }

    let mut grid = BinaryGrid::new(w, h);
    let thresh_mag = threshold * 0.75;
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let p00 = gray[(y - 1) * w + (x - 1)];
            let p01 = gray[(y - 1) * w + x];
            let p02 = gray[(y - 1) * w + (x + 1)];
            let p10 = gray[y * w + (x - 1)];
            let p12 = gray[y * w + (x + 1)];
            let p20 = gray[(y + 1) * w + (x - 1)];
            let p21 = gray[(y + 1) * w + x];
            let p22 = gray[(y + 1) * w + (x + 1)];

            let gx = (-p00 + p02) + 2.0 * (-p10 + p12) + (-p20 + p22);
            let gy = (-p00 - 2.0 * p01 - p02) + (p20 + 2.0 * p21 + p22);
            let mag = (gx * gx + gy * gy).sqrt();

            grid.set(x, y, mag >= thresh_mag);
        }
    }
    grid
}

/// Quantizes RGBA image into K color layers and returns each color with its binary mask
pub fn quantize_colors(
    w: usize,
    h: usize,
    rgba: &[u8],
    k: usize,
) -> Vec<(Color, BinaryGrid)> {
    let k = k.clamp(2, 16);

    // Collect valid opaque pixels for color clustering
    let mut samples = Vec::new();
    let step = ((w * h) / 4000).max(1);
    for i in (0..w * h).step_by(step) {
        let idx = i * 4;
        let a = rgba[idx + 3];
        if a >= 64 {
            samples.push([
                rgba[idx] as f32 / 255.0,
                rgba[idx + 1] as f32 / 255.0,
                rgba[idx + 2] as f32 / 255.0,
            ]);
        }
    }

    if samples.is_empty() {
        return Vec::new();
    }

    // Initialize centroids evenly spaced by luminance
    samples.sort_by(|a, b| {
        let lum_a = 0.299 * a[0] + 0.587 * a[1] + 0.114 * a[2];
        let lum_b = 0.299 * b[0] + 0.587 * b[1] + 0.114 * b[2];
        lum_a.partial_cmp(&lum_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut centroids = Vec::with_capacity(k);
    for i in 0..k {
        let idx = (i * samples.len()) / k;
        centroids.push(samples[idx]);
    }

    // Run simple k-means clustering (5 iterations)
    for _ in 0..5 {
        let mut sums = vec![[0.0f32; 3]; k];
        let mut counts = vec![0usize; k];

        for p in &samples {
            let mut best_dist = f32::MAX;
            let mut best_c = 0;
            for (ci, c) in centroids.iter().enumerate() {
                let dr = p[0] - c[0];
                let dg = p[1] - c[1];
                let db = p[2] - c[2];
                let dist = dr * dr + dg * dg + db * db;
                if dist < best_dist {
                    best_dist = dist;
                    best_c = ci;
                }
            }
            sums[best_c][0] += p[0];
            sums[best_c][1] += p[1];
            sums[best_c][2] += p[2];
            counts[best_c] += 1;
        }

        for (ci, count) in counts.iter().enumerate() {
            if *count > 0 {
                centroids[ci][0] = sums[ci][0] / *count as f32;
                centroids[ci][1] = sums[ci][1] / *count as f32;
                centroids[ci][2] = sums[ci][2] / *count as f32;
            }
        }
    }

    // Sort centroids by luminance so darker or lighter layers stack intuitively
    centroids.sort_by(|a, b| {
        let lum_a = 0.299 * a[0] + 0.587 * a[1] + 0.114 * a[2];
        let lum_b = 0.299 * b[0] + 0.587 * b[1] + 0.114 * b[2];
        lum_a.partial_cmp(&lum_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Create binary grid per centroid
    let mut result: Vec<(Color, BinaryGrid)> = centroids
        .iter()
        .map(|c| (Color::new(c[0], c[1], c[2], 1.0), BinaryGrid::new(w, h)))
        .collect();

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            let a = rgba[idx + 3];
            if a < 64 {
                continue;
            }
            let p = [
                rgba[idx] as f32 / 255.0,
                rgba[idx + 1] as f32 / 255.0,
                rgba[idx + 2] as f32 / 255.0,
            ];
            let mut best_dist = f32::MAX;
            let mut best_c = 0;
            for (ci, c) in centroids.iter().enumerate() {
                let dr = p[0] - c[0];
                let dg = p[1] - c[1];
                let db = p[2] - c[2];
                let dist = dr * dr + dg * dg + db * db;
                if dist < best_dist {
                    best_dist = dist;
                    best_c = ci;
                }
            }
            result[best_c].1.set(x, y, true);
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Marching Squares Contour Extraction
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GridPointKey(i32, i32);

impl GridPointKey {
    fn from_f32(p: Point) -> Self {
        Self((p.x * 2.0).round() as i32, (p.y * 2.0).round() as i32)
    }
}

/// Directed edge between midpoints of grid pixel boundaries
#[derive(Debug, Clone, Copy)]
struct ContourSegment {
    p1: Point,
    p2: Point,
}

/// Traces binary grid into closed polygonal loops using Marching Squares
pub fn extract_marching_squares_contours(grid: &BinaryGrid, min_area: usize) -> Vec<Vec<Point>> {
    let orig_w = grid.width;
    let orig_h = grid.height;
    if orig_w < 2 || orig_h < 2 {
        return Vec::new();
    }

    // 1-pixel boundary padding guarantees every contour closes cleanly without boundary truncation
    let padded = grid.with_padding(1);
    let pw = padded.width;
    let ph = padded.height;

    let mut segments = Vec::new();

    for y in 0..ph.saturating_sub(1) {
        for x in 0..pw.saturating_sub(1) {
            let tl = padded.get(x, y);
            let tr = padded.get(x + 1, y);
            let bl = padded.get(x, y + 1);
            let br = padded.get(x + 1, y + 1);

            let cell_case = ((tl as u8) << 3) | ((tr as u8) << 2) | ((br as u8) << 1) | (bl as u8);
            if cell_case == 0 || cell_case == 15 {
                continue;
            }

            let fx = x as f32;
            let fy = y as f32;
            let top = Point::new(fx + 0.5, fy);
            let right = Point::new(fx + 1.0, fy + 0.5);
            let bottom = Point::new(fx + 0.5, fy + 1.0);
            let left = Point::new(fx, fy + 0.5);

            match cell_case {
                1 => segments.push(ContourSegment { p1: bottom, p2: left }),
                2 => segments.push(ContourSegment { p1: right, p2: bottom }),
                3 => segments.push(ContourSegment { p1: right, p2: left }),
                4 => segments.push(ContourSegment { p1: top, p2: right }),
                5 => {
                    // Saddle resolution
                    segments.push(ContourSegment { p1: top, p2: right });
                    segments.push(ContourSegment { p1: bottom, p2: left });
                }
                6 => segments.push(ContourSegment { p1: top, p2: bottom }),
                7 => segments.push(ContourSegment { p1: top, p2: left }),
                8 => segments.push(ContourSegment { p1: left, p2: top }),
                9 => segments.push(ContourSegment { p1: bottom, p2: top }),
                10 => {
                    // Saddle resolution
                    segments.push(ContourSegment { p1: left, p2: top });
                    segments.push(ContourSegment { p1: right, p2: bottom });
                }
                11 => segments.push(ContourSegment { p1: right, p2: top }),
                12 => segments.push(ContourSegment { p1: left, p2: right }),
                13 => segments.push(ContourSegment { p1: bottom, p2: right }),
                14 => segments.push(ContourSegment { p1: left, p2: bottom }),
                _ => {}
            }
        }
    }

    if segments.is_empty() {
        return Vec::new();
    }

    // Build lookup table from starting point to outgoing segment index
    use std::collections::HashMap;
    let mut start_map: HashMap<GridPointKey, Vec<usize>> = HashMap::new();
    for (idx, seg) in segments.iter().enumerate() {
        start_map.entry(GridPointKey::from_f32(seg.p1)).or_default().push(idx);
    }

    let mut visited = vec![false; segments.len()];
    let mut loops = Vec::new();

    for i in 0..segments.len() {
        if visited[i] {
            continue;
        }

        let mut current_loop = Vec::new();
        let cur_idx = i;
        visited[cur_idx] = true;
        let start_point = segments[cur_idx].p1;
        current_loop.push(start_point);

        let mut next_pt = segments[cur_idx].p2;
        let mut max_steps = segments.len() + 10;
        let mut closed = false;

        while max_steps > 0 {
            max_steps -= 1;
            current_loop.push(next_pt);

            if GridPointKey::from_f32(next_pt) == GridPointKey::from_f32(start_point) {
                closed = true;
                break;
            }

            let key = GridPointKey::from_f32(next_pt);
            let mut found_next = false;
            if let Some(candidate_indices) = start_map.get(&key) {
                for &cand in candidate_indices {
                    if !visited[cand] {
                        visited[cand] = true;
                        next_pt = segments[cand].p2;
                        found_next = true;
                        break;
                    }
                }
            }

            if !found_next {
                break;
            }
        }

        if !closed {
            continue;
        }

        // Remove redundant identical endpoint
        if current_loop.first() == current_loop.last() {
            current_loop.pop();
        }

        // Must have at least 3 points to form a closed polygon
        if current_loop.len() >= 3 {
            // Map from padded coordinates back to original unpadded image coordinates:
            // Point (x, y) on padded grid corresponds to (x - 0.5, y - 0.5) in [0.0..orig_w, 0.0..orig_h]
            let unpadded_loop: Vec<Point> = current_loop
                .into_iter()
                .map(|p| {
                    Point::new(
                        (p.x - 0.5).clamp(0.0, orig_w as f32),
                        (p.y - 0.5).clamp(0.0, orig_h as f32),
                    )
                })
                .collect();

            let area = polygon_signed_area(&unpadded_loop).abs();
            if area >= min_area as f32 {
                loops.push(unpadded_loop);
            }
        }
    }

    loops
}

/// Computes the signed polygon area using the shoelace formula
pub fn polygon_signed_area(points: &[Point]) -> f32 {
    if points.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    let n = points.len();
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].x * points[j].y;
        area -= points[j].x * points[i].y;
    }
    area * 0.5
}

// ─────────────────────────────────────────────────────────────────────────────
// Ramer-Douglas-Peucker (RDP) Simplification
// ─────────────────────────────────────────────────────────────────────────────

fn perpendicular_distance(pt: Point, line_start: Point, line_end: Point) -> f32 {
    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;
    let mag_sq = dx * dx + dy * dy;
    if mag_sq < 0.0001 {
        return pt.distance_to(line_start);
    }
    let u = ((pt.x - line_start.x) * dx + (pt.y - line_start.y) * dy) / mag_sq;
    let clamped_u = u.clamp(0.0, 1.0);
    let proj = Point::new(line_start.x + clamped_u * dx, line_start.y + clamped_u * dy);
    pt.distance_to(proj)
}

fn rdp_recursive(points: &[Point], epsilon: f32, out: &mut Vec<Point>) {
    if points.len() < 2 {
        if let Some(&p) = points.first() {
            out.push(p);
        }
        return;
    }

    let start = points[0];
    let end = points[points.len() - 1];

    let mut max_dist = 0.0;
    let mut max_idx = 0;

    for i in 1..points.len() - 1 {
        let dist = perpendicular_distance(points[i], start, end);
        if dist > max_dist {
            max_dist = dist;
            max_idx = i;
        }
    }

    if max_dist > epsilon {
        rdp_recursive(&points[..=max_idx], epsilon, out);
        out.pop(); // remove duplicate middle point
        rdp_recursive(&points[max_idx..], epsilon, out);
    } else {
        out.push(start);
        out.push(end);
    }
}

/// Simplifies a closed polygonal loop using Ramer-Douglas-Peucker
pub fn simplify_closed_contour(points: &[Point], epsilon: f32) -> Vec<Point> {
    if points.len() <= 4 || epsilon <= 0.05 {
        return points.to_vec();
    }

    // Split closed loop into two arcs at the point farthest from vertex 0
    let start = points[0];
    let mut max_dist = 0.0;
    let mut split_idx = points.len() / 2;
    for (i, p) in points.iter().enumerate().skip(1) {
        let d = p.distance_to(start);
        if d > max_dist {
            max_dist = d;
            split_idx = i;
        }
    }

    let mut arc1 = Vec::new();
    rdp_recursive(&points[0..=split_idx], epsilon, &mut arc1);

    let mut arc2_pts: Vec<Point> = points[split_idx..].to_vec();
    arc2_pts.push(points[0]);
    let mut arc2 = Vec::new();
    rdp_recursive(&arc2_pts, epsilon, &mut arc2);

    arc1.pop(); // remove joint
    arc2.pop(); // remove wrap around start

    arc1.extend(arc2);
    if arc1.len() >= 3 {
        arc1
    } else {
        points.to_vec()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bézier Curve Fitting & Handle Tangents
// ─────────────────────────────────────────────────────────────────────────────

/// Converts a simplified closed polygon into `PathNode`s with smooth Bézier handles
pub fn fit_bezier_handles(
    points: &[Point],
    smoothness: f32,
    corner_threshold: f32,
) -> Vec<PathNode> {
    let n = points.len();
    if n < 3 {
        return points.iter().map(|&p| PathNode::new(p)).collect();
    }

    let mut nodes = Vec::with_capacity(n);
    let tension = smoothness.clamp(0.0, 1.0) * 0.35;

    for i in 0..n {
        let p = points[i];
        let prev = points[(i + n - 1) % n];
        let next = points[(i + 1) % n];

        let v_in = Point::new(p.x - prev.x, p.y - prev.y);
        let v_out = Point::new(next.x - p.x, next.y - p.y);

        let len_in = (v_in.x * v_in.x + v_in.y * v_in.y).sqrt();
        let len_out = (v_out.x * v_out.x + v_out.y * v_out.y).sqrt();

        if len_in < 0.001 || len_out < 0.001 || tension <= 0.01 {
            nodes.push(PathNode::new(p));
            continue;
        }

        let u_in = Point::new(v_in.x / len_in, v_in.y / len_in);
        let u_out = Point::new(v_out.x / len_out, v_out.y / len_out);

        // Dot product between normalized incoming and outgoing vectors
        let dot = (u_in.x * u_out.x + u_in.y * u_out.y).clamp(-1.0, 1.0);
        let angle = dot.acos();

        // If turn angle is too sharp, keep as a corner
        if angle > corner_threshold {
            nodes.push(PathNode::new(p));
            continue;
        }

        // Tangent is the bisector direction of incoming and outgoing vectors
        let mut tangent = Point::new(u_in.x + u_out.x, u_in.y + u_out.y);
        let t_len = (tangent.x * tangent.x + tangent.y * tangent.y).sqrt();
        if t_len < 0.001 {
            nodes.push(PathNode::new(p));
            continue;
        }
        tangent.x /= t_len;
        tangent.y /= t_len;

        let handle_len_in = len_in * tension;
        let handle_len_out = len_out * tension;

        let handle_in = Some(Point::new(
            p.x - tangent.x * handle_len_in,
            p.y - tangent.y * handle_len_in,
        ));
        let handle_out = Some(Point::new(
            p.x + tangent.x * handle_len_out,
            p.y + tangent.y * handle_len_out,
        ));

        nodes.push(PathNode::with_handles(p, handle_in, handle_out));
    }

    nodes
}

// ─────────────────────────────────────────────────────────────────────────────
// High-Level Tracing Pipeline
// ─────────────────────────────────────────────────────────────────────────────

/// Maps nodes from pixel grid [0..grid_w, 0..grid_h] to target world Rect
fn map_nodes_to_rect(nodes: &mut [PathNode], grid_w: f32, grid_h: f32, target_rect: Rect) {
    if grid_w <= 0.0 || grid_h <= 0.0 {
        return;
    }
    let sx = target_rect.width / grid_w;
    let sy = target_rect.height / grid_h;

    for node in nodes {
        node.point.x = target_rect.x + node.point.x * sx;
        node.point.y = target_rect.y + node.point.y * sy;

        if let Some(ref mut h) = node.handle_in {
            h.x = target_rect.x + h.x * sx;
            h.y = target_rect.y + h.y * sy;
        }
        if let Some(ref mut h) = node.handle_out {
            h.x = target_rect.x + h.x * sx;
            h.y = target_rect.y + h.y * sy;
        }
    }
}

/// Builds a single compound `PathElement` from multiple closed contours
pub fn build_compound_path(
    contours: &[Vec<Point>],
    grid_w: f32,
    grid_h: f32,
    target_rect: Rect,
    config: &TraceConfig,
    fill_color: Option<Color>,
) -> Option<PathElement> {
    if contours.is_empty() {
        return None;
    }

    let mut all_nodes = Vec::new();
    let mut subpath_lengths = Vec::new();

    for contour in contours {
        let simplified = simplify_closed_contour(contour, config.detail);
        if simplified.len() < 3 {
            continue;
        }
        let mut nodes = fit_bezier_handles(&simplified, config.smoothness, config.corner_threshold);
        map_nodes_to_rect(&mut nodes, grid_w, grid_h, target_rect);
        subpath_lengths.push(nodes.len());
        all_nodes.extend(nodes);
    }

    if all_nodes.is_empty() {
        return None;
    }

    let mut path_elem = PathElement::new_compound(
        all_nodes,
        subpath_lengths,
        true,
        fill_color,
        None,
        0.0,
    );
    path_elem.name = Some("Traced Vector Path".to_string());
    Some(path_elem)
}

/// Main entry point: Traces an `ImageElement` into vector `Element::Path` or `Element::Group`
pub fn trace_image_element(
    image: &ImageElement,
    config: &TraceConfig,
) -> Result<Element, String> {
    let (w, h, rgba) = extract_image_pixels(image, 1024)
        .ok_or_else(|| "Failed to decode image data or read pixels".to_string())?;

    let target_rect = image.rect.normalize();
    let grid_w = w as f32;
    let grid_h = h as f32;

    match config.mode {
        TraceMode::BrightnessCutoff => {
            let grid = binarize_brightness(w as usize, h as usize, &rgba, config.threshold, config.invert);
            let contours = extract_marching_squares_contours(&grid, config.despeckle);
            let path = build_compound_path(
                &contours,
                grid_w,
                grid_h,
                target_rect,
                config,
                config.fill_color,
            )
            .ok_or_else(|| "No vector paths could be generated from the selected threshold".to_string())?;
            Ok(Element::Path(path))
        }
        TraceMode::EdgeDetection => {
            let grid = binarize_edges(w as usize, h as usize, &rgba, config.threshold);
            let contours = extract_marching_squares_contours(&grid, config.despeckle);
            let path = build_compound_path(
                &contours,
                grid_w,
                grid_h,
                target_rect,
                config,
                config.fill_color,
            )
            .ok_or_else(|| "No edge contours detected at the current threshold".to_string())?;
            Ok(Element::Path(path))
        }
        TraceMode::ColorQuantization => {
            let layers = quantize_colors(w as usize, h as usize, &rgba, config.num_colors);
            let mut paths = Vec::new();

            for (color, grid) in layers {
                let contours = extract_marching_squares_contours(&grid, config.despeckle);
                if let Some(mut path) = build_compound_path(
                    &contours,
                    grid_w,
                    grid_h,
                    target_rect,
                    config,
                    Some(color),
                ) {
                    path.name = Some(format!("Traced Color Layer ({})", color.to_hex()));
                    paths.push(Element::Path(path));
                }
            }

            if paths.is_empty() {
                return Err("Color quantization did not produce any valid paths".to_string());
            }

            let mut group = GroupElement::new(paths);
            group.name = Some("Traced Color Layers".to_string());
            Ok(Element::Group(group))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_grid_basic() {
        let mut grid = BinaryGrid::new(10, 10);
        assert!(!grid.get(3, 4));
        grid.set(3, 4, true);
        assert!(grid.get(3, 4));
        assert!(!grid.get(4, 4));
    }

    #[test]
    fn test_marching_squares_box() {
        let mut grid = BinaryGrid::new(20, 20);
        for y in 5..15 {
            for x in 5..15 {
                grid.set(x, y, true);
            }
        }
        let contours = extract_marching_squares_contours(&grid, 4);
        assert!(!contours.is_empty(), "Should extract box contour");
        let area = polygon_signed_area(&contours[0]).abs();
        assert!(area > 80.0, "Expected area around 100, got {}", area);
    }

    #[test]
    fn test_rdp_simplification() {
        let pts = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.1),
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(0.0, 2.0),
        ];
        let simplified = simplify_closed_contour(&pts, 0.5);
        assert!(simplified.len() <= pts.len());
    }

    #[test]
    fn test_bezier_handles() {
        let pts = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        let nodes = fit_bezier_handles(&pts, 0.5, 1.2);
        assert_eq!(nodes.len(), 4);
    }

    #[test]
    fn test_marching_squares_touching_border() {
        let mut grid = BinaryGrid::new(20, 20);
        for y in 0..10 {
            for x in 0..10 {
                grid.set(x, y, true);
            }
        }
        let contours = extract_marching_squares_contours(&grid, 4);
        assert!(!contours.is_empty(), "Should extract contour touching border!");
    }
}
