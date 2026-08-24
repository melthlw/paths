pub mod brush;
pub mod group;
pub mod path;
pub mod rect;
pub mod style;
pub mod text;

#[allow(unused_imports)]
pub use brush::BrushStroke;
#[allow(unused_imports)]
pub use group::{CloneElement, GroupElement, ImageElement};
#[allow(unused_imports)]
pub use path::{dist_to_segment, ArcMode, PathElement, PathNode, ShapeOrigin};
#[allow(unused_imports)]
pub use rect::{CornerRadii, CornerStyle, RectElement};
#[allow(unused_imports)]
pub use style::{
    create_fill_paint, create_pattern_shader, create_skia_gradient_shader, render_mesh_gradient,
    BlendMode, FillLayer, FillStyle, Gradient, GradientStop, GradientType, MeshGradient, MeshNode,
    PatternType, StrokeLayer, StrokeStyle,
};
#[allow(unused_imports)]
pub use text::{get_cached_typeface, get_system_font_families, TextAlign, TextElement};

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
        match self {
            Element::Rect(r) => r.bounds(),
            Element::Brush(b) => b.bounds(),
            Element::Path(p) => p.bounds(),
            Element::Text(t) => t.bounds(),
            Element::Group(g) => g.bounds(),
            Element::Image(i) => i.bounds(),
            Element::Clone(c) => c.bounds(),
        }
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
        match self {
            Element::Rect(r) => r.translate(dx, dy),
            Element::Brush(b) => b.translate(dx, dy),
            Element::Path(p) => p.translate(dx, dy),
            Element::Text(t) => t.translate(dx, dy),
            Element::Group(g) => g.translate(dx, dy),
            Element::Image(i) => i.translate(dx, dy),
            Element::Clone(c) => c.translate(dx, dy),
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        match self {
            Element::Rect(r) => r.scale(origin, sx, sy),
            Element::Brush(b) => b.scale(origin, sx, sy),
            Element::Path(p) => p.scale(origin, sx, sy),
            Element::Text(t) => t.scale(origin, sx, sy),
            Element::Group(g) => g.scale(origin, sx, sy),
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

    pub fn stroke_color(&self) -> Option<Color> {
        match self {
            Element::Rect(r) => r.stroke_color,
            Element::Path(p) => p.stroke_color,
            Element::Brush(b) => Some(b.color),
            Element::Text(_) => None,
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
                    out.push(Some(fc));
                }
                for f in &r.fills {
                    if f.enabled {
                        out.push(Some(f.color));
                        if f.secondary_color.a > 0.01 {
                            out.push(Some(f.secondary_color));
                        }
                    }
                }
                if let Some(s) = r.stroke_color {
                    out.push(Some(s));
                }
                for s in &r.strokes {
                    if s.enabled {
                        out.push(Some(s.color));
                    }
                }
            }
            Element::Path(p) => {
                if let Some(fc) = p.fill_color {
                    out.push(Some(fc));
                }
                for f in &p.fills {
                    if f.enabled {
                        out.push(Some(f.color));
                        if f.secondary_color.a > 0.01 {
                            out.push(Some(f.secondary_color));
                        }
                    }
                }
                if let Some(s) = p.stroke_color {
                    out.push(Some(s));
                }
                for s in &p.strokes {
                    if s.enabled {
                        out.push(Some(s.color));
                    }
                }
            }
            Element::Text(t) => {
                out.push(Some(t.color));
            }
            Element::Brush(b) => {
                out.push(Some(b.color));
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
            Element::Text(_) => {}
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
            Element::Text(t) => {
                let r = t.bounds();
                let mut builder = skia::PathBuilder::new();
                builder.add_rect(r.to_skia(), None, None);
                builder.detach()
            }
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

    pub fn render(&self, canvas: &skia::Canvas) {
        match self {
            Element::Rect(r) => r.render(canvas),
            Element::Brush(b) => b.render(canvas),
            Element::Path(p) => p.render(canvas),
            Element::Text(t) => t.render(canvas),
            Element::Group(g) => g.render(canvas),
            Element::Image(i) => i.render(canvas),
            Element::Clone(c) => c.render(canvas),
        }
    }

    pub fn stroke_width(&self) -> f32 {
        match self {
            Element::Rect(r) => r.stroke_width,
            Element::Brush(b) => b.width,
            Element::Path(p) => p.stroke_width,
            Element::Text(_) => 0.0,
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
                r.fills = fills;
            }
            Element::Path(p) => {
                p.fill_color = fills
                    .iter()
                    .find(|f| f.enabled)
                    .map(|f| f.color.with_alpha(f.color.a * f.opacity));
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
            Element::Text(_) => vec![],
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
            Element::Text(_) => {}
            Element::Group(_) => {}
            Element::Image(_) => {}
            Element::Clone(_) => {}
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Element::Rect(_) => "/io/github/lewis/GnomePaths/icons/tool-square.svg",
            Element::Brush(_) => "/io/github/lewis/GnomePaths/icons/tool-pen.svg",
            Element::Path(p) => match &p.shape_origin {
                Some(ShapeOrigin::Rectangle { .. }) => {
                    "/io/github/lewis/GnomePaths/icons/tool-square.svg"
                }
                Some(ShapeOrigin::Triangle { .. }) => {
                    "/io/github/lewis/GnomePaths/icons/tool-triangle.svg"
                }
                Some(ShapeOrigin::Star { .. }) => "/io/github/lewis/GnomePaths/icons/tool-star.svg",
                Some(ShapeOrigin::Circle { .. }) => {
                    "/io/github/lewis/GnomePaths/icons/tool-circle.svg"
                }
                Some(ShapeOrigin::Spiral { .. }) => {
                    "/io/github/lewis/GnomePaths/icons/tool-spiral.svg"
                }
                None => "/io/github/lewis/GnomePaths/icons/tool-vector-pen.svg",
            },
            Element::Text(_) => "/io/github/lewis/GnomePaths/icons/tool-text.svg",
            Element::Group(g) => {
                if g.clip_element.is_some() {
                    "crop-symbolic"
                } else {
                    "folder-symbolic"
                }
            }
            Element::Image(_) => "image-x-generic-symbolic",
            Element::Clone(_) => "/io/github/lewis/GnomePaths/icons/clone.svg",
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
        let paint = create_fill_paint(&pattern_fill, bounds);
        assert!(paint.shader().is_some());

        let types = [
            PatternType::Checkerboard,
            PatternType::Dots,
            PatternType::Stripes,
            PatternType::Grid,
            PatternType::Hexagon,
        ];
        for pt in types {
            let shader = create_pattern_shader(pt, Color::RED, Color::BLUE, 20.0, 45.0);
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
    }
}
