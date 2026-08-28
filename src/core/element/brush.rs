use skia_safe as skia;

use super::ElementId;
use super::path::dist_to_segment;
use super::style::BlendMode;
use crate::core::NodeType;
use crate::core::color::Color;
use crate::core::element::path::{PathElement, PathNode};
use crate::core::geometry::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum StrokeCap {
    #[default]
    Round,
    Square,
    Butt,
}

impl StrokeCap {
    pub fn to_skia(self) -> skia::PaintCap {
        match self {
            StrokeCap::Round => skia::PaintCap::Round,
            StrokeCap::Square => skia::PaintCap::Square,
            StrokeCap::Butt => skia::PaintCap::Butt,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum StrokeJoin {
    #[default]
    Round,
    Miter,
    Bevel,
}

impl StrokeJoin {
    pub fn to_skia(self) -> skia::PaintJoin {
        match self {
            StrokeJoin::Round => skia::PaintJoin::Round,
            StrokeJoin::Miter => skia::PaintJoin::Miter,
            StrokeJoin::Bevel => skia::PaintJoin::Bevel,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum MarkerShape {
    #[default]
    None,
    Arrow,
    StealthArrow,
    Circle,
    Diamond,
    Square,
    Triangle,
    Star,
    CustomPath(String),
}

#[allow(dead_code)]
impl MarkerShape {
    pub fn name(&self) -> String {
        match self {
            MarkerShape::None => crate::core::gettext("None"),
            MarkerShape::Arrow => crate::core::gettext("Arrow"),
            MarkerShape::StealthArrow => crate::core::gettext("Stealth Arrow"),
            MarkerShape::Circle => crate::core::gettext("Circle Cap"),
            MarkerShape::Diamond => crate::core::gettext("Diamond"),
            MarkerShape::Square => crate::core::gettext("Square Block"),
            MarkerShape::Triangle => crate::core::gettext("Triangle Point"),
            MarkerShape::Star => crate::core::gettext("Star Motif"),
            MarkerShape::CustomPath(_) => crate::core::gettext("Custom Path"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum BrushStyle {
    #[default]
    Round,
    Pencil,
    Calligraphy,
    Ink,
    Marker,
    Airbrush,
    Charcoal,
    Watercolor,
    NeonGlow,
    Chalk,
    SprayPaint,
    StarTrail,
    BeadChain,
    ArrowTrail,
}

#[allow(dead_code)]
impl BrushStyle {
    pub fn name(self) -> String {
        match self {
            BrushStyle::Round => crate::core::gettext("Solid Round"),
            BrushStyle::Pencil => crate::core::gettext("Pencil Graphite"),
            BrushStyle::Calligraphy => crate::core::gettext("Calligraphy Ribbon"),
            BrushStyle::Ink => crate::core::gettext("Ink Pen"),
            BrushStyle::Marker => crate::core::gettext("Highlighter Marker"),
            BrushStyle::Airbrush => crate::core::gettext("Airbrush Soft"),
            BrushStyle::Charcoal => crate::core::gettext("Charcoal Sketch"),
            BrushStyle::Watercolor => crate::core::gettext("Watercolor Wash"),
            BrushStyle::NeonGlow => crate::core::gettext("Neon Glow Ribbon"),
            BrushStyle::Chalk => crate::core::gettext("Grainy Chalk"),
            BrushStyle::SprayPaint => crate::core::gettext("Spray Splatter"),
            BrushStyle::StarTrail => crate::core::gettext("Star Trail"),
            BrushStyle::BeadChain => crate::core::gettext("Pearl Bead Chain"),
            BrushStyle::ArrowTrail => crate::core::gettext("Vector Arrow Trail"),
        }
    }

    pub fn desc(self) -> String {
        match self {
            BrushStyle::Round => crate::core::gettext("Smooth uniform solid vector stroke"),
            BrushStyle::Pencil => crate::core::gettext("Fine textured graphite pencil line"),
            BrushStyle::Calligraphy => crate::core::gettext("45° chisel nib calligraphy ribbon"),
            BrushStyle::Ink => crate::core::gettext("Tapered fluid inking pen"),
            BrushStyle::Marker => {
                crate::core::gettext("Semi-transparent multiplicative highlighter")
            }
            BrushStyle::Airbrush => crate::core::gettext("Soft radial blur spray wash"),
            BrushStyle::Charcoal => crate::core::gettext("Rough grainy sketching charcoal"),
            BrushStyle::Watercolor => crate::core::gettext("Soft blended wet paint wash"),
            BrushStyle::NeonGlow => crate::core::gettext("Vibrant glowing light ribbon"),
            BrushStyle::Chalk => crate::core::gettext("Textured pastel chalk line"),
            BrushStyle::SprayPaint => crate::core::gettext("Multi-particle spray splatter"),
            BrushStyle::StarTrail => crate::core::gettext("Repeated star motif contour"),
            BrushStyle::BeadChain => crate::core::gettext("Connected pearl bead chain"),
            BrushStyle::ArrowTrail => crate::core::gettext("Sequential vector arrow trail"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum BrushMode {
    #[default]
    Brush,
    Pencil,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BrushStroke {
    pub id: ElementId,
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
    pub style: BrushStyle,
    pub smoothing: f32,
    pub pressure_points: Vec<f32>,
    pub calligraphy_angle: f32,
    pub auto_close: bool,
    pub cap_style: StrokeCap,
    pub join_style: StrokeJoin,
    pub taper_start: bool,
    pub taper_end: bool,
    pub start_marker: MarkerShape,
    pub body_marker: MarkerShape,
    pub end_marker: MarkerShape,
    pub body_spacing: f32,
    pub marker_scale: f32,
    pub name: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur: f32,
    #[serde(default)]
    pub modifiers: Vec<crate::core::modifier::Modifier>,
}

impl BrushStroke {
    pub fn new(points: Vec<Point>, color: Color, width: f32) -> Self {
        let count = points.len();
        Self {
            id: ElementId::new(),
            points,
            color,
            width: width.max(1.0),
            style: BrushStyle::Round,
            smoothing: 0.5,
            pressure_points: vec![1.0; count],
            calligraphy_angle: 45.0,
            auto_close: false,
            cap_style: StrokeCap::Round,
            join_style: StrokeJoin::Round,
            taper_start: false,
            taper_end: false,
            start_marker: MarkerShape::None,
            body_marker: MarkerShape::None,
            end_marker: MarkerShape::None,
            body_spacing: 2.0,
            marker_scale: 1.0,
            name: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            blur: 0.0,
            modifiers: Vec::new(),
        }
    }

    pub fn bounds(&self) -> Rect {
        if self.points.is_empty() {
            return Rect::ZERO;
        }
        let mut min_x = self.points[0].x;
        let mut min_y = self.points[0].y;
        let mut max_x = self.points[0].x;
        let mut max_y = self.points[0].y;

        for p in &self.points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        let margin = self.width + self.blur * 2.0 + 4.0;
        Rect::new(
            min_x - margin,
            min_y - margin,
            (max_x - min_x) + margin * 2.0,
            (max_y - min_y) + margin * 2.0,
        )
    }

    pub fn hit_test(&self, p: Point) -> bool {
        let threshold = (self.width / 2.0 + 6.0).max(8.0);
        if !self.bounds().expand(threshold).contains(p) {
            return false;
        }

        for window in self.points.windows(2) {
            let p1 = window[0];
            let p2 = window[1];
            if dist_to_segment(p, p1, p2) <= threshold {
                return true;
            }
        }
        false
    }

    pub fn translate_with_options(
        &mut self,
        dx: f32,
        dy: f32,
        _options: &crate::core::TransformOptions,
    ) {
        for p in &mut self.points {
            p.x += dx;
            p.y += dy;
        }
    }

    pub fn scale_with_options(
        &mut self,
        origin: Point,
        sx: f32,
        sy: f32,
        options: &crate::core::TransformOptions,
    ) {
        for p in &mut self.points {
            p.x = origin.x + (p.x - origin.x) * sx;
            p.y = origin.y + (p.y - origin.y) * sy;
        }
        if options.scale_stroke_width {
            let avg_scale = (sx.abs() + sy.abs()) / 2.0;
            self.width = (self.width * avg_scale).max(0.1);
        }
    }

    pub fn rotate(&mut self, center: Point, angle_rad: f32) {
        for p in &mut self.points {
            *p = crate::core::geometry::rotate_point(*p, center, angle_rad);
        }
    }

    /// Convert smoothed brush stroke into an editable vector Bézier PathElement
    pub fn to_path_element(&self) -> PathElement {
        if self.points.is_empty() {
            return PathElement::new(Vec::new(), false, None, Some(self.color), self.width);
        }

        let n = self.points.len();
        let mut nodes = Vec::with_capacity(n);

        if n == 1 {
            nodes.push(PathNode::new(self.points[0]));
        } else if n == 2 {
            nodes.push(PathNode::new(self.points[0]));
            nodes.push(PathNode::new(self.points[1]));
        } else {
            // Catmull-Rom to Cubic Bézier spline conversion for silky smooth curves
            for i in 0..n {
                let p_curr = self.points[i];
                let p_prev = if i > 0 { self.points[i - 1] } else { p_curr };
                let p_next = if i + 1 < n {
                    self.points[i + 1]
                } else {
                    p_curr
                };

                // Handle out towards next
                let handle_out = if i + 1 < n {
                    let dx = (p_next.x - p_prev.x) / 6.0;
                    let dy = (p_next.y - p_prev.y) / 6.0;
                    Some(Point::new(p_curr.x + dx, p_curr.y + dy))
                } else {
                    None
                };

                // Handle in from previous
                let handle_in = if i > 0 {
                    let p_prev2 = if i >= 2 { self.points[i - 2] } else { p_prev };
                    let dx = (p_next.x - p_prev2.x) / 6.0;
                    let dy = (p_next.y - p_prev2.y) / 6.0;
                    Some(Point::new(p_curr.x - dx, p_curr.y - dy))
                } else {
                    None
                };

                let mut node = PathNode::with_handles(p_curr, handle_in, handle_out);
                node.node_type = NodeType::Smooth;
                nodes.push(node);
            }
        }

        let mut path_elem = PathElement::new(
            nodes,
            self.auto_close,
            if self.auto_close {
                Some(self.color.with_alpha(0.3))
            } else {
                None
            },
            Some(self.color),
            self.width,
        );
        path_elem.opacity = self.opacity;
        path_elem.blur = self.blur;
        path_elem.blend_mode = self.blend_mode;
        path_elem
    }

    pub fn render(&self, canvas: &skia::Canvas) {
        if self.points.is_empty() {
            return;
        }

        let mut paint = skia::Paint::default();
        let mut base_color = self.color;
        base_color.a *= self.opacity;

        // Marker style: slightly transparent with blend mode
        if self.style == BrushStyle::Marker {
            base_color.a *= 0.5;
            paint.set_blend_mode(skia::BlendMode::Multiply);
        }

        if self.style == BrushStyle::Watercolor {
            base_color.a *= 0.35;
            paint.set_color4f(base_color.to_skia(), None);
            paint.set_blend_mode(skia::BlendMode::Multiply);
            if let Some(mask) =
                skia::MaskFilter::blur(skia::BlurStyle::Normal, self.width * 0.5, false)
            {
                paint.set_mask_filter(mask);
            }
        }

        paint.set_color4f(base_color.to_skia(), None);
        paint.set_anti_alias(true);

        if self.style == BrushStyle::NeonGlow {
            let mut glow_paint = paint.clone();
            glow_paint.set_style(skia::PaintStyle::Stroke);
            glow_paint.set_stroke_width(self.width * 2.2);
            if let Some(mask) =
                skia::MaskFilter::blur(skia::BlurStyle::Normal, self.width * 0.8, false)
            {
                glow_paint.set_mask_filter(mask);
            }
            let mut glow_path = skia::PathBuilder::new();
            if !self.points.is_empty() {
                glow_path.move_to(self.points[0].to_skia());
                for p in &self.points[1..] {
                    glow_path.line_to(p.to_skia());
                }
            }
            canvas.draw_path(&glow_path.detach(), &glow_paint);

            let mut core_paint = paint.clone();
            core_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 0.95), None);
            core_paint.set_stroke_width((self.width * 0.4).max(1.5));
            core_paint.set_mask_filter(None);
            let mut core_path = skia::PathBuilder::new();
            if !self.points.is_empty() {
                core_path.move_to(self.points[0].to_skia());
                for p in &self.points[1..] {
                    core_path.line_to(p.to_skia());
                }
            }
            canvas.draw_path(&core_path.detach(), &core_paint);
            return;
        }

        if self.blur > 0.001 || self.style == BrushStyle::Airbrush {
            let blur_amt = if self.style == BrushStyle::Airbrush {
                (self.width * 0.4).max(self.blur)
            } else {
                self.blur
            };
            if let Some(mask) = skia::MaskFilter::blur(skia::BlurStyle::Normal, blur_amt, false) {
                paint.set_mask_filter(mask);
            }
        }

        // Calligraphy ribbon rendering
        if self.style == BrushStyle::Calligraphy && self.points.len() > 1 {
            let angle_rad = self.calligraphy_angle.to_radians();
            let half_w = self.width * 0.5;
            let offset_x = angle_rad.cos() * half_w;
            let offset_y = angle_rad.sin() * half_w;

            let mut path_builder = skia::PathBuilder::new();
            let n = self.points.len();

            // Forward side
            path_builder.move_to(skia::Point::new(
                self.points[0].x + offset_x,
                self.points[0].y + offset_y,
            ));
            for i in 1..n {
                path_builder.line_to(skia::Point::new(
                    self.points[i].x + offset_x,
                    self.points[i].y + offset_y,
                ));
            }
            // Backward side
            for i in (0..n).rev() {
                path_builder.line_to(skia::Point::new(
                    self.points[i].x - offset_x,
                    self.points[i].y - offset_y,
                ));
            }
            path_builder.close();

            paint.set_style(skia::PaintStyle::Fill);
            canvas.draw_path(&path_builder.detach(), &paint);
            return;
        }

        paint.set_style(skia::PaintStyle::Stroke);
        paint.set_stroke_width(self.width);
        paint.set_stroke_cap(self.cap_style.to_skia());
        paint.set_stroke_join(self.join_style.to_skia());

        if self.points.len() == 1 {
            canvas.draw_point(self.points[0].to_skia(), &paint);
            return;
        }

        // Standard smooth quadratic spline rendering
        let mut builder = skia::PathBuilder::new();
        builder.move_to(self.points[0].to_skia());

        if self.points.len() == 2 {
            builder.line_to(self.points[1].to_skia());
        } else {
            for i in 1..self.points.len() - 1 {
                let p0 = self.points[i];
                let p1 = self.points[i + 1];
                let mid = Point::new((p0.x + p1.x) / 2.0, (p0.y + p1.y) / 2.0);
                builder.quad_to(p0.to_skia(), mid.to_skia());
            }
            if let Some(last) = self.points.last() {
                builder.line_to(last.to_skia());
            }
        }

        // Special Preset Style Renderers
        if self.style == BrushStyle::SprayPaint {
            let mut dot_paint = paint.clone();
            dot_paint.set_style(skia::PaintStyle::Fill);
            let radius = (self.width * 0.8).max(3.0);

            for (idx, p) in self.points.iter().enumerate() {
                let seed = (p.x * 12.9898 + p.y * 78.233 + (idx as f32) * 43758.5453)
                    .sin()
                    .abs();
                let num_dots = 6 + ((seed * 8.0) as usize);

                for d in 0..num_dots {
                    let angle = (seed * 17.0 + d as f32 * 2.39).sin() * std::f32::consts::PI * 2.0;
                    let dist = ((seed * 31.0 + d as f32 * 1.7).cos().abs()) * radius;
                    let dot_x = p.x + angle.cos() * dist;
                    let dot_y = p.y + angle.sin() * dist;
                    let dot_r = (1.0 + (seed * 3.0)).min(radius * 0.4);

                    let mut col = base_color;
                    col.a *= (0.2 + (seed * 0.6)).min(1.0);
                    dot_paint.set_color4f(col.to_skia(), None);
                    canvas.draw_circle(skia::Point::new(dot_x, dot_y), dot_r, &dot_paint);
                }
            }
            return;
        }

        if self.style == BrushStyle::Charcoal || self.style == BrushStyle::Chalk {
            let mut stroke_paint = paint.clone();
            stroke_paint.set_style(skia::PaintStyle::Stroke);
            stroke_paint.set_stroke_width((self.width * 0.5).max(1.0));

            if self.style == BrushStyle::Chalk {
                if let Some(pe) = skia::PathEffect::dash(&[6.0, 4.0, 2.0, 3.0], 0.0) {
                    stroke_paint.set_path_effect(pe);
                }
            }

            let offsets = [
                (0.0f32, 0.0f32, 0.7f32),
                (0.8f32, -0.6f32, 0.35f32),
                (-0.7f32, 0.8f32, 0.35f32),
            ];

            for (dx, dy, opacity_mult) in offsets {
                let mut path_b = skia::PathBuilder::new();
                if !self.points.is_empty() {
                    path_b.move_to(skia::Point::new(
                        self.points[0].x + dx,
                        self.points[0].y + dy,
                    ));
                    for p in &self.points[1..] {
                        path_b.line_to(skia::Point::new(p.x + dx, p.y + dy));
                    }
                }
                let mut c = base_color;
                c.a *= opacity_mult;
                stroke_paint.set_color4f(c.to_skia(), None);
                canvas.draw_path(&path_b.detach(), &stroke_paint);
            }
            return;
        }

        if self.style == BrushStyle::Pencil {
            let mut stroke_paint = paint.clone();
            stroke_paint.set_style(skia::PaintStyle::Stroke);
            stroke_paint.set_stroke_width((self.width * 0.4).max(1.0));

            let offsets = [(0.0f32, 0.0f32, 0.6f32), (0.4f32, -0.3f32, 0.4f32)];
            for (dx, dy, opacity_mult) in offsets {
                let mut path_b = skia::PathBuilder::new();
                if !self.points.is_empty() {
                    path_b.move_to(skia::Point::new(
                        self.points[0].x + dx,
                        self.points[0].y + dy,
                    ));
                    for p in &self.points[1..] {
                        path_b.line_to(skia::Point::new(p.x + dx, p.y + dy));
                    }
                }
                let mut c = base_color;
                c.a *= opacity_mult;
                stroke_paint.set_color4f(c.to_skia(), None);
                canvas.draw_path(&path_b.detach(), &stroke_paint);
            }
            return;
        }

        if self.auto_close {
            builder.close();
            let path = builder.detach();
            let mut fill_paint = paint.clone();
            fill_paint.set_style(skia::PaintStyle::Fill);
            let mut fill_col = base_color;
            fill_col.a *= 0.35;
            fill_paint.set_color4f(fill_col.to_skia(), None);
            canvas.draw_path(&path, &fill_paint);
            canvas.draw_path(&path, &paint);
        } else {
            let path = builder.detach();
            canvas.draw_path(&path, &paint);
        }

        // Determine Effective Markers for Presets
        let effective_start_marker = if self.style == BrushStyle::StarTrail {
            MarkerShape::Star
        } else if self.style == BrushStyle::ArrowTrail {
            MarkerShape::Arrow
        } else {
            self.start_marker.clone()
        };

        let effective_body_marker = if self.style == BrushStyle::StarTrail {
            MarkerShape::Star
        } else if self.style == BrushStyle::BeadChain {
            MarkerShape::Circle
        } else if self.style == BrushStyle::ArrowTrail {
            MarkerShape::StealthArrow
        } else {
            self.body_marker.clone()
        };

        let effective_end_marker = if self.style == BrushStyle::StarTrail {
            MarkerShape::Star
        } else if self.style == BrushStyle::ArrowTrail {
            MarkerShape::Arrow
        } else {
            self.end_marker.clone()
        };

        // Render Start, Body/Middle, and End Markers
        let n = self.points.len();
        if n >= 2 {
            let marker_scale_factor = (self.marker_scale * (self.width / 6.0)).max(0.2);

            // Start Marker
            if effective_start_marker != MarkerShape::None {
                let p0 = self.points[0];
                let p1 = self.points[1];
                let angle = (p1.y - p0.y).atan2(p1.x - p0.x);
                draw_marker_shape(
                    canvas,
                    &effective_start_marker,
                    p0,
                    angle,
                    marker_scale_factor,
                    base_color,
                );
            }

            // End Marker
            if effective_end_marker != MarkerShape::None {
                let p_last = self.points[n - 1];
                let p_prev = self.points[n - 2];
                let angle = (p_last.y - p_prev.y).atan2(p_last.x - p_prev.x);
                draw_marker_shape(
                    canvas,
                    &effective_end_marker,
                    p_last,
                    angle,
                    marker_scale_factor,
                    base_color,
                );
            }

            // Body / Middle Repeat Marker
            if effective_body_marker != MarkerShape::None {
                let step_dist = (self.body_spacing * self.width).max(8.0);
                let mut accumulated = 0.0;
                for i in 0..n - 1 {
                    let p0 = self.points[i];
                    let p1 = self.points[i + 1];
                    let seg_len = p0.distance_to(p1);
                    let angle = (p1.y - p0.y).atan2(p1.x - p0.x);

                    while accumulated + step_dist <= seg_len {
                        accumulated += step_dist;
                        let t = accumulated / seg_len;
                        let pos = Point::new(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y));
                        draw_marker_shape(
                            canvas,
                            &effective_body_marker,
                            pos,
                            angle,
                            marker_scale_factor,
                            base_color,
                        );
                    }
                    accumulated -= seg_len;
                    if accumulated < 0.0 {
                        accumulated = 0.0;
                    }
                }
            }
        }
    }
}

fn draw_marker_shape(
    canvas: &skia::Canvas,
    shape: &MarkerShape,
    pos: Point,
    angle_rad: f32,
    scale: f32,
    color: Color,
) {
    if *shape == MarkerShape::None {
        return;
    }
    let mut paint = skia::Paint::default();
    paint.set_color4f(color.to_skia(), None);
    paint.set_anti_alias(true);
    paint.set_style(skia::PaintStyle::Fill);

    canvas.save();
    canvas.translate((pos.x, pos.y));
    canvas.rotate(angle_rad.to_degrees(), None);
    canvas.scale((scale, scale));

    match shape {
        MarkerShape::Arrow => {
            let mut builder = skia::PathBuilder::new();
            builder.move_to((10.0, 0.0));
            builder.line_to((-10.0, -7.0));
            builder.line_to((-6.0, 0.0));
            builder.line_to((-10.0, 7.0));
            builder.close();
            canvas.draw_path(&builder.detach(), &paint);
        }
        MarkerShape::StealthArrow => {
            let mut builder = skia::PathBuilder::new();
            builder.move_to((12.0, 0.0));
            builder.line_to((-10.0, -8.0));
            builder.line_to((-4.0, 0.0));
            builder.line_to((-10.0, 8.0));
            builder.close();
            canvas.draw_path(&builder.detach(), &paint);
        }
        MarkerShape::Circle => {
            canvas.draw_circle((0.0, 0.0), 6.0, &paint);
        }
        MarkerShape::Diamond => {
            let mut builder = skia::PathBuilder::new();
            builder.move_to((8.0, 0.0));
            builder.line_to((0.0, -6.0));
            builder.line_to((-8.0, 0.0));
            builder.line_to((0.0, 6.0));
            builder.close();
            canvas.draw_path(&builder.detach(), &paint);
        }
        MarkerShape::Square => {
            let rect = skia::Rect::from_point_and_size((-6.0, -6.0), (12.0, 12.0));
            canvas.draw_rect(rect, &paint);
        }
        MarkerShape::Triangle => {
            let mut builder = skia::PathBuilder::new();
            builder.move_to((10.0, 0.0));
            builder.line_to((-8.0, -7.0));
            builder.line_to((-8.0, 7.0));
            builder.close();
            canvas.draw_path(&builder.detach(), &paint);
        }
        MarkerShape::Star => {
            let mut builder = skia::PathBuilder::new();
            for i in 0..10 {
                let r = if i % 2 == 0 { 8.0 } else { 3.5 };
                let a = (i as f32) * std::f32::consts::PI / 5.0;
                let px = a.cos() * r;
                let py = a.sin() * r;
                if i == 0 {
                    builder.move_to((px, py));
                } else {
                    builder.line_to((px, py));
                }
            }
            builder.close();
            canvas.draw_path(&builder.detach(), &paint);
        }
        MarkerShape::CustomPath(path_d) => {
            let subpaths = crate::core::svg_import::parse_svg_path_data_subpaths(path_d);
            let mut builder = skia::PathBuilder::new();
            for sub in &subpaths {
                if sub.is_empty() {
                    continue;
                }
                builder.move_to((sub[0].point.x, sub[0].point.y));
                for n in sub.iter().skip(1) {
                    builder.line_to((n.point.x, n.point.y));
                }
            }
            canvas.draw_path(&builder.detach(), &paint);
        }
        MarkerShape::None => {}
    }

    canvas.restore();
}
