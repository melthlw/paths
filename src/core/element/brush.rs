use skia_safe as skia;

use super::path::dist_to_segment;
use super::style::BlendMode;
use super::ElementId;
use crate::core::color::Color;
use crate::core::element::path::{PathElement, PathNode};
use crate::core::geometry::{Point, Rect};
use crate::core::NodeType;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum BrushStyle {
    #[default]
    Round,
    Pencil,
    Calligraphy,
    Ink,
    Marker,
    Airbrush,
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
    pub name: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur: f32,
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
            name: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            blur: 0.0,
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

        paint.set_color4f(base_color.to_skia(), None);
        paint.set_anti_alias(true);

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
    }
}
