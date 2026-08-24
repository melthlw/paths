use skia_safe as skia;

use crate::core::color::Color;
use crate::core::geometry::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl BlendMode {
    pub fn to_skia(self) -> skia::BlendMode {
        match self {
            BlendMode::Normal => skia::BlendMode::SrcOver,
            BlendMode::Multiply => skia::BlendMode::Multiply,
            BlendMode::Screen => skia::BlendMode::Screen,
            BlendMode::Overlay => skia::BlendMode::Overlay,
            BlendMode::Darken => skia::BlendMode::Darken,
            BlendMode::Lighten => skia::BlendMode::Lighten,
            BlendMode::ColorDodge => skia::BlendMode::ColorDodge,
            BlendMode::ColorBurn => skia::BlendMode::ColorBurn,
            BlendMode::HardLight => skia::BlendMode::HardLight,
            BlendMode::SoftLight => skia::BlendMode::SoftLight,
            BlendMode::Difference => skia::BlendMode::Difference,
            BlendMode::Exclusion => skia::BlendMode::Exclusion,
            BlendMode::Hue => skia::BlendMode::Hue,
            BlendMode::Saturation => skia::BlendMode::Saturation,
            BlendMode::Color => skia::BlendMode::Color,
            BlendMode::Luminosity => skia::BlendMode::Luminosity,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => BlendMode::Normal,
            1 => BlendMode::Multiply,
            2 => BlendMode::Screen,
            3 => BlendMode::Overlay,
            4 => BlendMode::Darken,
            5 => BlendMode::Lighten,
            6 => BlendMode::ColorDodge,
            7 => BlendMode::ColorBurn,
            8 => BlendMode::HardLight,
            9 => BlendMode::SoftLight,
            10 => BlendMode::Difference,
            11 => BlendMode::Exclusion,
            12 => BlendMode::Hue,
            13 => BlendMode::Saturation,
            14 => BlendMode::Color,
            15 => BlendMode::Luminosity,
            _ => BlendMode::Normal,
        }
    }

    pub fn to_index(self) -> u32 {
        match self {
            BlendMode::Normal => 0,
            BlendMode::Multiply => 1,
            BlendMode::Screen => 2,
            BlendMode::Overlay => 3,
            BlendMode::Darken => 4,
            BlendMode::Lighten => 5,
            BlendMode::ColorDodge => 6,
            BlendMode::ColorBurn => 7,
            BlendMode::HardLight => 8,
            BlendMode::SoftLight => 9,
            BlendMode::Difference => 10,
            BlendMode::Exclusion => 11,
            BlendMode::Hue => 12,
            BlendMode::Saturation => 13,
            BlendMode::Color => 14,
            BlendMode::Luminosity => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum StrokeStyle {
    #[default]
    Solid = 0,
    Dashed = 1,
    Dotted = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum FillStyle {
    #[default]
    Solid = 0,
    LinearGradient = 1,
    RadialGradient = 2,
    Mesh = 3,
    Pattern = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum PatternType {
    #[default]
    Grid = 0,
    Dots = 1,
    Stripes = 2,
    Checkerboard = 3,
    Hexagon = 4,
    Crosshatch = 5,
    Brick = 6,
    Scales = 7,
    Houndstooth = 8,
    Basketweave = 9,
}

impl PatternType {
    pub fn label(&self) -> String {
        match self {
            PatternType::Grid => crate::core::gettext("Technical Grid"),
            PatternType::Dots => crate::core::gettext("Halftone Dots"),
            PatternType::Stripes => crate::core::gettext("Diagonal Stripes"),
            PatternType::Checkerboard => crate::core::gettext("Checkerboard"),
            PatternType::Hexagon => crate::core::gettext("Honeycomb"),
            PatternType::Crosshatch => crate::core::gettext("Crosshatch"),
            PatternType::Brick => crate::core::gettext("Brick Wall"),
            PatternType::Scales => crate::core::gettext("Seigaiha Scales"),
            PatternType::Houndstooth => crate::core::gettext("Houndstooth"),
            PatternType::Basketweave => crate::core::gettext("Basketweave"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FillLayer {
    pub style: FillStyle,
    pub color: Color,
    pub secondary_color: Color,
    pub angle: f32,
    pub opacity: f32,
    pub enabled: bool,
    pub pattern_type: PatternType,
    pub pattern_scale: f32,
    pub mesh: Option<MeshGradient>,
}

impl Default for FillLayer {
    fn default() -> Self {
        Self {
            style: FillStyle::Solid,
            color: Color::BLACK,
            secondary_color: Color::WHITE,
            angle: 90.0,
            opacity: 1.0,
            enabled: true,
            pattern_type: PatternType::Checkerboard,
            pattern_scale: 16.0,
            mesh: None,
        }
    }
}

impl FillLayer {
    pub fn new(color: Color) -> Self {
        Self {
            style: FillStyle::Solid,
            color,
            secondary_color: Color::WHITE,
            angle: 0.0,
            opacity: 1.0,
            enabled: true,
            pattern_type: PatternType::Checkerboard,
            pattern_scale: 16.0,
            mesh: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_gradient(
        style: FillStyle,
        color: Color,
        secondary_color: Color,
        angle: f32,
    ) -> Self {
        Self {
            style,
            color,
            secondary_color,
            angle,
            opacity: 1.0,
            enabled: true,
            pattern_type: PatternType::Checkerboard,
            pattern_scale: 16.0,
            mesh: None,
        }
    }

    #[allow(dead_code)]
    pub fn new_pattern(pattern_type: PatternType, scale: f32, c1: Color, c2: Color) -> Self {
        Self {
            style: FillStyle::Pattern,
            color: c1,
            secondary_color: c2,
            angle: 0.0,
            opacity: 1.0,
            enabled: true,
            pattern_type,
            pattern_scale: scale,
            mesh: None,
        }
    }
}

pub fn create_pattern_shader(
    pattern_type: PatternType,
    c1: Color,
    c2: Color,
    scale: f32,
    angle: f32,
) -> Option<skia::Shader> {
    let sz = scale.clamp(4.0, 1024.0);
    let (tile_w, tile_h) = match pattern_type {
        PatternType::Hexagon => (sz, (sz * 1.7320508).max(4.0)),
        PatternType::Brick | PatternType::Scales => (sz, (sz * 0.5).max(4.0)),
        _ => (sz, sz),
    };
    let tile_rect = skia::Rect::from_xywh(0.0, 0.0, tile_w, tile_h);
    let mut recorder = skia::PictureRecorder::new();
    let canvas = recorder.begin_recording(tile_rect, false);

    let mut bg_paint = skia::Paint::default();
    bg_paint.set_color4f(c1.to_skia(), None);
    bg_paint.set_style(skia::PaintStyle::Fill);
    canvas.draw_rect(tile_rect, &bg_paint);

    let mut paint = skia::Paint::default();
    paint.set_color4f(c2.to_skia(), None);
    paint.set_anti_alias(true);

    match pattern_type {
        PatternType::Checkerboard => {
            let half = sz * 0.5;
            canvas.draw_rect(skia::Rect::from_xywh(0.0, 0.0, half, half), &paint);
            canvas.draw_rect(skia::Rect::from_xywh(half, half, half, half), &paint);
        }
        PatternType::Dots => {
            let r = sz * 0.22;
            canvas.draw_circle(skia::Point::new(sz * 0.5, sz * 0.5), r, &paint);
        }
        PatternType::Stripes => {
            paint.set_stroke_width((sz * 0.35).max(1.0));
            paint.set_style(skia::PaintStyle::Stroke);
            canvas.draw_line(skia::Point::new(0.0, 0.0), skia::Point::new(sz, sz), &paint);
            canvas.draw_line(
                skia::Point::new(-sz * 0.5, sz * 0.5),
                skia::Point::new(sz * 0.5, sz * 1.5),
                &paint,
            );
            canvas.draw_line(
                skia::Point::new(sz * 0.5, -sz * 0.5),
                skia::Point::new(sz * 1.5, sz * 0.5),
                &paint,
            );
        }
        PatternType::Grid => {
            paint.set_stroke_width((sz * 0.12).max(1.0));
            paint.set_style(skia::PaintStyle::Stroke);
            canvas.draw_rect(skia::Rect::from_xywh(0.0, 0.0, sz, sz), &paint);
        }
        PatternType::Hexagon => {
            // True seamless regular hexagonal honeycomb lattice (120° shared walls)
            paint.set_stroke_width((sz * 0.08).max(1.0));
            paint.set_style(skia::PaintStyle::Stroke);
            let w = tile_w;
            let h = tile_h;
            let half_w = w * 0.5;
            let h_6 = h / 6.0;
            let h_2 = h * 0.5;
            let h_23 = h * (2.0 / 3.0);

            let mut p1 = skia::PathBuilder::new();
            p1.move_to(skia::Point::new(0.0, 0.0));
            p1.line_to(skia::Point::new(half_w, h_6));
            p1.line_to(skia::Point::new(half_w, h_2));
            p1.line_to(skia::Point::new(0.0, h_23));
            p1.line_to(skia::Point::new(0.0, h));
            canvas.draw_path(&p1.detach(), &paint);

            let mut p2 = skia::PathBuilder::new();
            p2.move_to(skia::Point::new(w, 0.0));
            p2.line_to(skia::Point::new(half_w, h_6));
            canvas.draw_path(&p2.detach(), &paint);

            let mut p3 = skia::PathBuilder::new();
            p3.move_to(skia::Point::new(half_w, h_2));
            p3.line_to(skia::Point::new(w, h_23));
            p3.line_to(skia::Point::new(w, h));
            canvas.draw_path(&p3.detach(), &paint);
        }
        PatternType::Brick => {
            // True 50% staggered running bond brick wall
            paint.set_stroke_width((sz * 0.08).max(1.0));
            paint.set_style(skia::PaintStyle::Stroke);
            let w = tile_w;
            let h = tile_h;
            let half_h = h * 0.5;

            // 1. Horizontal course mortar lines
            canvas.draw_line(skia::Point::new(0.0, 0.0), skia::Point::new(w, 0.0), &paint);
            canvas.draw_line(skia::Point::new(0.0, half_h), skia::Point::new(w, half_h), &paint);

            // 2. Vertical joints: top course at 0 and W
            canvas.draw_line(skia::Point::new(0.0, 0.0), skia::Point::new(0.0, half_h), &paint);
            canvas.draw_line(skia::Point::new(w, 0.0), skia::Point::new(w, half_h), &paint);

            // 3. Vertical joint: bottom course staggered by 50% at W/2
            canvas.draw_line(skia::Point::new(w * 0.5, half_h), skia::Point::new(w * 0.5, h), &paint);
        }
        PatternType::Crosshatch => {
            paint.set_stroke_width((sz * 0.12).max(1.0));
            paint.set_style(skia::PaintStyle::Stroke);
            canvas.draw_line(skia::Point::new(0.0, 0.0), skia::Point::new(sz, sz), &paint);
            canvas.draw_line(skia::Point::new(0.0, sz), skia::Point::new(sz, 0.0), &paint);
        }
        PatternType::Scales => {
            // Authentic Japanese Seigaiha (concentric wave/fan fish scales)
            paint.set_stroke_width((sz * 0.07).max(1.0));
            paint.set_style(skia::PaintStyle::Stroke);
            let w = tile_w;
            let h = tile_h;
            let r_base = w * 0.5;
            let radii = [r_base, r_base * 0.70, r_base * 0.40];

            let draw_arcs = |canvas: &skia::Canvas, cx: f32, cy: f32| {
                for &r in &radii {
                    let oval = skia::Rect::from_xywh(cx - r, cy - r, r * 2.0, r * 2.0);
                    let mut path = skia::PathBuilder::new();
                    path.add_arc(oval, 180.0, 180.0);
                    canvas.draw_path(&path.detach(), &paint);
                }
            };

            // Bottom row center fan
            draw_arcs(canvas, w * 0.5, h);
            // Top row side fans
            draw_arcs(canvas, 0.0, 0.0);
            draw_arcs(canvas, w, 0.0);
            // Corner helper fans
            draw_arcs(canvas, 0.0, h);
            draw_arcs(canvas, w, h);
        }
        PatternType::Houndstooth => {
            let half = sz * 0.5;
            let mut path = skia::PathBuilder::new();
            path.move_to(skia::Point::new(0.0, 0.0));
            path.line_to(skia::Point::new(half, 0.0));
            path.line_to(skia::Point::new(sz, half));
            path.line_to(skia::Point::new(half, half));
            path.line_to(skia::Point::new(half, sz));
            path.line_to(skia::Point::new(0.0, half));
            path.close();
            canvas.draw_path(&path.detach(), &paint);
        }
        PatternType::Basketweave => {
            let half = sz * 0.5;
            let mut p_h = paint.clone();
            p_h.set_stroke_width((sz * 0.15).max(1.0));
            p_h.set_style(skia::PaintStyle::Stroke);
            canvas.draw_line(skia::Point::new(0.0, half * 0.5), skia::Point::new(half, half * 0.5), &p_h);
            canvas.draw_line(skia::Point::new(half * 0.5, half), skia::Point::new(half * 0.5, sz), &p_h);
            canvas.draw_line(skia::Point::new(half, sz * 0.75), skia::Point::new(sz, sz * 0.75), &p_h);
            canvas.draw_line(skia::Point::new(sz * 0.75, 0.0), skia::Point::new(sz * 0.75, half), &p_h);
        }
    }

    let picture = recorder.finish_recording_as_picture(None)?;
    let mut matrix = skia::Matrix::default();
    if angle.abs() > 0.01 {
        matrix.set_rotate(angle, Some(skia::Point::new(tile_w * 0.5, tile_h * 0.5)));
    }
    Some(picture.to_shader(
        (skia::TileMode::Repeat, skia::TileMode::Repeat),
        skia::FilterMode::Linear,
        Some(&matrix),
        Some(&tile_rect),
    ))
}

pub fn create_fill_paint(fill: &FillLayer, bounds: Rect) -> skia::Paint {
    let mut paint = skia::Paint::default();
    paint.set_style(skia::PaintStyle::Fill);
    paint.set_anti_alias(true);

    let c1 = fill.color.with_alpha(fill.color.a * fill.opacity);
    let c2 = fill
        .secondary_color
        .with_alpha(fill.secondary_color.a * fill.opacity);

    match fill.style {
        FillStyle::Solid => {
            paint.set_color4f(c1.to_skia(), None);
        }
        FillStyle::LinearGradient => {
            let angle_rad = fill.angle.to_radians();
            let cx = bounds.x + bounds.width * 0.5;
            let cy = bounds.y + bounds.height * 0.5;
            let dx = angle_rad.cos() * (bounds.width * 0.5).max(1.0);
            let dy = angle_rad.sin() * (bounds.height * 0.5).max(1.0);
            let start = skia::Point::new(cx - dx, cy - dy);
            let end = skia::Point::new(cx + dx, cy + dy);

            let colors = [c1.to_skia(), c2.to_skia()];
            let pos = [0.0f32, 1.0f32];
            let colors_desc = skia::gradient::Colors::new(
                &colors[..],
                Some(&pos[..]),
                skia::TileMode::Clamp,
                None,
            );
            let grad_desc = skia::gradient::Gradient::new(
                colors_desc,
                skia::gradient::Interpolation::default(),
            );
            if let Some(shader) =
                skia::gradient::shaders::linear_gradient((start, end), &grad_desc, None)
            {
                paint.set_shader(shader);
            } else {
                paint.set_color4f(c1.to_skia(), None);
            }
        }
        FillStyle::RadialGradient => {
            let cx = bounds.x + bounds.width * 0.5;
            let cy = bounds.y + bounds.height * 0.5;
            let center = skia::Point::new(cx, cy);
            let radius = (bounds.width.max(bounds.height) * 0.5).max(1.0);

            let colors = [c1.to_skia(), c2.to_skia()];
            let pos = [0.0f32, 1.0f32];
            let colors_desc = skia::gradient::Colors::new(
                &colors[..],
                Some(&pos[..]),
                skia::TileMode::Clamp,
                None,
            );
            let grad_desc = skia::gradient::Gradient::new(
                colors_desc,
                skia::gradient::Interpolation::default(),
            );
            if let Some(shader) =
                skia::gradient::shaders::radial_gradient((center, radius), &grad_desc, None)
            {
                paint.set_shader(shader);
            } else {
                paint.set_color4f(c1.to_skia(), None);
            }
        }
        FillStyle::Pattern => {
            if let Some(shader) =
                create_pattern_shader(fill.pattern_type, c1, c2, fill.pattern_scale, fill.angle)
            {
                paint.set_shader(shader);
            } else {
                paint.set_color4f(c1.to_skia(), None);
            }
        }
        FillStyle::Mesh => {
            paint.set_color4f(c1.to_skia(), None);
        }
    }
    paint
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StrokeLayer {
    pub style: StrokeStyle,
    pub color: Color,
    pub opacity: f32,
    pub width: f32,
    pub enabled: bool,
}

impl Default for StrokeLayer {
    fn default() -> Self {
        Self {
            style: StrokeStyle::Solid,
            color: Color::BLACK,
            opacity: 1.0,
            width: 2.0,
            enabled: true,
        }
    }
}

impl StrokeLayer {
    pub fn new(color: Color, width: f32) -> Self {
        Self {
            style: StrokeStyle::Solid,
            color,
            opacity: color.a,
            width: width.max(0.5),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GradientType {
    Linear,
    Radial,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GradientStop {
    pub offset: f32,
    pub color: Color,
}

impl GradientStop {
    pub fn new(offset: f32, color: Color) -> Self {
        Self { offset, color }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Gradient {
    pub kind: GradientType,
    pub start: Point,
    pub end: Point,
    pub stops: Vec<GradientStop>,
}

impl Gradient {
    pub fn new_linear(start: Point, end: Point, start_color: Color, end_color: Color) -> Self {
        Self {
            kind: GradientType::Linear,
            start,
            end,
            stops: vec![
                GradientStop::new(0.0, start_color),
                GradientStop::new(1.0, end_color),
            ],
        }
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.start.translate(dx, dy);
        self.end.translate(dx, dy);
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        self.start.x = origin.x + (self.start.x - origin.x) * sx;
        self.start.y = origin.y + (self.start.y - origin.y) * sy;
        self.end.x = origin.x + (self.end.x - origin.x) * sx;
        self.end.y = origin.y + (self.end.y - origin.y) * sy;
    }
}

pub fn create_skia_gradient_shader(grad: &Gradient) -> Option<skia::Shader> {
    if grad.stops.is_empty() {
        return None;
    }
    let colors: Vec<skia::Color4f> = grad.stops.iter().map(|s| s.color.to_skia()).collect();
    let pos: Vec<f32> = grad
        .stops
        .iter()
        .map(|s| s.offset.clamp(0.0, 1.0))
        .collect();

    let colors_desc =
        skia::gradient::Colors::new(&colors[..], Some(&pos[..]), skia::TileMode::Clamp, None);
    let grad_desc =
        skia::gradient::Gradient::new(colors_desc, skia::gradient::Interpolation::default());

    match grad.kind {
        GradientType::Linear => skia::gradient::shaders::linear_gradient(
            (grad.start.to_skia(), grad.end.to_skia()),
            &grad_desc,
            None,
        ),
        GradientType::Radial => {
            let center = grad.start.to_skia();
            let radius = grad.start.distance_to(grad.end).max(1.0);
            skia::gradient::shaders::radial_gradient((center, radius), &grad_desc, None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MeshNode {
    pub point: Point,
    pub color: Color,
}

impl MeshNode {
    pub fn new(point: Point, color: Color) -> Self {
        Self { point, color }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MeshGradient {
    pub rows: usize,
    pub cols: usize,
    pub nodes: Vec<MeshNode>,
}

impl MeshGradient {
    pub fn new_grid(
        rect: Rect,
        rows: usize,
        cols: usize,
        primary_color: Color,
        secondary_color: Color,
    ) -> Self {
        let mut nodes = Vec::with_capacity(rows * cols);
        let r = rect.normalize();
        for row in 0..rows {
            let v = if rows > 1 {
                row as f32 / (rows - 1) as f32
            } else {
                0.0
            };
            for col in 0..cols {
                let u = if cols > 1 {
                    col as f32 / (cols - 1) as f32
                } else {
                    0.0
                };
                let px = r.x + u * r.width;
                let py = r.y + v * r.height;
                let t = (u + v) / 2.0;
                let c = Color::new(
                    primary_color.r + t * (secondary_color.r - primary_color.r),
                    primary_color.g + t * (secondary_color.g - primary_color.g),
                    primary_color.b + t * (secondary_color.b - primary_color.b),
                    primary_color.a + t * (secondary_color.a - primary_color.a),
                );
                nodes.push(MeshNode::new(Point::new(px, py), c));
            }
        }
        Self { rows, cols, nodes }
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        for n in &mut self.nodes {
            n.point.translate(dx, dy);
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        for n in &mut self.nodes {
            n.point.x = origin.x + (n.point.x - origin.x) * sx;
            n.point.y = origin.y + (n.point.y - origin.y) * sy;
        }
    }
}

pub fn render_mesh_gradient(canvas: &skia::Canvas, clip_path: &skia::Path, mesh: &MeshGradient) {
    if mesh.rows < 2 || mesh.cols < 2 || mesh.nodes.len() != mesh.rows * mesh.cols {
        return;
    }
    canvas.save();
    canvas.clip_path(clip_path, None, true);

    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for n in &mesh.nodes {
        positions.push(n.point.to_skia());
        let c = n.color;
        let c_u8 = skia::Color::from_argb(
            (c.a * 255.0).round() as u8,
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
        );
        colors.push(c_u8);
    }

    for r in 0..(mesh.rows - 1) {
        for c in 0..(mesh.cols - 1) {
            let top_left = (r * mesh.cols + c) as u16;
            let top_right = (r * mesh.cols + c + 1) as u16;
            let bot_left = ((r + 1) * mesh.cols + c) as u16;
            let bot_right = ((r + 1) * mesh.cols + c + 1) as u16;

            indices.push(top_left);
            indices.push(top_right);
            indices.push(bot_left);

            indices.push(top_right);
            indices.push(bot_right);
            indices.push(bot_left);
        }
    }

    let vertices = skia::Vertices::new_copy(
        skia::vertices::VertexMode::Triangles,
        &positions,
        &positions,
        &colors,
        Some(&indices),
    );
    let mut paint = skia::Paint::default();
    paint.set_anti_alias(true);
    canvas.draw_vertices(&vertices, skia::BlendMode::SrcOver, &paint);

    canvas.restore();
}
