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
    Custom = 10,
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
            PatternType::Custom => crate::core::gettext("Custom Pattern"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FillLayer {
    pub style: FillStyle,
    pub color: Color,
    pub secondary_color: Color,
    #[serde(default)]
    pub stops: Vec<GradientStop>,
    pub angle: f32,
    pub opacity: f32,
    pub enabled: bool,
    pub pattern_type: PatternType,
    pub pattern_scale: f32,
    #[serde(default)]
    pub pattern_offset: Point,
    #[serde(default)]
    pub custom_pattern_path: Option<String>,
    pub mesh: Option<MeshGradient>,
}

impl Default for FillLayer {
    fn default() -> Self {
        Self {
            style: FillStyle::Solid,
            color: Color::BLACK,
            secondary_color: Color::WHITE,
            stops: Vec::new(),
            angle: 90.0,
            opacity: 1.0,
            enabled: true,
            pattern_type: PatternType::Checkerboard,
            pattern_scale: 16.0,
            pattern_offset: Point::ZERO,
            custom_pattern_path: None,
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
            stops: Vec::new(),
            angle: 0.0,
            opacity: 1.0,
            enabled: true,
            pattern_type: PatternType::Checkerboard,
            pattern_scale: 16.0,
            pattern_offset: Point::ZERO,
            custom_pattern_path: None,
            mesh: None,
        }
    }

    pub fn effective_stops(&self) -> Vec<GradientStop> {
        if self.stops.len() >= 2 {
            self.stops.clone()
        } else {
            vec![
                GradientStop::new(0.0, self.color),
                GradientStop::new(1.0, self.secondary_color),
            ]
        }
    }

    pub fn sample_at(&self, point: Point, bounds: Rect) -> Color {
        let base_c = self.color;
        match self.style {
            FillStyle::Solid => base_c.with_alpha(base_c.a * self.opacity),
            FillStyle::LinearGradient => {
                let angle_rad = self.angle.to_radians();
                let cx = bounds.x + bounds.width * 0.5;
                let cy = bounds.y + bounds.height * 0.5;
                let dx = angle_rad.cos() * (bounds.width * 0.5).max(1.0);
                let dy = angle_rad.sin() * (bounds.height * 0.5).max(1.0);
                let start = Point::new(cx - dx, cy - dy);
                let end = Point::new(cx + dx, cy + dy);

                let grad = Gradient {
                    start,
                    end,
                    stops: self.effective_stops(),
                    kind: GradientType::Linear,
                };
                let sampled = grad.sample_at(point);
                sampled.with_alpha(sampled.a * self.opacity)
            }
            FillStyle::RadialGradient => {
                let cx = bounds.x + bounds.width * 0.5;
                let cy = bounds.y + bounds.height * 0.5;
                let center = Point::new(cx, cy);
                let radius = (bounds.width.max(bounds.height) * 0.5).max(1.0);
                let end = Point::new(cx + radius, cy);

                let grad = Gradient {
                    start: center,
                    end,
                    stops: self.effective_stops(),
                    kind: GradientType::Radial,
                };
                let sampled = grad.sample_at(point);
                sampled.with_alpha(sampled.a * self.opacity)
            }
            FillStyle::Mesh => {
                if let Some(m) = &self.mesh {
                    m.sample_at(point)
                } else {
                    base_c.with_alpha(base_c.a * self.opacity)
                }
            }
            FillStyle::Pattern => {
                base_c.with_alpha(base_c.a * self.opacity)
            }
        }
    }
}

pub fn create_pattern_shader(
    pattern_type: PatternType,
    c1: Color,
    c2: Color,
    scale: f32,
    angle: f32,
    offset: Point,
    custom_path: Option<&str>,
) -> Option<skia::Shader> {
    if pattern_type == PatternType::Custom || custom_path.is_some() {
        if let Some(cp) = custom_path {
            if let Some(shader) = crate::core::create_custom_pattern_shader(cp, c1, c2, scale, angle, offset) {
                return Some(shader);
            }
        }
    }
    let sz = scale.clamp(4.0, 2048.0);
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
            canvas.draw_line(
                skia::Point::new(0.0, half_h),
                skia::Point::new(w, half_h),
                &paint,
            );

            // 2. Vertical joints: top course at 0 and W
            canvas.draw_line(
                skia::Point::new(0.0, 0.0),
                skia::Point::new(0.0, half_h),
                &paint,
            );
            canvas.draw_line(
                skia::Point::new(w, 0.0),
                skia::Point::new(w, half_h),
                &paint,
            );

            // 3. Vertical joint: bottom course staggered by 50% at W/2
            canvas.draw_line(
                skia::Point::new(w * 0.5, half_h),
                skia::Point::new(w * 0.5, h),
                &paint,
            );
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
            canvas.draw_line(
                skia::Point::new(0.0, half * 0.5),
                skia::Point::new(half, half * 0.5),
                &p_h,
            );
            canvas.draw_line(
                skia::Point::new(half * 0.5, half),
                skia::Point::new(half * 0.5, sz),
                &p_h,
            );
            canvas.draw_line(
                skia::Point::new(half, sz * 0.75),
                skia::Point::new(sz, sz * 0.75),
                &p_h,
            );
            canvas.draw_line(
                skia::Point::new(sz * 0.75, 0.0),
                skia::Point::new(sz * 0.75, half),
                &p_h,
            );
        }
        PatternType::Custom => {
            paint.set_stroke_width((sz * 0.1).max(1.0));
            paint.set_style(skia::PaintStyle::Stroke);
            canvas.draw_line(skia::Point::new(sz * 0.5, 0.0), skia::Point::new(sz, sz * 0.5), &paint);
            canvas.draw_line(skia::Point::new(sz, sz * 0.5), skia::Point::new(sz * 0.5, sz), &paint);
            canvas.draw_line(skia::Point::new(sz * 0.5, sz), skia::Point::new(0.0, sz * 0.5), &paint);
            canvas.draw_line(skia::Point::new(0.0, sz * 0.5), skia::Point::new(sz * 0.5, 0.0), &paint);
        }
    }

    let picture = recorder.finish_recording_as_picture(None)?;
    let mut matrix = skia::Matrix::default();
    matrix.pre_translate((offset.x, offset.y));
    if angle.abs() > 0.01 {
        matrix.post_rotate(
            angle,
            Some(skia::Point::new(
                offset.x + tile_w * 0.5,
                offset.y + tile_h * 0.5,
            )),
        );
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

            let eff_stops = fill.effective_stops();
            let colors: Vec<skia::Color4f> = eff_stops
                .iter()
                .map(|s| s.color.with_alpha(s.color.a * fill.opacity).to_skia())
                .collect();
            let pos: Vec<f32> = eff_stops.iter().map(|s| s.offset.clamp(0.0, 1.0)).collect();

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

            let eff_stops = fill.effective_stops();
            let colors: Vec<skia::Color4f> = eff_stops
                .iter()
                .map(|s| s.color.with_alpha(s.color.a * fill.opacity).to_skia())
                .collect();
            let pos: Vec<f32> = eff_stops.iter().map(|s| s.offset.clamp(0.0, 1.0)).collect();

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
            if let Some(shader) = create_pattern_shader(
                fill.pattern_type,
                c1,
                c2,
                fill.pattern_scale,
                fill.angle,
                fill.pattern_offset,
                fill.custom_pattern_path.as_deref(),
            ) {
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

    pub fn sample_at(&self, point: Point) -> Color {
        if self.stops.is_empty() {
            return Color::WHITE;
        }
        if self.stops.len() == 1 {
            return self.stops[0].color;
        }
        let eff_stops = &self.stops;

        let t = match self.kind {
            GradientType::Linear => {
                let v = self.end - self.start;
                let len_sq = v.x * v.x + v.y * v.y;
                if len_sq < 0.0001 {
                    0.0
                } else {
                    let p = point - self.start;
                    ((p.x * v.x + p.y * v.y) / len_sq).clamp(0.0, 1.0)
                }
            }
            GradientType::Radial => {
                let radius = self.start.distance_to(self.end).max(1.0);
                let dist = point.distance_to(self.start);
                (dist / radius).clamp(0.0, 1.0)
            }
        };

        // Interpolate along stops
        let mut left_stop = &eff_stops[0];
        let mut right_stop = &eff_stops[eff_stops.len() - 1];

        for s in eff_stops {
            if s.offset <= t && s.offset >= left_stop.offset {
                left_stop = s;
            }
            if s.offset >= t && s.offset <= right_stop.offset {
                right_stop = s;
            }
        }

        if (right_stop.offset - left_stop.offset).abs() < 0.0001 {
            return left_stop.color;
        }

        let factor = ((t - left_stop.offset) / (right_stop.offset - left_stop.offset)).clamp(0.0, 1.0);
        Color::new(
            left_stop.color.r + factor * (right_stop.color.r - left_stop.color.r),
            left_stop.color.g + factor * (right_stop.color.g - left_stop.color.g),
            left_stop.color.b + factor * (right_stop.color.b - left_stop.color.b),
            left_stop.color.a + factor * (right_stop.color.a - left_stop.color.a),
        )
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
    #[serde(default)]
    pub smooth_curves: bool,
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
        Self {
            rows,
            cols,
            nodes,
            smooth_curves: true,
        }
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

    pub fn reset_to_bounds(&mut self, bounds: Rect) {
        if self.rows < 2 || self.cols < 2 || self.nodes.len() != self.rows * self.cols {
            return;
        }
        let r = bounds.normalize();
        for row in 0..self.rows {
            let v = row as f32 / (self.rows - 1) as f32;
            for col in 0..self.cols {
                let u = col as f32 / (self.cols - 1) as f32;
                let px = r.x + u * r.width;
                let py = r.y + v * r.height;
                self.nodes[row * self.cols + col].point = Point::new(px, py);
            }
        }
    }

    pub fn split_row(&mut self, row_idx: Option<usize>) -> usize {
        if self.rows < 2 || self.cols < 2 {
            return 0;
        }
        let target_r = row_idx.unwrap_or(self.rows / 2).clamp(0, self.rows - 2);
        let mut final_nodes = Vec::with_capacity((self.rows + 1) * self.cols);

        for r in 0..self.rows {
            for c in 0..self.cols {
                final_nodes.push(self.nodes[r * self.cols + c].clone());
            }
            if r == target_r {
                for c in 0..self.cols {
                    let top_node = &self.nodes[r * self.cols + c];
                    let bot_node = &self.nodes[(r + 1) * self.cols + c];
                    let mid_p = Point::new(
                        (top_node.point.x + bot_node.point.x) * 0.5,
                        (top_node.point.y + bot_node.point.y) * 0.5,
                    );
                    let mid_c = Color::new(
                        (top_node.color.r + bot_node.color.r) * 0.5,
                        (top_node.color.g + bot_node.color.g) * 0.5,
                        (top_node.color.b + bot_node.color.b) * 0.5,
                        (top_node.color.a + bot_node.color.a) * 0.5,
                    );
                    final_nodes.push(MeshNode::new(mid_p, mid_c));
                }
            }
        }
        self.rows += 1;
        self.nodes = final_nodes;
        (target_r + 1) * self.cols
    }

    pub fn split_col(&mut self, col_idx: Option<usize>) -> usize {
        if self.rows < 2 || self.cols < 2 {
            return 0;
        }
        let target_c = col_idx.unwrap_or(self.cols / 2).clamp(0, self.cols - 2);
        let mut final_nodes = Vec::with_capacity(self.rows * (self.cols + 1));

        for r in 0..self.rows {
            for c in 0..self.cols {
                final_nodes.push(self.nodes[r * self.cols + c].clone());
                if c == target_c {
                    let left_node = &self.nodes[r * self.cols + c];
                    let right_node = &self.nodes[r * self.cols + (c + 1)];
                    let mid_p = Point::new(
                        (left_node.point.x + right_node.point.x) * 0.5,
                        (left_node.point.y + right_node.point.y) * 0.5,
                    );
                    let mid_c = Color::new(
                        (left_node.color.r + right_node.color.r) * 0.5,
                        (left_node.color.g + right_node.color.g) * 0.5,
                        (left_node.color.b + right_node.color.b) * 0.5,
                        (left_node.color.a + right_node.color.a) * 0.5,
                    );
                    final_nodes.push(MeshNode::new(mid_p, mid_c));
                }
            }
        }
        self.cols += 1;
        self.nodes = final_nodes;
        target_c + 1
    }

    pub fn delete_node(&mut self, node_idx: usize) -> bool {
        if node_idx >= self.nodes.len() || self.rows < 2 || self.cols < 2 {
            return false;
        }
        if self.rows <= 2 && self.cols <= 2 {
            return false;
        }

        let r = node_idx / self.cols;
        let c = node_idx % self.cols;

        if self.cols > 2 && (self.rows <= 2 || self.cols >= self.rows) {
            // Delete column `c`
            let mut new_nodes = Vec::with_capacity(self.rows * (self.cols - 1));
            for row in 0..self.rows {
                for col in 0..self.cols {
                    if col != c {
                        new_nodes.push(self.nodes[row * self.cols + col].clone());
                    }
                }
            }
            self.cols -= 1;
            self.nodes = new_nodes;
            true
        } else if self.rows > 2 {
            // Delete row `r`
            let mut new_nodes = Vec::with_capacity((self.rows - 1) * self.cols);
            for row in 0..self.rows {
                if row != r {
                    for col in 0..self.cols {
                        new_nodes.push(self.nodes[row * self.cols + col].clone());
                    }
                }
            }
            self.rows -= 1;
            self.nodes = new_nodes;
            true
        } else {
            false
        }
    }

    pub fn smooth_grid_spacing(&mut self) {
        if self.rows <= 2 || self.cols <= 2 || self.nodes.len() != self.rows * self.cols {
            return;
        }

        // Perform Laplacian relaxation on internal grid nodes for even distribution
        let mut new_pts = self.nodes.iter().map(|n| n.point).collect::<Vec<_>>();
        for _ in 0..4 {
            for r in 1..(self.rows - 1) {
                for c in 1..(self.cols - 1) {
                    let top = new_pts[(r - 1) * self.cols + c];
                    let bot = new_pts[(r + 1) * self.cols + c];
                    let left = new_pts[r * self.cols + (c - 1)];
                    let right = new_pts[r * self.cols + (c + 1)];

                    new_pts[r * self.cols + c] = Point::new(
                        (top.x + bot.x + left.x + right.x) * 0.25,
                        (top.y + bot.y + left.y + right.y) * 0.25,
                    );
                }
            }
        }
        for (i, p) in new_pts.into_iter().enumerate() {
            self.nodes[i].point = p;
        }
    }

    pub fn subdivide_at(&mut self, pt: Point, new_color: Option<Color>) -> usize {
        if self.rows < 2 || self.cols < 2 || self.nodes.len() != self.rows * self.cols {
            return 0;
        }

        // Find which column interval pt.x falls into
        let mut insert_c = 1;
        for c in 0..self.cols - 1 {
            let x0 = self.nodes[c].point.x;
            let x1 = self.nodes[c + 1].point.x;
            let (min_x, max_x) = if x0 < x1 { (x0, x1) } else { (x1, x0) };
            if pt.x >= min_x && pt.x <= max_x {
                insert_c = c + 1;
                break;
            }
        }
        let insert_c = insert_c.clamp(1, self.cols - 1);

        // Find which row interval pt.y falls into
        let mut insert_r = 1;
        for r in 0..self.rows - 1 {
            let y0 = self.nodes[r * self.cols].point.y;
            let y1 = self.nodes[(r + 1) * self.cols].point.y;
            let (min_y, max_y) = if y0 < y1 { (y0, y1) } else { (y1, y0) };
            if pt.y >= min_y && pt.y <= max_y {
                insert_r = r + 1;
                break;
            }
        }
        let insert_r = insert_r.clamp(1, self.rows - 1);

        // 1. Insert new column at insert_c
        let mut new_nodes = Vec::with_capacity(self.rows * (self.cols + 1));
        for r in 0..self.rows {
            for c in 0..self.cols {
                if c == insert_c {
                    let left_node = &self.nodes[r * self.cols + (c - 1)];
                    let right_node = &self.nodes[r * self.cols + c];
                    let span_x = (right_node.point.x - left_node.point.x).abs().max(1.0);
                    let t = ((pt.x - left_node.point.x).abs() / span_x).clamp(0.0, 1.0);
                    let py = left_node.point.y + t * (right_node.point.y - left_node.point.y);
                    let col = Color::new(
                        left_node.color.r + t * (right_node.color.r - left_node.color.r),
                        left_node.color.g + t * (right_node.color.g - left_node.color.g),
                        left_node.color.b + t * (right_node.color.b - left_node.color.b),
                        left_node.color.a + t * (right_node.color.a - left_node.color.a),
                    );
                    new_nodes.push(MeshNode::new(Point::new(pt.x, py), col));
                }
                new_nodes.push(self.nodes[r * self.cols + c].clone());
            }
        }
        self.cols += 1;
        self.nodes = new_nodes;

        // 2. Insert new row at insert_r
        let mut final_nodes = Vec::with_capacity((self.rows + 1) * self.cols);
        for r in 0..self.rows {
            if r == insert_r {
                for c in 0..self.cols {
                    let top_node = &self.nodes[(r - 1) * self.cols + c];
                    let bot_node = &self.nodes[r * self.cols + c];
                    let span_y = (bot_node.point.y - top_node.point.y).abs().max(1.0);
                    let t = ((pt.y - top_node.point.y).abs() / span_y).clamp(0.0, 1.0);
                    let px = top_node.point.x + t * (bot_node.point.x - top_node.point.x);
                    let col = if c == insert_c && new_color.is_some() {
                        new_color.unwrap()
                    } else {
                        Color::new(
                            top_node.color.r + t * (bot_node.color.r - top_node.color.r),
                            top_node.color.g + t * (bot_node.color.g - top_node.color.g),
                            top_node.color.b + t * (bot_node.color.b - top_node.color.b),
                            top_node.color.a + t * (bot_node.color.a - top_node.color.a),
                        )
                    };
                    final_nodes.push(MeshNode::new(Point::new(px, pt.y), col));
                }
            }
            for c in 0..self.cols {
                final_nodes.push(self.nodes[r * self.cols + c].clone());
            }
        }
        self.rows += 1;
        self.nodes = final_nodes;

        let selected_idx = insert_r * self.cols + insert_c;
        if let Some(col) = new_color {
            if selected_idx < self.nodes.len() {
                self.nodes[selected_idx].color = col;
            }
        }
        selected_idx
    }

    pub fn sample_at(&self, point: Point) -> Color {
        if self.nodes.is_empty() {
            return Color::WHITE;
        }
        if self.nodes.len() == 1 || self.rows < 2 || self.cols < 2 {
            return self.nodes[0].color;
        }

        // Check quadrilateral patches
        for r in 0..(self.rows - 1) {
            for c in 0..(self.cols - 1) {
                let p00 = self.nodes[r * self.cols + c].point;
                let p10 = self.nodes[r * self.cols + (c + 1)].point;
                let p01 = self.nodes[(r + 1) * self.cols + c].point;
                let p11 = self.nodes[(r + 1) * self.cols + (c + 1)].point;

                let c00 = self.nodes[r * self.cols + c].color;
                let c10 = self.nodes[r * self.cols + (c + 1)].color;
                let c01 = self.nodes[(r + 1) * self.cols + c].color;
                let c11 = self.nodes[(r + 1) * self.cols + (c + 1)].color;

                // Triangle 1: (p00, p10, p11)
                if let Some((u, v, w)) = point_in_triangle(point, p00, p10, p11) {
                    return Color::new(
                        u * c00.r + v * c10.r + w * c11.r,
                        u * c00.g + v * c10.g + w * c11.g,
                        u * c00.b + v * c10.b + w * c11.b,
                        u * c00.a + v * c10.a + w * c11.a,
                    );
                }

                // Triangle 2: (p00, p11, p01)
                if let Some((u, v, w)) = point_in_triangle(point, p00, p11, p01) {
                    return Color::new(
                        u * c00.r + v * c11.r + w * c01.r,
                        u * c00.g + v * c11.g + w * c01.g,
                        u * c00.b + v * c11.b + w * c01.b,
                        u * c00.a + v * c11.a + w * c01.a,
                    );
                }
            }
        }

        // Outside all patches: use Inverse Distance Weighting from closest nodes
        let mut total_weight = 0.0f32;
        let mut accum_r = 0.0f32;
        let mut accum_g = 0.0f32;
        let mut accum_b = 0.0f32;
        let mut accum_a = 0.0f32;

        for node in &self.nodes {
            let dist = point.distance_to(node.point);
            if dist < 0.5 {
                return node.color;
            }
            let weight = 1.0 / (dist * dist);
            total_weight += weight;
            accum_r += node.color.r * weight;
            accum_g += node.color.g * weight;
            accum_b += node.color.b * weight;
            accum_a += node.color.a * weight;
        }

        if total_weight > 0.0 {
            Color::new(
                accum_r / total_weight,
                accum_g / total_weight,
                accum_b / total_weight,
                accum_a / total_weight,
            )
        } else {
            self.nodes[0].color
        }
    }
}

fn point_in_triangle(p: Point, a: Point, b: Point, c: Point) -> Option<(f32, f32, f32)> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = v0.x * v0.x + v0.y * v0.y;
    let d01 = v0.x * v1.x + v0.y * v1.y;
    let d11 = v1.x * v1.x + v1.y * v1.y;
    let d20 = v2.x * v0.x + v2.y * v0.y;
    let d21 = v2.x * v1.x + v2.y * v1.y;
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 0.00001 {
        return None;
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    if u >= -0.01 && v >= -0.01 && w >= -0.01 {
        Some((u.clamp(0.0, 1.0), v.clamp(0.0, 1.0), w.clamp(0.0, 1.0)))
    } else {
        None
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

    if mesh.smooth_curves {
        // High-fidelity smooth sub-patches (3x3 quads per patch = 18 triangles per patch)
        let sub = 3usize;
        let mut vert_idx = 0u16;

        for r in 0..(mesh.rows - 1) {
            for c in 0..(mesh.cols - 1) {
                let p00 = mesh.nodes[r * mesh.cols + c].point;
                let p10 = mesh.nodes[r * mesh.cols + (c + 1)].point;
                let p01 = mesh.nodes[(r + 1) * mesh.cols + c].point;
                let p11 = mesh.nodes[(r + 1) * mesh.cols + (c + 1)].point;

                let c00 = mesh.nodes[r * mesh.cols + c].color;
                let c10 = mesh.nodes[r * mesh.cols + (c + 1)].color;
                let c01 = mesh.nodes[(r + 1) * mesh.cols + c].color;
                let c11 = mesh.nodes[(r + 1) * mesh.cols + (c + 1)].color;

                let base_idx = vert_idx;
                for sr in 0..=sub {
                    let v = sr as f32 / sub as f32;
                    for sc in 0..=sub {
                        let u = sc as f32 / sub as f32;
                        // Bilinear/Coons interpolation
                        let px = (1.0 - u) * (1.0 - v) * p00.x
                            + u * (1.0 - v) * p10.x
                            + (1.0 - u) * v * p01.x
                            + u * v * p11.x;
                        let py = (1.0 - u) * (1.0 - v) * p00.y
                            + u * (1.0 - v) * p10.y
                            + (1.0 - u) * v * p01.y
                            + u * v * p11.y;

                        let cr = (1.0 - u) * (1.0 - v) * c00.r
                            + u * (1.0 - v) * c10.r
                            + (1.0 - u) * v * c01.r
                            + u * v * c11.r;
                        let cg = (1.0 - u) * (1.0 - v) * c00.g
                            + u * (1.0 - v) * c10.g
                            + (1.0 - u) * v * c01.g
                            + u * v * c11.g;
                        let cb = (1.0 - u) * (1.0 - v) * c00.b
                            + u * (1.0 - v) * c10.b
                            + (1.0 - u) * v * c01.b
                            + u * v * c11.b;
                        let ca = (1.0 - u) * (1.0 - v) * c00.a
                            + u * (1.0 - v) * c10.a
                            + (1.0 - u) * v * c01.a
                            + u * v * c11.a;

                        positions.push(Point::new(px, py).to_skia());
                        colors.push(skia::Color::from_argb(
                            (ca * 255.0).round() as u8,
                            (cr * 255.0).round() as u8,
                            (cg * 255.0).round() as u8,
                            (cb * 255.0).round() as u8,
                        ));
                        vert_idx += 1;
                    }
                }

                let stride = (sub + 1) as u16;
                for sr in 0..sub as u16 {
                    for sc in 0..sub as u16 {
                        let tl = base_idx + sr * stride + sc;
                        let tr = tl + 1;
                        let bl = tl + stride;
                        let br = bl + 1;

                        indices.push(tl);
                        indices.push(tr);
                        indices.push(bl);

                        indices.push(tr);
                        indices.push(br);
                        indices.push(bl);
                    }
                }
            }
        }
    } else {
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
    canvas.draw_vertices(&vertices, skia::BlendMode::Dst, &paint);

    canvas.restore();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::Rect;

    #[test]
    fn test_render_mesh_gradient_execution() {
        let mut surface = skia::surfaces::raster_n32_premul((200, 200)).unwrap();
        let canvas = surface.canvas();
        let path = skia::Path::rect(skia::Rect::from_xywh(0.0, 0.0, 100.0, 100.0), None);
        let mesh = MeshGradient::new_grid(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            3,
            3,
            Color::RED,
            Color::BLUE,
        );
        render_mesh_gradient(canvas, &path, &mesh);

        let image = surface.image_snapshot();
        let mut pixel = [0u8; 4];
        let info = skia::ImageInfo::new(
            (1, 1),
            skia::ColorType::RGBA8888,
            skia::AlphaType::Premul,
            None,
        );
        let success = image.read_pixels(&info, &mut pixel, 4, (5, 5), skia::image::CachingHint::Disallow);
        assert!(success);
        // Pixel at (5,5) near top-left should be reddish, NOT black (0,0,0,255)
        println!("Rendered pixel at (5,5): RGBA({}, {}, {}, {})", pixel[0], pixel[1], pixel[2], pixel[3]);
        assert!(pixel[0] > 100, "Red channel should be > 100, got {}", pixel[0]);
    }

    #[test]
    fn test_mesh_gradient_manipulation_methods() {
        let mut mesh = MeshGradient::new_grid(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            3,
            3,
            Color::RED,
            Color::BLUE,
        );
        assert_eq!(mesh.rows, 3);
        assert_eq!(mesh.cols, 3);
        assert_eq!(mesh.nodes.len(), 9);

        // Test Split Row
        mesh.split_row(Some(1));
        assert_eq!(mesh.rows, 4);
        assert_eq!(mesh.cols, 3);
        assert_eq!(mesh.nodes.len(), 12);

        // Test Split Col
        mesh.split_col(Some(1));
        assert_eq!(mesh.rows, 4);
        assert_eq!(mesh.cols, 4);
        assert_eq!(mesh.nodes.len(), 16);

        // Move a node and test reset_to_bounds
        mesh.nodes[5].point = Point::new(999.0, 999.0);
        mesh.reset_to_bounds(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert!(mesh.nodes[5].point.x < 100.0);
        assert!(mesh.nodes[5].point.y < 100.0);

        // Test Smooth Grid Spacing
        mesh.smooth_grid_spacing();
        assert_eq!(mesh.nodes.len(), 16);

        // Test Delete Node
        let deleted = mesh.delete_node(5);
        assert!(deleted);
        assert_eq!(mesh.cols, 3);
        assert_eq!(mesh.nodes.len(), 12);
    }

    #[test]
    fn test_gradient_stop_manipulation() {
        let mut grad = Gradient::new_linear(
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Color::RED,
            Color::BLUE,
        );
        assert_eq!(grad.stops.len(), 2);
        assert_eq!(grad.stops[0].offset, 0.0);
        assert_eq!(grad.stops[1].offset, 1.0);

        // Add middle stop
        grad.stops.push(GradientStop::new(0.5, Color::EMERALD));
        grad.stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        assert_eq!(grad.stops.len(), 3);
        assert_eq!(grad.stops[1].color, Color::EMERALD);

        // Reverse stops
        for s in &mut grad.stops {
            s.offset = (1.0 - s.offset).clamp(0.0, 1.0);
        }
        grad.stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        assert_eq!(grad.stops[0].color, Color::BLUE);
        assert_eq!(grad.stops[2].color, Color::RED);
    }

    #[test]
    fn test_gradient_and_mesh_color_sampling() {
        let grad = Gradient::new_linear(
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Color::BLACK,
            Color::WHITE,
        );
        let mid_c = grad.sample_at(Point::new(50.0, 0.0));
        assert!((mid_c.r - 0.5).abs() < 0.05);
        assert!((mid_c.g - 0.5).abs() < 0.05);
        assert!((mid_c.b - 0.5).abs() < 0.05);

        let mesh = MeshGradient::new_grid(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            3,
            3,
            Color::BLACK,
            Color::WHITE,
        );
        let sample_center = mesh.sample_at(Point::new(50.0, 50.0));
        assert!(sample_center.r > 0.0 && sample_center.r < 1.0);
    }
}

