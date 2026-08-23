use skia_safe as skia;

use crate::core::color::Color;
use crate::core::geometry::{Point, Rect};
use super::path::dist_to_segment;
use super::style::BlendMode;
use super::ElementId;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BrushStroke {
    pub id: ElementId,
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
    pub name: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur: f32,
}

impl BrushStroke {
    pub fn new(points: Vec<Point>, color: Color, width: f32) -> Self {
        Self {
            id: ElementId::new(),
            points,
            color,
            width: width.max(1.0),
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

        let margin = self.width / 2.0 + 2.0;
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

    pub fn translate(&mut self, dx: f32, dy: f32) {
        for p in &mut self.points {
            p.x += dx;
            p.y += dy;
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        for p in &mut self.points {
            p.x = origin.x + (p.x - origin.x) * sx;
            p.y = origin.y + (p.y - origin.y) * sy;
        }
    }

    pub fn rotate(&mut self, center: Point, angle_rad: f32) {
        for p in &mut self.points {
            *p = crate::core::geometry::rotate_point(*p, center, angle_rad);
        }
    }

    pub fn render(&self, canvas: &skia::Canvas) {
        if self.points.is_empty() {
            return;
        }

        let mut paint = skia::Paint::default();
        paint.set_color4f(self.color.to_skia(), None);
        paint.set_style(skia::PaintStyle::Stroke);
        paint.set_stroke_width(self.width);
        paint.set_stroke_cap(skia::PaintCap::Round);
        paint.set_stroke_join(skia::PaintJoin::Round);
        paint.set_anti_alias(true);

        if self.points.len() == 1 {
            canvas.draw_point(self.points[0].to_skia(), &paint);
            return;
        }

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

        let path = builder.detach();
        canvas.draw_path(&path, &paint);
    }
}
