pub mod brush;
pub mod group;
pub mod path;
pub mod rect;
pub mod style;
pub mod text;

pub use brush::{BrushMode, BrushStroke, BrushStyle, MarkerShape, StrokeCap, StrokeJoin};

pub use group::{CloneElement, GroupElement, ImageElement};

pub use path::{ArcMode, PathElement, PathNode, ShapeOrigin, dist_to_segment};

pub use rect::{CornerRadii, CornerStyle, RectElement};

pub use style::{
    BlendMode, ElementStyleSnapshot, FillLayer, FillStyle, Gradient, GradientStop, GradientType,
    MeshGradient, PatternType, StrokeLayer, StrokeStyle,
};

pub use text::{
    OpenTypeFeatures, PathGlyphOrientation, PathVerticalAlign, TextAlign, TextBaseline, TextCase,
    TextElement, get_curated_font_glyphs, get_system_font_families,
};

use skia_safe as skia;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::color::Color;
use crate::core::geometry::{Point, Rect};

static NEXT_ELEMENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ElementId(pub u64);

impl ElementId {
    pub fn new() -> Self {
        Self(NEXT_ELEMENT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Element {
    Rect(RectElement),
    Brush(BrushStroke),
    Path(PathElement),
    Text(TextElement),
    Group(GroupElement),
    Image(ImageElement),
    Clone(CloneElement),
}

impl Element {
    pub fn id(&self) -> ElementId {
        match self {
            Element::Rect(r) => r.id,
            Element::Brush(b) => b.id,
            Element::Path(p) => p.id,
            Element::Text(t) => t.id,
            Element::Group(g) => g.id,
            Element::Image(i) => i.id,
            Element::Clone(c) => c.id,
        }
    }

    pub fn bounds(&self) -> Rect {
        let base_b = match self {
            Element::Rect(r) => r.bounds(),
            Element::Brush(b) => b.bounds(),
            Element::Path(p) => p.bounds(),
            Element::Text(t) => t.bounds(),
            Element::Group(g) => g.bounds(),
            Element::Image(i) => i.bounds(),
            Element::Clone(c) => c.bounds(),
        };

        self.evaluate_modifier_bounds(base_b)
    }

    pub fn evaluate_modifier_bounds(&self, base_b: Rect) -> Rect {
        let mods = self.modifiers();
        if mods.is_empty() {
            return base_b;
        }

        let mut boxes = vec![base_b];

        for m in mods {
            if !m.enabled() {
                continue;
            }
            match m {
                crate::core::modifier::Modifier::Array(arr) => {
                    let mut next_boxes = Vec::new();
                    for cur_b in &boxes {
                        let center = cur_b.center();
                        match &arr.mode {
                            crate::core::modifier::ArrayMode::Linear {
                                count,
                                offset_x,
                                offset_y,
                                scale_step,
                                rotate_step_deg: _,
                            } => {
                                for i in 0..*count {
                                    let scale = scale_step.powi(i as i32);
                                    let dx = offset_x * i as f32;
                                    let dy = offset_y * i as f32;
                                    let w = cur_b.width * scale;
                                    let h = cur_b.height * scale;
                                    let copy_r = Rect::new(
                                        cur_b.x + dx,
                                        cur_b.y + dy,
                                        w,
                                        h,
                                    );
                                    next_boxes.push(copy_r);
                                }
                            }
                            crate::core::modifier::ArrayMode::Radial {
                                count,
                                radius,
                                start_angle_deg,
                                total_angle_deg,
                                rotate_copies: _,
                            } => {
                                let step = if *count > 1 {
                                    total_angle_deg / (*count as f32)
                                } else {
                                    0.0
                                };
                                for i in 0..*count {
                                    let angle_deg = start_angle_deg + step * i as f32;
                                    let rad = angle_deg.to_radians();
                                    let cx = center.x + radius * rad.cos();
                                    let cy = center.y + radius * rad.sin();
                                    let copy_r = Rect::new(
                                        cx - cur_b.width / 2.0,
                                        cy - cur_b.height / 2.0,
                                        cur_b.width,
                                        cur_b.height,
                                    );
                                    next_boxes.push(copy_r);
                                }
                            }
                            crate::core::modifier::ArrayMode::Grid {
                                rows,
                                cols,
                                spacing_x,
                                spacing_y,
                            } => {
                                for r in 0..*rows {
                                    for c in 0..*cols {
                                        let copy_r = Rect::new(
                                            cur_b.x + spacing_x * c as f32,
                                            cur_b.y + spacing_y * r as f32,
                                            cur_b.width,
                                            cur_b.height,
                                        );
                                        next_boxes.push(copy_r);
                                    }
                                }
                            }
                        }
                    }
                    boxes = next_boxes;
                }
                crate::core::modifier::Modifier::EnvelopeWarp(env) => {
                    let mut next_boxes = Vec::new();
                    for cur_b in &boxes {
                        let norm = cur_b.normalize();
                        let p1 = Point::new(norm.x + env.top_left_offset.x, norm.y + env.top_left_offset.y);
                        let p2 = Point::new(norm.x + norm.width + env.top_right_offset.x, norm.y + env.top_right_offset.y);
                        let p3 = Point::new(norm.x + norm.width + env.bottom_right_offset.x, norm.y + norm.height + env.bottom_right_offset.y);
                        let p4 = Point::new(norm.x + env.bottom_left_offset.x, norm.y + norm.height + env.bottom_left_offset.y);

                        let min_x = norm.x.min(p1.x).min(p2.x).min(p3.x).min(p4.x);
                        let min_y = norm.y.min(p1.y).min(p2.y).min(p3.y).min(p4.y);
                        let max_x = (norm.x + norm.width).max(p1.x).max(p2.x).max(p3.x).max(p4.x);
                        let max_y = (norm.y + norm.height).max(p1.y).max(p2.y).max(p3.y).max(p4.y);

                        next_boxes.push(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y));
                    }
                    boxes = next_boxes;
                }
                _ => {}
            }
        }

        let mut total_bounds = boxes[0];
        for b in &boxes[1..] {
            total_bounds = total_bounds.union(*b);
        }

        total_bounds
    }

    pub fn hit_test(&self, p: Point) -> bool {
        match self {
            Element::Rect(r) => r.hit_test(p),
            Element::Brush(b) => b.hit_test(p),
            Element::Path(path) => path.hit_test(p),
            Element::Text(t) => t.hit_test(p),
            Element::Group(g) => g.hit_test(p),
            Element::Image(i) => i.hit_test(p),
            Element::Clone(c) => c.hit_test(p),
        }
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.translate_with_options(dx, dy, &crate::core::TransformOptions::default());
    }

    pub fn translate_with_options(
        &mut self,
        dx: f32,
        dy: f32,
        options: &crate::core::TransformOptions,
    ) {
        match self {
            Element::Rect(r) => r.translate_with_options(dx, dy, options),
            Element::Brush(b) => b.translate_with_options(dx, dy, options),
            Element::Path(p) => p.translate_with_options(dx, dy, options),
            Element::Text(t) => t.translate(dx, dy),
            Element::Group(g) => g.translate_with_options(dx, dy, options),
            Element::Image(i) => i.translate(dx, dy),
            Element::Clone(c) => c.translate(dx, dy),
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        self.scale_with_options(origin, sx, sy, &crate::core::TransformOptions::default());
    }

    pub fn scale_with_options(
        &mut self,
        origin: Point,
        sx: f32,
        sy: f32,
        options: &crate::core::TransformOptions,
    ) {
        match self {
            Element::Rect(r) => r.scale_with_options(origin, sx, sy, options),
            Element::Brush(b) => b.scale_with_options(origin, sx, sy, options),
            Element::Path(p) => p.scale_with_options(origin, sx, sy, options),
            Element::Text(t) => t.scale(origin, sx, sy),
            Element::Group(g) => g.scale_with_options(origin, sx, sy, options),
            Element::Image(i) => i.scale(origin, sx, sy),
            Element::Clone(c) => c.scale(origin, sx, sy),
        }
    }

    pub fn rotate(&mut self, center: Point, angle_rad: f32) {
        match self {
            Element::Rect(r) => {
                let mut path = r.to_path_element();
                path.rotate(center, angle_rad);
                *self = Element::Path(path);
            }
            Element::Brush(b) => b.rotate(center, angle_rad),
            Element::Path(p) => p.rotate(center, angle_rad),
            Element::Text(t) => t.rotate(center, angle_rad),
            Element::Group(g) => g.rotate(center, angle_rad),
            Element::Image(i) => {
                i.translate(center.x - i.rect.x, center.y - i.rect.y);
            }
            Element::Clone(c) => c.rotate(center, angle_rad),
        }
    }

    pub fn fill_color(&self) -> Option<Color> {
        match self {
            Element::Rect(r) => r.fill_color,
            Element::Path(p) => p.fill_color,
            Element::Brush(_) => None,
            Element::Text(t) => Some(t.color),
            Element::Group(g) => g.children.first().and_then(|c| c.fill_color()),
            Element::Image(_) => None,
            Element::Clone(_) => None,
        }
    }

    pub fn sample_color_at(&self, point: Point) -> Option<Color> {
        match self {
            Element::Rect(r) => {
                if let Some(mesh) = &r.mesh_gradient {
                    return Some(mesh.sample_at(point));
                }
                if let Some(grad) = &r.gradient {
                    return Some(grad.sample_at(point));
                }
                if let Some(f0) = r.fills.first() {
                    if f0.enabled {
                        return Some(f0.sample_at(point, r.bounds()));
                    }
                }
                r.fill_color
            }
            Element::Path(p) => {
                if let Some(mesh) = &p.mesh_gradient {
                    return Some(mesh.sample_at(point));
                }
                if let Some(grad) = &p.gradient {
                    return Some(grad.sample_at(point));
                }
                if let Some(f0) = p.fills.first() {
                    if f0.enabled {
                        return Some(f0.sample_at(point, p.bounds()));
                    }
                }
                p.fill_color
            }
            Element::Brush(b) => Some(b.color),
            Element::Text(t) => Some(t.color),
            Element::Group(g) => {
                for c in g.children.iter().rev() {
                    if c.hit_test(point) {
                        if let Some(col) = c.sample_color_at(point) {
                            return Some(col);
                        }
                    }
                }
                g.children.first().and_then(|c| c.sample_color_at(point))
            }
            Element::Clone(_) => None,
            Element::Image(_) => None,
        }
    }

    pub fn stroke_color(&self) -> Option<Color> {
        match self {
            Element::Rect(r) => r.stroke_color,
            Element::Path(p) => p.stroke_color,
            Element::Brush(b) => Some(b.color),
            Element::Text(t) => {
                if !t.strokes.is_empty() {
                    t.strokes.iter().find(|s| s.enabled).map(|s| s.color)
                } else {
                    t.stroke_color
                }
            }
            Element::Group(g) => g.children.first().and_then(|c| c.stroke_color()),
            Element::Image(_) => None,
            Element::Clone(_) => None,
        }
    }

    pub fn set_fill_color(&mut self, color: Option<Color>) {
        match self {
            Element::Rect(r) => {
                r.fill_color = color;
                if let Some(c) = color {
                    if r.fills.is_empty() {
                        r.fills.push(FillLayer::new(c));
                    } else {
                        r.fills[0].color = c;
                        r.fills[0].enabled = true;
                    }
                } else {
                    r.fills.clear();
                }
                r.gradient = None;
                r.mesh_gradient = None;
            }
            Element::Path(p) => {
                p.fill_color = color;
                if let Some(c) = color {
                    if p.fills.is_empty() {
                        p.fills.push(FillLayer::new(c));
                    } else {
                        p.fills[0].color = c;
                        p.fills[0].enabled = true;
                    }
                } else {
                    p.fills.clear();
                }
                p.gradient = None;
                p.mesh_gradient = None;
            }
            Element::Brush(b) => {
                if let Some(c) = color {
                    b.color = c;
                }
            }
            Element::Text(t) => {
                if let Some(c) = color {
                    t.color = c;
                }
            }
            Element::Group(g) => {
                for c in &mut g.children {
                    c.set_fill_color(color);
                }
            }
            Element::Image(_) => {}
            Element::Clone(_) => {}
        }
    }

    pub fn collect_colors(&self, out: &mut Vec<Option<Color>>) {
        match self {
            Element::Rect(r) => {
                if let Some(fc) = r.fill_color {
                    if fc.a > 0.001 {
                        out.push(Some(fc));
                    }
                }
                for f in &r.fills {
                    if f.enabled {
                        if f.color.a > 0.001 {
                            out.push(Some(f.color));
                        }
                        if f.style != FillStyle::Solid && f.secondary_color.a > 0.001 {
                            out.push(Some(f.secondary_color));
                        }
                        for stop in &f.stops {
                            if stop.color.a > 0.001 {
                                out.push(Some(stop.color));
                            }
                        }
                        if let Some(ref m) = f.mesh {
                            for node in &m.nodes {
                                if node.color.a > 0.001 {
                                    out.push(Some(node.color));
                                }
                            }
                        }
                    }
                }
                if let Some(s) = r.stroke_color {
                    if s.a > 0.001 {
                        out.push(Some(s));
                    }
                }
                for s in &r.strokes {
                    if s.enabled && s.color.a > 0.001 {
                        out.push(Some(s.color));
                    }
                }
            }
            Element::Path(p) => {
                if let Some(fc) = p.fill_color {
                    if fc.a > 0.001 {
                        out.push(Some(fc));
                    }
                }
                for f in &p.fills {
                    if f.enabled {
                        if f.color.a > 0.001 {
                            out.push(Some(f.color));
                        }
                        if f.style != FillStyle::Solid && f.secondary_color.a > 0.001 {
                            out.push(Some(f.secondary_color));
                        }
                        for stop in &f.stops {
                            if stop.color.a > 0.001 {
                                out.push(Some(stop.color));
                            }
                        }
                        if let Some(ref m) = f.mesh {
                            for node in &m.nodes {
                                if node.color.a > 0.001 {
                                    out.push(Some(node.color));
                                }
                            }
                        }
                    }
                }
                if let Some(s) = p.stroke_color {
                    if s.a > 0.001 {
                        out.push(Some(s));
                    }
                }
                for s in &p.strokes {
                    if s.enabled && s.color.a > 0.001 {
                        out.push(Some(s.color));
                    }
                }
            }
            Element::Text(t) => {
                if t.color.a > 0.001 {
                    out.push(Some(t.color));
                }
            }
            Element::Brush(b) => {
                if b.color.a > 0.001 {
                    out.push(Some(b.color));
                }
            }
            Element::Group(g) => {
                for child in &g.children {
                    child.collect_colors(out);
                }
            }
            Element::Image(_) => {}
            Element::Clone(_) => {}
        }
    }

    pub fn set_stroke_color(&mut self, color: Option<Color>) {
        match self {
            Element::Rect(r) => {
                r.stroke_color = color;
                if let Some(c) = color {
                    if r.strokes.is_empty() {
                        r.strokes.push(StrokeLayer::new(c, r.stroke_width));
                    } else {
                        r.strokes[0].color = c;
                        r.strokes[0].enabled = true;
                    }
                } else {
                    r.strokes.clear();
                }
            }
            Element::Path(p) => {
                p.stroke_color = color;
                if let Some(c) = color {
                    if p.strokes.is_empty() {
                        p.strokes.push(StrokeLayer::new(c, p.stroke_width));
                    } else {
                        p.strokes[0].color = c;
                        p.strokes[0].enabled = true;
                    }
                } else {
                    p.strokes.clear();
                }
            }
            Element::Brush(b) => {
                if let Some(c) = color {
                    b.color = c;
                }
            }
            Element::Text(t) => {
                t.stroke_color = color;
                if let Some(c) = color {
                    if t.strokes.is_empty() {
                        t.strokes.push(StrokeLayer::new(c, t.stroke_width));
                    } else {
                        t.strokes[0].color = c;
                        t.strokes[0].enabled = true;
                    }
                } else {
                    t.strokes.clear();
                }
            }
            Element::Group(g) => {
                for c in &mut g.children {
                    c.set_stroke_color(color);
                }
            }
            Element::Image(_) => {}
            Element::Clone(_) => {}
        }
    }

    pub fn to_skia_path(&self) -> skia::Path {
        match self {
            Element::Rect(r) => r.to_path_element().to_skia_path(),
            Element::Path(p) => p.to_skia_path(),
            Element::Brush(b) => {
                let mut builder = skia::PathBuilder::new();
                if !b.points.is_empty() {
                    builder.move_to(b.points[0].to_skia());
                    for p in &b.points[1..] {
                        builder.line_to(p.to_skia());
                    }
                }
                builder.detach()
            }
            Element::Text(t) => t.to_skia_path(),
            Element::Group(g) => {
                if let Some(clip) = &g.clip_element {
                    clip.to_skia_path()
                } else if !g.children.is_empty() {
                    g.children[0].to_skia_path()
                } else {
                    skia::PathBuilder::new().detach()
                }
            }
            Element::Image(i) => {
                let r = i.bounds();
                let mut builder = skia::PathBuilder::new();
                builder.add_rect(r.to_skia(), None, None);
                builder.detach()
            }
            Element::Clone(c) => {
                let r = c.bounds();
                let mut builder = skia::PathBuilder::new();
                builder.add_rect(r.to_skia(), None, None);
                builder.detach()
            }
        }
    }

    pub fn modifiers(&self) -> &[crate::core::modifier::Modifier] {
        match self {
            Element::Rect(r) => &r.modifiers,
            Element::Brush(b) => &b.modifiers,
            Element::Path(p) => &p.modifiers,
            Element::Text(t) => &t.modifiers,
            Element::Group(g) => &g.modifiers,
            Element::Image(i) => &i.modifiers,
            Element::Clone(_) => &[],
        }
    }

    pub fn modifiers_mut(&mut self) -> Option<&mut Vec<crate::core::modifier::Modifier>> {
        match self {
            Element::Rect(r) => Some(&mut r.modifiers),
            Element::Brush(b) => Some(&mut b.modifiers),
            Element::Path(p) => Some(&mut p.modifiers),
            Element::Text(t) => Some(&mut t.modifiers),
            Element::Group(g) => Some(&mut g.modifiers),
            Element::Image(i) => Some(&mut i.modifiers),
            Element::Clone(_) => None,
        }
    }

    pub fn apply_modifier(&mut self, mod_idx: usize) {
        let mods_len = self.modifiers().len();
        if mod_idx >= mods_len {
            return;
        }

        let m = self.modifiers_mut().unwrap().remove(mod_idx);
        if !m.enabled() {
            return;
        }

        match m {
            crate::core::modifier::Modifier::Array(arr) => {
                let mut children = Vec::new();
                let base_bounds = self.bounds();
                let center = base_bounds.center();

                let mut generate_copy = |dx: f32, dy: f32, rot_deg: f32, scale: f32| {
                    let mut child = self.clone();
                    child.translate(dx, dy);
                    if rot_deg.abs() > 0.001 || (scale - 1.0).abs() > 0.001 {
                        child.scale(center, scale, scale);
                    }
                    children.push(child);
                };

                match &arr.mode {
                    crate::core::modifier::ArrayMode::Linear {
                        count,
                        offset_x,
                        offset_y,
                        scale_step,
                        rotate_step_deg,
                    } => {
                        for i in 0..*count {
                            let dx = offset_x * i as f32;
                            let dy = offset_y * i as f32;
                            let scale = scale_step.powi(i as i32);
                            let rot_deg = rotate_step_deg * i as f32;
                            generate_copy(dx, dy, rot_deg, scale);
                        }
                    }
                    crate::core::modifier::ArrayMode::Radial {
                        count,
                        radius,
                        start_angle_deg,
                        total_angle_deg,
                        rotate_copies: _,
                    } => {
                        let step = if *count > 1 {
                            total_angle_deg / (*count as f32)
                        } else {
                            0.0
                        };
                        for i in 0..*count {
                            let angle_deg = start_angle_deg + step * i as f32;
                            let rad = angle_deg.to_radians();
                            let dx = radius * rad.cos();
                            let dy = radius * rad.sin();
                            generate_copy(dx, dy, angle_deg, 1.0);
                        }
                    }
                    crate::core::modifier::ArrayMode::Grid {
                        rows,
                        cols,
                        spacing_x,
                        spacing_y,
                    } => {
                        for r in 0..*rows {
                            for c in 0..*cols {
                                let dx = spacing_x * c as f32;
                                let dy = spacing_y * r as f32;
                                generate_copy(dx, dy, 0.0, 1.0);
                            }
                        }
                    }
                }

                if children.len() == 1 {
                    *self = children.remove(0);
                } else if !children.is_empty() {
                    let mut grp = crate::core::element::group::GroupElement::new(children);
                    grp.modifiers = self.modifiers().to_vec();
                    *self = Element::Group(grp);
                }
            }
            _ => {
                let evaluated_skia = match self {
                    Element::Path(p) => p.to_skia_path_evaluated(),
                    Element::Rect(r) => r.to_path_element().to_skia_path_evaluated(),
                    _ => self.to_skia_path(),
                };

                let fills = match self {
                    Element::Path(p) => p.fills.clone(),
                    Element::Rect(r) => r.fills.clone(),
                    _ => Vec::new(),
                };
                let strokes = match self {
                    Element::Path(p) => p.strokes.clone(),
                    Element::Rect(r) => r.strokes.clone(),
                    _ => Vec::new(),
                };
                let fill_c = self.fill_color();
                let stroke_c = self.stroke_color();
                let stroke_w = self.stroke_width();

                let mut new_paths = crate::core::PathElement::from_skia_path(
                    &evaluated_skia,
                    fill_c,
                    stroke_c,
                    stroke_w,
                );

                if !new_paths.is_empty() {
                    let mut new_path = new_paths.remove(0);
                    new_path.fills = fills;
                    new_path.strokes = strokes;
                    new_path.modifiers = self.modifiers().to_vec();
                    *self = Element::Path(new_path);
                }
            }
        }
    }

    pub fn render(&self, canvas: &skia::Canvas) {
        self.render_with_doc(canvas, None);
    }

    pub fn render_base_with_doc(&self, canvas: &skia::Canvas, doc: Option<&crate::core::document::Document>) {
        match self {
            Element::Rect(r) => r.render(canvas),
            Element::Brush(b) => b.render(canvas),
            Element::Path(p) => p.render(canvas),
            Element::Text(t) => t.render_with_doc(canvas, doc),
            Element::Group(g) => g.render(canvas),
            Element::Image(i) => i.render(canvas),
            Element::Clone(c) => c.render(canvas),
        }
    }

    pub fn render_with_doc(&self, canvas: &skia::Canvas, doc: Option<&crate::core::document::Document>) {
        let enabled_mods: Vec<&crate::core::modifier::Modifier> = self
            .modifiers()
            .iter()
            .filter(|m| m.enabled())
            .collect();

        if enabled_mods.is_empty() {
            self.render_base_with_doc(canvas, doc);
        } else {
            self.render_modifier_stack_step(0, &enabled_mods, canvas, doc);
        }
    }

    fn render_modifier_stack_step(
        &self,
        step_idx: usize,
        mods: &[&crate::core::modifier::Modifier],
        canvas: &skia::Canvas,
        doc: Option<&crate::core::document::Document>,
    ) {
        if step_idx >= mods.len() {
            self.render_base_with_doc(canvas, doc);
            return;
        }

        let m = mods[step_idx];
        let bounds = match self {
            Element::Rect(r) => r.bounds(),
            Element::Brush(b) => b.bounds(),
            Element::Path(p) => p.bounds(),
            Element::Text(t) => t.bounds(),
            Element::Group(g) => g.bounds(),
            Element::Image(i) => i.bounds(),
            Element::Clone(c) => c.bounds(),
        };
        let center = bounds.center();

        match m {
            crate::core::modifier::Modifier::Array(arr) => {
                match &arr.mode {
                    crate::core::modifier::ArrayMode::Linear {
                        count,
                        offset_x,
                        offset_y,
                        scale_step,
                        rotate_step_deg,
                    } => {
                        for i in 0..*count {
                            canvas.save();
                            let dx = offset_x * i as f32;
                            let dy = offset_y * i as f32;
                            let scale = scale_step.powi(i as i32);
                            let rot_deg = rotate_step_deg * i as f32;

                            canvas.translate((center.x + dx, center.y + dy));
                            if rot_deg.abs() > 0.001 {
                                canvas.rotate(rot_deg, None);
                            }
                            if (scale - 1.0).abs() > 0.001 {
                                canvas.scale((scale, scale));
                            }
                            canvas.translate((-center.x, -center.y));

                            self.render_modifier_stack_step(step_idx + 1, mods, canvas, doc);
                            canvas.restore();
                        }
                    }
                    crate::core::modifier::ArrayMode::Radial {
                        count,
                        radius,
                        start_angle_deg,
                        total_angle_deg,
                        rotate_copies,
                    } => {
                        let total_c = (*count).max(1) as f32;
                        let angle_step = if *count > 1 {
                            total_angle_deg / total_c
                        } else {
                            0.0
                        };

                        for i in 0..*count {
                            canvas.save();
                            let angle_deg = start_angle_deg + angle_step * i as f32;
                            let rad = angle_deg.to_radians();
                            let dx = radius * rad.cos();
                            let dy = radius * rad.sin();

                            canvas.translate((center.x + dx, center.y + dy));
                            if *rotate_copies {
                                canvas.rotate(angle_deg, None);
                            }
                            canvas.translate((-center.x, -center.y));

                            self.render_modifier_stack_step(step_idx + 1, mods, canvas, doc);
                            canvas.restore();
                        }
                    }
                    crate::core::modifier::ArrayMode::Grid {
                        rows,
                        cols,
                        spacing_x,
                        spacing_y,
                    } => {
                        for r in 0..*rows {
                            for c in 0..*cols {
                                canvas.save();
                                let dx = spacing_x * c as f32;
                                let dy = spacing_y * r as f32;

                                canvas.translate((dx, dy));
                                self.render_modifier_stack_step(step_idx + 1, mods, canvas, doc);
                                canvas.restore();
                            }
                        }
                    }
                }
            }
            crate::core::modifier::Modifier::Extrude3D(ext) => {
                let fill_c = self.fill_color();
                let stroke_c = self.stroke_color();
                let stroke_w = self.stroke_width();
                let path = self.to_skia_path();

                crate::core::modifier::apply_extrude_3d_to_canvas(
                    &path,
                    fill_c,
                    stroke_c,
                    stroke_w,
                    ext,
                    canvas,
                );
                self.render_modifier_stack_step(step_idx + 1, mods, canvas, doc);
            }
            _ => {
                // Non-generative modifiers (Twist, Wave, Warp, Chamfer) - continue to next step
                self.render_modifier_stack_step(step_idx + 1, mods, canvas, doc);
            }
        }
    }

    pub fn stroke_width(&self) -> f32 {
        match self {
            Element::Rect(r) => {
                if !r.strokes.is_empty() {
                    r.strokes.iter().find(|s| s.enabled).map(|s| s.width).unwrap_or(0.0)
                } else if r.stroke_color.is_some() {
                    r.stroke_width
                } else {
                    0.0
                }
            }
            Element::Brush(b) => b.width,
            Element::Path(p) => {
                if !p.strokes.is_empty() {
                    p.strokes.iter().find(|s| s.enabled).map(|s| s.width).unwrap_or(0.0)
                } else if p.stroke_color.is_some() {
                    p.stroke_width
                } else {
                    0.0
                }
            }
            Element::Text(t) => {
                if !t.strokes.is_empty() {
                    t.strokes.iter().find(|s| s.enabled).map(|s| s.width).unwrap_or(0.0)
                } else if t.stroke_color.is_some() {
                    t.stroke_width
                } else {
                    0.0
                }
            }
            Element::Group(g) => g.children.first().map(|c| c.stroke_width()).unwrap_or(0.0),
            Element::Image(_) => 0.0,
            Element::Clone(_) => 0.0,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Element::Rect(r) => r
                .name
                .clone()
                .unwrap_or_else(|| format!("Retângulo #{}", r.id.0)),
            Element::Brush(b) => b
                .name
                .clone()
                .unwrap_or_else(|| format!("Pincel #{}", b.id.0)),
            Element::Path(p) => p.name.clone().unwrap_or_else(|| match &p.shape_origin {
                Some(ShapeOrigin::Rectangle { .. }) => format!("Retângulo #{}", p.id.0),
                Some(ShapeOrigin::Triangle { .. }) => format!("Polígono #{}", p.id.0),
                Some(ShapeOrigin::Star { .. }) => format!("Estrela #{}", p.id.0),
                Some(ShapeOrigin::Circle { .. }) => format!("Círculo #{}", p.id.0),
                Some(ShapeOrigin::Spiral { .. }) => format!("Espiral #{}", p.id.0),
                None => format!("Caminho #{}", p.id.0),
            }),
            Element::Text(t) => t.name.clone().unwrap_or_else(|| {
                let preview = if t.text.len() > 14 {
                    format!("{}...", &t.text[..14])
                } else {
                    t.text.clone()
                };
                if preview.is_empty() {
                    format!("Texto #{}", t.id.0)
                } else {
                    format!("\"{}\"", preview)
                }
            }),
            Element::Group(g) => g.name.clone().unwrap_or_else(|| {
                if g.clip_element.is_some() {
                    format!("Grupo de Recorte #{}", g.id.0)
                } else {
                    format!("Grupo #{}", g.id.0)
                }
            }),
            Element::Image(i) => i
                .name
                .clone()
                .unwrap_or_else(|| format!("Imagem #{}", i.id.0)),
            Element::Clone(c) => c
                .name
                .clone()
                .unwrap_or_else(|| format!("Clone #{}", c.id.0)),
        }
    }

    pub fn visible(&self) -> bool {
        match self {
            Element::Rect(r) => r.visible,
            Element::Brush(b) => b.visible,
            Element::Path(p) => p.visible,
            Element::Text(t) => t.visible,
            Element::Group(g) => g.visible,
            Element::Image(i) => i.visible,
            Element::Clone(c) => c.visible,
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        match self {
            Element::Rect(r) => r.visible = visible,
            Element::Brush(b) => b.visible = visible,
            Element::Path(p) => p.visible = visible,
            Element::Text(t) => t.visible = visible,
            Element::Group(g) => g.visible = visible,
            Element::Image(i) => i.visible = visible,
            Element::Clone(c) => c.visible = visible,
        }
    }

    pub fn locked(&self) -> bool {
        match self {
            Element::Rect(r) => r.locked,
            Element::Brush(b) => b.locked,
            Element::Path(p) => p.locked,
            Element::Text(t) => t.locked,
            Element::Group(g) => g.locked,
            Element::Image(i) => i.locked,
            Element::Clone(c) => c.locked,
        }
    }

    pub fn set_locked(&mut self, locked: bool) {
        match self {
            Element::Rect(r) => r.locked = locked,
            Element::Brush(b) => b.locked = locked,
            Element::Path(p) => p.locked = locked,
            Element::Text(t) => t.locked = locked,
            Element::Group(g) => g.locked = locked,
            Element::Image(i) => i.locked = locked,
            Element::Clone(c) => c.locked = locked,
        }
    }

    pub fn opacity(&self) -> f32 {
        match self {
            Element::Rect(r) => r.opacity,
            Element::Brush(b) => b.opacity,
            Element::Path(p) => p.opacity,
            Element::Text(t) => t.opacity,
            Element::Group(g) => g.opacity,
            Element::Image(i) => i.opacity,
            Element::Clone(c) => c.opacity,
        }
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        let op = opacity.clamp(0.0, 1.0);
        match self {
            Element::Rect(r) => r.opacity = op,
            Element::Brush(b) => b.opacity = op,
            Element::Path(p) => p.opacity = op,
            Element::Text(t) => t.opacity = op,
            Element::Group(g) => g.opacity = op,
            Element::Image(i) => i.opacity = op,
            Element::Clone(c) => c.opacity = op,
        }
    }

    pub fn blend_mode(&self) -> BlendMode {
        match self {
            Element::Rect(r) => r.blend_mode,
            Element::Brush(b) => b.blend_mode,
            Element::Path(p) => p.blend_mode,
            Element::Text(t) => t.blend_mode,
            Element::Group(g) => g.blend_mode,
            Element::Image(i) => i.blend_mode,
            Element::Clone(c) => c.blend_mode,
        }
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        match self {
            Element::Rect(r) => r.blend_mode = mode,
            Element::Brush(b) => b.blend_mode = mode,
            Element::Path(p) => p.blend_mode = mode,
            Element::Text(t) => t.blend_mode = mode,
            Element::Group(g) => g.blend_mode = mode,
            Element::Image(i) => i.blend_mode = mode,
            Element::Clone(c) => c.blend_mode = mode,
        }
    }

    pub fn blur(&self) -> f32 {
        match self {
            Element::Rect(r) => r.blur,
            Element::Brush(b) => b.blur,
            Element::Path(p) => p.blur,
            Element::Text(t) => t.blur,
            Element::Group(g) => g.blur,
            Element::Image(i) => i.blur,
            Element::Clone(c) => c.blur_radius,
        }
    }

    pub fn set_blur(&mut self, blur: f32) {
        let b_val = blur.clamp(0.0, 1.0);
        match self {
            Element::Rect(r) => r.blur = b_val,
            Element::Brush(b) => b.blur = b_val,
            Element::Path(p) => p.blur = b_val,
            Element::Text(t) => t.blur = b_val,
            Element::Group(g) => g.blur = b_val,
            Element::Image(i) => i.blur = b_val,
            Element::Clone(c) => c.blur_radius = b_val,
        }
    }

    pub fn fills(&self) -> Vec<FillLayer> {
        match self {
            Element::Rect(r) => {
                if !r.fills.is_empty() {
                    r.fills.clone()
                } else if let Some(f) = r.fill_color {
                    vec![FillLayer::new(f)]
                } else {
                    vec![]
                }
            }
            Element::Path(p) => {
                if !p.fills.is_empty() {
                    p.fills.clone()
                } else if let Some(f) = p.fill_color {
                    vec![FillLayer::new(f)]
                } else {
                    vec![]
                }
            }
            Element::Brush(b) => vec![FillLayer::new(b.color)],
            Element::Text(t) => vec![FillLayer::new(t.color)],
            Element::Group(_) => vec![],
            Element::Image(_) => vec![],
            Element::Clone(_) => vec![],
        }
    }

    pub fn set_fills(&mut self, fills: Vec<FillLayer>) {
        match self {
            Element::Rect(r) => {
                r.fill_color = fills
                    .iter()
                    .find(|f| f.enabled)
                    .map(|f| f.color.with_alpha(f.color.a * f.opacity));
                if let Some(f0) = fills.first() {
                    match f0.style {
                        FillStyle::Solid => {
                            r.mesh_gradient = None;
                            r.gradient = None;
                        }
                        FillStyle::LinearGradient | FillStyle::RadialGradient => {
                            r.mesh_gradient = None;
                        }
                        FillStyle::Mesh => {
                            r.gradient = None;
                            if let Some(m) = &f0.mesh {
                                r.mesh_gradient = Some(m.clone());
                            }
                        }
                        FillStyle::Pattern => {
                            r.mesh_gradient = None;
                            r.gradient = None;
                        }
                    }
                }
                r.fills = fills;
            }
            Element::Path(p) => {
                p.fill_color = fills
                    .iter()
                    .find(|f| f.enabled)
                    .map(|f| f.color.with_alpha(f.color.a * f.opacity));
                if let Some(f0) = fills.first() {
                    match f0.style {
                        FillStyle::Solid => {
                            p.mesh_gradient = None;
                            p.gradient = None;
                        }
                        FillStyle::LinearGradient | FillStyle::RadialGradient => {
                            p.mesh_gradient = None;
                        }
                        FillStyle::Mesh => {
                            p.gradient = None;
                            if let Some(m) = &f0.mesh {
                                p.mesh_gradient = Some(m.clone());
                            }
                        }
                        FillStyle::Pattern => {
                            p.mesh_gradient = None;
                            p.gradient = None;
                        }
                    }
                }
                p.fills = fills;
            }
            Element::Brush(b) => {
                if let Some(f) = fills.iter().find(|f| f.enabled) {
                    b.color = f.color.with_alpha(f.color.a * f.opacity);
                }
            }
            Element::Text(t) => {
                if let Some(f) = fills.iter().find(|f| f.enabled) {
                    t.color = f.color.with_alpha(f.color.a * f.opacity);
                }
            }
            Element::Group(_) => {}
            Element::Image(_) => {}
            Element::Clone(_) => {}
        }
    }

    pub fn strokes(&self) -> Vec<StrokeLayer> {
        match self {
            Element::Rect(r) => {
                if !r.strokes.is_empty() {
                    r.strokes.clone()
                } else if let Some(s) = r.stroke_color {
                    vec![StrokeLayer::new(s, r.stroke_width)]
                } else {
                    vec![]
                }
            }
            Element::Path(p) => {
                if !p.strokes.is_empty() {
                    p.strokes.clone()
                } else if let Some(s) = p.stroke_color {
                    vec![StrokeLayer::new(s, p.stroke_width)]
                } else {
                    vec![]
                }
            }
            Element::Brush(b) => vec![StrokeLayer::new(b.color, b.width)],
            Element::Text(t) => {
                if !t.strokes.is_empty() {
                    t.strokes.clone()
                } else if let Some(s) = t.stroke_color {
                    vec![StrokeLayer::new(s, t.stroke_width)]
                } else {
                    vec![]
                }
            }
            Element::Group(_) => vec![],
            Element::Image(_) => vec![],
            Element::Clone(_) => vec![],
        }
    }

    pub fn set_strokes(&mut self, strokes: Vec<StrokeLayer>) {
        match self {
            Element::Rect(r) => {
                r.stroke_color = strokes
                    .iter()
                    .find(|s| s.enabled)
                    .map(|s| s.color.with_alpha(s.color.a * s.opacity));
                r.stroke_width = strokes
                    .iter()
                    .find(|s| s.enabled)
                    .map(|s| s.width)
                    .unwrap_or(2.0);
                r.strokes = strokes;
            }
            Element::Path(p) => {
                p.stroke_color = strokes
                    .iter()
                    .find(|s| s.enabled)
                    .map(|s| s.color.with_alpha(s.color.a * s.opacity));
                p.stroke_width = strokes
                    .iter()
                    .find(|s| s.enabled)
                    .map(|s| s.width)
                    .unwrap_or(2.0);
                p.strokes = strokes;
            }
            Element::Brush(b) => {
                if let Some(s) = strokes.iter().find(|s| s.enabled) {
                    b.color = s.color.with_alpha(s.color.a * s.opacity);
                    b.width = s.width;
                }
            }
            Element::Text(t) => {
                t.stroke_color = strokes
                    .iter()
                    .find(|s| s.enabled)
                    .map(|s| s.color.with_alpha(s.color.a * s.opacity));
                t.stroke_width = strokes
                    .iter()
                    .find(|s| s.enabled)
                    .map(|s| s.width)
                    .unwrap_or(1.0);
                t.strokes = strokes;
            }
            Element::Group(_) => {}
            Element::Image(_) => {}
            Element::Clone(_) => {}
        }
    }

    pub fn set_stroke_width(&mut self, width: f32) {
        match self {
            Element::Rect(r) => {
                r.stroke_width = width;
                if !r.strokes.is_empty() {
                    r.strokes[0].width = width;
                }
            }
            Element::Brush(b) => b.width = width,
            Element::Path(p) => {
                p.stroke_width = width;
                if !p.strokes.is_empty() {
                    p.strokes[0].width = width;
                }
            }
            Element::Text(t) => {
                t.stroke_width = width;
                if !t.strokes.is_empty() {
                    t.strokes[0].width = width;
                }
            }
            Element::Group(_) => {}
            Element::Image(_) => {}
            Element::Clone(_) => {}
        }
    }

    pub fn extract_style_snapshot(&self) -> ElementStyleSnapshot {
        ElementStyleSnapshot {
            fill_color: self.fill_color(),
            stroke_color: self.stroke_color(),
            stroke_width: self.stroke_width(),
            fills: self.fills(),
            strokes: self.strokes(),
            opacity: self.opacity(),
            blend_mode: self.blend_mode(),
            blur: self.blur(),
        }
    }

    pub fn apply_style_snapshot(&mut self, style: &ElementStyleSnapshot) {
        if !style.fills.is_empty() {
            self.set_fills(style.fills.clone());
        } else {
            self.set_fill_color(style.fill_color);
        }

        if !style.strokes.is_empty() {
            self.set_strokes(style.strokes.clone());
        } else {
            self.set_stroke_color(style.stroke_color);
            self.set_stroke_width(style.stroke_width);
        }

        self.set_opacity(style.opacity);
        self.set_blend_mode(style.blend_mode);
        self.set_blur(style.blur);
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Element::Rect(_) => "/io/gitlab/lewisHeart/GnomePaths/icons/tool-square.svg",
            Element::Brush(_) => "/io/gitlab/lewisHeart/GnomePaths/icons/tool-pen.svg",
            Element::Path(p) => match &p.shape_origin {
                Some(ShapeOrigin::Rectangle { .. }) => {
                    "/io/gitlab/lewisHeart/GnomePaths/icons/tool-square.svg"
                }
                Some(ShapeOrigin::Triangle { .. }) => {
                    "/io/gitlab/lewisHeart/GnomePaths/icons/tool-triangle.svg"
                }
                Some(ShapeOrigin::Star { .. }) => "/io/gitlab/lewisHeart/GnomePaths/icons/tool-star.svg",
                Some(ShapeOrigin::Circle { .. }) => {
                    "/io/gitlab/lewisHeart/GnomePaths/icons/tool-circle.svg"
                }
                Some(ShapeOrigin::Spiral { .. }) => {
                    "/io/gitlab/lewisHeart/GnomePaths/icons/tool-spiral.svg"
                }
                None => "/io/gitlab/lewisHeart/GnomePaths/icons/tool-vector-pen.svg",
            },
            Element::Text(_) => "/io/gitlab/lewisHeart/GnomePaths/icons/tool-text.svg",
            Element::Group(g) => {
                if g.clip_element.is_some() {
                    "crop-symbolic"
                } else {
                    "folder-symbolic"
                }
            }
            Element::Image(_) => "image-x-generic-symbolic",
            Element::Clone(_) => "/io/gitlab/lewisHeart/GnomePaths/icons/clone.svg",
        }
    }

    pub fn element_type_name(&self) -> &'static str {
        match self {
            Element::Rect(_) => "Retângulo",
            Element::Brush(_) => "Pincel",
            Element::Path(p) => match &p.shape_origin {
                Some(ShapeOrigin::Rectangle { .. }) => "Retângulo",
                Some(ShapeOrigin::Triangle { .. }) => "Polígono",
                Some(ShapeOrigin::Star { .. }) => "Estrela",
                Some(ShapeOrigin::Circle { .. }) => "Círculo",
                Some(ShapeOrigin::Spiral { .. }) => "Espiral",
                None => "Caminho Vetorial",
            },
            Element::Text(_) => "Texto",
            Element::Group(g) => {
                if g.clip_element.is_some() {
                    "Grupo de Recorte"
                } else {
                    "Grupo"
                }
            }
            Element::Image(_) => "Imagem",
            Element::Clone(_) => "Clone",
        }
    }

    pub fn type_id(&self) -> u8 {
        match self {
            Element::Rect(_) => 0,
            Element::Brush(_) => 1,
            Element::Path(p) => match &p.shape_origin {
                Some(ShapeOrigin::Rectangle { .. }) => 2,
                Some(ShapeOrigin::Triangle { .. }) => 3,
                Some(ShapeOrigin::Star { .. }) => 4,
                Some(ShapeOrigin::Circle { .. }) => 5,
                Some(ShapeOrigin::Spiral { .. }) => 6,
                None => 7,
            },
            Element::Text(_) => 8,
            Element::Group(_) => 9,
            Element::Image(_) => 10,
            Element::Clone(_) => 11,
        }
    }

    pub fn clone_with_new_id(&self) -> Element {
        let new_id = ElementId::new();
        match self {
            Element::Rect(r) => {
                let mut el = r.clone();
                el.id = new_id;
                Element::Rect(el)
            }
            Element::Brush(b) => {
                let mut el = b.clone();
                el.id = new_id;
                Element::Brush(el)
            }
            Element::Path(p) => {
                let mut el = p.clone();
                el.id = new_id;
                Element::Path(el)
            }
            Element::Text(t) => {
                let mut el = t.clone();
                el.id = new_id;
                Element::Text(el)
            }
            Element::Group(g) => {
                let mut el = g.clone();
                el.id = new_id;
                el.children = g.children.iter().map(|c| c.clone_with_new_id()).collect();
                if let Some(clip) = &g.clip_element {
                    el.clip_element = Some(Box::new(clip.clone_with_new_id()));
                }
                Element::Group(el)
            }
            Element::Image(i) => {
                let mut el = i.clone();
                el.id = new_id;
                Element::Image(el)
            }
            Element::Clone(c) => {
                let mut el = c.clone();
                el.id = new_id;
                Element::Clone(el)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_element_properties() {
        let mut text = TextElement::new(
            Point::new(100.0, 100.0),
            "Hello World\nLine 2".to_string(),
            24.0,
            Color::BLACK,
        );
        text.font_family = "Inter".to_string();
        text.font_weight = 700;
        text.line_height = 1.5;
        text.alignment = TextAlign::Center;

        let bounds = text.bounds();
        assert!(bounds.width > 0.0);
        assert!(bounds.height > 24.0);
    }

    #[test]
    fn test_gradient_transformations() {
        let mut grad = Gradient::new_linear(
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
            Color::BLUE,
            Color::RED,
        );
        grad.translate(50.0, -20.0);
        assert_eq!(grad.start, Point::new(50.0, -20.0));
        assert_eq!(grad.end, Point::new(150.0, 80.0));

        grad.scale(Point::new(50.0, -20.0), 2.0, 2.0);
        assert_eq!(grad.start, Point::new(50.0, -20.0));
        assert_eq!(grad.end, Point::new(250.0, 180.0));
    }

    #[test]
    fn test_mesh_gradient_grid() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut mesh = MeshGradient::new_grid(rect, 3, 3, Color::BLUE, Color::RED);
        assert_eq!(mesh.nodes.len(), 9);
        assert_eq!(mesh.nodes[0].point, Point::new(0.0, 0.0));
        assert_eq!(mesh.nodes[8].point, Point::new(100.0, 100.0));

        mesh.translate(10.0, 10.0);
        assert_eq!(mesh.nodes[0].point, Point::new(10.0, 10.0));
    }

    #[test]
    fn test_pattern_and_mesh_fill_layers() {
        let mut pattern_fill = FillLayer::new(Color::WHITE);
        pattern_fill.style = FillStyle::Pattern;
        pattern_fill.secondary_color = Color::BLACK;
        pattern_fill.pattern_type = PatternType::Checkerboard;
        pattern_fill.pattern_scale = 16.0;
        assert_eq!(pattern_fill.style, FillStyle::Pattern);
        assert_eq!(pattern_fill.pattern_type, PatternType::Checkerboard);

        let bounds = Rect::new(0.0, 0.0, 200.0, 200.0);
        let paint = style::create_fill_paint(&pattern_fill, bounds);
        assert!(paint.shader().is_some());

        let types = [
            PatternType::Checkerboard,
            PatternType::Dots,
            PatternType::Stripes,
            PatternType::Grid,
            PatternType::Hexagon,
        ];
        for pt in types {
            let shader =
                style::create_pattern_shader(pt, Color::RED, Color::BLUE, 20.0, 45.0, Point::ZERO, None);
            assert!(shader.is_some());
        }

        let mesh = MeshGradient::new_grid(bounds, 2, 2, Color::RED, Color::BLUE);
        let mut mesh_fill = FillLayer::new(Color::new(0.2, 0.5, 0.9, 1.0));
        mesh_fill.style = FillStyle::Mesh;
        mesh_fill.mesh = Some(mesh);
        assert_eq!(mesh_fill.style, FillStyle::Mesh);
        assert!(mesh_fill.mesh.is_some());
    }

    #[test]
    fn test_rect_corner_radii_and_styles() {
        let mut rect = RectElement::new(Rect::new(0.0, 0.0, 100.0, 100.0), None, None);
        assert_eq!(rect.effective_radii().max_radius(), 0.0);
        let sharp_path = rect.to_path_element();
        assert_eq!(sharp_path.nodes.len(), 4);

        rect.corner_radius = 20.0;
        assert_eq!(rect.effective_radii().max_radius(), 20.0);
        let round_path = rect.to_path_element();
        assert_eq!(round_path.nodes.len(), 8);

        rect.corner_style = CornerStyle::Chamfer;
        let chamfer_path = rect.to_path_element();
        assert_eq!(chamfer_path.nodes.len(), 8);

        rect.corner_style = CornerStyle::Concave;
        let concave_path = rect.to_path_element();
        assert_eq!(concave_path.nodes.len(), 8);

        // Non-uniform corner radii
        rect.corner_radii = CornerRadii::new(10.0, 0.0, 15.0, 0.0);
        rect.corner_style = CornerStyle::Round;
        let indiv_path = rect.to_path_element();
        assert_eq!(indiv_path.nodes.len(), 6);
    }

    #[test]
    fn test_path_node_editing_and_de_casteljau() {
        use crate::core::NodeType;

        let mut path = PathElement::new(
            vec![
                PathNode::new(Point::new(0.0, 0.0)),
                PathNode::new(Point::new(100.0, 0.0)),
                PathNode::new(Point::new(100.0, 100.0)),
                PathNode::new(Point::new(0.0, 100.0)),
            ],
            true,
            None,
            None,
            1.0,
        );

        path.convert_nodes_type(&[0], NodeType::Smooth);
        assert_eq!(path.nodes[0].node_type, NodeType::Smooth);
        assert!(path.nodes[0].handle_in.is_some() || path.nodes[0].handle_out.is_some());

        path.convert_nodes_type(&[1], NodeType::Symmetric);
        assert_eq!(path.nodes[1].node_type, NodeType::Symmetric);

        path.convert_nodes_type(&[0, 1], NodeType::Corner);
        assert_eq!(path.nodes[0].node_type, NodeType::Corner);
        assert_eq!(path.nodes[1].node_type, NodeType::Corner);

        path.make_segments_curve(&[0, 1]);
        assert!(path.nodes[0].handle_out.is_some());
        assert!(path.nodes[1].handle_in.is_some());

        path.make_segments_straight(&[0, 1]);
        assert!(path.nodes[0].handle_out.is_none());
        assert!(path.nodes[1].handle_in.is_none());

        let initial_count = path.nodes.len();
        let inserted_idx = path.insert_node_at_segment(0, 0.5);
        assert_eq!(path.nodes.len(), initial_count + 1);
        assert_eq!(inserted_idx, 1);
        assert_eq!(path.nodes[1].point.x.round(), 50.0);
        assert_eq!(path.nodes[1].point.y.round(), 0.0);

        path.align_nodes_horizontal(&[0, 1, 2]);
        assert_eq!(path.nodes[0].point.y, path.nodes[1].point.y);
        assert_eq!(path.nodes[1].point.y, path.nodes[2].point.y);

        assert!(path.is_closed);
        path.toggle_closed();
        assert!(!path.is_closed);

        let first_pt = path.nodes.first().unwrap().point;
        let last_pt = path.nodes.last().unwrap().point;
        path.reverse_direction();
        assert_eq!(path.nodes.first().unwrap().point, last_pt);
        assert_eq!(path.nodes.last().unwrap().point, first_pt);

        path.delete_nodes(&[0]);
        assert_eq!(path.nodes.len(), initial_count);

        // Test toggle_node_smooth_corner
        path.toggle_node_smooth_corner(0);
        assert_eq!(path.nodes[0].node_type, NodeType::Smooth);
        assert!(path.nodes[0].handle_in.is_some() || path.nodes[0].handle_out.is_some());

        path.toggle_node_smooth_corner(0);
        assert_eq!(path.nodes[0].node_type, NodeType::Corner);
        assert!(path.nodes[0].handle_in.is_none() && path.nodes[0].handle_out.is_none());

        // Test bend_segment
        path.bend_segment(0, Point::new(10.0, 20.0), 0.5);
        assert!(path.nodes[0].handle_out.is_some());
        assert!(path.nodes[1].handle_in.is_some());
    }

    #[test]
    fn test_brush_stroke_and_path_conversion() {
        let pts = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 25.0),
            Point::new(100.0, 0.0),
            Point::new(150.0, 50.0),
        ];
        let mut stroke = BrushStroke::new(pts, Color::BLACK, 4.0);
        stroke.style = BrushStyle::Calligraphy;
        stroke.smoothing = 0.5;
        stroke.auto_close = false;
        stroke.cap_style = StrokeCap::Square;

        assert_eq!(stroke.style, BrushStyle::Calligraphy);
        assert_eq!(stroke.cap_style, StrokeCap::Square);
        assert!(stroke.bounds().width > 0.0);

        let path_elem = stroke.to_path_element();
        assert_eq!(path_elem.nodes.len(), 4);
        assert!(!path_elem.is_closed);
        assert_eq!(path_elem.stroke_width, 4.0);
        // Catmull-Rom generated smooth handles for intermediate nodes
        assert!(path_elem.nodes[1].handle_in.is_some());
        assert!(path_elem.nodes[1].handle_out.is_some());
    }

    #[test]
    fn test_mesh_gradient_per_node_colors() {
        let mut mesh = MeshGradient::new_grid(
            crate::core::Rect::new(0.0, 0.0, 100.0, 100.0),
            3,
            3,
            Color::RED,
            Color::BLUE,
        );
        assert_eq!(mesh.nodes.len(), 9);
        assert_eq!(mesh.nodes[0].color, Color::RED);

        // Customize each node independently
        let new_col = Color::from_hex("#00ff00").unwrap();
        mesh.nodes[4].color = new_col;
        assert_eq!(mesh.nodes[4].color, new_col);
        assert_ne!(mesh.nodes[0].color, mesh.nodes[4].color);
    }

    #[test]
    fn test_mesh_gradient_subdivision() {
        let mut mesh = MeshGradient::new_grid(
            crate::core::Rect::new(0.0, 0.0, 100.0, 100.0),
            2,
            2,
            Color::RED,
            Color::BLUE,
        );
        assert_eq!(mesh.rows, 2);
        assert_eq!(mesh.cols, 2);
        assert_eq!(mesh.nodes.len(), 4);

        // Subdivide at point (50.0, 50.0) with green color
        let green = Color::from_hex("#00ff00").unwrap();
        let new_idx = mesh.subdivide_at(Point::new(50.0, 50.0), Some(green));
        assert_eq!(mesh.rows, 3);
        assert_eq!(mesh.cols, 3);
        assert_eq!(mesh.nodes.len(), 9);
        assert_eq!(mesh.nodes[new_idx].color, green);
        assert_eq!(mesh.nodes[new_idx].point, Point::new(50.0, 50.0));
    }

    #[test]
    fn test_element_apply_modifier_baking() {
        use crate::core::modifier::{ArrayMode, ArrayModifier, Modifier};
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut rect_elem = RectElement::new(rect, Some(Color::RED), None);
        rect_elem.modifiers.push(Modifier::Array(ArrayModifier {
            enabled: true,
            mode: ArrayMode::Linear {
                count: 3,
                offset_x: 40.0,
                offset_y: 0.0,
                scale_step: 1.0,
                rotate_step_deg: 0.0,
            },
        }));

        let mut elem = Element::Rect(rect_elem);
        assert_eq!(elem.modifiers().len(), 1);

        elem.apply_modifier(0);

        // After baking, modifier is removed and element is baked into a Group!
        assert_eq!(elem.modifiers().len(), 0);
        match elem {
            Element::Group(g) => {
                assert_eq!(g.children.len(), 3);
            }
            _ => panic!("Expected GroupElement after applying ArrayModifier"),
        }
    }
}
