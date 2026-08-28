use crate::core::element::CornerStyle;
use crate::core::geometry::{Point, Rect};
use skia_safe as skia;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ArrayMode {
    Linear {
        count: u32,
        offset_x: f32,
        offset_y: f32,
        scale_step: f32,
        rotate_step_deg: f32,
    },
    Radial {
        count: u32,
        radius: f32,
        start_angle_deg: f32,
        total_angle_deg: f32,
        rotate_copies: bool,
    },
    Grid {
        rows: u32,
        cols: u32,
        spacing_x: f32,
        spacing_y: f32,
    },
}

impl Default for ArrayMode {
    fn default() -> Self {
        ArrayMode::Linear {
            count: 3,
            offset_x: 40.0,
            offset_y: 0.0,
            scale_step: 1.0,
            rotate_step_deg: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArrayModifier {
    pub enabled: bool,
    pub mode: ArrayMode,
}

impl Default for ArrayModifier {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: ArrayMode::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnvelopeWarpModifier {
    pub enabled: bool,
    // Bounding box corner displacements
    pub top_left_offset: Point,
    pub top_right_offset: Point,
    pub bottom_right_offset: Point,
    pub bottom_left_offset: Point,
}

impl Default for EnvelopeWarpModifier {
    fn default() -> Self {
        Self {
            enabled: true,
            top_left_offset: Point::new(0.0, 0.0),
            top_right_offset: Point::new(0.0, 0.0),
            bottom_right_offset: Point::new(0.0, 0.0),
            bottom_left_offset: Point::new(0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChamferRoundingModifier {
    pub enabled: bool,
    pub radius: f32,
    pub style: CornerStyle,
}

impl Default for ChamferRoundingModifier {
    fn default() -> Self {
        Self {
            enabled: true,
            radius: 10.0,
            style: CornerStyle::Round,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OffsetPathModifier {
    pub enabled: bool,
    pub offset: f32,
    pub miter_limit: f32,
}

impl Default for OffsetPathModifier {
    fn default() -> Self {
        Self {
            enabled: true,
            offset: 12.0,
            miter_limit: 4.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ZigZagModifier {
    pub enabled: bool,
    pub ridges: u32,
    pub amplitude: f32,
}

impl Default for ZigZagModifier {
    fn default() -> Self {
        Self {
            enabled: true,
            ridges: 6,
            amplitude: 10.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WaveDeformModifier {
    pub enabled: bool,
    pub amplitude: f32,
    pub wavelength: f32,
}

impl Default for WaveDeformModifier {
    fn default() -> Self {
        Self {
            enabled: true,
            amplitude: 15.0,
            wavelength: 60.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Extrude3DMode {
    Isometric,
    Cabinet,
    Perspective,
    Custom3D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Bevel3DStyle {
    Round,
    Chamfer,
    Convex,
    Steps,
}

fn default_roughness() -> f32 {
    0.35
}

fn default_rim_light() -> f32 {
    0.30
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Material3DPreset {
    Default,
    Chrome,
    Gold,
    Glass,
    MatteClay,
    Neon,
    Titanium,
    Velvet,
}

impl Default for Material3DPreset {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Extrude3DModifier {
    pub enabled: bool,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub perspective: f32,
    pub depth: f32,
    pub taper: f32,
    pub twist_deg: f32,
    pub corner_radius_2d: f32,
    pub bevel_style: Bevel3DStyle,
    pub bevel_radius: f32,
    pub bevel_segments: usize,
    pub shading: bool,
    pub shading_intensity: f32,
    pub ambient_light: f32,
    pub light_angle_deg: f32,
    pub light_elevation_deg: f32,
    pub gloss_specular: f32,
    #[serde(default)]
    pub material_preset: Material3DPreset,
    #[serde(default)]
    pub metallic: f32,
    #[serde(default = "default_roughness")]
    pub roughness: f32,
    #[serde(default = "default_rim_light")]
    pub rim_light: f32,
    pub custom_side_color: Option<crate::core::Color>,
    pub mode: Extrude3DMode,
    pub angle_deg: f32,
}

impl Default for Extrude3DModifier {
    fn default() -> Self {
        Self {
            enabled: true,
            rot_x: 20.0,
            rot_y: -30.0,
            rot_z: 0.0,
            perspective: 600.0,
            depth: 40.0,
            taper: 1.0,
            twist_deg: 0.0,
            corner_radius_2d: 0.0,
            bevel_style: Bevel3DStyle::Round,
            bevel_radius: 0.0,
            bevel_segments: 6,
            shading: true,
            shading_intensity: 0.65,
            ambient_light: 0.4,
            light_angle_deg: 135.0,
            light_elevation_deg: 45.0,
            gloss_specular: 0.35,
            material_preset: Material3DPreset::Default,
            metallic: 0.0,
            roughness: 0.35,
            rim_light: 0.30,
            custom_side_color: None,
            mode: Extrude3DMode::Custom3D,
            angle_deg: 45.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TwistModifier {
    pub enabled: bool,
    pub angle_deg: f32,
    pub radius: f32,
}

impl Default for TwistModifier {
    fn default() -> Self {
        Self {
            enabled: true,
            angle_deg: 90.0,
            radius: 100.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Modifier {
    Array(ArrayModifier),
    Extrude3D(Extrude3DModifier),
    Twist(TwistModifier),
    OffsetPath(OffsetPathModifier),
    ZigZag(ZigZagModifier),
    WaveDeform(WaveDeformModifier),
    EnvelopeWarp(EnvelopeWarpModifier),
    ChamferRounding(ChamferRoundingModifier),
}

impl Modifier {
    pub fn name(&self) -> &'static str {
        match self {
            Modifier::Array(_) => "Array Modifier",
            Modifier::Extrude3D(_) => "3D Extrude & Lighting",
            Modifier::Twist(_) => "Twist & Swirl Distortion",
            Modifier::OffsetPath(_) => "Offset Path / Outline",
            Modifier::ZigZag(_) => "ZigZag Distortion",
            Modifier::WaveDeform(_) => "Sine Wave Ripple",
            Modifier::EnvelopeWarp(_) => "Envelope Warp Modifier",
            Modifier::ChamferRounding(_) => "Dynamic Chamfer & Rounding",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Modifier::Array(_) => "view-grid-symbolic",
            Modifier::Extrude3D(_) => "orientation-landscape-symbolic",
            Modifier::Twist(_) => "emblem-synchronizing-symbolic",
            Modifier::OffsetPath(_) => "zoom-out-symbolic",
            Modifier::ZigZag(_) => "edit-cut-symbolic",
            Modifier::WaveDeform(_) => "view-refresh-symbolic",
            Modifier::EnvelopeWarp(_) => "transform-symbolic",
            Modifier::ChamferRounding(_) => "tool-node-symbolic",
        }
    }

    pub fn enabled(&self) -> bool {
        match self {
            Modifier::Array(m) => m.enabled,
            Modifier::Extrude3D(m) => m.enabled,
            Modifier::Twist(m) => m.enabled,
            Modifier::OffsetPath(m) => m.enabled,
            Modifier::ZigZag(m) => m.enabled,
            Modifier::WaveDeform(m) => m.enabled,
            Modifier::EnvelopeWarp(m) => m.enabled,
            Modifier::ChamferRounding(m) => m.enabled,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        match self {
            Modifier::Array(m) => m.enabled = enabled,
            Modifier::Extrude3D(m) => m.enabled = enabled,
            Modifier::Twist(m) => m.enabled = enabled,
            Modifier::OffsetPath(m) => m.enabled = enabled,
            Modifier::ZigZag(m) => m.enabled = enabled,
            Modifier::WaveDeform(m) => m.enabled = enabled,
            Modifier::EnvelopeWarp(m) => m.enabled = enabled,
            Modifier::ChamferRounding(m) => m.enabled = enabled,
        }
    }
}

/// Applies dynamic corner rounding to a Skia path
pub fn apply_chamfer_rounding_to_path(
    path: &skia::Path,
    radius: f32,
    style: CornerStyle,
) -> skia::Path {
    if radius <= 0.1 || path.count_points() < 3 {
        return path.clone();
    }

    let bounds = path.bounds();
    let effective_r = radius.min(bounds.width().min(bounds.height()) * 0.5);

    // 1. If it's an axis-aligned rectangle, create a perfect rounded RRect
    let pts = path.points();
    if pts.len() >= 4 && pts.len() <= 5 {
        let is_rect = (pts[0].x == pts[1].x || pts[0].y == pts[1].y)
            && (pts[1].x == pts[2].x || pts[1].y == pts[2].y);
        if is_rect && style == CornerStyle::Round {
            let mut b = skia::PathBuilder::new();
            let radii = [skia::Point::new(effective_r, effective_r); 4];
            b.add_rrect(
                skia::RRect::new_rect_radii(bounds, &radii),
                Some(skia::PathDirection::CW),
                0,
            );
            return b.detach();
        }
    }

    // 2. Multi-contour path handling (e.g. text or complex paths)
    let loops = extract_subpath_polygon_loops(path);
    if loops.is_empty() {
        return path.clone();
    }

    let mut result_builder = skia::PathBuilder::new();

    for loop_pts in loops {
        let n = loop_pts.len();
        if n < 3 {
            continue;
        }

        let mut raw_pts: Vec<Point> = Vec::new();
        for p in loop_pts {
            let pt = Point::new(p.x, p.y);
            if raw_pts.last() != Some(&pt) {
                raw_pts.push(pt);
            }
        }
        if raw_pts.len() > 3 && raw_pts.first() == raw_pts.last() {
            raw_pts.pop();
        }
        let count = raw_pts.len();
        if count < 3 {
            continue;
        }

        for i in 0..count {
            let prev = raw_pts[(i + count - 1) % count];
            let curr = raw_pts[i];
            let next = raw_pts[(i + 1) % count];

            let v1 = Point::new(prev.x - curr.x, prev.y - curr.y);
            let v2 = Point::new(next.x - curr.x, next.y - curr.y);
            let len1 = (v1.x * v1.x + v1.y * v1.y).sqrt();
            let len2 = (v2.x * v2.x + v2.y * v2.y).sqrt();

            if len1 < 0.1 || len2 < 0.1 {
                if i == 0 {
                    result_builder.move_to(curr.to_skia());
                } else {
                    result_builder.line_to(curr.to_skia());
                }
                continue;
            }

            let u1 = Point::new(v1.x / len1, v1.y / len1);
            let u2 = Point::new(v2.x / len2, v2.y / len2);

            let t = effective_r.min(len1 * 0.48).min(len2 * 0.48);
            let t1 = Point::new(curr.x + u1.x * t, curr.y + u1.y * t);
            let t2 = Point::new(curr.x + u2.x * t, curr.y + u2.y * t);

            if i == 0 {
                result_builder.move_to(t1.to_skia());
            } else {
                result_builder.line_to(t1.to_skia());
            }

            match style {
                CornerStyle::Round => {
                    result_builder.quad_to(curr.to_skia(), t2.to_skia());
                }
                CornerStyle::Chamfer => {
                    result_builder.line_to(t2.to_skia());
                }
                CornerStyle::Concave => {
                    let mid = Point::new(
                        curr.x + (u1.x + u2.x) * t * 0.25,
                        curr.y + (u1.y + u2.y) * t * 0.25,
                    );
                    result_builder.quad_to(mid.to_skia(), t2.to_skia());
                }
            }
        }
        result_builder.close();
    }

    result_builder.detach()
}

/// Applies 4-point envelope warp distortion to a Skia path within a bounding rectangle
pub fn apply_envelope_warp_to_path(
    path: &skia::Path,
    bounds: Rect,
    tl: Point,
    tr: Point,
    br: Point,
    bl: Point,
) -> skia::Path {
    if bounds.width <= 0.001 || bounds.height <= 0.001 {
        return path.clone();
    }
    if tl == Point::ZERO && tr == Point::ZERO && br == Point::ZERO && bl == Point::ZERO {
        return path.clone();
    }

    let orig_tl = Point::new(bounds.x, bounds.y);
    let orig_tr = Point::new(bounds.x + bounds.width, bounds.y);
    let orig_br = Point::new(bounds.x + bounds.width, bounds.y + bounds.height);
    let orig_bl = Point::new(bounds.x, bounds.y + bounds.height);

    let new_tl = Point::new(orig_tl.x + tl.x, orig_tl.y + tl.y);
    let new_tr = Point::new(orig_tr.x + tr.x, orig_tr.y + tr.y);
    let new_br = Point::new(orig_br.x + br.x, orig_br.y + br.y);
    let new_bl = Point::new(orig_bl.x + bl.x, orig_bl.y + bl.y);

    let warp_point = |p: Point| -> Point {
        let u = ((p.x - bounds.x) / bounds.width).clamp(0.0, 1.0);
        let v = ((p.y - bounds.y) / bounds.height).clamp(0.0, 1.0);

        let top = Point::new(
            (1.0 - u) * new_tl.x + u * new_tr.x,
            (1.0 - u) * new_tl.y + u * new_tr.y,
        );
        let bot = Point::new(
            (1.0 - u) * new_bl.x + u * new_br.x,
            (1.0 - u) * new_bl.y + u * new_br.y,
        );

        Point::new((1.0 - v) * top.x + v * bot.x, (1.0 - v) * top.y + v * bot.y)
    };

    let mut builder = skia::PathBuilder::new();
    let count = path.count_points();
    if count == 0 {
        return path.clone();
    }

    let mut iter = skia::path::Iter::new(path, false);
    while let Some((verb, pts)) = iter.next() {
        match verb {
            skia::path::Verb::Move => {
                if !pts.is_empty() {
                    let wp = warp_point(Point::new(pts[0].x, pts[0].y));
                    builder.move_to(wp.to_skia());
                }
            }
            skia::path::Verb::Line => {
                if pts.len() >= 2 {
                    let wp = warp_point(Point::new(pts[1].x, pts[1].y));
                    builder.line_to(wp.to_skia());
                }
            }
            skia::path::Verb::Quad => {
                if pts.len() >= 3 {
                    let wp1 = warp_point(Point::new(pts[1].x, pts[1].y));
                    let wp2 = warp_point(Point::new(pts[2].x, pts[2].y));
                    builder.quad_to(wp1.to_skia(), wp2.to_skia());
                }
            }
            skia::path::Verb::Conic => {
                if pts.len() >= 3 {
                    let wp1 = warp_point(Point::new(pts[1].x, pts[1].y));
                    let wp2 = warp_point(Point::new(pts[2].x, pts[2].y));
                    let w = iter.conic_weight().unwrap_or(1.0);
                    builder.conic_to(wp1.to_skia(), wp2.to_skia(), w);
                }
            }
            skia::path::Verb::Cubic => {
                if pts.len() >= 4 {
                    let wp1 = warp_point(Point::new(pts[1].x, pts[1].y));
                    let wp2 = warp_point(Point::new(pts[2].x, pts[2].y));
                    let wp3 = warp_point(Point::new(pts[3].x, pts[3].y));
                    builder.cubic_to(wp1.to_skia(), wp2.to_skia(), wp3.to_skia());
                }
            }
            skia::path::Verb::Close => {
                builder.close();
            }
            _ => {}
        }
    }

    builder.detach()
}

/// Expands or contracts a Skia path contour by offset distance
pub fn apply_offset_path_to_path(path: &skia::Path, offset: f32, miter_limit: f32) -> skia::Path {
    if offset.abs() < 0.1 {
        return path.clone();
    }
    let mut paint = skia::Paint::default();
    paint.set_style(skia::PaintStyle::Stroke);
    paint.set_stroke_width(offset.abs() * 2.0);
    paint.set_stroke_miter(miter_limit);
    paint.set_stroke_join(skia::PaintJoin::Miter);

    let mut builder = skia::PathBuilder::new();
    if skia::path_utils::fill_path_with_paint(path, &paint, &mut builder, None, None) {
        builder.detach()
    } else {
        path.clone()
    }
}

/// Applies ZigZag serrated distortion to a Skia path
pub fn apply_zigzag_to_path(path: &skia::Path, ridges: u32, amplitude: f32) -> skia::Path {
    if ridges == 0 || amplitude.abs() < 0.1 {
        return path.clone();
    }
    let pts = path.points();
    if pts.len() < 2 {
        return path.clone();
    }

    let mut builder = skia::PathBuilder::new();
    let num_ridges = ridges.max(1) as usize;

    for i in 0..pts.len() {
        let p = pts[i];
        if i == 0 {
            builder.move_to(p);
        } else {
            let prev = pts[i - 1];
            let dx = p.x - prev.x;
            let dy = p.y - prev.y;
            let seg_len = (dx * dx + dy * dy).sqrt();

            let nx = -dy / seg_len.max(0.001);
            let ny = dx / seg_len.max(0.001);

            let sub_steps = num_ridges * 2;
            for s in 1..=sub_steps {
                let t = s as f32 / sub_steps as f32;
                let base_x = prev.x + t * dx;
                let base_y = prev.y + t * dy;

                let sign = if s % 2 == 1 { 1.0 } else { -1.0 };
                let off_x = base_x + nx * amplitude * sign;
                let off_y = base_y + ny * amplitude * sign;

                builder.line_to(skia::Point::new(off_x, off_y));
            }
        }
    }
    if path.is_last_contour_closed() {
        builder.close();
    }
    builder.detach()
}

/// Applies sine wave deformation to a Skia path
pub fn apply_wave_deform_to_path(path: &skia::Path, amplitude: f32, wavelength: f32) -> skia::Path {
    if amplitude.abs() < 0.1 || wavelength <= 1.0 {
        return path.clone();
    }
    let pts = path.points();
    if pts.len() < 2 {
        return path.clone();
    }

    let mut builder = skia::PathBuilder::new();
    let mut accum_dist = 0.0f32;

    for i in 0..pts.len() {
        let p = pts[i];
        if i == 0 {
            let wave_off = amplitude * (accum_dist * 2.0 * std::f32::consts::PI / wavelength).sin();
            builder.move_to(skia::Point::new(p.x, p.y + wave_off));
        } else {
            let prev = pts[i - 1];
            let dx = p.x - prev.x;
            let dy = p.y - prev.y;
            let seg_len = (dx * dx + dy * dy).sqrt();
            accum_dist += seg_len;

            let wave_off = amplitude * (accum_dist * 2.0 * std::f32::consts::PI / wavelength).sin();
            let nx = -dy / seg_len.max(0.001);
            let ny = dx / seg_len.max(0.001);

            let wave_pt = skia::Point::new(p.x + nx * wave_off, p.y + ny * wave_off);
            builder.line_to(wave_pt);
        }
    }
    if path.is_last_contour_closed() {
        builder.close();
    }
    builder.detach()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn normalize(&self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len > 0.0001 {
            Self::new(self.x / len, self.y / len, self.z / len)
        } else {
            Self::new(0.0, 0.0, 1.0)
        }
    }

    pub fn rotate_euler(&self, rx_deg: f32, ry_deg: f32, rz_deg: f32) -> Self {
        let rx = rx_deg.to_radians();
        let ry = ry_deg.to_radians();
        let rz = rz_deg.to_radians();

        // Rx (Pitch)
        let (sx, cx) = rx.sin_cos();
        let y1 = self.y * cx - self.z * sx;
        let z1 = self.y * sx + self.z * cx;
        let x1 = self.x;

        // Ry (Yaw)
        let (sy, cy) = ry.sin_cos();
        let x2 = x1 * cy + z1 * sy;
        let y2 = y1;
        let z2 = -x1 * sy + z1 * cy;

        // Rz (Roll)
        let (sz, cz) = rz.sin_cos();
        let x3 = x2 * cz - y2 * sz;
        let y3 = x2 * sz + y2 * cz;
        let z3 = z2;

        Self::new(x3, y3, z3)
    }

    pub fn project_to_screen(&self, center: Point, perspective: f32) -> Point {
        let scale = if perspective > 10.0 {
            let denom = (perspective - self.z).max(40.0);
            perspective / denom
        } else {
            1.0
        };
        Point::new(center.x + self.x * scale, center.y + self.y * scale)
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

/// Transforms a Skia Path into 3D projective space

#[allow(dead_code)]
pub fn transform_skia_path_3d(
    path: &skia::Path,
    global_center: Point,
    local_z: f32,
    scale_factor: f32,
    twist_deg: f32,
    rot_x: f32,
    rot_y: f32,
    rot_z: f32,
    perspective: f32,
) -> skia::Path {
    let t_rad = twist_deg.to_radians();
    let (st, ct) = t_rad.sin_cos();

    // 1. Separate into subpath contours to preserve individual glyph/shape centers
    let mut subpaths: Vec<skia::Path> = Vec::new();
    let mut cur_builder = skia::PathBuilder::new();
    let mut has_points = false;

    let iter = skia::path::Iter::new(path, false);
    for (verb, points) in iter {
        match verb {
            skia::path::Verb::Move => {
                if has_points {
                    subpaths.push(cur_builder.detach());
                    cur_builder = skia::PathBuilder::new();
                }
                if !points.is_empty() {
                    cur_builder.move_to(points[0]);
                    has_points = true;
                }
            }
            skia::path::Verb::Line => {
                if points.len() >= 2 {
                    cur_builder.line_to(points[1]);
                }
            }
            skia::path::Verb::Quad => {
                if points.len() >= 3 {
                    cur_builder.quad_to(points[1], points[2]);
                }
            }
            skia::path::Verb::Conic => {
                if points.len() >= 3 {
                    cur_builder.quad_to(points[1], points[2]);
                }
            }
            skia::path::Verb::Cubic => {
                if points.len() >= 4 {
                    cur_builder.cubic_to(points[1], points[2], points[3]);
                }
            }
            skia::path::Verb::Close => {
                cur_builder.close();
                subpaths.push(cur_builder.detach());
                cur_builder = skia::PathBuilder::new();
                has_points = false;
            }
            skia::path::Verb::Done => break,
        }
    }
    if has_points {
        subpaths.push(cur_builder.detach());
    }

    if subpaths.is_empty() {
        return path.clone();
    }

    let is_compound = subpaths.len() > 1;
    let mut result_builder = skia::PathBuilder::new();

    for sp in subpaths {
        let sub_b = sp.bounds();
        let sub_center = Point::new(sub_b.center_x(), sub_b.center_y());

        let transform_pt = |p: skia::Point| -> skia::Point {
            let (dx, dy) = if is_compound {
                let local_dx =
                    (p.x - sub_center.x) * scale_factor + (sub_center.x - global_center.x);
                let local_dy =
                    (p.y - sub_center.y) * scale_factor + (sub_center.y - global_center.y);
                (local_dx, local_dy)
            } else {
                (
                    (p.x - global_center.x) * scale_factor,
                    (p.y - global_center.y) * scale_factor,
                )
            };

            let (dx_t, dy_t) = if twist_deg.abs() > 0.01 {
                (dx * ct - dy * st, dx * st + dy * ct)
            } else {
                (dx, dy)
            };

            let v = Vec3::new(dx_t, dy_t, local_z).rotate_euler(rot_x, rot_y, rot_z);
            let proj = v.project_to_screen(global_center, perspective);
            skia::Point::new(proj.x, proj.y)
        };

        let iter = skia::path::Iter::new(&sp, false);
        for (verb, points) in iter {
            match verb {
                skia::path::Verb::Move => {
                    if !points.is_empty() {
                        result_builder.move_to(transform_pt(points[0]));
                    }
                }
                skia::path::Verb::Line => {
                    if points.len() >= 2 {
                        result_builder.line_to(transform_pt(points[1]));
                    }
                }
                skia::path::Verb::Quad => {
                    if points.len() >= 3 {
                        result_builder.quad_to(transform_pt(points[1]), transform_pt(points[2]));
                    }
                }
                skia::path::Verb::Conic => {
                    if points.len() >= 3 {
                        result_builder.quad_to(transform_pt(points[1]), transform_pt(points[2]));
                    }
                }
                skia::path::Verb::Cubic => {
                    if points.len() >= 4 {
                        result_builder.cubic_to(
                            transform_pt(points[1]),
                            transform_pt(points[2]),
                            transform_pt(points[3]),
                        );
                    }
                }
                skia::path::Verb::Close => {
                    result_builder.close();
                }
                skia::path::Verb::Done => break,
            }
        }
    }

    result_builder.detach()
}

/// Extracts distinct subpath polygon loops from a Skia Path with adaptive curve subdivision
pub fn extract_subpath_polygon_loops(path: &skia::Path) -> Vec<Vec<skia::Point>> {
    let mut loops = Vec::new();
    let mut current_loop: Vec<skia::Point> = Vec::new();

    let iter = skia::path::Iter::new(path, false);
    for (verb, points) in iter {
        match verb {
            skia::path::Verb::Move => {
                if !current_loop.is_empty() {
                    loops.push(current_loop);
                    current_loop = Vec::new();
                }
                if !points.is_empty() {
                    current_loop.push(points[0]);
                }
            }
            skia::path::Verb::Line => {
                if points.len() >= 2 {
                    current_loop.push(points[1]);
                }
            }
            skia::path::Verb::Quad => {
                if points.len() >= 3 {
                    let p0 = if let Some(&last) = current_loop.last() {
                        last
                    } else {
                        points[0]
                    };
                    let p1 = points[1];
                    let p2 = points[2];
                    let steps = 12usize;
                    for s in 1..=steps {
                        let t = s as f32 / steps as f32;
                        let inv_t = 1.0 - t;
                        let x = inv_t * inv_t * p0.x + 2.0 * inv_t * t * p1.x + t * t * p2.x;
                        let y = inv_t * inv_t * p0.y + 2.0 * inv_t * t * p1.y + t * t * p2.y;
                        current_loop.push(skia::Point::new(x, y));
                    }
                }
            }
            skia::path::Verb::Conic => {
                if points.len() >= 3 {
                    let p0 = if let Some(&last) = current_loop.last() {
                        last
                    } else {
                        points[0]
                    };
                    let p1 = points[1];
                    let p2 = points[2];
                    let steps = 12usize;
                    for s in 1..=steps {
                        let t = s as f32 / steps as f32;
                        let inv_t = 1.0 - t;
                        let x = inv_t * inv_t * p0.x + 2.0 * inv_t * t * p1.x + t * t * p2.x;
                        let y = inv_t * inv_t * p0.y + 2.0 * inv_t * t * p1.y + t * t * p2.y;
                        current_loop.push(skia::Point::new(x, y));
                    }
                }
            }
            skia::path::Verb::Cubic => {
                if points.len() >= 4 {
                    let p0 = if let Some(&last) = current_loop.last() {
                        last
                    } else {
                        points[0]
                    };
                    let p1 = points[1];
                    let p2 = points[2];
                    let p3 = points[3];
                    let steps = 24usize;
                    for s in 1..=steps {
                        let t = s as f32 / steps as f32;
                        let inv_t = 1.0 - t;
                        let x = inv_t.powi(3) * p0.x
                            + 3.0 * inv_t.powi(2) * t * p1.x
                            + 3.0 * inv_t * t.powi(2) * p2.x
                            + t.powi(3) * p3.x;
                        let y = inv_t.powi(3) * p0.y
                            + 3.0 * inv_t.powi(2) * t * p1.y
                            + 3.0 * inv_t * t.powi(2) * p2.y
                            + t.powi(3) * p3.y;
                        current_loop.push(skia::Point::new(x, y));
                    }
                }
            }
            skia::path::Verb::Close => {
                if !current_loop.is_empty() {
                    loops.push(current_loop);
                    current_loop = Vec::new();
                }
            }
            skia::path::Verb::Done => break,
        }
    }
    if !current_loop.is_empty() {
        loops.push(current_loop);
    }

    let mut clean_loops: Vec<Vec<skia::Point>> = Vec::new();
    for lp in loops {
        if lp.len() < 3 {
            continue;
        }
        let mut cl: Vec<skia::Point> = Vec::with_capacity(lp.len());
        for p in lp {
            if let Some(last) = cl.last() {
                let dx = p.x - last.x;
                let dy = p.y - last.y;
                if (dx * dx + dy * dy) < 0.001 {
                    continue;
                }
            }
            cl.push(p);
        }
        if cl.len() >= 3 {
            clean_loops.push(cl);
        }
    }
    clean_loops
}

/// Extrudes a 2D vector path into a true topological 3D mesh solid

#[allow(dead_code)]
pub fn apply_extrude_3d_to_path(path: &skia::Path, ext: &Extrude3DModifier) -> skia::Path {
    if !ext.enabled {
        return path.clone();
    }
    let bounds = path.bounds();
    let center = Point::new(bounds.center_x(), bounds.center_y());

    let (rot_x, rot_y, rot_z, perspective) = match ext.mode {
        Extrude3DMode::Isometric => (35.264, -45.0, 0.0, 0.0),
        Extrude3DMode::Cabinet => (0.0, 0.0, 0.0, 0.0),
        Extrude3DMode::Perspective => (20.0, -30.0, 0.0, 600.0),
        Extrude3DMode::Custom3D => (ext.rot_x, ext.rot_y, ext.rot_z, ext.perspective),
    };

    let rounded_path = if ext.corner_radius_2d > 0.1 {
        apply_chamfer_rounding_to_path(path, ext.corner_radius_2d, CornerStyle::Round)
    } else {
        path.clone()
    };

    let loops = extract_subpath_polygon_loops(&rounded_path);
    if loops.is_empty() {
        return path.clone();
    }
    let is_compound = loops.len() > 1;

    let mut builder = skia::PathBuilder::new();
    let mut front_builder = skia::PathBuilder::new();
    let mut back_builder = skia::PathBuilder::new();

    for loop_pts in &loops {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for p in loop_pts {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
        let sub_center = Point::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);

        let transform_pt = |p: skia::Point, lz: f32, scale: f32, twist: f32| -> skia::Point {
            let (dx, dy) = if is_compound {
                let local_dx = (p.x - sub_center.x) * scale + (sub_center.x - center.x) * scale;
                let local_dy = (p.y - sub_center.y) * scale + (sub_center.y - center.y) * scale;
                (local_dx, local_dy)
            } else {
                ((p.x - center.x) * scale, (p.y - center.y) * scale)
            };

            let (dx_t, dy_t) = if twist.abs() > 0.01 {
                let t_rad = twist.to_radians();
                let (st, ct) = t_rad.sin_cos();
                (dx * ct - dy * st, dx * st + dy * ct)
            } else {
                (dx, dy)
            };

            let v3d = Vec3::new(dx_t, dy_t, lz).rotate_euler(rot_x, rot_y, rot_z);
            let proj = v3d.project_to_screen(center, perspective);
            skia::Point::new(proj.x, proj.y)
        };

        let n_pts = loop_pts.len();
        if n_pts >= 3 {
            let p_front_0 = transform_pt(loop_pts[0], 0.0, 1.0, 0.0);
            front_builder.move_to(p_front_0);
            for p in &loop_pts[1..] {
                front_builder.line_to(transform_pt(*p, 0.0, 1.0, 0.0));
            }
            front_builder.close();

            let p_back_0 = transform_pt(loop_pts[0], -ext.depth, ext.taper, ext.twist_deg);
            back_builder.move_to(p_back_0);
            for p in &loop_pts[1..] {
                back_builder.line_to(transform_pt(*p, -ext.depth, ext.taper, ext.twist_deg));
            }
            back_builder.close();

            for i in 0..n_pts {
                let next_i = (i + 1) % n_pts;
                let p0 = transform_pt(loop_pts[i], 0.0, 1.0, 0.0);
                let p1 = transform_pt(loop_pts[next_i], 0.0, 1.0, 0.0);
                let p2 = transform_pt(loop_pts[next_i], -ext.depth, ext.taper, ext.twist_deg);
                let p3 = transform_pt(loop_pts[i], -ext.depth, ext.taper, ext.twist_deg);

                let mut quad = skia::PathBuilder::new();
                quad.move_to(p0);
                quad.line_to(p1);
                quad.line_to(p2);
                quad.line_to(p3);
                quad.close();
                builder.add_path(&quad.detach(), skia::path::AddPathMode::Append);
            }
        }
    }

    builder.add_path(&back_builder.detach(), skia::path::AddPathMode::Append);
    builder.add_path(&front_builder.detach(), skia::path::AddPathMode::Append);
    builder.detach()
}

/// Applies a parametric twist/swirl distortion around path center
pub fn apply_twist_to_path(path: &skia::Path, angle_deg: f32, max_radius: f32) -> skia::Path {
    if angle_deg.abs() < 0.1 || max_radius <= 1.0 {
        return path.clone();
    }
    let pts = path.points();
    if pts.len() < 2 {
        return path.clone();
    }

    let bounds = path.bounds();
    let center = skia::Point::new(
        bounds.x() + bounds.width() / 2.0,
        bounds.y() + bounds.height() / 2.0,
    );

    let max_rad_rad = angle_deg.to_radians();
    let mut builder = skia::PathBuilder::new();

    for i in 0..pts.len() {
        let p = pts[i];
        let dx = p.x - center.x;
        let dy = p.y - center.y;
        let dist = (dx * dx + dy * dy).sqrt();

        let twist_factor = (1.0 - (dist / max_radius).min(1.0)).max(0.0);
        let cur_angle = max_rad_rad * twist_factor;

        let cos_a = cur_angle.cos();
        let sin_a = cur_angle.sin();

        let rx = dx * cos_a - dy * sin_a;
        let ry = dx * sin_a + dy * cos_a;

        let np = skia::Point::new(center.x + rx, center.y + ry);
        if i == 0 {
            builder.move_to(np);
        } else {
            builder.line_to(np);
        }
    }
    builder.close();
    builder.detach()
}

#[derive(Debug, Clone)]
pub struct ExtrudedFaceElement {
    pub path: skia::Path,
    pub color: crate::core::Color,
    pub is_front_cap: bool,
}

fn is_point_in_polygon(pt: skia::Point, poly: &[skia::Point]) -> bool {
    let mut inside = false;
    let n = poly.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let pi = poly[i];
        let pj = poly[j];
        if ((pi.y > pt.y) != (pj.y > pt.y))
            && (pt.x < (pj.x - pi.x) * (pt.y - pi.y) / (pj.y - pi.y).max(0.00001) + pi.x)
        {
            inside = !inside;
        }
    }
    inside
}

fn group_loops_into_glyphs(loops: &[Vec<skia::Point>]) -> Vec<(usize, Vec<usize>)> {
    let mut glyphs: Vec<(usize, Vec<usize>)> = Vec::new();
    let mut is_hole = vec![false; loops.len()];

    for i in 0..loops.len() {
        if loops[i].is_empty() {
            continue;
        }
        let pt = loops[i][0];
        let mut enclosure_count = 0;
        let mut parent_idx = None;
        for j in 0..loops.len() {
            if i != j && !loops[j].is_empty() && is_point_in_polygon(pt, &loops[j]) {
                enclosure_count += 1;
                parent_idx = Some(j);
            }
        }
        if enclosure_count % 2 == 1 {
            is_hole[i] = true;
            if let Some(p) = parent_idx {
                if let Some(g) = glyphs.iter_mut().find(|(outer, _)| *outer == p) {
                    g.1.push(i);
                } else {
                    glyphs.push((p, vec![i]));
                }
            }
        }
    }

    for (i, &hole) in is_hole.iter().enumerate() {
        if !hole {
            if !glyphs.iter().any(|(outer, _)| *outer == i) {
                glyphs.push((i, Vec::new()));
            }
        }
    }

    glyphs
}

#[allow(dead_code)]
fn compute_inward_vertex_normals(pts: &[skia::Point]) -> Vec<skia::Point> {
    let n = pts.len();
    if n < 3 {
        return vec![skia::Point::new(0.0, 0.0); n];
    }
    let mut signed_area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        signed_area += pts[i].x * pts[j].y - pts[j].x * pts[i].y;
    }
    let is_cw = signed_area > 0.0;

    let mut normals = Vec::with_capacity(n);
    for i in 0..n {
        let prev_i = if i == 0 { n - 1 } else { i - 1 };
        let next_i = (i + 1) % n;

        let p_prev = pts[prev_i];
        let p_curr = pts[i];
        let p_next = pts[next_i];

        let d1 = skia::Point::new(p_curr.x - p_prev.x, p_curr.y - p_prev.y);
        let l1 = (d1.x * d1.x + d1.y * d1.y).sqrt().max(0.0001);
        let u1 = skia::Point::new(d1.x / l1, d1.y / l1);

        let d2 = skia::Point::new(p_next.x - p_curr.x, p_next.y - p_curr.y);
        let l2 = (d2.x * d2.x + d2.y * d2.y).sqrt().max(0.0001);
        let u2 = skia::Point::new(d2.x / l2, d2.y / l2);

        let (n1, n2) = if is_cw {
            (skia::Point::new(-u1.y, u1.x), skia::Point::new(-u2.y, u2.x))
        } else {
            (skia::Point::new(u1.y, -u1.x), skia::Point::new(u2.y, -u2.x))
        };

        let dot = (n1.x * n2.x + n1.y * n2.y).clamp(-0.9, 1.0);
        let scale = (2.0 / (1.0 + dot)).min(2.5);
        let miter_n = skia::Point::new(
            (n1.x + n2.x) * 0.5 * scale,
            (n1.y + n2.y) * 0.5 * scale,
        );
        normals.push(miter_n);
    }
    normals
}

/// Bakes 3D extrusion into clean vector path elements for persistence and SVG export
pub fn bake_extrude_3d_faces(
    path: &skia::Path,
    ext: &Extrude3DModifier,
    base_fill: crate::core::Color,
) -> Vec<ExtrudedFaceElement> {
    if !ext.enabled || ext.depth.abs() < 0.1 {
        return vec![ExtrudedFaceElement {
            path: path.clone(),
            color: base_fill,
            is_front_cap: true,
        }];
    }

    let bounds = path.bounds();
    let global_center = Point::new(bounds.center_x(), bounds.center_y());
    let max_dim = bounds.width().min(bounds.height()).max(10.0);

    let (rot_x, rot_y, rot_z, perspective) = match ext.mode {
        Extrude3DMode::Isometric => (35.264, -45.0, 0.0, 0.0),
        Extrude3DMode::Cabinet => (0.0, 0.0, 0.0, 0.0),
        Extrude3DMode::Perspective => (20.0, -30.0, 0.0, 600.0),
        Extrude3DMode::Custom3D => (ext.rot_x, ext.rot_y, ext.rot_z, ext.perspective),
    };

    let bevel_r = ext.bevel_radius.min(ext.depth * 0.45).min(max_dim * 0.35);
    let rounding_r = ext.corner_radius_2d.max(if bevel_r > 0.5 { bevel_r.min(15.0) } else { 0.0 });
    let rounded_path = if rounding_r > 0.1 {
        apply_chamfer_rounding_to_path(path, rounding_r, CornerStyle::Round)
    } else {
        path.clone()
    };

    let loops = extract_subpath_polygon_loops(&rounded_path);
    if loops.is_empty() {
        return vec![ExtrudedFaceElement {
            path: path.clone(),
            color: base_fill,
            is_front_cap: true,
        }];
    }
    let glyphs = group_loops_into_glyphs(&loops);

    let side_base = ext.custom_side_color.unwrap_or_else(|| {
        crate::core::Color::new(
            base_fill.r * 0.75,
            base_fill.g * 0.75,
            base_fill.b * 0.75,
            base_fill.a,
        )
    });

    let (light_dir, light_intensity) = if ext.shading {
        let (sx, cx) = ext.light_elevation_deg.to_radians().sin_cos();
        let (sy, cy) = ext.light_angle_deg.to_radians().sin_cos();
        let dir = Vec3::new(cy * cx, sy * cx, sx).normalize();
        (dir, 1.0f32)
    } else {
        (Vec3::new(0.0, 0.0, 1.0), 0.0f32)
    };

    let view_dir = Vec3::new(0.0, 0.0, 1.0);
    let n_front = Vec3::new(0.0, 0.0, 1.0)
        .rotate_euler(rot_x, rot_y, rot_z)
        .normalize();
    let n_back = Vec3::new(0.0, 0.0, -1.0)
        .rotate_euler(rot_x, rot_y, rot_z)
        .normalize();

    let compute_color = |base: crate::core::Color, normal: Vec3| -> crate::core::Color {
        if !ext.shading || light_intensity <= 0.01 {
            return base;
        }

        let base_r = base.r.max(0.08);
        let base_g = base.g.max(0.08);
        let base_b = base.b.max(0.08);

        let n_dot_l = normal.dot(&light_dir).max(0.0);
        let ambient = 0.32;
        let diffuse = 0.58 * n_dot_l;

        let half_vec = (light_dir + view_dir).normalize();
        let n_dot_h = normal.dot(&half_vec).max(0.0);
        let shininess = (1.0 - ext.roughness * 0.85).max(0.05) * 64.0;
        let specular_intensity = n_dot_h.powf(shininess) * ext.gloss_specular;

        let n_dot_v = normal.dot(&view_dir).abs();
        let fresnel = (1.0 - n_dot_v).powi(3) * ext.rim_light * 0.55;

        let total_light = ambient + diffuse + fresnel;

        let lit_r = (base_r * total_light * (1.0 - ext.metallic * 0.4)
            + specular_intensity * (if ext.metallic > 0.5 { base_r } else { 1.0 }))
        .clamp(0.0, 1.0);
        let lit_g = (base_g * total_light * (1.0 - ext.metallic * 0.4)
            + specular_intensity * (if ext.metallic > 0.5 { base_g } else { 1.0 }))
        .clamp(0.0, 1.0);
        let lit_b = (base_b * total_light * (1.0 - ext.metallic * 0.4)
            + specular_intensity * (if ext.metallic > 0.5 { base_b } else { 1.0 }))
        .clamp(0.0, 1.0);

        crate::core::Color::new(lit_r, lit_g, lit_b, base.a)
    };

    let transform_pt_to_3d_and_2d = |p: skia::Point,
                                     local_z: f32,
                                     taper_s: f32,
                                     twist_deg: f32|
     -> (Vec3, skia::Point) {
        let dx = (p.x - global_center.x) * taper_s;
        let dy = (p.y - global_center.y) * taper_s;

        let (dx_t, dy_t) = if twist_deg.abs() > 0.01 {
            let t_rad = twist_deg.to_radians();
            let (st, ct) = t_rad.sin_cos();
            (dx * ct - dy * st, dx * st + dy * ct)
        } else {
            (dx, dy)
        };

        let v3d = Vec3::new(dx_t, dy_t, local_z).rotate_euler(rot_x, rot_y, rot_z);
        let proj = v3d.project_to_screen(global_center, perspective);
        (v3d, skia::Point::new(proj.x, proj.y))
    };

    struct RingLevel {
        local_z: f32,
        inset_dist: f32,
        taper_scale: f32,
        twist: f32,
    }

    let bevel_r = ext.bevel_radius.min(ext.depth * 0.45).min(max_dim * 0.35);
    let has_bevel = bevel_r > 0.5;

    let mut ring_levels: Vec<RingLevel> = Vec::new();

    if has_bevel {
        let bevel_steps = 6usize;
        // Front shoulder fillet
        for k in 0..=bevel_steps {
            let u = k as f32 / bevel_steps as f32;
            let theta = u * std::f32::consts::FRAC_PI_2;
            let (z_off, d_off) = match ext.bevel_style {
                Bevel3DStyle::Round => {
                    let z = -bevel_r * (1.0 - theta.cos());
                    let d = bevel_r * theta.cos();
                    (z, d)
                }
                Bevel3DStyle::Chamfer => {
                    let z = -bevel_r * u;
                    let d = bevel_r * (1.0 - u);
                    (z, d)
                }
                Bevel3DStyle::Convex => {
                    let z = -bevel_r * (1.0 - theta.cos());
                    let d = bevel_r * theta.cos() * (1.0 + 0.15 * (2.0 * theta).sin());
                    (z, d)
                }
                Bevel3DStyle::Steps => {
                    let step = (u * 3.0).floor() / 3.0;
                    let z = -bevel_r * step;
                    let d = bevel_r * (1.0 - step);
                    (z, d)
                }
            };
            let norm_t = (z_off.abs() / ext.depth.max(0.1)).clamp(0.0, 1.0);
            let taper = 1.0 - (1.0 - ext.taper) * norm_t;
            let twist = ext.twist_deg * norm_t;
            ring_levels.push(RingLevel {
                local_z: z_off,
                inset_dist: d_off.max(0.0),
                taper_scale: taper.max(0.001),
                twist,
            });
        }

        // Straight body between front and rear shoulders
        if ext.depth > 2.0 * bevel_r + 1.0 {
            let z_body = -bevel_r + (-ext.depth + 2.0 * bevel_r) * 0.5;
            let norm_t = (z_body.abs() / ext.depth.max(0.1)).clamp(0.0, 1.0);
            let taper = 1.0 - (1.0 - ext.taper) * norm_t;
            let twist = ext.twist_deg * norm_t;
            ring_levels.push(RingLevel {
                local_z: z_body,
                inset_dist: 0.0,
                taper_scale: taper.max(0.001),
                twist,
            });
        }

        // Rear shoulder fillet
        for k in 1..=bevel_steps {
            let u = k as f32 / bevel_steps as f32;
            let theta = u * std::f32::consts::FRAC_PI_2;
            let (z_off, d_off) = match ext.bevel_style {
                Bevel3DStyle::Round => {
                    let z = -ext.depth + bevel_r * (1.0 - theta.sin());
                    let d = bevel_r * (1.0 - theta.cos());
                    (z, d)
                }
                Bevel3DStyle::Chamfer => {
                    let z = -ext.depth + bevel_r * (1.0 - u);
                    let d = bevel_r * u;
                    (z, d)
                }
                Bevel3DStyle::Convex => {
                    let z = -ext.depth + bevel_r * (1.0 - theta.sin());
                    let d = bevel_r * (1.0 - theta.cos()) * (1.0 + 0.15 * (2.0 * theta).sin());
                    (z, d)
                }
                Bevel3DStyle::Steps => {
                    let step = ((1.0 - u) * 3.0).floor() / 3.0;
                    let z = -ext.depth + bevel_r * step;
                    let d = bevel_r * (1.0 - step);
                    (z, d)
                }
            };
            let norm_t = (z_off.abs() / ext.depth.max(0.1)).clamp(0.0, 1.0);
            let taper = 1.0 - (1.0 - ext.taper) * norm_t;
            let twist = ext.twist_deg * norm_t;
            ring_levels.push(RingLevel {
                local_z: z_off,
                inset_dist: d_off.max(0.0),
                taper_scale: taper.max(0.001),
                twist,
            });
        }
    } else {
        ring_levels.push(RingLevel {
            local_z: 0.0,
            inset_dist: 0.0,
            taper_scale: 1.0,
            twist: 0.0,
        });
        let has_twist_or_taper = ext.twist_deg.abs() > 0.01 || (ext.taper - 1.0).abs() > 0.01;
        if has_twist_or_taper {
            for k in 1..3 {
                let t = k as f32 / 3.0;
                ring_levels.push(RingLevel {
                    local_z: -ext.depth * t,
                    inset_dist: 0.0,
                    taper_scale: (1.0 - (1.0 - ext.taper) * t).max(0.001),
                    twist: ext.twist_deg * t,
                });
            }
        }
        ring_levels.push(RingLevel {
            local_z: -ext.depth,
            inset_dist: 0.0,
            taper_scale: ext.taper.max(0.001),
            twist: ext.twist_deg,
        });
    }

    struct FaceCandidate {
        path: skia::Path,
        color: crate::core::Color,
        avg_z: f32,
        is_front_cap: bool,
    }

    let mut face_candidates: Vec<FaceCandidate> = Vec::new();
    let mut all_rings: Vec<Vec<Vec<(Vec3, skia::Point)>>> = Vec::with_capacity(loops.len());

    for loop_pts in &loops {
        let v_normals = compute_inward_vertex_normals(loop_pts);
        let mut rings: Vec<Vec<(Vec3, skia::Point)>> = Vec::with_capacity(ring_levels.len());
        for rl in &ring_levels {
            let mut ring_pts = Vec::with_capacity(loop_pts.len());
            for (i, &p) in loop_pts.iter().enumerate() {
                let offset_p = skia::Point::new(
                    p.x + v_normals[i].x * rl.inset_dist,
                    p.y + v_normals[i].y * rl.inset_dist,
                );
                let (v3d, p2d) =
                    transform_pt_to_3d_and_2d(offset_p, rl.local_z, rl.taper_scale, rl.twist);
                ring_pts.push((v3d, p2d));
            }
            rings.push(ring_pts);
        }
        all_rings.push(rings);
    }

    // Per-glyph Front Caps (with counter-hole cutouts via EvenOdd)
    if n_front.dot(&view_dir) > -0.001 {
        for (outer_idx, holes) in &glyphs {
            let mut cap_b = skia::PathBuilder::new();
            let outer_first_ring = &all_rings[*outer_idx][0];
            if let Some(p_start) = outer_first_ring.first() {
                cap_b.move_to(p_start.1);
                for pt in &outer_first_ring[1..] {
                    cap_b.line_to(pt.1);
                }
                cap_b.close();
            }
            for &h_idx in holes {
                let hole_first_ring = &all_rings[h_idx][0];
                if let Some(p_start) = hole_first_ring.first() {
                    cap_b.move_to(p_start.1);
                    for pt in &hole_first_ring[1..] {
                        cap_b.line_to(pt.1);
                    }
                    cap_b.close();
                }
            }
            let mut cap_path = cap_b.detach();
            cap_path.set_fill_type(skia::PathFillType::EvenOdd);
            let cap_z = outer_first_ring.iter().map(|p| p.0.z).sum::<f32>() / outer_first_ring.len().max(1) as f32;
            face_candidates.push(FaceCandidate {
                path: cap_path,
                color: compute_color(base_fill, n_front),
                avg_z: cap_z + 0.05,
                is_front_cap: true,
            });
        }
    }

    // Per-glyph Back Caps (only if facing viewer)
    if n_back.dot(&view_dir) > 0.001 {
        for (outer_idx, holes) in &glyphs {
            let mut cap_b = skia::PathBuilder::new();
            let outer_last_ring = all_rings[*outer_idx].last().unwrap();
            if let Some(p_start) = outer_last_ring.first() {
                cap_b.move_to(p_start.1);
                for pt in &outer_last_ring[1..] {
                    cap_b.line_to(pt.1);
                }
                cap_b.close();
            }
            for &h_idx in holes {
                let hole_last_ring = all_rings[h_idx].last().unwrap();
                if let Some(p_start) = hole_last_ring.first() {
                    cap_b.move_to(p_start.1);
                    for pt in &hole_last_ring[1..] {
                        cap_b.line_to(pt.1);
                    }
                    cap_b.close();
                }
            }
            let mut cap_path = cap_b.detach();
            cap_path.set_fill_type(skia::PathFillType::EvenOdd);
            let cap_z = outer_last_ring.iter().map(|p| p.0.z).sum::<f32>() / outer_last_ring.len().max(1) as f32;
            face_candidates.push(FaceCandidate {
                path: cap_path,
                color: compute_color(side_base, n_back),
                avg_z: cap_z - 0.05,
                is_front_cap: false,
            });
        }
    }

    // Side Wall Quads with Back-Face Culling
    for (l_idx, loop_pts) in loops.iter().enumerate() {
        let is_hole = glyphs.iter().any(|(_, holes)| holes.contains(&l_idx));
        let rings = &all_rings[l_idx];
        let n_pts = loop_pts.len();
        if rings.len() >= 2 {
            for r in 0..(rings.len() - 1) {
                let ring_a = &rings[r];
                let ring_b = &rings[r + 1];

                for i in 0..n_pts {
                    let next_i = (i + 1) % n_pts;
                    let p0 = ring_a[i];
                    let p1 = ring_a[next_i];
                    let p2 = ring_b[next_i];
                    let p3 = ring_b[i];

                    let e1 = p1.0 - p0.0;
                    let e2 = p3.0 - p0.0;
                    let normal = if !is_hole {
                        e2.cross(&e1).normalize()
                    } else {
                        e1.cross(&e2).normalize()
                    };

                    // Back-face culling: skip faces pointing away from camera
                    if normal.dot(&view_dir) <= 0.001 {
                        continue;
                    }

                    let avg_z = (p0.0.z + p1.0.z + p2.0.z + p3.0.z) * 0.25;

                    let mut quad_builder = skia::PathBuilder::new();
                    quad_builder.move_to(p0.1);
                    quad_builder.line_to(p1.1);
                    quad_builder.line_to(p2.1);
                    quad_builder.line_to(p3.1);
                    quad_builder.close();

                    let col = compute_color(side_base, normal);
                    face_candidates.push(FaceCandidate {
                        path: quad_builder.detach(),
                        color: col,
                        avg_z,
                        is_front_cap: false,
                    });
                }
            }
        }
    }

    face_candidates.sort_by(|a, b| {
        a.avg_z
            .partial_cmp(&b.avg_z)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    face_candidates
        .into_iter()
        .map(|f| ExtrudedFaceElement {
            path: f.path,
            color: f.color,
            is_front_cap: f.is_front_cap,
        })
        .collect()
}

struct Face3D {
    polygon_2d: skia::Path,
    normal: Vec3,
    avg_z: f32,
    base_color: crate::core::Color,
    is_front_cap: bool,
}

/// Renders a full 3D extruded solid with true topological quad mesh faces,
/// depth-sorted painter's algorithm rendering, and PBR shading.
pub fn apply_extrude_3d_to_canvas(
    path: &skia::Path,
    fill_color: Option<crate::core::Color>,
    stroke_color: Option<crate::core::Color>,
    stroke_width: f32,
    ext: &Extrude3DModifier,
    canvas: &skia::Canvas,
) {
    if !ext.enabled || ext.depth.abs() < 0.1 {
        let mut paint = skia::Paint::default();
        if let Some(fc) = fill_color {
            paint.set_color4f(fc.to_skia(), None);
            paint.set_style(skia::PaintStyle::Fill);
            paint.set_anti_alias(true);
            canvas.draw_path(path, &paint);
        }
        if let Some(sc) = stroke_color {
            paint.set_color4f(sc.to_skia(), None);
            paint.set_style(skia::PaintStyle::Stroke);
            paint.set_stroke_width(stroke_width);
            paint.set_anti_alias(true);
            canvas.draw_path(path, &paint);
        }
        return;
    }

    let bounds = path.bounds();
    let global_center = Point::new(bounds.center_x(), bounds.center_y());
    let max_dim = bounds.width().min(bounds.height()).max(10.0);

    let (rot_x, rot_y, rot_z, perspective) = match ext.mode {
        Extrude3DMode::Isometric => (35.264, -45.0, 0.0, 0.0),
        Extrude3DMode::Cabinet => (0.0, 0.0, 0.0, 0.0),
        Extrude3DMode::Perspective => (20.0, -30.0, 0.0, 600.0),
        Extrude3DMode::Custom3D => (ext.rot_x, ext.rot_y, ext.rot_z, ext.perspective),
    };

    let bevel_r = ext.bevel_radius.min(ext.depth * 0.45).min(max_dim * 0.35);
    let rounding_r = ext.corner_radius_2d.max(if bevel_r > 0.5 { bevel_r.min(15.0) } else { 0.0 });
    let rounded_path = if rounding_r > 0.1 {
        apply_chamfer_rounding_to_path(path, rounding_r, CornerStyle::Round)
    } else {
        path.clone()
    };

    let loops = extract_subpath_polygon_loops(&rounded_path);
    if loops.is_empty() {
        return;
    }
    let glyphs = group_loops_into_glyphs(&loops);

    canvas.save();

    let base_fill = fill_color.unwrap_or(crate::core::Color::new(0.3, 0.55, 0.9, 1.0));
    let side_base = ext.custom_side_color.unwrap_or_else(|| {
        crate::core::Color::new(
            base_fill.r * 0.75,
            base_fill.g * 0.75,
            base_fill.b * 0.75,
            base_fill.a,
        )
    });

    let (light_dir, light_intensity) = if ext.shading {
        let (sx, cx) = ext.light_elevation_deg.to_radians().sin_cos();
        let (sy, cy) = ext.light_angle_deg.to_radians().sin_cos();
        let dir = Vec3::new(cy * cx, sy * cx, sx).normalize();
        (dir, 1.0f32)
    } else {
        (Vec3::new(0.0, 0.0, 1.0), 0.0f32)
    };

    let view_dir = Vec3::new(0.0, 0.0, 1.0);

    let n_front = Vec3::new(0.0, 0.0, 1.0)
        .rotate_euler(rot_x, rot_y, rot_z)
        .normalize();
    let n_back = Vec3::new(0.0, 0.0, -1.0)
        .rotate_euler(rot_x, rot_y, rot_z)
        .normalize();

    let compute_face_paint = |base: crate::core::Color, normal: Vec3| -> skia::Paint {
        let mut paint = skia::Paint::default();
        paint.set_anti_alias(true);
        paint.set_style(skia::PaintStyle::Fill);

        if !ext.shading || light_intensity <= 0.01 {
            paint.set_color4f(base.to_skia(), None);
            return paint;
        }

        let base_r = base.r.max(0.08);
        let base_g = base.g.max(0.08);
        let base_b = base.b.max(0.08);

        let n_dot_l = normal.dot(&light_dir).max(0.0);
        let ambient = 0.32;
        let diffuse = 0.58 * n_dot_l;

        let half_vec = (light_dir + view_dir).normalize();
        let n_dot_h = normal.dot(&half_vec).max(0.0);
        let shininess = (1.0 - ext.roughness * 0.85).max(0.05) * 64.0;
        let specular_intensity = n_dot_h.powf(shininess) * ext.gloss_specular;

        let n_dot_v = normal.dot(&view_dir).abs();
        let fresnel = (1.0 - n_dot_v).powi(3) * ext.rim_light * 0.55;

        let total_light = ambient + diffuse + fresnel;

        let lit_r = (base_r * total_light * (1.0 - ext.metallic * 0.4)
            + specular_intensity * (if ext.metallic > 0.5 { base_r } else { 1.0 }))
        .clamp(0.0, 1.0);
        let lit_g = (base_g * total_light * (1.0 - ext.metallic * 0.4)
            + specular_intensity * (if ext.metallic > 0.5 { base_g } else { 1.0 }))
        .clamp(0.0, 1.0);
        let lit_b = (base_b * total_light * (1.0 - ext.metallic * 0.4)
            + specular_intensity * (if ext.metallic > 0.5 { base_b } else { 1.0 }))
        .clamp(0.0, 1.0);

        let shaded_color = crate::core::Color::new(lit_r, lit_g, lit_b, base.a);
        paint.set_color4f(shaded_color.to_skia(), None);
        paint
    };

    let transform_pt_to_3d_and_2d = |p: skia::Point,
                                     local_z: f32,
                                     taper_s: f32,
                                     twist_deg: f32|
     -> (Vec3, skia::Point) {
        let dx = (p.x - global_center.x) * taper_s;
        let dy = (p.y - global_center.y) * taper_s;

        let (dx_t, dy_t) = if twist_deg.abs() > 0.01 {
            let t_rad = twist_deg.to_radians();
            let (st, ct) = t_rad.sin_cos();
            (dx * ct - dy * st, dx * st + dy * ct)
        } else {
            (dx, dy)
        };

        let v3d = Vec3::new(dx_t, dy_t, local_z).rotate_euler(rot_x, rot_y, rot_z);
        let proj = v3d.project_to_screen(global_center, perspective);
        (v3d, skia::Point::new(proj.x, proj.y))
    };

    struct RingLevel {
        local_z: f32,
        inset_dist: f32,
        taper_scale: f32,
        twist: f32,
    }

    let bevel_r = ext.bevel_radius.min(ext.depth * 0.45).min(max_dim * 0.35);
    let has_bevel = bevel_r > 0.5;

    let mut ring_levels: Vec<RingLevel> = Vec::new();

    if has_bevel {
        let bevel_steps = 6usize;
        // Front shoulder fillet
        for k in 0..=bevel_steps {
            let u = k as f32 / bevel_steps as f32;
            let theta = u * std::f32::consts::FRAC_PI_2;
            let (z_off, d_off) = match ext.bevel_style {
                Bevel3DStyle::Round => {
                    let z = -bevel_r * (1.0 - theta.cos());
                    let d = bevel_r * theta.cos();
                    (z, d)
                }
                Bevel3DStyle::Chamfer => {
                    let z = -bevel_r * u;
                    let d = bevel_r * (1.0 - u);
                    (z, d)
                }
                Bevel3DStyle::Convex => {
                    let z = -bevel_r * (1.0 - theta.cos());
                    let d = bevel_r * theta.cos() * (1.0 + 0.15 * (2.0 * theta).sin());
                    (z, d)
                }
                Bevel3DStyle::Steps => {
                    let step = (u * 3.0).floor() / 3.0;
                    let z = -bevel_r * step;
                    let d = bevel_r * (1.0 - step);
                    (z, d)
                }
            };
            let norm_t = (z_off.abs() / ext.depth.max(0.1)).clamp(0.0, 1.0);
            let taper = 1.0 - (1.0 - ext.taper) * norm_t;
            let twist = ext.twist_deg * norm_t;
            ring_levels.push(RingLevel {
                local_z: z_off,
                inset_dist: d_off.max(0.0),
                taper_scale: taper.max(0.001),
                twist,
            });
        }

        // Straight body between front and rear shoulders
        if ext.depth > 2.0 * bevel_r + 1.0 {
            let z_body = -bevel_r + (-ext.depth + 2.0 * bevel_r) * 0.5;
            let norm_t = (z_body.abs() / ext.depth.max(0.1)).clamp(0.0, 1.0);
            let taper = 1.0 - (1.0 - ext.taper) * norm_t;
            let twist = ext.twist_deg * norm_t;
            ring_levels.push(RingLevel {
                local_z: z_body,
                inset_dist: 0.0,
                taper_scale: taper.max(0.001),
                twist,
            });
        }

        // Rear shoulder fillet
        for k in 1..=bevel_steps {
            let u = k as f32 / bevel_steps as f32;
            let theta = u * std::f32::consts::FRAC_PI_2;
            let (z_off, d_off) = match ext.bevel_style {
                Bevel3DStyle::Round => {
                    let z = -ext.depth + bevel_r * (1.0 - theta.sin());
                    let d = bevel_r * (1.0 - theta.cos());
                    (z, d)
                }
                Bevel3DStyle::Chamfer => {
                    let z = -ext.depth + bevel_r * (1.0 - u);
                    let d = bevel_r * u;
                    (z, d)
                }
                Bevel3DStyle::Convex => {
                    let z = -ext.depth + bevel_r * (1.0 - theta.sin());
                    let d = bevel_r * (1.0 - theta.cos()) * (1.0 + 0.15 * (2.0 * theta).sin());
                    (z, d)
                }
                Bevel3DStyle::Steps => {
                    let step = ((1.0 - u) * 3.0).floor() / 3.0;
                    let z = -ext.depth + bevel_r * step;
                    let d = bevel_r * (1.0 - step);
                    (z, d)
                }
            };
            let norm_t = (z_off.abs() / ext.depth.max(0.1)).clamp(0.0, 1.0);
            let taper = 1.0 - (1.0 - ext.taper) * norm_t;
            let twist = ext.twist_deg * norm_t;
            ring_levels.push(RingLevel {
                local_z: z_off,
                inset_dist: d_off.max(0.0),
                taper_scale: taper.max(0.001),
                twist,
            });
        }
    } else {
        ring_levels.push(RingLevel {
            local_z: 0.0,
            inset_dist: 0.0,
            taper_scale: 1.0,
            twist: 0.0,
        });
        let has_twist_or_taper = ext.twist_deg.abs() > 0.01 || (ext.taper - 1.0).abs() > 0.01;
        if has_twist_or_taper {
            for k in 1..3 {
                let t = k as f32 / 3.0;
                ring_levels.push(RingLevel {
                    local_z: -ext.depth * t,
                    inset_dist: 0.0,
                    taper_scale: (1.0 - (1.0 - ext.taper) * t).max(0.001),
                    twist: ext.twist_deg * t,
                });
            }
        }
        ring_levels.push(RingLevel {
            local_z: -ext.depth,
            inset_dist: 0.0,
            taper_scale: ext.taper.max(0.001),
            twist: ext.twist_deg,
        });
    }

    let mut faces: Vec<Face3D> = Vec::new();
    let mut all_rings: Vec<Vec<Vec<(Vec3, skia::Point)>>> = Vec::with_capacity(loops.len());

    for loop_pts in &loops {
        let v_normals = compute_inward_vertex_normals(loop_pts);
        let mut rings: Vec<Vec<(Vec3, skia::Point)>> = Vec::with_capacity(ring_levels.len());
        for rl in &ring_levels {
            let mut ring_pts = Vec::with_capacity(loop_pts.len());
            for (i, &p) in loop_pts.iter().enumerate() {
                let offset_p = skia::Point::new(
                    p.x + v_normals[i].x * rl.inset_dist,
                    p.y + v_normals[i].y * rl.inset_dist,
                );
                let (v3d, p2d) =
                    transform_pt_to_3d_and_2d(offset_p, rl.local_z, rl.taper_scale, rl.twist);
                ring_pts.push((v3d, p2d));
            }
            rings.push(ring_pts);
        }
        all_rings.push(rings);
    }

    // Per-glyph Front Caps (with counter-hole cutouts via EvenOdd)
    if n_front.dot(&view_dir) > -0.001 {
        for (outer_idx, holes) in &glyphs {
            let mut cap_b = skia::PathBuilder::new();
            let outer_first_ring = &all_rings[*outer_idx][0];
            if let Some(p_start) = outer_first_ring.first() {
                cap_b.move_to(p_start.1);
                for pt in &outer_first_ring[1..] {
                    cap_b.line_to(pt.1);
                }
                cap_b.close();
            }
            for &h_idx in holes {
                let hole_first_ring = &all_rings[h_idx][0];
                if let Some(p_start) = hole_first_ring.first() {
                    cap_b.move_to(p_start.1);
                    for pt in &hole_first_ring[1..] {
                        cap_b.line_to(pt.1);
                    }
                    cap_b.close();
                }
            }
            let mut cap_path = cap_b.detach();
            cap_path.set_fill_type(skia::PathFillType::EvenOdd);
            let cap_z = outer_first_ring.iter().map(|p| p.0.z).sum::<f32>() / outer_first_ring.len().max(1) as f32;
            faces.push(Face3D {
                polygon_2d: cap_path,
                normal: n_front,
                avg_z: cap_z + 0.05,
                base_color: base_fill,
                is_front_cap: true,
            });
        }
    }

    // Per-glyph Back Caps (only if facing viewer)
    if n_back.dot(&view_dir) > 0.001 {
        for (outer_idx, holes) in &glyphs {
            let mut cap_b = skia::PathBuilder::new();
            let outer_last_ring = all_rings[*outer_idx].last().unwrap();
            if let Some(p_start) = outer_last_ring.first() {
                cap_b.move_to(p_start.1);
                for pt in &outer_last_ring[1..] {
                    cap_b.line_to(pt.1);
                }
                cap_b.close();
            }
            for &h_idx in holes {
                let hole_last_ring = all_rings[h_idx].last().unwrap();
                if let Some(p_start) = hole_last_ring.first() {
                    cap_b.move_to(p_start.1);
                    for pt in &hole_last_ring[1..] {
                        cap_b.line_to(pt.1);
                    }
                    cap_b.close();
                }
            }
            let mut cap_path = cap_b.detach();
            cap_path.set_fill_type(skia::PathFillType::EvenOdd);
            let cap_z = outer_last_ring.iter().map(|p| p.0.z).sum::<f32>() / outer_last_ring.len().max(1) as f32;
            faces.push(Face3D {
                polygon_2d: cap_path,
                normal: n_back,
                avg_z: cap_z - 0.05,
                base_color: side_base,
                is_front_cap: false,
            });
        }
    }

    // Side Wall Quads with Back-Face Culling
    for (l_idx, loop_pts) in loops.iter().enumerate() {
        let is_hole = glyphs.iter().any(|(_, holes)| holes.contains(&l_idx));
        let rings = &all_rings[l_idx];
        let n_pts = loop_pts.len();
        if rings.len() >= 2 {
            for r in 0..(rings.len() - 1) {
                let ring_a = &rings[r];
                let ring_b = &rings[r + 1];

                for i in 0..n_pts {
                    let next_i = (i + 1) % n_pts;
                    let p0 = ring_a[i];
                    let p1 = ring_a[next_i];
                    let p2 = ring_b[next_i];
                    let p3 = ring_b[i];

                    let e1 = p1.0 - p0.0;
                    let e2 = p3.0 - p0.0;
                    let normal = if !is_hole {
                        e2.cross(&e1).normalize()
                    } else {
                        e1.cross(&e2).normalize()
                    };

                    // Back-face culling: skip faces pointing away from camera
                    if normal.dot(&view_dir) <= 0.001 {
                        continue;
                    }

                    let avg_z = (p0.0.z + p1.0.z + p2.0.z + p3.0.z) * 0.25;

                    let mut quad_builder = skia::PathBuilder::new();
                    quad_builder.move_to(p0.1);
                    quad_builder.line_to(p1.1);
                    quad_builder.line_to(p2.1);
                    quad_builder.line_to(p3.1);
                    quad_builder.close();

                    faces.push(Face3D {
                        polygon_2d: quad_builder.detach(),
                        normal,
                        avg_z,
                        base_color: side_base,
                        is_front_cap: false,
                    });
                }
            }
        }
    }

    faces.sort_by(|a, b| {
        a.avg_z
            .partial_cmp(&b.avg_z)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for face in faces {
        let paint = compute_face_paint(face.base_color, face.normal);
        canvas.draw_path(&face.polygon_2d, &paint);

        if face.is_front_cap && stroke_color.is_some() && stroke_width > 0.05 {
            if let Some(sc) = stroke_color {
                let mut stroke_paint = skia::Paint::default();
                stroke_paint.set_style(skia::PaintStyle::Stroke);
                stroke_paint.set_stroke_width(stroke_width);
                stroke_paint.set_anti_alias(true);
                stroke_paint.set_color4f(sc.to_skia(), None);
                canvas.draw_path(&face.polygon_2d, &stroke_paint);
            }
        }
    }

    canvas.restore();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations_and_euler_rotation() {
        let v = Vec3::new(1.0, 0.0, 0.0);
        // Rotate 90 degrees around Z axis -> (0, 1, 0)
        let rz = v.rotate_euler(0.0, 0.0, 90.0);
        assert!((rz.x).abs() < 0.001);
        assert!((rz.y - 1.0).abs() < 0.001);
        assert!((rz.z).abs() < 0.001);

        // Rotate 90 degrees around Y axis -> (0, 0, -1)
        let ry = v.rotate_euler(0.0, 90.0, 0.0);
        assert!((ry.x).abs() < 0.001);
        assert!((ry.y).abs() < 0.001);
        assert!((ry.z - (-1.0)).abs() < 0.001);

        // Normalization
        let v_norm = Vec3::new(3.0, 4.0, 0.0).normalize();
        assert!((v_norm.x - 0.6).abs() < 0.001);
        assert!((v_norm.y - 0.8).abs() < 0.001);

        // Dot product
        let d = Vec3::new(1.0, 2.0, 3.0).dot(&Vec3::new(4.0, -5.0, 6.0));
        assert_eq!(d, 4.0 - 10.0 + 18.0);

        // Cross product
        let c = Vec3::new(1.0, 0.0, 0.0).cross(&Vec3::new(0.0, 1.0, 0.0));
        assert!((c.x).abs() < 0.001);
        assert!((c.y).abs() < 0.001);
        assert!((c.z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_extract_subpath_polygon_loops() {
        let mut builder = skia::PathBuilder::new();
        builder.add_rect(
            skia::Rect::from_xywh(10.0, 10.0, 50.0, 50.0),
            Some(skia::PathDirection::CW),
            0,
        );
        builder.add_rect(
            skia::Rect::from_xywh(70.0, 10.0, 50.0, 50.0),
            Some(skia::PathDirection::CW),
            0,
        );
        let path = builder.detach();

        let loops = extract_subpath_polygon_loops(&path);
        assert_eq!(loops.len(), 2);
        assert!(loops[0].len() >= 4);
        assert!(loops[1].len() >= 4);
    }

    #[test]
    fn test_vec3_perspective_projection() {
        let center = Point::new(100.0, 100.0);
        let v_front = Vec3::new(20.0, 30.0, 0.0);
        let p_front = v_front.project_to_screen(center, 600.0);
        assert_eq!(p_front.x, 120.0);
        assert_eq!(p_front.y, 130.0);

        // Point closer to camera (positive Z) appears larger (foreshortened)
        let v_near = Vec3::new(20.0, 30.0, 300.0);
        let p_near = v_near.project_to_screen(center, 600.0);
        assert_eq!(p_near.x, 140.0);
        assert_eq!(p_near.y, 160.0);
    }

    #[test]
    fn test_transform_skia_path_3d() {
        let mut builder = skia::PathBuilder::new();
        builder.move_to(skia::Point::new(0.0, 0.0));
        builder.line_to(skia::Point::new(100.0, 0.0));
        builder.line_to(skia::Point::new(100.0, 100.0));
        builder.close();
        let path = builder.detach();

        let center = Point::new(50.0, 50.0);
        let transformed =
            transform_skia_path_3d(&path, center, 0.0, 1.0, 0.0, 45.0, -30.0, 0.0, 800.0);
        assert!(transformed.count_points() >= 3);
        assert!(transformed.is_last_contour_closed());
    }

    #[test]
    fn test_apply_extrude_3d_to_canvas_no_panic() {
        let mut builder = skia::PathBuilder::new();
        builder.add_rect(
            skia::Rect::from_xywh(20.0, 20.0, 80.0, 80.0),
            Some(skia::PathDirection::CW),
            0,
        );
        let path = builder.detach();

        let ext = Extrude3DModifier {
            enabled: true,
            rot_x: 25.0,
            rot_y: -40.0,
            rot_z: 10.0,
            perspective: 600.0,
            depth: 35.0,
            taper: 0.9,
            twist_deg: 15.0,
            corner_radius_2d: 5.0,
            bevel_style: Bevel3DStyle::Round,
            bevel_radius: 2.0,
            bevel_segments: 6,
            shading: true,
            shading_intensity: 0.7,
            ambient_light: 0.4,
            light_angle_deg: 120.0,
            light_elevation_deg: 50.0,
            gloss_specular: 0.5,
            material_preset: Material3DPreset::Default,
            metallic: 0.0,
            roughness: 0.4,
            rim_light: 0.3,
            custom_side_color: Some(crate::core::Color::RED),
            mode: Extrude3DMode::Custom3D,
            angle_deg: 45.0,
        };

        let mut surface = skia::surfaces::raster_n32_premul((200, 200)).unwrap();
        apply_extrude_3d_to_canvas(
            &path,
            Some(crate::core::Color::BLUE),
            Some(crate::core::Color::WHITE),
            2.0,
            &ext,
            surface.canvas(),
        );

        let solid_path = apply_extrude_3d_to_path(&path, &ext);
        assert!(solid_path.count_points() > path.count_points());
    }

    #[test]
    fn test_material_presets_and_pbr_properties() {
        let mut ext = Extrude3DModifier::default();
        assert_eq!(ext.material_preset, Material3DPreset::Default);
        assert_eq!(ext.metallic, 0.0);
        assert_eq!(ext.roughness, 0.35);
        assert_eq!(ext.rim_light, 0.30);

        ext.material_preset = Material3DPreset::Gold;
        ext.metallic = 0.92;
        ext.roughness = 0.12;
        ext.rim_light = 0.60;
        ext.gloss_specular = 0.88;

        let json = serde_json::to_string(&ext).unwrap();
        let deserialized: Extrude3DModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.material_preset, Material3DPreset::Gold);
        assert_eq!(deserialized.metallic, 0.92);
        assert_eq!(deserialized.roughness, 0.12);
        assert_eq!(deserialized.rim_light, 0.60);
    }

    #[test]
    fn test_apply_chamfer_rounding_to_polygon_and_rect() {
        let mut builder = skia::PathBuilder::new();
        builder.move_to(skia::Point::new(0.0, 0.0));
        builder.line_to(skia::Point::new(100.0, 0.0));
        builder.line_to(skia::Point::new(100.0, 100.0));
        builder.line_to(skia::Point::new(0.0, 100.0));
        builder.close();
        let sharp_poly = builder.detach();

        let rounded = apply_chamfer_rounding_to_path(&sharp_poly, 15.0, CornerStyle::Round);
        assert!(rounded.count_points() > sharp_poly.count_points());

        let chamfered = apply_chamfer_rounding_to_path(&sharp_poly, 15.0, CornerStyle::Chamfer);
        assert!(chamfered.count_points() > sharp_poly.count_points());
    }
}
