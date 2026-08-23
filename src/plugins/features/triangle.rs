use skia_safe as skia;

use super::star::StarFeature;
use crate::core::{
    calculate_resize_scales, hit_transform_handle, Color, Element, ElementId, PathElement,
    PathNode, Point, PointerButton, PointerEvent, Rect, ShapeOrigin, TransformHandle, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriangleHandle {
    CornerRadius,
}

impl TriangleHandle {
    pub fn position(self, rect: Rect, corner_radius: f32) -> Point {
        let r = rect.normalize();
        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;
        let ry = r.height / 2.0;

        let offset = corner_radius.min(ry * 0.45);
        Point::new(cx, (cy - ry) + offset).round()
    }
}

pub fn hit_triangle_handle(
    rect: Rect,
    corner_radius: f32,
    p: Point,
    zoom: f32,
) -> Option<TriangleHandle> {
    let hit_r = (8.0 / zoom).max(6.0);
    let pos = TriangleHandle::CornerRadius.position(rect, corner_radius);
    if p.distance_to(pos) <= hit_r {
        Some(TriangleHandle::CornerRadius)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TriangleToolState {
    Idle,
    Creating {
        start_pos: Point,
        current_pos: Point,
        is_square_locked: bool,
        is_center_anchored: bool,
    },
    DraggingHandle {
        handle: TriangleHandle,
        elem_id: ElementId,
        rect: Rect,
    },
    MovingElement {
        start_world: Point,
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

pub struct TriangleFeature {
    state: TriangleToolState,
    pub sides: u32,
    pub corner_radius: f32,
}

impl Default for TriangleFeature {
    fn default() -> Self {
        Self {
            state: TriangleToolState::Idle,
            sides: 3,
            corner_radius: 0.0,
        }
    }
}

impl TriangleFeature {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_rect(&self) -> Option<Rect> {
        if let TriangleToolState::Creating {
            start_pos,
            current_pos,
            is_square_locked,
            is_center_anchored,
        } = self.state
        {
            if is_center_anchored {
                let dx = (current_pos.x - start_pos.x).abs();
                let dy = (current_pos.y - start_pos.y).abs();
                let (rx, ry) = if is_square_locked {
                    let r = dx.max(dy);
                    (r, r)
                } else {
                    (dx, dy)
                };
                Some(Rect::new(start_pos.x - rx, start_pos.y - ry, rx * 2.0, ry * 2.0).round())
            } else if is_square_locked {
                let dx = current_pos.x - start_pos.x;
                let dy = current_pos.y - start_pos.y;
                let size = dx.abs().max(dy.abs());
                let sx = if dx >= 0.0 { size } else { -size };
                let sy = if dy >= 0.0 { size } else { -size };
                let p2_adj = Point::new(start_pos.x + sx, start_pos.y + sy);
                Some(Rect::from_points(start_pos, p2_adj).round())
            } else {
                let dx = current_pos.x - start_pos.x;
                let dy = current_pos.y - start_pos.y;
                let p2_adj = Point::new(start_pos.x + dx, start_pos.y + dy);
                Some(Rect::from_points(start_pos, p2_adj).round())
            }
        } else {
            None
        }
    }

    pub fn create_triangle_path(
        rect: Rect,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
        stroke_width: f32,
        sides: u32,
        corner_radius: f32,
    ) -> PathElement {
        let r = rect.normalize();
        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;
        let rx = r.width / 2.0;
        let ry = r.height / 2.0;

        let n = sides.max(3);
        let mut raw_points = Vec::with_capacity(n as usize);

        for i in 0..n {
            let angle =
                -std::f32::consts::FRAC_PI_2 + (i as f32) * (std::f32::consts::TAU / n as f32);
            let px = cx + rx * angle.cos();
            let py = cy + ry * angle.sin();
            raw_points.push(Point::new(px.round(), py.round()));
        }

        let nodes = if corner_radius > 0.001 {
            StarFeature::apply_corner_rounding(&raw_points, corner_radius, true)
        } else {
            raw_points.iter().map(|p| PathNode::new(*p)).collect()
        };

        let mut elem = PathElement::new(nodes, true, fill_color, stroke_color, stroke_width);
        elem.shape_origin = Some(ShapeOrigin::Triangle {
            corner_radius,
            sides: n,
        });
        elem.shape_rect = Some(r);
        elem.stroke_width = stroke_width;
        elem
    }
}

impl FeaturePlugin for TriangleFeature {
    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        // 0. Check if clicking on active selection handles (Resize or Rotate)
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
                    self.state = TriangleToolState::Rotating {
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
                    self.state = TriangleToolState::Resizing {
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

        // 1. Check if clicking on the corner radius handle of the selected polygon
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Path(p)) = ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                if let Some(ShapeOrigin::Triangle { corner_radius, .. }) = p.shape_origin {
                    let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());
                    if let Some(h) = hit_triangle_handle(
                        base_rect,
                        corner_radius,
                        event.world_pos,
                        ctx.viewport.zoom,
                    ) {
                        self.state = TriangleToolState::DraggingHandle {
                            handle: h,
                            elem_id: sel_id,
                            rect: base_rect,
                        };
                        ctx.set_cursor("grab");
                        ctx.request_redraw();
                        return;
                    }
                }
            }
        }

        // 2. Check if clicking on an existing triangle to select it
        let hit_tri = ctx.document.elements.iter().rev().find_map(|e| {
            if let Element::Path(p) = e {
                if let Some(ShapeOrigin::Triangle {
                    sides,
                    corner_radius,
                }) = p.shape_origin
                {
                    if p.bounds().contains(event.world_pos) {
                        return Some((e.id(), sides, corner_radius));
                    }
                }
            }
            None
        });

        if let Some((id, s, r_rad)) = hit_tri {
            ctx.document.select(id, false);
            self.sides = s;
            self.corner_radius = r_rad;
            self.state = TriangleToolState::MovingElement {
                start_world: event.world_pos,
                elem_id: id,
                has_dragged: false,
            };
            ctx.set_cursor("pointer");
            ctx.request_redraw();
            return;
        }

        // 3. Start creating new triangle/polygon
        if !event.shift_pressed {
            ctx.document.deselect_all();
        }

        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude).round();
        self.state = TriangleToolState::Creating {
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
            TriangleToolState::Creating {
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
                ctx.set_cursor("crosshair");
                ctx.request_redraw();
            }
            TriangleToolState::DraggingHandle { elem_id, rect, .. } => {
                let r = rect.normalize();
                let cy = r.y + r.height / 2.0;
                let ry = r.height / 2.0;

                let top_y = cy - ry;
                let offset = (event.world_pos.y - top_y).clamp(0.0, ry * 0.45);
                self.corner_radius = offset.round();

                let target_id = *elem_id;
                let base_rect = *rect;
                if let Some(Element::Path(p)) = ctx
                    .document
                    .elements
                    .iter_mut()
                    .find(|e| e.id() == target_id)
                {
                    let fill = p.fill_color;
                    let stroke = p.stroke_color;
                    let sw = p.stroke_width;
                    let mut new_elem = Self::create_triangle_path(
                        base_rect,
                        fill,
                        stroke,
                        sw,
                        self.sides,
                        self.corner_radius,
                    );
                    new_elem.id = target_id;
                    new_elem.shape_rect = Some(base_rect);
                    *p = new_elem;
                }

                ctx.set_cursor("grabbing");
                ctx.request_redraw();
            }
            TriangleToolState::MovingElement {
                start_world,
                elem_id,
                has_dragged,
            } => {
                let dx = event.world_pos.x - start_world.x;
                let dy = event.world_pos.y - start_world.y;
                if dx.abs() > 1.0 || dy.abs() > 1.0 {
                    *has_dragged = true;
                    let target_id = *elem_id;
                    if let Some(e) = ctx
                        .document
                        .elements
                        .iter_mut()
                        .find(|e| e.id() == target_id)
                    {
                        e.translate(dx, dy);
                    }
                    *start_world = event.world_pos;
                    ctx.set_cursor("move");
                    ctx.request_redraw();
                }
            }
            TriangleToolState::Resizing {
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
            TriangleToolState::Rotating {
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
            TriangleToolState::Idle => {
                if let Some(bounds) = ctx.document.selection_bounds() {
                    if let Some(handle) = hit_transform_handle(bounds, event.world_pos, ctx.viewport.zoom) {
                        ctx.set_cursor(handle.cursor_name());
                        return;
                    }
                }

                let mut hover_handle = false;
                if ctx.document.selected_ids.len() == 1 {
                    let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
                    if let Some(Element::Path(p)) =
                        ctx.document.elements.iter().find(|e| e.id() == sel_id)
                    {
                        if let Some(ShapeOrigin::Triangle { corner_radius, .. }) = p.shape_origin {
                            let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());
                            if hit_triangle_handle(
                                base_rect,
                                corner_radius,
                                event.world_pos,
                                ctx.viewport.zoom,
                            )
                            .is_some()
                            {
                                hover_handle = true;
                            }
                        }
                    }
                }

                if hover_handle {
                    ctx.set_cursor("grab");
                } else {
                    let is_hovering = ctx.document.elements.iter().any(|e| {
                        if let Element::Path(p) = e {
                            matches!(p.shape_origin, Some(ShapeOrigin::Triangle { .. }))
                                && p.bounds().contains(event.world_pos)
                        } else {
                            false
                        }
                    });
                    if is_hovering {
                        ctx.set_cursor("pointer");
                    } else {
                        ctx.set_cursor("crosshair");
                    }
                }
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        match self.state {
            TriangleToolState::Creating { .. } => {
                if let Some(rect) = self.current_rect() {
                    if rect.width > 2.0 && rect.height > 2.0 {
                        let elem = Self::create_triangle_path(
                            rect,
                            Some(ctx.active_fill_color),
                            ctx.active_stroke_color,
                            ctx.active_stroke_width,
                            self.sides,
                            self.corner_radius,
                        );
                        let new_id = elem.id;
                        ctx.document.add_element(Element::Path(elem));
                        ctx.document.select(new_id, false);
                    }
                }
            }
            _ => {}
        }

        self.state = TriangleToolState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("crosshair");
        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.state = TriangleToolState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        // 1. Draw creation preview
        if let Some(rect) = self.current_rect() {
            let elem = Self::create_triangle_path(
                rect,
                Some(ctx.active_fill_color),
                ctx.active_stroke_color,
                ctx.active_stroke_width,
                self.sides,
                self.corner_radius,
            );
            let path = elem.to_skia_path();

            let mut fill_paint = skia::Paint::default();
            fill_paint.set_color4f(ctx.active_fill_color.to_skia(), None);
            fill_paint.set_style(skia::PaintStyle::Fill);
            fill_paint.set_anti_alias(true);
            canvas.draw_path(&path, &fill_paint);

            if let Some(stroke_color) = ctx.active_stroke_color {
                let mut stroke_paint = skia::Paint::default();
                stroke_paint.set_color4f(stroke_color.to_skia(), None);
                stroke_paint.set_style(skia::PaintStyle::Stroke);
                stroke_paint.set_stroke_width((ctx.active_stroke_width / viewport.zoom).max(1.0));
                stroke_paint.set_anti_alias(true);
                canvas.draw_path(&path, &stroke_paint);
            }
        }

        // 2. Draw selection bounding box and corner radius handle if a Triangle is selected
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Path(p)) = ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                if let Some(ShapeOrigin::Triangle { corner_radius, .. }) = p.shape_origin {
                    let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());

                    // Corner radius handle
                    let handle_radius = (4.5 / viewport.zoom).clamp(3.5, 6.5);
                    let stroke_w = (1.5 / viewport.zoom).max(1.0);

                    let mut fill_paint = skia::Paint::default();
                    fill_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                    fill_paint.set_style(skia::PaintStyle::Fill);
                    fill_paint.set_anti_alias(true);

                    let mut handle_stroke = skia::Paint::default();
                    handle_stroke.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 1.0), None);
                    handle_stroke.set_style(skia::PaintStyle::Stroke);
                    handle_stroke.set_stroke_width(stroke_w);
                    handle_stroke.set_anti_alias(true);

                    let pos = TriangleHandle::CornerRadius.position(base_rect, corner_radius);
                    canvas.draw_circle(pos.to_skia(), handle_radius, &fill_paint);
                    canvas.draw_circle(pos.to_skia(), handle_radius, &handle_stroke);
                }
            }
        }
    }

    fn get_shape_params(&self) -> Option<ShapeOrigin> {
        Some(ShapeOrigin::Triangle {
            corner_radius: self.corner_radius,
            sides: self.sides,
        })
    }

    fn set_shape_params(&mut self, origin: &ShapeOrigin) {
        if let ShapeOrigin::Triangle {
            corner_radius,
            sides,
        } = origin
        {
            self.corner_radius = *corner_radius;
            self.sides = *sides;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_path_vertices() {
        let elem = TriangleFeature::create_triangle_path(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            None,
            None,
            1.0,
            3,
            0.0,
        );
        assert_eq!(elem.nodes.len(), 3);
        assert!(elem.is_closed);
    }
}
