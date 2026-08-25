use skia_safe as skia;

use crate::core::{
    calculate_resize_scales, hit_transform_handle, Color, Element, ElementId, PathElement,
    PathNode, Point, PointerButton, PointerEvent, Rect, ShapeOrigin, TransformHandle, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpiralHandle {
    InnerRadius,
    Turns,
}

impl SpiralHandle {
    pub fn position(self, rect: Rect, turns: f32, inner_radius: f32) -> Point {
        let r = rect.normalize();
        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;
        let rx = r.width / 2.0;
        let ry = r.height / 2.0;

        match self {
            SpiralHandle::InnerRadius => {
                let cur_rx = rx * inner_radius;
                Point::new(cx + cur_rx, cy)
            }
            SpiralHandle::Turns => {
                let total_turns = turns.max(0.25);
                let total_angle = total_turns * std::f32::consts::TAU;
                Point::new(cx + rx * total_angle.cos(), cy + ry * total_angle.sin())
            }
        }
    }
}

pub fn hit_spiral_handle(
    rect: Rect,
    turns: f32,
    inner_radius: f32,
    p: Point,
    zoom: f32,
) -> Option<SpiralHandle> {
    let hit_r = (8.0 / zoom).max(6.0);
    let handles = [SpiralHandle::InnerRadius, SpiralHandle::Turns];
    for h in handles {
        let pos = h.position(rect, turns, inner_radius);
        if p.distance_to(pos) <= hit_r {
            return Some(h);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
enum SpiralToolState {
    Idle,
    Creating {
        start_pos: Point,
        current_pos: Point,
        is_square_locked: bool,
        is_center_anchored: bool,
    },
    DraggingHandle {
        handle: SpiralHandle,
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

pub struct SpiralFeature {
    state: SpiralToolState,
    pub turns: f32,
    pub divergence: f32,
    pub inner_radius: f32,
}

impl Default for SpiralFeature {
    fn default() -> Self {
        Self {
            state: SpiralToolState::Idle,
            turns: 3.0,
            divergence: 1.0,
            inner_radius: 0.0,
        }
    }
}

impl SpiralFeature {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_rect(&self) -> Option<Rect> {
        if let SpiralToolState::Creating {
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
                Some(Rect::new(start_pos.x - rx, start_pos.y - ry, rx * 2.0, ry * 2.0))
            } else if is_square_locked {
                let dx = current_pos.x - start_pos.x;
                let dy = current_pos.y - start_pos.y;
                let size = dx.abs().max(dy.abs());
                let sx = if dx >= 0.0 { size } else { -size };
                let sy = if dy >= 0.0 { size } else { -size };
                let p2_adj = Point::new(start_pos.x + sx, start_pos.y + sy);
                Some(Rect::from_points(start_pos, p2_adj))
            } else {
                let dx = current_pos.x - start_pos.x;
                let dy = current_pos.y - start_pos.y;
                let p2_adj = Point::new(start_pos.x + dx, start_pos.y + dy);
                Some(Rect::from_points(start_pos, p2_adj))
            }
        } else {
            None
        }
    }

    pub fn create_spiral_path(
        rect: Rect,
        stroke_color: Option<Color>,
        stroke_width: f32,
        turns: f32,
        divergence: f32,
        inner_radius: f32,
    ) -> PathElement {
        let r = rect.normalize();
        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;
        let rx = r.width / 2.0;
        let ry = r.height / 2.0;

        let total_turns = turns.max(0.25);
        let total_angle = total_turns * std::f32::consts::TAU;
        let div = divergence.max(0.05);
        let r_inner = inner_radius.clamp(0.0, 0.999);

        // 8 cubic Bezier segments per complete turn (every 45 degrees) for silky smooth curves
        let num_steps = ((total_turns * 8.0).ceil() as usize).max(8);
        let dt = 1.0 / (num_steps as f32);

        // Evaluate spiral position and derivative at parametric parameter t in [0.0, 1.0]
        let eval_spiral = |t: f32| -> (Point, Point) {
            let angle = t * total_angle;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let r_factor = r_inner + (1.0 - r_inner) * t.powf(div);
            let px = cx + rx * r_factor * cos_a;
            let py = cy + ry * r_factor * sin_a;

            let dr_dt = if t <= 0.0 {
                if (div - 1.0).abs() < 1e-4 {
                    1.0 - r_inner
                } else if div > 1.0 {
                    0.0
                } else {
                    ((1.0 - r_inner) * div * (0.0001f32).powf(div - 1.0)).min(50.0)
                }
            } else {
                ((1.0 - r_inner) * div * t.powf(div - 1.0)).min(100.0)
            };

            // Tangent: dP/dt = (rx * (r' * cos(a) - r * theta_max * sin(a)), ry * (r' * sin(a) + r * theta_max * cos(a)))
            let vx = rx * (dr_dt * cos_a - r_factor * total_angle * sin_a);
            let vy = ry * (dr_dt * sin_a + r_factor * total_angle * cos_a);

            (Point::new(px, py), Point::new(vx, vy))
        };

        let mut samples = Vec::with_capacity(num_steps + 1);
        for i in 0..=num_steps {
            let t = i as f32 * dt;
            samples.push(eval_spiral(t));
        }

        let mut nodes = Vec::with_capacity(num_steps + 1);
        for i in 0..=num_steps {
            let (p, v) = samples[i];
            let handle_in = if i > 0 {
                Some(Point::new(p.x - (dt / 3.0) * v.x, p.y - (dt / 3.0) * v.y))
            } else {
                None
            };
            let handle_out = if i < num_steps {
                Some(Point::new(p.x + (dt / 3.0) * v.x, p.y + (dt / 3.0) * v.y))
            } else {
                None
            };

            nodes.push(PathNode::with_handles(p, handle_in, handle_out));
        }

        let mut elem = PathElement::new(nodes, false, None, stroke_color, stroke_width);
        elem.shape_origin = Some(ShapeOrigin::Spiral {
            turns,
            divergence,
            inner_radius,
        });
        elem.shape_rect = Some(r);
        elem.stroke_width = stroke_width;
        elem
    }
}

impl FeaturePlugin for SpiralFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:spiral");
    }

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
                    self.state = SpiralToolState::Rotating {
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
                    self.state = SpiralToolState::Resizing {
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

        // 1. Check if clicking on an on-canvas handle of the selected spiral
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Path(p)) = ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                if let Some(ShapeOrigin::Spiral {
                    turns,
                    inner_radius,
                    ..
                }) = p.shape_origin
                {
                    let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());
                    if let Some(h) = hit_spiral_handle(
                        base_rect,
                        turns,
                        inner_radius,
                        event.world_pos,
                        ctx.viewport.zoom,
                    ) {
                        self.state = SpiralToolState::DraggingHandle {
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

        // 2. Check if clicking on ANY existing element to select it
        if let Some(hit_id) = ctx.document.hit_test(event.world_pos) {
            ctx.document.select(hit_id, event.shift_pressed);
            if let Some(Element::Path(p)) = ctx.document.find_element(hit_id) {
                if let Some(ShapeOrigin::Spiral {
                    turns,
                    divergence,
                    inner_radius,
                }) = p.shape_origin
                {
                    self.turns = turns;
                    self.divergence = divergence;
                    self.inner_radius = inner_radius;
                }
            }
            self.state = SpiralToolState::MovingElement {
                start_world: event.world_pos,
                elem_id: hit_id,
                has_dragged: false,
            };
            ctx.set_cursor("move");
            ctx.request_redraw();
            return;
        }

        // 3. Start creating new spiral
        if !event.shift_pressed {
            ctx.document.deselect_all();
        }

        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude).round();
        self.state = SpiralToolState::Creating {
            start_pos: snapped,
            current_pos: snapped,
            is_square_locked: event.shift_pressed,
            is_center_anchored: event.alt_pressed,
        };
        ctx.set_cursor("tool:spiral");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match &mut self.state {
            SpiralToolState::Creating {
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
                ctx.set_cursor("tool:spiral");
                ctx.request_redraw();
            }
            SpiralToolState::DraggingHandle {
                handle,
                elem_id,
                rect,
            } => {
                let r = rect.normalize();
                let cx = r.x + r.width / 2.0;
                let cy = r.y + r.height / 2.0;
                let rx = r.width / 2.0;

                match handle {
                    SpiralHandle::InnerRadius => {
                        let offset = (event.world_pos.x - cx) / rx.max(1.0);
                        self.inner_radius = offset.clamp(0.0, 0.95);
                        self.inner_radius = (self.inner_radius * 100.0).round() / 100.0;
                    }
                    SpiralHandle::Turns => {
                        let rad = (event.world_pos.y - cy).atan2(event.world_pos.x - cx);
                        let norm_rad = if rad < 0.0 {
                            rad + std::f32::consts::TAU
                        } else {
                            rad
                        };
                        let cur_base_turns = self.turns.floor();
                        let new_turns =
                            (cur_base_turns + norm_rad / std::f32::consts::TAU).clamp(0.5, 20.0);
                        self.turns = (new_turns * 10.0).round() / 10.0;
                    }
                }

                let target_id = *elem_id;
                let base_rect = *rect;
                if let Some(Element::Path(p)) = ctx
                    .document
                    .elements
                    .iter_mut()
                    .find(|e| e.id() == target_id)
                {
                    let stroke = p.stroke_color;
                    let sw = p.stroke_width;
                    let mut new_elem = Self::create_spiral_path(
                        base_rect,
                        stroke,
                        sw,
                        self.turns,
                        self.divergence,
                        self.inner_radius,
                    );
                    new_elem.id = target_id;
                    new_elem.shape_rect = Some(base_rect);
                    *p = new_elem;
                }

                ctx.set_cursor("grabbing");
                ctx.request_redraw();
            }
            SpiralToolState::MovingElement {
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
            SpiralToolState::Resizing {
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
            SpiralToolState::Rotating {
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
            SpiralToolState::Idle => {
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
                        if let Some(ShapeOrigin::Spiral {
                            turns,
                            inner_radius,
                            ..
                        }) = p.shape_origin
                        {
                            let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());
                            if hit_spiral_handle(
                                base_rect,
                                turns,
                                inner_radius,
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
                            matches!(p.shape_origin, Some(ShapeOrigin::Spiral { .. }))
                                && p.bounds().contains(event.world_pos)
                        } else {
                            false
                        }
                    });
                    if is_hovering {
                        ctx.set_cursor("pointer");
                    } else {
                        ctx.set_cursor("tool:spiral");
                    }
                }
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        match self.state {
            SpiralToolState::Creating { .. } => {
                if let Some(rect) = self.current_rect() {
                    if rect.width > 2.0 && rect.height > 2.0 {
                        ctx.document.snapshot();
                        let stroke_col = ctx.active_stroke_color.or(Some(ctx.active_fill_color));
                        let elem = Self::create_spiral_path(
                            rect,
                            stroke_col,
                            ctx.active_stroke_width,
                            self.turns,
                            self.divergence,
                            self.inner_radius,
                        );
                        let new_id = elem.id;
                        ctx.document.add_element(Element::Path(elem));
                        ctx.document.select(new_id, false);
                    }
                }
            }
            SpiralToolState::DraggingHandle { .. } => {
                ctx.document.snapshot();
            }
            SpiralToolState::MovingElement {
                has_dragged: true, ..
            } => {
                ctx.document.snapshot();
            }
            _ => {}
        }

        self.state = SpiralToolState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("tool:spiral");
        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.state = SpiralToolState::Idle;
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
            let stroke_col = ctx.active_stroke_color.unwrap_or(ctx.active_fill_color);
            let elem = Self::create_spiral_path(
                rect,
                Some(stroke_col),
                ctx.active_stroke_width,
                self.turns,
                self.divergence,
                self.inner_radius,
            );
            let path = elem.to_skia_path();

            let mut stroke_paint = skia::Paint::default();
            stroke_paint.set_color4f(stroke_col.to_skia(), None);
            stroke_paint.set_style(skia::PaintStyle::Stroke);
            stroke_paint.set_stroke_width((ctx.active_stroke_width / viewport.zoom).max(1.0));
            stroke_paint.set_anti_alias(true);
            canvas.draw_path(&path, &stroke_paint);
        }

        // 2. Draw selection bounding box and handles if a Spiral is selected
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Path(p)) = ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                if let Some(ShapeOrigin::Spiral {
                    turns,
                    inner_radius,
                    ..
                }) = p.shape_origin
                {
                    let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());

                    // On-canvas handles
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

                    let handles = [SpiralHandle::InnerRadius, SpiralHandle::Turns];
                    for h in handles {
                        let pos = h.position(base_rect, turns, inner_radius);
                        canvas.draw_circle(pos.to_skia(), handle_radius, &fill_paint);
                        canvas.draw_circle(pos.to_skia(), handle_radius, &handle_stroke);
                    }
                }
            }
        }
    }

    fn get_shape_params(&self) -> Option<ShapeOrigin> {
        Some(ShapeOrigin::Spiral {
            turns: self.turns,
            divergence: self.divergence,
            inner_radius: self.inner_radius,
        })
    }

    fn set_shape_params(&mut self, origin: &ShapeOrigin) {
        if let ShapeOrigin::Spiral {
            turns,
            divergence,
            inner_radius,
        } = origin
        {
            self.turns = *turns;
            self.divergence = *divergence;
            self.inner_radius = *inner_radius;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spiral_path_nodes() {
        let elem = SpiralFeature::create_spiral_path(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            None,
            1.0,
            3.0,
            1.0,
            0.0,
        );
        assert!(!elem.nodes.is_empty());
        assert!(!elem.is_closed);
        // Ensure nodes have smooth handles
        assert!(elem.nodes[0].handle_out.is_some());
        assert!(elem.nodes[1].handle_in.is_some());
        assert!(elem.nodes[1].handle_out.is_some());
        let last = elem.nodes.len() - 1;
        assert!(elem.nodes[last].handle_in.is_some());
        assert!(elem.nodes[last].handle_out.is_none());
    }
}
