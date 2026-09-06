use skia_safe as skia;

use crate::core::{
    calculate_resize_scales, hit_transform_handle, Element, ElementId, ImageElement, Point,
    PointerButton, PointerEvent, Rect, TransformHandle, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, PartialEq)]
enum ImageToolState {
    Idle,
    Creating {
        start_pos: Point,
        current_pos: Point,
        is_square_locked: bool,
        is_center_anchored: bool,
    },
    MovingElement {
        start_world: Point,
        initial_rect: Rect,
        elem_id: ElementId,
        has_dragged: bool,
    },
    Resizing {
        handle: TransformHandle,
        initial_bounds: Rect,
        initial_elements: Vec<(ElementId, Element)>,
        origin: Point,
    },
    Rotating {
        center: Point,
        start_angle: f32,
        initial_elements: Vec<(ElementId, Element)>,
    },
}

pub struct ImageFeature {
    state: ImageToolState,
}

impl Default for ImageFeature {
    fn default() -> Self {
        Self {
            state: ImageToolState::Idle,
        }
    }
}

impl ImageFeature {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_rect(&self) -> Option<Rect> {
        if let ImageToolState::Creating {
            start_pos,
            current_pos,
            is_square_locked,
            is_center_anchored,
        } = self.state
        {
            let mut dx = current_pos.x - start_pos.x;
            let mut dy = current_pos.y - start_pos.y;

            if is_square_locked {
                let side = dx.abs().max(dy.abs()).round();
                dx = if dx < 0.0 { -side } else { side };
                dy = if dy < 0.0 { -side } else { side };
            }

            if is_center_anchored {
                let half_w = dx.abs().round();
                let half_h = dy.abs().round();
                Some(Rect::new(
                    (start_pos.x - half_w).round(),
                    (start_pos.y - half_h).round(),
                    (half_w * 2.0).max(1.0),
                    (half_h * 2.0).max(1.0),
                ))
            } else {
                let p2_adj = Point::new(start_pos.x + dx, start_pos.y + dy);
                Some(Rect::from_points(start_pos, p2_adj).round())
            }
        } else {
            None
        }
    }
}

impl FeaturePlugin for ImageFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:image");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        // 1. Check if clicking on active selection handles (Resize or Rotate)
        if let Some(bounds) = ctx.document.selection_bounds() {
            if let Some(handle) = hit_transform_handle(bounds, event.world_pos, ctx.viewport.zoom) {
                ctx.document.snapshot();
                let initial_elements: Vec<(ElementId, Element)> = ctx
                    .document
                    .selected_ids
                    .iter()
                    .filter_map(|&id| {
                        ctx.document
                            .elements
                            .iter()
                            .find(|e| e.id() == id)
                            .map(|el| (id, el.clone()))
                    })
                    .collect();

                if handle == TransformHandle::Rotate {
                    let center = bounds.center();
                    let start_angle =
                        (event.world_pos.y - center.y).atan2(event.world_pos.x - center.x);
                    self.state = ImageToolState::Rotating {
                        center,
                        start_angle,
                        initial_elements,
                    };
                    ctx.set_cursor("grabbing");
                } else {
                    let origin = if event.alt_pressed {
                        bounds.center()
                    } else {
                        handle.opposite_anchor(bounds)
                    };
                    self.state = ImageToolState::Resizing {
                        handle,
                        initial_bounds: bounds,
                        initial_elements,
                        origin,
                    };
                    ctx.set_cursor(handle.cursor_name());
                }
                ctx.request_redraw();
                return;
            }
        }

        // 2. Check if clicking on an existing element to select it
        if let Some(hit_id) = ctx.document.hit_test(event.world_pos) {
            ctx.document.select(hit_id, event.shift_pressed);
            let initial_rect = if let Some(Element::Image(img)) = ctx.document.find_element(hit_id) {
                img.rect.normalize()
            } else {
                ctx.document.find_element(hit_id).map(|e| e.bounds()).unwrap_or(Rect::ZERO)
            };
            self.state = ImageToolState::MovingElement {
                start_world: event.world_pos,
                initial_rect,
                elem_id: hit_id,
                has_dragged: false,
            };
            ctx.set_cursor("move");
            ctx.request_redraw();
            return;
        }

        // 3. Otherwise, click on empty canvas -> deselect and start creating new Image frame
        if !event.shift_pressed {
            ctx.document.deselect_all();
        }
        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude).round();
        self.state = ImageToolState::Creating {
            start_pos: snapped,
            current_pos: snapped,
            is_square_locked: event.shift_pressed,
            is_center_anchored: event.alt_pressed,
        };
        ctx.set_cursor("crosshair");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match &mut self.state {
            ImageToolState::MovingElement {
                start_world,
                initial_rect,
                elem_id,
                has_dragged,
            } => {
                let dx = (event.world_pos.x - start_world.x).round();
                let dy = (event.world_pos.y - start_world.y).round();
                if dx.abs() > 1.0 || dy.abs() > 1.0 {
                    *has_dragged = true;
                    if let Some(el) = ctx
                        .document
                        .elements
                        .iter_mut()
                        .find(|e| e.id() == *elem_id)
                    {
                        if let Element::Image(img) = el {
                            img.rect = Rect::new(
                                initial_rect.x + dx,
                                initial_rect.y + dy,
                                initial_rect.width,
                                initial_rect.height,
                            );
                        } else {
                            el.translate(dx, dy);
                        }
                    }
                    ctx.set_cursor("move");
                    ctx.request_redraw();
                }
            }
            ImageToolState::Creating {
                current_pos,
                is_square_locked,
                is_center_anchored,
                ..
            } => {
                let empty_exclude = std::collections::HashSet::new();
                let snapped = ctx.snap_point(event.world_pos, &empty_exclude).round();
                *current_pos = snapped;
                *is_square_locked = event.shift_pressed;
                *is_center_anchored = event.alt_pressed;
                ctx.request_redraw();
            }
            ImageToolState::Resizing {
                handle,
                initial_bounds,
                initial_elements,
                origin,
            } => {
                let selected_ids = ctx.document.selected_ids.clone();
                let snapped_pos = ctx.snap_point(event.world_pos, &selected_ids);
                let (sx, sy) = calculate_resize_scales(
                    *handle,
                    *origin,
                    *initial_bounds,
                    snapped_pos,
                    event.shift_pressed,
                );
                for (id, initial_el) in initial_elements.iter() {
                    if let Some(el) = ctx.document.elements.iter_mut().find(|e| e.id() == *id) {
                        let mut modified = initial_el.clone();
                        modified.scale(*origin, sx, sy);
                        *el = modified;
                    }
                }
                ctx.set_cursor(handle.cursor_name());
                ctx.request_redraw();
            }
            ImageToolState::Rotating {
                center,
                start_angle,
                initial_elements,
            } => {
                let cur_angle = (event.world_pos.y - center.y).atan2(event.world_pos.x - center.x);
                let mut delta_angle = cur_angle - *start_angle;
                if event.shift_pressed {
                    let step = std::f32::consts::PI / 12.0;
                    delta_angle = (delta_angle / step).round() * step;
                }
                for (id, initial_el) in initial_elements.iter() {
                    if let Some(el) = ctx.document.elements.iter_mut().find(|e| e.id() == *id) {
                        let mut modified = initial_el.clone();
                        modified.rotate(*center, delta_angle);
                        *el = modified;
                    }
                }
                ctx.set_cursor("grabbing");
                ctx.request_redraw();
            }
            ImageToolState::Idle => {
                if let Some(bounds) = ctx.document.selection_bounds() {
                    if let Some(handle) =
                        hit_transform_handle(bounds, event.world_pos, ctx.viewport.zoom)
                    {
                        ctx.set_cursor(handle.cursor_name());
                        return;
                    }
                }

                let is_hovering_image = ctx.document.elements.iter().any(|e| {
                    if let Element::Image(img) = e {
                        img.bounds().contains(event.world_pos)
                    } else {
                        false
                    }
                });

                if is_hovering_image {
                    ctx.set_cursor("pointer");
                } else {
                    ctx.set_cursor("tool:image");
                }
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }
        match self.state {
            ImageToolState::Creating { .. } => {
                if let Some(rect) = self.current_rect() {
                    if rect.width > 2.0 && rect.height > 2.0 {
                        ctx.document.snapshot();
                        let elem = ImageElement::new_placeholder(rect, Some("Image Frame".to_string()));
                        let new_id = elem.id;
                        ctx.document.add_element(Element::Image(elem));
                        ctx.document.select(new_id, false);
                    }
                }
            }
            ImageToolState::MovingElement { elem_id, has_dragged, .. } => {
                if !has_dragged && !event.shift_pressed {
                    ctx.document.select(elem_id, false);
                }
            }
            ImageToolState::Idle => {
                let hit = ctx.document.elements.iter().rev().find_map(|e| {
                    if let Element::Image(img) = e {
                        if img.bounds().contains(event.world_pos) {
                            return Some(img.id);
                        }
                    }
                    None
                });

                if let Some(id) = hit {
                    ctx.document.select(id, event.shift_pressed);
                } else if !event.shift_pressed {
                    ctx.document.selected_ids.clear();
                }
            }
            _ => {}
        }

        self.state = ImageToolState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("tool:image");
        ctx.request_redraw();
    }

    fn on_double_click(&mut self, ctx: &mut PluginContext, event: &PointerEvent) -> bool {
        if let Some(hit_id) = ctx.document.hit_test(event.world_pos) {
            if let Some(Element::Image(_)) = ctx.document.find_element(hit_id) {
                ctx.document.select(hit_id, false);
                return true;
            }
        }
        false
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.state = ImageToolState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        _ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        // Draw creation preview if actively dragging
        if let Some(rect) = self.current_rect() {
            let r = rect.normalize();
            let dst = skia::Rect::from_xywh(r.x, r.y, r.width, r.height);

            // Semi-transparent frame background
            let mut bg_paint = skia::Paint::default();
            bg_paint.set_color4f(skia::Color4f::new(0.20, 0.52, 0.89, 0.12), None);
            bg_paint.set_style(skia::PaintStyle::Fill);
            canvas.draw_rect(dst, &bg_paint);

            // Frame border
            let mut border_paint = skia::Paint::default();
            border_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.9), None);
            border_paint.set_style(skia::PaintStyle::Stroke);
            border_paint.set_stroke_width((1.5 / viewport.zoom).max(1.0));
            border_paint.set_anti_alias(true);
            canvas.draw_rect(dst, &border_paint);

            // Diagonal frame cross-lines
            let mut cross_paint = skia::Paint::default();
            cross_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.35), None);
            cross_paint.set_style(skia::PaintStyle::Stroke);
            cross_paint.set_stroke_width((1.0 / viewport.zoom).max(0.75));
            cross_paint.set_anti_alias(true);
            canvas.draw_line(skia::Point::new(r.x, r.y), skia::Point::new(r.x + r.width, r.y + r.height), &cross_paint);
            canvas.draw_line(skia::Point::new(r.x + r.width, r.y), skia::Point::new(r.x, r.y + r.height), &cross_paint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Color, Document, GridConfig, PathEditorConfig, RulerConfig, SnapConfig, TransformOptions, Viewport};

    #[test]
    fn test_image_feature_creation_and_placeholder() {
        let mut feat = ImageFeature::new();
        let mut doc = Document::new();
        let mut vp = Viewport::default();
        let grid_cfg = GridConfig::default();
        let snap_cfg = SnapConfig::default();
        let ruler_cfg = RulerConfig::default();
        let pe_cfg = PathEditorConfig::default();
        let trans_opt = TransformOptions::default();
        let mut guides = Vec::new();

        let mut ctx = PluginContext {
            document: &mut doc,
            viewport: &mut vp,
            grid_config: &grid_cfg,
            snap_config: &snap_cfg,
            ruler_config: &ruler_cfg,
            path_editor_config: &pe_cfg,
            transform_options: &trans_opt,
            active_snap_guides: &mut guides,
            active_fill_color: Color::BLACK,
            active_stroke_color: None,
            active_stroke_width: 1.0,
            widget_size: (800.0, 600.0),
            needs_redraw: false,
            cursor_name: None,
        };

        // 1. Activate
        feat.on_activate(&mut ctx);
        assert_eq!(ctx.cursor_name, Some("tool:image"));

        // 2. Drag to create image frame
        let p_down = PointerEvent {
            screen_pos: Point::new(100.0, 100.0),
            world_pos: Point::new(100.0, 100.0),
            button: Some(PointerButton::Primary),
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        };
        feat.on_pointer_down(&mut ctx, &p_down);

        let p_move = PointerEvent {
            screen_pos: Point::new(300.0, 250.0),
            world_pos: Point::new(300.0, 250.0),
            button: Some(PointerButton::Primary),
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        };
        feat.on_pointer_move(&mut ctx, &p_move);

        let p_up = PointerEvent {
            screen_pos: Point::new(300.0, 250.0),
            world_pos: Point::new(300.0, 250.0),
            button: Some(PointerButton::Primary),
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        };
        feat.on_pointer_up(&mut ctx, &p_up);

        assert_eq!(ctx.document.elements.len(), 1);
        if let Element::Image(img) = &ctx.document.elements[0] {
            assert_eq!(img.rect.width, 200.0);
            assert_eq!(img.rect.height, 150.0);
            assert!(img.image_data.is_empty());
        } else {
            panic!("Expected Image element");
        }
    }
}

