use crate::core::color::Color;
use crate::core::geometry::{Point, Rect};
use super::path::PathElement;
use super::style::{BlendMode, FillLayer, Gradient, MeshGradient, StrokeLayer};
use super::ElementId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CornerStyle {
    #[default]
    Round = 0,
    Chamfer = 1,
    Concave = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left: top_left.max(0.0),
            top_right: top_right.max(0.0),
            bottom_right: bottom_right.max(0.0),
            bottom_left: bottom_left.max(0.0),
        }
    }

    pub fn uniform(radius: f32) -> Self {
        let r = radius.max(0.0);
        Self {
            top_left: r,
            top_right: r,
            bottom_right: r,
            bottom_left: r,
        }
    }

    pub fn max_radius(&self) -> f32 {
        self.top_left
            .max(self.top_right)
            .max(self.bottom_right)
            .max(self.bottom_left)
    }

    pub fn is_uniform(&self) -> bool {
        (self.top_left - self.top_right).abs() < 0.001
            && (self.top_right - self.bottom_right).abs() < 0.001
            && (self.bottom_right - self.bottom_left).abs() < 0.001
    }

    pub fn clamped(&self, width: f32, height: f32) -> Self {
        let max_r = (width / 2.0).min(height / 2.0).max(0.0);
        Self {
            top_left: self.top_left.min(max_r),
            top_right: self.top_right.min(max_r),
            bottom_right: self.bottom_right.min(max_r),
            bottom_left: self.bottom_left.min(max_r),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RectElement {
    pub id: ElementId,
    pub rect: Rect,
    pub fills: Vec<FillLayer>,
    pub strokes: Vec<StrokeLayer>,
    pub fill_color: Option<Color>,
    pub stroke_color: Option<Color>,
    pub stroke_width: f32,
    pub corner_radius: f32,
    pub corner_radii: CornerRadii,
    pub corner_style: CornerStyle,
    pub gradient: Option<Gradient>,
    pub mesh_gradient: Option<MeshGradient>,
    pub name: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur: f32,
}

impl RectElement {
    pub fn new(rect: Rect, fill_color: Option<Color>, stroke_color: Option<Color>) -> Self {
        let mut fills = Vec::new();
        if let Some(f) = fill_color {
            fills.push(FillLayer::new(f));
        }
        let mut strokes = Vec::new();
        if let Some(s) = stroke_color {
            strokes.push(StrokeLayer::new(s, 2.0));
        }
        Self {
            id: ElementId::new(),
            rect: rect.round(),
            fills,
            strokes,
            fill_color,
            stroke_color,
            stroke_width: 2.0,
            corner_radius: 0.0,
            corner_radii: CornerRadii::default(),
            corner_style: CornerStyle::default(),
            gradient: None,
            mesh_gradient: None,
            name: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            blur: 0.0,
        }
    }

    pub fn bounds(&self) -> Rect {
        let expand = if !self.strokes.is_empty() {
            let max_stroke = self
                .strokes
                .iter()
                .filter(|s| s.enabled)
                .map(|s| s.width)
                .fold(0.0f32, f32::max);
            max_stroke / 2.0
        } else if self.stroke_color.is_some() {
            self.stroke_width / 2.0
        } else {
            0.0
        };
        self.rect.expand(expand)
    }

    pub fn hit_test(&self, p: Point) -> bool {
        self.bounds().contains(p)
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.rect.x = (self.rect.x + dx).round();
        self.rect.y = (self.rect.y + dy).round();
        if let Some(g) = &mut self.gradient {
            g.translate(dx, dy);
        }
        if let Some(m) = &mut self.mesh_gradient {
            m.translate(dx, dy);
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        let mut min_x = origin.x + (self.rect.x - origin.x) * sx;
        let mut min_y = origin.y + (self.rect.y - origin.y) * sy;
        let mut max_x = origin.x + (self.rect.x + self.rect.width - origin.x) * sx;
        let mut max_y = origin.y + (self.rect.y + self.rect.height - origin.y) * sy;
        if min_x > max_x {
            std::mem::swap(&mut min_x, &mut max_x);
        }
        if min_y > max_y {
            std::mem::swap(&mut min_y, &mut max_y);
        }
        self.rect = Rect::new(
            min_x.round(),
            min_y.round(),
            (max_x - min_x).round().max(1.0),
            (max_y - min_y).round().max(1.0),
        );
        if let Some(g) = &mut self.gradient {
            g.scale(origin, sx, sy);
        }
        if let Some(m) = &mut self.mesh_gradient {
            m.scale(origin, sx, sy);
        }
    }

    pub fn effective_radii(&self) -> CornerRadii {
        if self.corner_radii.max_radius() > 0.001 {
            self.corner_radii.clamped(self.rect.width, self.rect.height)
        } else if self.corner_radius > 0.001 {
            CornerRadii::uniform(self.corner_radius).clamped(self.rect.width, self.rect.height)
        } else {
            CornerRadii::uniform(0.0)
        }
    }

    pub fn to_path_element(&self) -> PathElement {
        let r = self.rect.normalize();
        let radii = self.effective_radii();
        let max_r = radii.max_radius();

        let nodes = if max_r <= 0.001 {
            vec![
                crate::core::PathNode::new(Point::new(r.x, r.y)),
                crate::core::PathNode::new(Point::new(r.x + r.width, r.y)),
                crate::core::PathNode::new(Point::new(r.x + r.width, r.y + r.height)),
                crate::core::PathNode::new(Point::new(r.x, r.y + r.height)),
            ]
        } else {
            let k = 0.55228475_f32;
            let mut n = Vec::with_capacity(8);

            let tl = radii.top_left;
            let tr = radii.top_right;
            let br = radii.bottom_right;
            let bl = radii.bottom_left;

            // 1. Top-Left corner start (point on top edge)
            if tl > 0.001 {
                match self.corner_style {
                    CornerStyle::Round => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + tl, r.y),
                            Some(Point::new(r.x + tl - tl * k, r.y)),
                            None,
                        ));
                    }
                    CornerStyle::Concave => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + tl, r.y),
                            Some(Point::new(r.x + tl, r.y + tl * k)),
                            None,
                        ));
                    }
                    CornerStyle::Chamfer => {
                        n.push(crate::core::PathNode::new(Point::new(r.x + tl, r.y)));
                    }
                }
            } else {
                n.push(crate::core::PathNode::new(Point::new(r.x, r.y)));
            }

            // 2. Top-Right corner
            if tr > 0.001 {
                match self.corner_style {
                    CornerStyle::Round => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + r.width - tr, r.y),
                            None,
                            Some(Point::new(r.x + r.width - tr + tr * k, r.y)),
                        ));
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + r.width, r.y + tr),
                            Some(Point::new(r.x + r.width, r.y + tr - tr * k)),
                            None,
                        ));
                    }
                    CornerStyle::Chamfer => {
                        n.push(crate::core::PathNode::new(Point::new(r.x + r.width - tr, r.y)));
                        n.push(crate::core::PathNode::new(Point::new(r.x + r.width, r.y + tr)));
                    }
                    CornerStyle::Concave => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + r.width - tr, r.y),
                            None,
                            Some(Point::new(r.x + r.width - tr, r.y + tr * k)),
                        ));
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + r.width, r.y + tr),
                            Some(Point::new(r.x + r.width - tr * k, r.y + tr)),
                            None,
                        ));
                    }
                }
            } else {
                n.push(crate::core::PathNode::new(Point::new(r.x + r.width, r.y)));
            }

            // 3. Bottom-Right corner
            if br > 0.001 {
                match self.corner_style {
                    CornerStyle::Round => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + r.width, r.y + r.height - br),
                            None,
                            Some(Point::new(r.x + r.width, r.y + r.height - br + br * k)),
                        ));
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + r.width - br, r.y + r.height),
                            Some(Point::new(r.x + r.width - br + br * k, r.y + r.height)),
                            None,
                        ));
                    }
                    CornerStyle::Chamfer => {
                        n.push(crate::core::PathNode::new(Point::new(r.x + r.width, r.y + r.height - br)));
                        n.push(crate::core::PathNode::new(Point::new(r.x + r.width - br, r.y + r.height)));
                    }
                    CornerStyle::Concave => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + r.width, r.y + r.height - br),
                            None,
                            Some(Point::new(r.x + r.width - br * k, r.y + r.height - br)),
                        ));
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + r.width - br, r.y + r.height),
                            Some(Point::new(r.x + r.width - br, r.y + r.height - br * k)),
                            None,
                        ));
                    }
                }
            } else {
                n.push(crate::core::PathNode::new(Point::new(r.x + r.width, r.y + r.height)));
            }

            // 4. Bottom-Left corner
            if bl > 0.001 {
                match self.corner_style {
                    CornerStyle::Round => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + bl, r.y + r.height),
                            None,
                            Some(Point::new(r.x + bl - bl * k, r.y + r.height)),
                        ));
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x, r.y + r.height - bl),
                            Some(Point::new(r.x, r.y + r.height - bl + bl * k)),
                            None,
                        ));
                    }
                    CornerStyle::Chamfer => {
                        n.push(crate::core::PathNode::new(Point::new(r.x + bl, r.y + r.height)));
                        n.push(crate::core::PathNode::new(Point::new(r.x, r.y + r.height - bl)));
                    }
                    CornerStyle::Concave => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x + bl, r.y + r.height),
                            None,
                            Some(Point::new(r.x + bl, r.y + r.height - bl * k)),
                        ));
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x, r.y + r.height - bl),
                            Some(Point::new(r.x + bl * k, r.y + r.height - bl)),
                            None,
                        ));
                    }
                }
            } else {
                n.push(crate::core::PathNode::new(Point::new(r.x, r.y + r.height)));
            }

            // 5. Top-Left closing node (point on left edge)
            if tl > 0.001 {
                match self.corner_style {
                    CornerStyle::Round => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x, r.y + tl),
                            None,
                            Some(Point::new(r.x, r.y + tl - tl * k)),
                        ));
                    }
                    CornerStyle::Chamfer => {
                        n.push(crate::core::PathNode::new(Point::new(r.x, r.y + tl)));
                    }
                    CornerStyle::Concave => {
                        n.push(crate::core::PathNode::with_handles(
                            Point::new(r.x, r.y + tl),
                            None,
                            Some(Point::new(r.x + tl * k, r.y + tl)),
                        ));
                    }
                }
            }

            n
        };

        PathElement {
            id: self.id,
            nodes,
            is_closed: true,
            subpath_lengths: Vec::new(),
            fills: self.fills.clone(),
            strokes: self.strokes.clone(),
            fill_color: self.fill_color,
            stroke_color: self.stroke_color,
            stroke_width: self.stroke_width,
            gradient: self.gradient.clone(),
            mesh_gradient: self.mesh_gradient.clone(),
            shape_origin: None,
            shape_rect: Some(r),
            name: self.name.clone(),
            visible: self.visible,
            locked: self.locked,
            opacity: self.opacity,
            blend_mode: self.blend_mode,
            blur: self.blur,
        }
    }

    pub fn render(&self, canvas: &skia_safe::Canvas) {
        self.to_path_element().render(canvas);
    }
}
