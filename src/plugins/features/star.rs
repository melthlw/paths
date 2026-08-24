use skia_safe as skia;

use crate::core::{
    calculate_resize_scales, hit_transform_handle, Color, Element, ElementId, PathElement,
    PathNode, Point, PointerButton, PointerEvent, Rect, ShapeOrigin, TransformHandle, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StarHandle {
    InnerRatio,
    CornerRadius,
}

impl StarHandle {
    pub fn position(
        self,
        rect: Rect,
        num_points: u32,
        inner_ratio: f32,
        corner_radius: f32,
    ) -> Point {
        let r = rect.normalize();
        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;
        let rx = r.width / 2.0;
        let ry = r.height / 2.0;

        match self {
            StarHandle::InnerRatio => {
                let total_vertices = (num_points * 2) as f32;
                let angle = -std::f32::consts::FRAC_PI_2 + (std::f32::consts::TAU / total_vertices);
                let cur_rx = rx * inner_ratio;
                let cur_ry = ry * inner_ratio;
                Point::new(cx + cur_rx * angle.cos(), cy + cur_ry * angle.sin()).round()
            }
            StarHandle::CornerRadius => {
                let offset = corner_radius.min(ry * 0.75);
                Point::new(cx, (cy - ry) + offset).round()
            }
        }
    }
}

pub fn hit_star_handle(
    rect: Rect,
    num_points: u32,
    inner_ratio: f32,
    corner_radius: f32,
    p: Point,
    zoom: f32,
) -> Option<StarHandle> {
    let hit_r = (8.0 / zoom).max(6.0);
    let handles = [StarHandle::InnerRatio, StarHandle::CornerRadius];
    for h in handles {
        let pos = h.position(rect, num_points, inner_ratio, corner_radius);
        if p.distance_to(pos) <= hit_r {
            return Some(h);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
enum StarToolState {
    Idle,
    Creating {
        start_pos: Point,
        current_pos: Point,
        is_square_locked: bool,
        is_center_anchored: bool,
    },
    DraggingHandle {
        handle: StarHandle,
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

pub struct StarFeature {
    state: StarToolState,
    pub num_points: u32,
    pub inner_ratio: f32,
    pub corner_radius: f32,
}

impl Default for StarFeature {
    fn default() -> Self {
        Self {
            state: StarToolState::Idle,
            num_points: 5,
            inner_ratio: 0.45,
            corner_radius: 0.0,
        }
    }
}

impl StarFeature {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_rect(&self) -> Option<Rect> {
        if let StarToolState::Creating {
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

    pub fn create_star_path(
        rect: Rect,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
        stroke_width: f32,
        num_points: u32,
        inner_ratio: f32,
        corner_radius: f32,
    ) -> PathElement {
        let r = rect.normalize();
        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;
        let rx = r.width / 2.0;
        let ry = r.height / 2.0;

        let total_vertices = num_points * 2;
        let mut raw_points = Vec::with_capacity(total_vertices as usize);

        for i in 0..total_vertices {
            let angle = -std::f32::consts::FRAC_PI_2
                + (i as f32) * (std::f32::consts::TAU / total_vertices as f32);
            let (cur_rx, cur_ry) = if i % 2 == 0 {
                (rx, ry)
            } else {
                (rx * inner_ratio, ry * inner_ratio)
            };
            let px = cx + cur_rx * angle.cos();
            let py = cy + cur_ry * angle.sin();
            raw_points.push(Point::new(px.round(), py.round()));
        }

        let nodes = if corner_radius > 0.001 {
            Self::apply_star_corner_rounding(&raw_points, corner_radius)
        } else {
            raw_points.iter().map(|p| PathNode::new(*p)).collect()
        };

        let mut elem = PathElement::new(nodes, true, fill_color, stroke_color, stroke_width);
        elem.shape_origin = Some(ShapeOrigin::Star {
            corner_radius,
            points: num_points,
            inner_ratio,
        });
        elem.shape_rect = Some(r);
        elem.stroke_width = stroke_width;
        elem
    }

    /// Apply corner rounding specifically to star outer tips without distorting inner valleys
    pub fn apply_star_corner_rounding(points: &[Point], radius: f32) -> Vec<PathNode> {
        let n = points.len();
        if n < 3 || radius < 0.001 {
            return points.iter().map(|p| PathNode::new(*p)).collect();
        }

        let mut nodes = Vec::with_capacity(n * 2);

        for i in 0..n {
            let prev = if i == 0 { points[n - 1] } else { points[i - 1] };
            let curr = points[i];
            let next = if i == n - 1 { points[0] } else { points[i + 1] };

            // On a star, round outer vertices (i % 2 == 0)
            if i % 2 == 0 {
                let d_in = curr.distance_to(prev);
                let d_out = curr.distance_to(next);
                let max_r = (d_in.min(d_out) * 0.85).min(radius);

                if max_r < 0.5 {
                    nodes.push(PathNode::new(curr));
                    continue;
                }

                let dir_in_x = (prev.x - curr.x) / d_in;
                let dir_in_y = (prev.y - curr.y) / d_in;
                let dir_out_x = (next.x - curr.x) / d_out;
                let dir_out_y = (next.y - curr.y) / d_out;

                let p_in = Point::new(curr.x + dir_in_x * max_r, curr.y + dir_in_y * max_r);
                let p_out = Point::new(curr.x + dir_out_x * max_r, curr.y + dir_out_y * max_r);

                let k = 0.55228475_f32;
                let handle_in = Point::new(
                    p_in.x + (curr.x - p_in.x) * k,
                    p_in.y + (curr.y - p_in.y) * k,
                );
                let handle_out = Point::new(
                    p_out.x + (curr.x - p_out.x) * k,
                    p_out.y + (curr.y - p_out.y) * k,
                );

                let mut node_in = PathNode::new(p_in);
                node_in.handle_out = Some(handle_in);

                let mut node_out = PathNode::new(p_out);
                node_out.handle_in = Some(handle_out);

                nodes.push(node_in);
                nodes.push(node_out);
            } else {
                // Inner valley remains a sharp, crisp corner
                nodes.push(PathNode::new(curr));
            }
        }

        nodes
    }

    /// Apply corner rounding to polygon vertices by replacing each corner
    /// with a short Bézier curve segment
    pub fn apply_corner_rounding(points: &[Point], radius: f32, is_closed: bool) -> Vec<PathNode> {
        let n = points.len();
        if n < 3 || radius < 0.001 {
            return points.iter().map(|p| PathNode::new(*p)).collect();
        }

        let mut nodes = Vec::with_capacity(n * 2);

        for i in 0..n {
            let prev = if i == 0 {
                if is_closed {
                    points[n - 1]
                } else {
                    continue;
                }
            } else {
                points[i - 1]
            };
            let curr = points[i];
            let next = if i == n - 1 {
                if is_closed {
                    points[0]
                } else {
                    continue;
                }
            } else {
                points[i + 1]
            };

            let d_in = curr.distance_to(prev);
            let d_out = curr.distance_to(next);
            let max_r = (d_in.min(d_out) / 2.0).min(radius);

            if max_r < 0.5 {
                nodes.push(PathNode::new(curr));
                continue;
            }

            // Direction vectors
            let dir_in_x = (prev.x - curr.x) / d_in;
            let dir_in_y = (prev.y - curr.y) / d_in;
            let dir_out_x = (next.x - curr.x) / d_out;
            let dir_out_y = (next.y - curr.y) / d_out;

            // Offset point along incoming edge
            let p_in = Point::new(curr.x + dir_in_x * max_r, curr.y + dir_in_y * max_r);
            // Offset point along outgoing edge
            let p_out = Point::new(curr.x + dir_out_x * max_r, curr.y + dir_out_y * max_r);

            let k = 0.55228475_f32;
            let handle_in = Point::new(
                p_in.x + (curr.x - p_in.x) * k,
                p_in.y + (curr.y - p_in.y) * k,
            );
            let handle_out = Point::new(
                p_out.x + (curr.x - p_out.x) * k,
                p_out.y + (curr.y - p_out.y) * k,
            );

            let mut node_in = PathNode::new(p_in);
            node_in.handle_out = Some(handle_in);

            let mut node_out = PathNode::new(p_out);
            node_out.handle_in = Some(handle_out);

            nodes.push(node_in);
            nodes.push(node_out);
        }

        nodes
    }
}

impl FeaturePlugin for StarFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:star");
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
                    self.state = StarToolState::Rotating {
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
                    self.state = StarToolState::Resizing {
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

        // 1. Check if clicking on an on-canvas handle of the currently selected star
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Path(p)) = ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                if let Some(ShapeOrigin::Star {
                    points,
                    inner_ratio,
                    corner_radius,
                }) = p.shape_origin
                {
                    let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());
                    if let Some(h) = hit_star_handle(
                        base_rect,
                        points,
                        inner_ratio,
                        corner_radius,
                        event.world_pos,
                        ctx.viewport.zoom,
                    ) {
                        self.state = StarToolState::DraggingHandle {
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

        // 2. Check if clicking on an existing star to select it
        let hit_star = ctx.document.elements.iter().rev().find_map(|e| {
            if let Element::Path(p) = e {
                if let Some(ShapeOrigin::Star {
                    points,
                    inner_ratio,
                    corner_radius,
                }) = p.shape_origin
                {
                    if p.bounds().contains(event.world_pos) {
                        return Some((e.id(), points, inner_ratio, corner_radius));
                    }
                }
            }
            None
        });

        if let Some((id, pts, ratio, r_rad)) = hit_star {
            ctx.document.select(id, false);
            self.num_points = pts;
            self.inner_ratio = ratio;
            self.corner_radius = r_rad;
            self.state = StarToolState::MovingElement {
                start_world: event.world_pos,
                elem_id: id,
                has_dragged: false,
            };
            ctx.set_cursor("pointer");
            ctx.request_redraw();
            return;
        }

        // 3. Start creating new star
        if !event.shift_pressed {
            ctx.document.deselect_all();
        }

        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude).round();
        self.state = StarToolState::Creating {
            start_pos: snapped,
            current_pos: snapped,
            is_square_locked: event.shift_pressed,
            is_center_anchored: event.alt_pressed,
        };
        ctx.set_cursor("tool:star");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match &mut self.state {
            StarToolState::Creating {
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
                ctx.set_cursor("tool:star");
                ctx.request_redraw();
            }
            StarToolState::DraggingHandle {
                handle,
                elem_id,
                rect,
            } => {
                let r = rect.normalize();
                let cx = r.x + r.width / 2.0;
                let cy = r.y + r.height / 2.0;
                let rx = r.width / 2.0;
                let ry = r.height / 2.0;

                match handle {
                    StarHandle::InnerRatio => {
                        let dist = event.world_pos.distance_to(Point::new(cx, cy));
                        let ratio = (dist / rx.max(1.0)).clamp(0.05, 0.95);
                        self.inner_ratio = (ratio * 100.0).round() / 100.0;
                    }
                    StarHandle::CornerRadius => {
                        let top_y = cy - ry;
                        let offset = (event.world_pos.y - top_y).clamp(0.0, ry * 0.75);
                        self.corner_radius = offset.round();
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
                    let fill = p.fill_color;
                    let stroke = p.stroke_color;
                    let sw = p.stroke_width;
                    let mut new_elem = Self::create_star_path(
                        base_rect,
                        fill,
                        stroke,
                        sw,
                        self.num_points,
                        self.inner_ratio,
                        self.corner_radius,
                    );
                    new_elem.id = target_id;
                    new_elem.shape_rect = Some(base_rect);
                    *p = new_elem;
                }

                ctx.set_cursor("grabbing");
                ctx.request_redraw();
            }
            StarToolState::MovingElement {
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
            StarToolState::Resizing {
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
            StarToolState::Rotating {
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
            StarToolState::Idle => {
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
                        if let Some(ShapeOrigin::Star {
                            points,
                            inner_ratio,
                            corner_radius,
                        }) = p.shape_origin
                        {
                            let base_rect = p.shape_rect.unwrap_or_else(|| p.bounds());
                            if hit_star_handle(
                                base_rect,
                                points,
                                inner_ratio,
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
                            matches!(p.shape_origin, Some(ShapeOrigin::Star { .. }))
                                && p.bounds().contains(event.world_pos)
                        } else {
                            false
                        }
                    });
                    if is_hovering {
                        ctx.set_cursor("pointer");
                    } else {
                        ctx.set_cursor("tool:star");
                    }
                }
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        match self.state {
            StarToolState::Creating { .. } => {
                if let Some(rect) = self.current_rect() {
                    if rect.width > 2.0 && rect.height > 2.0 {
                        let elem = Self::create_star_path(
                            rect,
                            Some(ctx.active_fill_color),
                            ctx.active_stroke_color,
                            ctx.active_stroke_width,
                            self.num_points,
                            self.inner_ratio,
                            self.corner_radius,
                        );
                        let new_id = elem.id;
                        ctx.document.add_element(Element::Path(elem));
                        ctx.document.select(new_id, false);
                    }
                }
            }
            StarToolState::DraggingHandle { .. } => {
                ctx.document.snapshot();
            }
            StarToolState::MovingElement { has_dragged, .. } => {
                if has_dragged {
                    ctx.document.snapshot();
                }
            }
            _ => {}
        }

        self.state = StarToolState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("tool:star");
        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.state = StarToolState::Idle;
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
            let elem = Self::create_star_path(
                rect,
                Some(ctx.active_fill_color),
                ctx.active_stroke_color,
                ctx.active_stroke_width,
                self.num_points,
                self.inner_ratio,
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

        // 2. Draw selection bounding box and handles if a Star is selected
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Path(p)) = ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                if let Some(ShapeOrigin::Star {
                    points,
                    inner_ratio,
                    corner_radius,
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

                    let handles = [StarHandle::InnerRatio, StarHandle::CornerRadius];
                    for h in handles {
                        let pos = h.position(base_rect, points, inner_ratio, corner_radius);
                        canvas.draw_circle(pos.to_skia(), handle_radius, &fill_paint);
                        canvas.draw_circle(pos.to_skia(), handle_radius, &handle_stroke);
                    }
                }
            }
        }
    }

    fn get_shape_params(&self) -> Option<ShapeOrigin> {
        Some(ShapeOrigin::Star {
            points: self.num_points,
            inner_ratio: self.inner_ratio,
            corner_radius: self.corner_radius,
        })
    }

    fn set_shape_params(&mut self, origin: &ShapeOrigin) {
        if let ShapeOrigin::Star {
            points,
            inner_ratio,
            corner_radius,
        } = origin
        {
            self.num_points = *points;
            self.inner_ratio = *inner_ratio;
            self.corner_radius = *corner_radius;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_star_path_vertices() {
        let elem = StarFeature::create_star_path(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            None,
            None,
            1.0,
            5,
            0.5,
            0.0,
        );
        assert_eq!(elem.nodes.len(), 10);
        assert!(elem.is_closed);
    }
}
