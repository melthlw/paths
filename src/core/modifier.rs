use skia_safe as skia;
use crate::core::element::CornerStyle;
use crate::core::geometry::{Point, Rect};

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Extrude3DMode {
    Isometric,
    Cabinet,
    Perspective,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Extrude3DModifier {
    pub enabled: bool,
    pub depth: f32,
    pub angle_deg: f32,
    pub shading: bool,
    pub shading_intensity: f32,
    pub custom_side_color: Option<crate::core::Color>,
    pub light_angle_deg: f32,
    pub mode: Extrude3DMode,
    pub taper: f32,
    pub twist_deg: f32,
    pub bevel_radius: f32,
    pub gloss_specular: f32,
}

impl Default for Extrude3DModifier {
    fn default() -> Self {
        Self {
            enabled: true,
            depth: 40.0,
            angle_deg: 45.0,
            shading: true,
            shading_intensity: 0.6,
            custom_side_color: None,
            light_angle_deg: 135.0,
            mode: Extrude3DMode::Isometric,
            taper: 1.0,
            twist_deg: 0.0,
            bevel_radius: 0.0,
            gloss_specular: 0.2,
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
    if radius <= 0.1 {
        return path.clone();
    }
    let effective_r = match style {
        CornerStyle::Round => radius,
        CornerStyle::Chamfer => radius * 0.7,
        CornerStyle::Concave => radius * 0.85,
    };
    if let Some(pe) = skia::PathEffect::corner_path(effective_r) {
        if let Some((mut builder, _)) = pe.filter_path(path, &skia::StrokeRec::new_fill(), &skia::Rect::default()) {
            return builder.detach();
        }
    }
    path.clone()
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

        Point::new(
            (1.0 - v) * top.x + v * bot.x,
            (1.0 - v) * top.y + v * bot.y,
        )
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

/// Extrudes a 2D vector path into a 3D isometric perspective solid
pub fn apply_extrude_3d_to_path(path: &skia::Path, depth: f32, angle_deg: f32) -> skia::Path {
    if depth.abs() < 0.5 {
        return path.clone();
    }
    let rad = angle_deg.to_radians();
    let dx = depth * rad.cos();
    let dy = depth * rad.sin();

    let mut builder = skia::PathBuilder::new();
    let pts = path.points();

    // 1. Add back offset shape
    for i in 0..pts.len() {
        let p = skia::Point::new(pts[i].x + dx, pts[i].y + dy);
        if i == 0 {
            builder.move_to(p);
        } else {
            builder.line_to(p);
        }
    }
    if path.is_last_contour_closed() {
        builder.close();
    }

    // 2. Connect side extrusion panels
    if pts.len() >= 2 {
        for i in 0..pts.len() - 1 {
            let p0 = pts[i];
            let p1 = pts[i + 1];

            builder.move_to(p0);
            builder.line_to(p1);
            builder.line_to(skia::Point::new(p1.x + dx, p1.y + dy));
            builder.line_to(skia::Point::new(p0.x + dx, p0.y + dy));
            builder.close();
        }
    }

    // 3. Add front shape
    for i in 0..pts.len() {
        let p = pts[i];
        if i == 0 {
            builder.move_to(p);
        } else {
            builder.line_to(p);
        }
    }
    if path.is_last_contour_closed() {
        builder.close();
    }

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
    let center = skia::Point::new(bounds.x() + bounds.width() / 2.0, bounds.y() + bounds.height() / 2.0);

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

        let rx = dx * cos_a - dy * sin_a + center.x;
        let ry = dx * sin_a + dy * cos_a + center.y;

        let twisted_pt = skia::Point::new(rx, ry);
        if i == 0 {
            builder.move_to(twisted_pt);
        } else {
            builder.line_to(twisted_pt);
        }
    }
    if path.is_last_contour_closed() {
        builder.close();
    }
    builder.detach()
}

/// Renders a 3D Extruded vector solid with ambient occlusion lighting shading directly onto Skia canvas
pub fn apply_extrude_3d_to_canvas(
    path: &skia::Path,
    fill_color: Option<crate::core::Color>,
    stroke_color: Option<crate::core::Color>,
    stroke_width: f32,
    ext: &Extrude3DModifier,
    canvas: &skia::Canvas,
) {
    if !ext.enabled || ext.depth.abs() < 0.5 {
        return;
    }
    let rad = ext.angle_deg.to_radians();
    let (base_dx, base_dy) = match ext.mode {
        Extrude3DMode::Cabinet => (ext.depth * 0.5 * rad.cos(), ext.depth * 0.5 * rad.sin()),
        Extrude3DMode::Perspective => (ext.depth * rad.cos(), ext.depth * rad.sin()),
        Extrude3DMode::Isometric => (ext.depth * rad.cos(), ext.depth * rad.sin()),
    };

    canvas.save();

    let default_fill = crate::core::Color::new(0.2, 0.5, 0.9, 1.0);
    let base_fill = fill_color.unwrap_or(default_fill);
    let side_base = ext.custom_side_color.unwrap_or(base_fill);

    let mut side_paint = skia::Paint::default();
    side_paint.set_style(skia::PaintStyle::Fill);
    side_paint.set_anti_alias(true);

    let mut stroke_side_paint = skia::Paint::default();
    if let Some(sc) = stroke_color {
        stroke_side_paint.set_color4f(sc.to_skia(), None);
        stroke_side_paint.set_style(skia::PaintStyle::Stroke);
        stroke_side_paint.set_stroke_width(stroke_width);
        stroke_side_paint.set_anti_alias(true);
    }

    let intensity = ext.shading_intensity.clamp(0.0, 1.0);
    let light_rad = ext.light_angle_deg.to_radians();
    let light_dot = (rad.cos() * light_rad.cos() + rad.sin() * light_rad.sin()).abs();

    // Fast adaptive step count (max 24 steps) for 60 FPS performance without lagging
    let steps = ((ext.depth.abs() / 3.0) as usize).clamp(6, 24);

    // 1. Render smooth side volume using adaptive steps
    for i in (1..=steps).rev() {
        let t = i as f32 / steps as f32;
        let step_dx = base_dx * t;
        let step_dy = base_dy * t;
        let scale = if ext.mode == Extrude3DMode::Perspective {
            1.0 - (1.0 - ext.taper) * t - 0.15 * t
        } else {
            1.0 - (1.0 - ext.taper) * t
        };

        canvas.save();
        canvas.translate((step_dx, step_dy));
        if (scale - 1.0).abs() > 0.001 {
            canvas.scale((scale, scale));
        }
        if ext.twist_deg.abs() > 0.01 {
            canvas.rotate(ext.twist_deg * t, None);
        }

        if ext.shading {
            let darkness = (1.0 - t * 0.65 * intensity * (0.6 + 0.4 * light_dot)).clamp(0.12, 1.0);
            let shaded_color = crate::core::Color::new(
                (side_base.r * darkness).clamp(0.0, 1.0),
                (side_base.g * darkness).clamp(0.0, 1.0),
                (side_base.b * darkness).clamp(0.0, 1.0),
                side_base.a,
            );
            side_paint.set_color4f(shaded_color.to_skia(), None);
        } else {
            side_paint.set_color4f(side_base.to_skia(), None);
        }
        canvas.draw_path(path, &side_paint);
        if stroke_color.is_some() {
            canvas.draw_path(path, &stroke_side_paint);
        }
        canvas.restore();
    }

    // 2. Render optional 3D Bevel edge highlight
    if ext.bevel_radius > 0.5 {
        let mut bevel_paint = skia::Paint::default();
        bevel_paint.set_style(skia::PaintStyle::Stroke);
        bevel_paint.set_stroke_width(ext.bevel_radius);
        bevel_paint.set_anti_alias(true);
        let bevel_col = crate::core::Color::new(
            (base_fill.r + 0.3 * ext.gloss_specular).clamp(0.0, 1.0),
            (base_fill.g + 0.3 * ext.gloss_specular).clamp(0.0, 1.0),
            (base_fill.b + 0.3 * ext.gloss_specular).clamp(0.0, 1.0),
            0.8,
        );
        bevel_paint.set_color4f(bevel_col.to_skia(), None);
        canvas.draw_path(path, &bevel_paint);
    }

    canvas.restore();
}
