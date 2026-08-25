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
pub enum Modifier {
    Array(ArrayModifier),
    EnvelopeWarp(EnvelopeWarpModifier),
    ChamferRounding(ChamferRoundingModifier),
}

impl Modifier {
    pub fn name(&self) -> &'static str {
        match self {
            Modifier::Array(_) => "Array Modifier",
            Modifier::EnvelopeWarp(_) => "Envelope Warp Modifier",
            Modifier::ChamferRounding(_) => "Dynamic Chamfer & Rounding",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Modifier::Array(_) => "view-grid-symbolic",
            Modifier::EnvelopeWarp(_) => "transform-symbolic",
            Modifier::ChamferRounding(_) => "tool-node-symbolic",
        }
    }

    pub fn enabled(&self) -> bool {
        match self {
            Modifier::Array(m) => m.enabled,
            Modifier::EnvelopeWarp(m) => m.enabled,
            Modifier::ChamferRounding(m) => m.enabled,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        match self {
            Modifier::Array(m) => m.enabled = enabled,
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
