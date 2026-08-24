use skia_safe as skia;

use crate::core::{
    calculate_resize_scales, hit_transform_handle, ArcMode, Color, Element, ElementId, PathElement,
    PathNode, Point, PointerButton, PointerEvent, Rect, ShapeOrigin, TransformHandle, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArcHandle {
    StartAngle,
    EndAngle,
}

impl ArcHandle {
    pub fn position(self, rect: Rect, start_angle: f32, end_angle: f32) -> Point {
        let r = rect.normalize();
        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;
        let rx = r.width / 2.0;
        let ry = r.height / 2.0;

        let angle_deg = match self {
            ArcHandle::StartAngle => start_angle,
            ArcHandle::EndAngle => end_angle,
        };
        let rad = angle_deg.to_radians();
        Point::new(cx + rx * rad.cos(), cy + ry * rad.sin()).round()
    }
}

pub fn hit_arc_handle(
    rect: Rect,
    start_angle: f32,
    end_angle: f32,
    p: Point,
    zoom: f32,
) -> Option<ArcHandle> {
    let hit_r = (8.0 / zoom).max(6.0);
    let handles = [ArcHandle::StartAngle, ArcHandle::EndAngle];
    for h in handles {
        let pos = h.position(rect, start_angle, end_angle);
        if p.distance_to(pos) <= hit_r {
            return Some(h);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
enum CircleToolState {
    Idle,
    Creating {
        start_pos: Point,
        current_pos: Point,
        is_circle_locked: bool,
        is_center_anchored: bool,
    },
    DraggingHandle {
        handle: ArcHandle,
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

pub struct CircleFeature {
    state: CircleToolState,
    pub arc_mode: ArcMode,
    pub start_angle: f32,
    pub end_angle: f32,
}

impl Default for CircleFeature {
    fn default() -> Self {
        Self {
            state: CircleToolState::Idle,
            arc_mode: ArcMode::Full,
            start_angle: 0.0,
            end_angle: 360.0,
        }
    }
}

impl CircleFeature {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_rect(&self) -> Option<Rect> {
        if let CircleToolState::Creating {
            start_pos,
            current_pos,
            is_circle_locked,
            is_center_anchored,
        } = self.state
        {
            if is_center_anchored {
                let dx = (current_pos.x - start_pos.x).abs();
                let dy = (current_pos.y - start_pos.y).abs();
                let (rx, ry) = if is_circle_locked {
                    let r = dx.max(dy);
                    (r, r)
                } else {
                    (dx, dy)
                };
                Some(Rect::new(start_pos.x - rx, start_pos.y - ry, rx * 2.0, ry * 2.0).round())
            } else if is_circle_locked {
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

    pub fn create_ellipse_path(
        rect: Rect,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
        stroke_width: f32,
        arc_mode: ArcMode,
        start_angle_deg: f32,
        end_angle_deg: f32,
    ) -> PathElement {
        let r = rect.normalize();
        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;
        let rx = r.width / 2.0;
        let ry = r.height / 2.0;

        let is_full =
            arc_mode == ArcMode::Full || ((end_angle_deg - start_angle_deg).abs() >= 359.99);

        if is_full {
            // Full ellipse using 4-point Bézier approximation
            let k = 0.55228475_f32;
            let nodes = vec![
                PathNode::with_handles(
                    Point::new(cx, cy - ry),
                    Some(Point::new(cx - rx * k, cy - ry)),
                    Some(Point::new(cx + rx * k, cy - ry)),
                ),
                PathNode::with_handles(
                    Point::new(cx + rx, cy),
                    Some(Point::new(cx + rx, cy - ry * k)),
                    Some(Point::new(cx + rx, cy + ry * k)),
                ),
                PathNode::with_handles(
                    Point::new(cx, cy + ry),
                    Some(Point::new(cx + rx * k, cy + ry)),
                    Some(Point::new(cx - rx * k, cy + ry)),
                ),
                PathNode::with_handles(
                    Point::new(cx - rx, cy),
                    Some(Point::new(cx - rx, cy + ry * k)),
                    Some(Point::new(cx - rx, cy - ry * k)),
                ),
            ];

            let mut elem = PathElement::new(nodes, true, fill_color, stroke_color, stroke_width);
            elem.shape_origin = Some(ShapeOrigin::Circle {
                arc_mode,
                start_angle: start_angle_deg,
                end_angle: end_angle_deg,
            });
            elem.shape_rect = Some(r);
            elem.stroke_width = stroke_width;
            elem
        } else {
            // Partial arc
            let start_rad = start_angle_deg.to_radians();
            let end_rad = end_angle_deg.to_radians();

            // Generate arc points using Bézier segments per 90° chunk
            let sweep = end_rad - start_rad;
            let num_segments = ((sweep.abs() / std::f32::consts::FRAC_PI_2).ceil() as usize).max(1);
            let seg_angle = sweep / num_segments as f32;
            let k = 4.0 / 3.0 * (seg_angle / 4.0).tan();

            let mut nodes = Vec::new();

            for s in 0..num_segments {
                let a1 = start_rad + s as f32 * seg_angle;
                let a2 = a1 + seg_angle;

                let p1 = Point::new(cx + rx * a1.cos(), cy + ry * a1.sin());
                let p2 = Point::new(cx + rx * a2.cos(), cy + ry * a2.sin());

                let h1 = Point::new(p1.x - rx * k * a1.sin(), p1.y + ry * k * a1.cos());
                let h2 = Point::new(p2.x + rx * k * a2.sin(), p2.y - ry * k * a2.cos());

                if s == 0 {
                    nodes.push(PathNode::with_handles(p1, None, Some(h1)));
                } else {
                    let last = nodes.last_mut().unwrap();
                    last.handle_out = Some(h1);
                }
                nodes.push(PathNode::with_handles(p2, Some(h2), None));
            }

            let is_closed = match arc_mode {
                ArcMode::Full | ArcMode::Chord => true,
                ArcMode::Segment => {
                    // Add center point and close
                    nodes.push(PathNode::new(Point::new(cx, cy)));
                    true
                }
                ArcMode::Arc => false,
            };

            let mut elem =
                PathElement::new(nodes, is_closed, fill_color, stroke_color, stroke_width);
            elem.shape_origin = Some(ShapeOrigin::Circle {
                arc_mode,
                start_angle: start_angle_deg,
                end_angle: end_angle_deg,
            });
            elem.shape_rect = Some(r);
            elem.stroke_width = stroke_width;
            elem
        }
    }
}

impl FeaturePlugin for CircleFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:circle");
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
                    self.state = CircleToolState::Rotating {
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
                    self.state = CircleToolState::Resizing {
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

        // 1. Check if clicking on an angle handle of the currently selected circle
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Path(p)) = ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                if let Some(ShapeOrigin::Circle {
                    start_angle,
                    end_angle,
                    ..
                }) = p.shape_origin
                {
                    let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());
                    if let Some(h) = hit_arc_handle(
                        base_rect,
                        start_angle,
                        end_angle,
                        event.world_pos,
                        ctx.viewport.zoom,
                    ) {
                        self.state = CircleToolState::DraggingHandle {
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

        // 2. Check if clicking on an existing circle to select it
        let hit_circle = ctx.document.elements.iter().rev().find_map(|e| {
            if let Element::Path(p) = e {
                if let Some(ShapeOrigin::Circle {
                    arc_mode,
                    start_angle,
                    end_angle,
                }) = p.shape_origin
                {
                    if p.bounds().contains(event.world_pos) {
                        return Some((e.id(), arc_mode, start_angle, end_angle));
                    }
                }
            }
            None
        });

        if let Some((id, mode, s_ang, e_ang)) = hit_circle {
            ctx.document.select(id, false);
            self.arc_mode = mode;
            self.start_angle = s_ang;
            self.end_angle = e_ang;
            self.state = CircleToolState::MovingElement {
                start_world: event.world_pos,
                elem_id: id,
                has_dragged: false,
            };
            ctx.set_cursor("pointer");
            ctx.request_redraw();
            return;
        }

        // 3. Start creating new circle
        if !event.shift_pressed {
            ctx.document.deselect_all();
        }

        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude).round();
        self.state = CircleToolState::Creating {
            start_pos: snapped,
            current_pos: snapped,
            is_circle_locked: event.shift_pressed,
            is_center_anchored: event.alt_pressed,
        };
        ctx.set_cursor("tool:circle");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match &mut self.state {
            CircleToolState::Creating {
                current_pos,
                is_circle_locked,
                is_center_anchored,
                ..
            } => {
                let empty_exclude = std::collections::HashSet::new();
                let snapped = ctx.snap_point(event.world_pos, &empty_exclude).round();
                *current_pos = snapped;
                *is_circle_locked = event.shift_pressed;
                *is_center_anchored = event.alt_pressed;
                ctx.set_cursor("tool:circle");
                ctx.request_redraw();
            }
            CircleToolState::DraggingHandle {
                handle,
                elem_id,
                rect,
            } => {
                let r = rect.normalize();
                let cx = r.x + r.width / 2.0;
                let cy = r.y + r.height / 2.0;

                let deg = (event.world_pos.y - cy)
                    .atan2(event.world_pos.x - cx)
                    .to_degrees();
                let mut norm_deg = if deg < 0.0 { deg + 360.0 } else { deg };
                if event.shift_pressed {
                    norm_deg = (norm_deg / 15.0).round() * 15.0;
                }
                norm_deg = norm_deg.clamp(0.0, 360.0);

                match handle {
                    ArcHandle::StartAngle => self.start_angle = norm_deg.round(),
                    ArcHandle::EndAngle => self.end_angle = norm_deg.round(),
                }

                if self.arc_mode == ArcMode::Full {
                    self.arc_mode = ArcMode::Segment;
                }

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
                    let mut new_elem = Self::create_ellipse_path(
                        base_rect,
                        fill,
                        stroke,
                        sw,
                        self.arc_mode,
                        self.start_angle,
                        self.end_angle,
                    );
                    new_elem.id = target_id;
                    new_elem.shape_rect = Some(base_rect);
                    *p = new_elem;
                }

                ctx.set_cursor("grabbing");
                ctx.request_redraw();
            }
            CircleToolState::MovingElement {
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
            CircleToolState::Resizing {
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
            CircleToolState::Rotating {
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
            CircleToolState::Idle => {
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
                        if let Some(ShapeOrigin::Circle {
                            start_angle,
                            end_angle,
                            ..
                        }) = p.shape_origin
                        {
                            let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());
                            if hit_arc_handle(
                                base_rect,
                                start_angle,
                                end_angle,
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
                            matches!(p.shape_origin, Some(ShapeOrigin::Circle { .. }))
                                && p.bounds().contains(event.world_pos)
                        } else {
                            false
                        }
                    });
                    if is_hovering {
                        ctx.set_cursor("pointer");
                    } else {
                        ctx.set_cursor("tool:circle");
                    }
                }
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        match self.state {
            CircleToolState::Creating { .. } => {
                if let Some(rect) = self.current_rect() {
                    if rect.width > 2.0 && rect.height > 2.0 {
                        let elem = Self::create_ellipse_path(
                            rect,
                            Some(ctx.active_fill_color),
                            ctx.active_stroke_color,
                            ctx.active_stroke_width,
                            self.arc_mode,
                            self.start_angle,
                            self.end_angle,
                        );
                        let new_id = elem.id;
                        ctx.document.add_element(Element::Path(elem));
                        ctx.document.select(new_id, false);
                    }
                }
            }
            CircleToolState::MovingElement { .. } => {
                self.state = CircleToolState::Idle;
                ctx.clear_snap_guides();
                ctx.set_cursor("default");
                ctx.request_redraw();
            }
            CircleToolState::Resizing { .. } | CircleToolState::Rotating { .. } => {
                self.state = CircleToolState::Idle;
                ctx.clear_snap_guides();
                ctx.set_cursor("default");
                ctx.request_redraw();
            }
            _ => {}
        }

        self.state = CircleToolState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("tool:circle");
        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.state = CircleToolState::Idle;
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
            let elem = Self::create_ellipse_path(
                rect,
                Some(ctx.active_fill_color),
                ctx.active_stroke_color,
                ctx.active_stroke_width,
                self.arc_mode,
                self.start_angle,
                self.end_angle,
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

        // 2. Draw selection bounding box and angle handles if a Circle is selected
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Path(p)) = ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                if let Some(ShapeOrigin::Circle {
                    start_angle,
                    end_angle,
                    ..
                }) = p.shape_origin
                {
                    let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());

                    // Angle handles
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

                    let handles = [ArcHandle::StartAngle, ArcHandle::EndAngle];
                    for h in handles {
                        let pos = h.position(base_rect, start_angle, end_angle);
                        canvas.draw_circle(pos.to_skia(), handle_radius, &fill_paint);
                        canvas.draw_circle(pos.to_skia(), handle_radius, &handle_stroke);
                    }
                }
            }
        }
    }

    fn get_shape_params(&self) -> Option<ShapeOrigin> {
        Some(ShapeOrigin::Circle {
            arc_mode: self.arc_mode,
            start_angle: self.start_angle,
            end_angle: self.end_angle,
        })
    }

    fn set_shape_params(&mut self, origin: &ShapeOrigin) {
        if let ShapeOrigin::Circle {
            arc_mode,
            start_angle,
            end_angle,
        } = origin
        {
            self.arc_mode = *arc_mode;
            self.start_angle = *start_angle;
            self.end_angle = *end_angle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_path_nodes() {
        let elem = CircleFeature::create_ellipse_path(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            None,
            None,
            1.0,
            ArcMode::Full,
            0.0,
            360.0,
        );
        assert_eq!(elem.nodes.len(), 4);
        assert!(elem.is_closed);
    }
}
