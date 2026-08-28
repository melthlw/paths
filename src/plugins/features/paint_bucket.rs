use skia_safe as skia;

use crate::core::element::{Element, PathElement};
use crate::core::{ElementId, Point, PointerButton, PointerEvent, Viewport};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum PaintBucketMode {
    CreateNew,
    PaintExisting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintBucketTarget {
    Fill,
    Stroke,
    Both,
}

pub struct PaintBucketFeature {
    pub mode: PaintBucketMode,
    pub target: PaintBucketTarget,
    pub detect_intersection: bool,
    pub auto_select: bool,
    pub offset: f32,
    hover_id: Option<ElementId>,
    hover_pos: Option<Point>,
}

impl Default for PaintBucketFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl PaintBucketFeature {
    pub fn new() -> Self {
        Self {
            mode: PaintBucketMode::CreateNew,
            target: PaintBucketTarget::Fill,
            detect_intersection: true,
            auto_select: true,
            offset: 0.0,
            hover_id: None,
            hover_pos: None,
        }
    }

    /// Computes the exact bounded sub-region path for the clicked element.
    /// If `detect_intersection` is true, intersects and subtracts overlapping elements.
    pub fn compute_paint_region(
        elements: &[Element],
        hit_idx: usize,
        pos: Point,
        detect_intersection: bool,
    ) -> Option<skia::Path> {
        let base_elem = elements.get(hit_idx)?;
        let mut sk_path = base_elem.to_skia_path();

        if detect_intersection {
            let base_bounds = base_elem.bounds();
            for (i, other) in elements.iter().enumerate() {
                if i == hit_idx {
                    continue;
                }
                if !other.bounds().intersects(base_bounds) {
                    continue;
                }
                let other_path = other.to_skia_path();
                if other.hit_test(pos) {
                    if let Some(inter) = sk_path.op(&other_path, skia::PathOp::Intersect) {
                        if !inter.is_empty() {
                            sk_path = inter;
                        }
                    }
                } else if let Some(diff) = sk_path.op(&other_path, skia::PathOp::Difference) {
                    if !diff.is_empty() {
                        sk_path = diff;
                    }
                }
            }
        }

        Some(sk_path)
    }
}

impl FeaturePlugin for PaintBucketFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:eyedropper");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        // Find clicked element (from topmost to bottom)
        let mut hit_idx = None;
        for (i, elem) in ctx.document.elements.iter().enumerate().rev() {
            if elem.hit_test(event.world_pos) {
                hit_idx = Some(i);
                break;
            }
        }

        let hit_idx = match hit_idx {
            Some(i) => i,
            None => return,
        };

        let base_elem = &ctx.document.elements[hit_idx];

        match self.mode {
            PaintBucketMode::CreateNew => {
                let sk_path = Self::compute_paint_region(
                    &ctx.document.elements,
                    hit_idx,
                    event.world_pos,
                    self.detect_intersection,
                )
                .unwrap_or_else(|| base_elem.to_skia_path());

                let mut is_stroke = self.target == PaintBucketTarget::Stroke;
                let is_both = self.target == PaintBucketTarget::Both;
                if event.shift_pressed {
                    // Shift toggles fill vs stroke
                    if is_stroke {
                        is_stroke = false;
                    } else if !is_both {
                        is_stroke = true;
                    }
                }

                let fill_color = if is_stroke {
                    None
                } else {
                    Some(ctx.active_fill_color)
                };

                let stroke_color = if is_stroke || is_both {
                    ctx.active_stroke_color.or(Some(ctx.active_fill_color))
                } else {
                    None
                };

                let stroke_width = if stroke_color.is_some() {
                    ctx.active_stroke_width.max(1.0)
                } else {
                    1.0
                };

                let mut candidate_elems =
                    PathElement::from_skia_path(&sk_path, fill_color, stroke_color, stroke_width);

                let chosen = if candidate_elems.len() == 1 {
                    candidate_elems.pop()
                } else {
                    let pos = event.world_pos;
                    if let Some(pos_idx) = candidate_elems.iter().position(|el| el.hit_test(pos)) {
                        Some(candidate_elems.remove(pos_idx))
                    } else {
                        candidate_elems.pop()
                    }
                };

                let mut new_elem = chosen.unwrap_or_else(|| {
                    let mut fallback = PathElement::from_skia_path(
                        &base_elem.to_skia_path(),
                        fill_color,
                        stroke_color,
                        stroke_width,
                    );
                    fallback.pop().unwrap_or_else(|| match base_elem {
                        Element::Rect(r) => {
                            let mut p = r.to_path_element();
                            p.id = ElementId::new();
                            p.fill_color = fill_color;
                            p.stroke_color = stroke_color;
                            p.stroke_width = stroke_width;
                            p
                        }
                        Element::Brush(b) => {
                            let mut p = b.to_path_element();
                            p.id = ElementId::new();
                            p.fill_color = fill_color;
                            p.stroke_color = stroke_color;
                            p.stroke_width = stroke_width;
                            p
                        }
                        Element::Path(p) => {
                            let mut p = p.clone();
                            p.id = ElementId::new();
                            p.fill_color = fill_color;
                            p.stroke_color = stroke_color;
                            p.stroke_width = stroke_width;
                            p
                        }
                        _ => PathElement::new(vec![], true, fill_color, stroke_color, stroke_width),
                    })
                });

                if self.offset.abs() > 0.001 {
                    let bounds = new_elem.bounds();
                    if bounds.width > 0.0 && bounds.height > 0.0 {
                        let center = bounds.center();
                        let sx = ((bounds.width + self.offset * 2.0).max(1.0)) / bounds.width;
                        let sy = ((bounds.height + self.offset * 2.0).max(1.0)) / bounds.height;
                        new_elem.scale(center, sx, sy);
                    }
                }

                ctx.document.snapshot();
                let new_id = new_elem.id;
                let insert_idx = (hit_idx + 1).min(ctx.document.elements.len());
                ctx.document
                    .elements
                    .insert(insert_idx, Element::Path(new_elem));
                if self.auto_select {
                    ctx.document.select(new_id, false);
                }
                ctx.request_redraw();
            }
            PaintBucketMode::PaintExisting => {
                ctx.document.snapshot();
                let el = &mut ctx.document.elements[hit_idx];
                let id = el.id();
                let is_stroke = event.shift_pressed || self.target == PaintBucketTarget::Stroke;

                if is_stroke {
                    let stroke_c = ctx.active_stroke_color.unwrap_or(ctx.active_fill_color);
                    el.set_stroke_color(Some(stroke_c));
                } else if self.target == PaintBucketTarget::Both {
                    el.set_fill_color(Some(ctx.active_fill_color));
                    let stroke_c = ctx.active_stroke_color.unwrap_or(ctx.active_fill_color);
                    el.set_stroke_color(Some(stroke_c));
                } else {
                    el.set_fill_color(Some(ctx.active_fill_color));
                }

                ctx.document.select(id, false);
                ctx.request_redraw();
            }
        }
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        let mut hit = None;
        for elem in ctx.document.elements.iter().rev() {
            if elem.hit_test(event.world_pos) {
                hit = Some(elem.id());
                break;
            }
        }

        self.hover_pos = Some(event.world_pos);

        if self.hover_id != hit {
            self.hover_id = hit;
            ctx.request_redraw();
        } else if self.mode == PaintBucketMode::CreateNew && self.detect_intersection {
            ctx.request_redraw();
        }
        ctx.set_cursor("crosshair");
    }

    fn on_pointer_up(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.hover_id = None;
        self.hover_pos = None;
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn as_paint_bucket_feature(&self) -> Option<&PaintBucketFeature> {
        Some(self)
    }

    fn as_paint_bucket_feature_mut(&mut self) -> Option<&mut PaintBucketFeature> {
        Some(self)
    }

    fn render_overlay(
        &self,
        ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        let (hit_idx, hover_pos) = match (self.hover_id, self.hover_pos) {
            (Some(id), Some(pos)) => {
                if let Some(idx) = ctx.document.elements.iter().position(|e| e.id() == id) {
                    (idx, pos)
                } else {
                    return;
                }
            }
            _ => return,
        };

        let zoom = viewport.zoom;
        let preview_path = if self.mode == PaintBucketMode::CreateNew {
            Self::compute_paint_region(
                &ctx.document.elements,
                hit_idx,
                hover_pos,
                self.detect_intersection,
            )
        } else {
            Some(ctx.document.elements[hit_idx].to_skia_path())
        };

        if let Some(sk_path) = preview_path {
            // Fill preview in CreateNew mode
            if self.mode == PaintBucketMode::CreateNew {
                let mut fill_paint = skia::Paint::default();
                fill_paint.set_color4f(skia::Color4f::new(0.2, 0.65, 1.0, 0.22), None);
                fill_paint.set_style(skia::PaintStyle::Fill);
                fill_paint.set_anti_alias(true);
                canvas.draw_path(&sk_path, &fill_paint);
            }

            // Outline
            let mut stroke_paint = skia::Paint::default();
            stroke_paint.set_color4f(skia::Color4f::new(0.2, 0.65, 1.0, 0.85), None);
            stroke_paint.set_stroke_width(2.0 / zoom);
            stroke_paint.set_style(skia::PaintStyle::Stroke);
            stroke_paint.set_anti_alias(true);
            let intervals = [4.0 / zoom, 3.0 / zoom];
            stroke_paint.set_path_effect(skia::dash_path_effect::new(&intervals, 0.0));
            canvas.draw_path(&sk_path, &stroke_paint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Rect;
    use crate::core::element::RectElement;

    #[test]
    fn test_paint_bucket_defaults() {
        let feature = PaintBucketFeature::new();
        assert_eq!(feature.mode, PaintBucketMode::CreateNew);
        assert_eq!(feature.target, PaintBucketTarget::Fill);
        assert!(feature.detect_intersection);
        assert!(feature.auto_select);
        assert_eq!(feature.offset, 0.0);
    }

    #[test]
    fn test_compute_paint_region_single_element() {
        let rect = RectElement::new(Rect::new(0.0, 0.0, 100.0, 100.0), None, None);
        let elements = vec![Element::Rect(rect)];

        let region =
            PaintBucketFeature::compute_paint_region(&elements, 0, Point::new(50.0, 50.0), true);

        assert!(region.is_some());
        let sk_path = region.unwrap();
        assert!(!sk_path.is_empty());
        assert!(sk_path.contains(skia_safe::Point::new(50.0, 50.0)));
    }

    #[test]
    fn test_compute_paint_region_overlapping_elements() {
        // Element 0: [0, 0, 100, 100]
        // Element 1: [50, 0, 100, 100] (overlaps between X=50..100)
        let rect1 = RectElement::new(Rect::new(0.0, 0.0, 100.0, 100.0), None, None);
        let rect2 = RectElement::new(Rect::new(50.0, 0.0, 100.0, 100.0), None, None);
        let elements = vec![Element::Rect(rect1), Element::Rect(rect2)];

        // Click in non-overlapping part of rect1 (X=25, Y=50)
        let region_diff =
            PaintBucketFeature::compute_paint_region(&elements, 0, Point::new(25.0, 50.0), true);
        assert!(region_diff.is_some());
        let path_diff = region_diff.unwrap();
        assert!(path_diff.contains(skia_safe::Point::new(25.0, 50.0)));
        // Should NOT contain the overlapping part
        assert!(!path_diff.contains(skia_safe::Point::new(75.0, 50.0)));

        // Click in overlapping intersection part (X=75, Y=50)
        let region_inter =
            PaintBucketFeature::compute_paint_region(&elements, 0, Point::new(75.0, 50.0), true);
        assert!(region_inter.is_some());
        let path_inter = region_inter.unwrap();
        assert!(path_inter.contains(skia_safe::Point::new(75.0, 50.0)));
        // Should NOT contain the non-overlapping parts
        assert!(!path_inter.contains(skia_safe::Point::new(25.0, 50.0)));
        assert!(!path_inter.contains(skia_safe::Point::new(125.0, 50.0)));
    }
}
