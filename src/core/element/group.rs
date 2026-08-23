use skia_safe as skia;

use crate::core::geometry::{Point, Rect};
use super::style::BlendMode;
use super::{Element, ElementId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GroupElement {
    pub id: ElementId,
    pub name: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur: f32,
    pub children: Vec<Element>,
    pub clip_element: Option<Box<Element>>,
}

impl GroupElement {
    pub fn new(children: Vec<Element>) -> Self {
        Self {
            id: ElementId::new(),
            name: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            blur: 0.0,
            children,
            clip_element: None,
        }
    }

    pub fn bounds(&self) -> Rect {
        if let Some(clip) = &self.clip_element {
            return clip.bounds();
        }
        if self.children.is_empty() {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }
        let mut b = self.children[0].bounds();
        for child in &self.children[1..] {
            b = b.union(child.bounds());
        }
        b
    }

    pub fn hit_test(&self, p: Point) -> bool {
        if let Some(clip) = &self.clip_element {
            if !clip.hit_test(p) {
                return false;
            }
        }
        self.children.iter().any(|c| c.hit_test(p))
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        for child in &mut self.children {
            child.translate(dx, dy);
        }
        if let Some(clip) = &mut self.clip_element {
            clip.translate(dx, dy);
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        for child in &mut self.children {
            child.scale(origin, sx, sy);
        }
        if let Some(clip) = &mut self.clip_element {
            clip.scale(origin, sx, sy);
        }
    }

    pub fn rotate(&mut self, center: Point, angle_rad: f32) {
        for child in &mut self.children {
            child.rotate(center, angle_rad);
        }
        if let Some(clip) = &mut self.clip_element {
            clip.rotate(center, angle_rad);
        }
    }

    pub fn render(&self, canvas: &skia::Canvas) {
        if !self.visible {
            return;
        }
        if let Some(clip) = &self.clip_element {
            let path = clip.to_skia_path();
            canvas.save();
            canvas.clip_path(&path, None, true);
            for child in &self.children {
                self.render_child(child, canvas);
            }
            canvas.restore();
        } else {
            for child in &self.children {
                self.render_child(child, canvas);
            }
        }
    }

    fn render_child(&self, child: &Element, canvas: &skia::Canvas) {
        if !child.visible() {
            return;
        }
        if child.opacity() < 0.999
            || child.blur() > 0.001
            || child.blend_mode() != BlendMode::Normal
        {
            let mut layer_paint = skia::Paint::default();
            layer_paint.set_alpha_f(child.opacity().clamp(0.0, 1.0));
            layer_paint.set_blend_mode(child.blend_mode().to_skia());
            if child.blur() > 0.001 {
                let sigma = child.blur() * 40.0;
                if let Some(blur_filter) =
                    skia::image_filters::blur((sigma, sigma), skia::TileMode::Decal, None, None)
                {
                    layer_paint.set_image_filter(blur_filter);
                }
            }
            canvas.save_layer(&skia::canvas::SaveLayerRec::default().paint(&layer_paint));
            child.render(canvas);
            canvas.restore();
        } else {
            child.render(canvas);
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImageElement {
    pub id: ElementId,
    pub name: Option<String>,
    pub rect: Rect,
    pub image_data: Vec<u8>,
    pub opacity: f32,
    pub visible: bool,
    pub locked: bool,
    pub blend_mode: BlendMode,
    pub blur: f32,
}

impl ImageElement {
    pub fn new(rect: Rect, image_data: Vec<u8>, name: Option<String>) -> Self {
        Self {
            id: ElementId::new(),
            name,
            rect,
            image_data,
            opacity: 1.0,
            visible: true,
            locked: false,
            blend_mode: BlendMode::Normal,
            blur: 0.0,
        }
    }

    pub fn bounds(&self) -> Rect {
        self.rect.normalize()
    }

    pub fn hit_test(&self, p: Point) -> bool {
        self.bounds().contains(p)
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.rect.x += dx;
        self.rect.y += dy;
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        self.rect.x = origin.x + (self.rect.x - origin.x) * sx;
        self.rect.y = origin.y + (self.rect.y - origin.y) * sy;
        self.rect.width *= sx;
        self.rect.height *= sy;
    }

    pub fn render(&self, canvas: &skia::Canvas) {
        if let Some(image) = skia::Image::from_encoded(skia::Data::new_copy(&self.image_data)) {
            let r = self.rect.normalize();
            let dst = skia::Rect::from_xywh(r.x, r.y, r.width, r.height);
            let mut paint = skia::Paint::default();
            paint.set_anti_alias(true);
            canvas.draw_image_rect_with_sampling_options(
                image,
                None,
                dst,
                skia::SamplingOptions::default(),
                &paint,
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CloneElement {
    pub id: ElementId,
    pub source_id: ElementId,
    pub offset: Point,
    pub scale: Point,
    pub rotation: f32,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur_radius: f32,
    pub visible: bool,
    pub locked: bool,
    pub name: Option<String>,
    #[serde(default)]
    pub cached_bounds: Option<Rect>,
}

impl CloneElement {
    pub fn new(source_id: ElementId, offset: Point) -> Self {
        Self {
            id: ElementId::new(),
            source_id,
            offset,
            scale: Point::new(1.0, 1.0),
            rotation: 0.0,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            blur_radius: 0.0,
            visible: true,
            locked: false,
            name: None,
            cached_bounds: None,
        }
    }

    pub fn bounds(&self) -> Rect {
        if let Some(b) = self.cached_bounds {
            b
        } else {
            Rect::new(self.offset.x, self.offset.y, 0.0, 0.0)
        }
    }

    pub fn hit_test(&self, p: Point) -> bool {
        self.bounds().contains(p)
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.offset.x += dx;
        self.offset.y += dy;
        if let Some(ref mut b) = self.cached_bounds {
            b.x += dx;
            b.y += dy;
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        self.offset.x = origin.x + (self.offset.x - origin.x) * sx;
        self.offset.y = origin.y + (self.offset.y - origin.y) * sy;
        self.scale.x *= sx;
        self.scale.y *= sy;
        if let Some(ref mut b) = self.cached_bounds {
            b.x = origin.x + (b.x - origin.x) * sx;
            b.y = origin.y + (b.y - origin.y) * sy;
            b.width *= sx.abs();
            b.height *= sy.abs();
        }
    }

    pub fn rotate(&mut self, center: Point, angle_rad: f32) {
        let dx = self.offset.x - center.x;
        let dy = self.offset.y - center.y;
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        self.offset.x = center.x + (dx * cos_a - dy * sin_a);
        self.offset.y = center.y + (dx * sin_a + dy * cos_a);
        self.rotation += angle_rad;
        if let Some(ref mut b) = self.cached_bounds {
            let cx = b.x + b.width * 0.5;
            let cy = b.y + b.height * 0.5;
            let bx = center.x + ((cx - center.x) * cos_a - (cy - center.y) * sin_a);
            let by = center.y + ((cx - center.x) * sin_a + (cy - center.y) * cos_a);
            b.x = bx - b.width * 0.5;
            b.y = by - b.height * 0.5;
        }
    }

    pub fn render(&self, _canvas: &skia::Canvas) {
        // Rendered via document/renderer with master lookup
    }
}
