use skia_safe as skia;

use crate::core::{
    calculate_resize_scales, hit_transform_handle, Element, ElementId, Point, PointerButton,
    PointerEvent, Rect, TransformHandle, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CornerRadiusHandle {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

impl CornerRadiusHandle {
    pub fn position(self, rect: Rect, corner_radius: f32, zoom: f32) -> Point {
        let r = rect.normalize();
        let max_r = (r.width / 2.0).min(r.height / 2.0);
        let min_inset = (10.0 / zoom).min(max_r * 0.5);
        let offset = if corner_radius > 0.001 {
            corner_radius.clamp(min_inset, max_r)
        } else {
            min_inset
        };
        match self {
            CornerRadiusHandle::TopLeft => Point::new(r.x + offset, r.y + offset),
            CornerRadiusHandle::TopRight => Point::new(r.x + r.width - offset, r.y + offset),
            CornerRadiusHandle::BottomRight => {
                Point::new(r.x + r.width - offset, r.y + r.height - offset)
            }
            CornerRadiusHandle::BottomLeft => Point::new(r.x + offset, r.y + r.height - offset),
        }
    }
}

fn hit_corner_radius_handle(
    rect: Rect,
    radii: crate::core::CornerRadii,
    p: Point,
    zoom: f32,
) -> Option<CornerRadiusHandle> {
    let handles = [
        CornerRadiusHandle::TopLeft,
        CornerRadiusHandle::TopRight,
        CornerRadiusHandle::BottomRight,
        CornerRadiusHandle::BottomLeft,
    ];
    let hit_r = (9.0 / zoom).max(6.0);
    for h in handles {
        let rad = match h {
            CornerRadiusHandle::TopLeft => radii.top_left,
            CornerRadiusHandle::TopRight => radii.top_right,
            CornerRadiusHandle::BottomRight => radii.bottom_right,
            CornerRadiusHandle::BottomLeft => radii.bottom_left,
        };
        let pos = h.position(rect, rad, zoom);
        if p.distance_to(pos) <= hit_r {
            return Some(h);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
enum SelectState {
    Idle,
    DraggingCornerRadius {
        handle: CornerRadiusHandle,
        rect: Rect,
        elem_id: ElementId,
    },
    DraggingElements {
        start_world: Point,
        initial_bounds: Rect,
        initial_elements: Vec<(ElementId, Element)>,
        hit_id: ElementId,
        has_dragged: bool,
    },
    Resizing {
        handle: TransformHandle,
        start_world: Point,
        initial_bounds: Rect,
        initial_elements: Vec<(ElementId, Element)>,
        origin: Point,
    },
    Rotating {
        center: Point,
        start_angle: f32,
        initial_elements: Vec<(ElementId, Element)>,
    },
    BoxSelecting {
        start_world: Point,
        current_world: Point,
    },
    Panning {
        last_screen: Point,
    },
}

pub struct SelectFeature {
    state: SelectState,
}

impl Default for SelectFeature {
    fn default() -> Self {
        Self {
            state: SelectState::Idle,
        }
    }
}

impl SelectFeature {
    pub fn new() -> Self {
        Self::default()
    }

    fn collect_selected_elements(ctx: &PluginContext) -> Vec<(ElementId, Element)> {
        ctx.document
            .elements
            .iter()
            .filter(|el| ctx.document.selected_ids.contains(&el.id()))
            .map(|el| (el.id(), el.clone()))
            .collect()
    }
}

impl FeaturePlugin for SelectFeature {
    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button == Some(PointerButton::Middle) {
            self.state = SelectState::Panning {
                last_screen: event.screen_pos,
            };
            ctx.set_cursor("grabbing");
            ctx.request_redraw();
            return;
        }

        if event.button != Some(PointerButton::Primary) {
            return;
        }

        // 1. Check if clicking on Corner Radius handles (for selected RectElement)
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            let corner_hit = ctx.document.elements.iter().find_map(|e| {
                if e.id() == sel_id {
                    if let Element::Rect(rect_el) = e {
                        let ch = hit_corner_radius_handle(
                            rect_el.rect,
                            rect_el.effective_radii(),
                            event.world_pos,
                            ctx.viewport.zoom,
                        )?;
                        return Some((ch, rect_el.rect.normalize(), sel_id));
                    }
                }
                None
            });

            if let Some((ch, norm_rect, elem_id)) = corner_hit {
                ctx.document.snapshot();
                self.state = SelectState::DraggingCornerRadius {
                    handle: ch,
                    rect: norm_rect,
                    elem_id,
                };
                ctx.set_cursor("crosshair");
                ctx.request_redraw();
                return;
            }
        }

        // 2. Check if clicking on active selection handles (Resize or Rotate)
        if let Some(bounds) = ctx.document.selection_bounds() {
            if let Some(handle) = hit_transform_handle(bounds, event.world_pos, ctx.viewport.zoom) {
                ctx.document.snapshot();
                let initial_elements = Self::collect_selected_elements(ctx);

                if handle == TransformHandle::Rotate {
                    let center = bounds.center();
                    let start_angle =
                        (event.world_pos.y - center.y).atan2(event.world_pos.x - center.x);
                    self.state = SelectState::Rotating {
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
                    self.state = SelectState::Resizing {
                        handle,
                        start_world: event.world_pos,
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

        // 2. Element hit test
        let hit_id = ctx.document.hit_test(event.world_pos);

        match hit_id {
            Some(id) => {
                if event.shift_pressed || event.ctrl_pressed {
                    if ctx.document.is_selected(id) {
                        ctx.document.selected_ids.remove(&id);
                    } else {
                        ctx.document.selected_ids.insert(id);
                    }
                } else if !ctx.document.is_selected(id) {
                    ctx.document.select(id, false);
                }

                ctx.document.snapshot();
                let initial_bounds = ctx.document.selection_bounds().unwrap_or(Rect::ZERO);
                let initial_elements = Self::collect_selected_elements(ctx);
                self.state = SelectState::DraggingElements {
                    start_world: event.world_pos,
                    initial_bounds,
                    initial_elements,
                    hit_id: id,
                    has_dragged: false,
                };
                ctx.set_cursor("move");
            }
            None => {
                if !event.shift_pressed {
                    ctx.document.deselect_all();
                }
                self.state = SelectState::BoxSelecting {
                    start_world: event.world_pos,
                    current_world: event.world_pos,
                };
                ctx.set_cursor("crosshair");
            }
        }
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match &mut self.state {
            SelectState::DraggingCornerRadius {
                handle,
                rect,
                elem_id,
            } => {
                let max_r = (rect.width / 2.0).min(rect.height / 2.0);
                let raw_dist = match handle {
                    CornerRadiusHandle::TopLeft => {
                        ((event.world_pos.x - rect.x) + (event.world_pos.y - rect.y)) / 2.0
                    }
                    CornerRadiusHandle::TopRight => {
                        ((rect.x + rect.width - event.world_pos.x) + (event.world_pos.y - rect.y))
                            / 2.0
                    }
                    CornerRadiusHandle::BottomRight => {
                        ((rect.x + rect.width - event.world_pos.x)
                            + (rect.y + rect.height - event.world_pos.y))
                            / 2.0
                    }
                    CornerRadiusHandle::BottomLeft => {
                        ((event.world_pos.x - rect.x) + (rect.y + rect.height - event.world_pos.y))
                            / 2.0
                    }
                };
                let min_threshold = (6.0 / ctx.viewport.zoom).min(max_r * 0.25);
                let new_radius = if raw_dist <= min_threshold {
                    0.0
                } else {
                    raw_dist.clamp(0.0, max_r).round()
                };

                if let Some(el) = ctx
                    .document
                    .elements
                    .iter_mut()
                    .find(|e| e.id() == *elem_id)
                {
                    if let Element::Rect(r) = el {
                        if event.alt_pressed {
                            match handle {
                                CornerRadiusHandle::TopLeft => r.corner_radii.top_left = new_radius,
                                CornerRadiusHandle::TopRight => r.corner_radii.top_right = new_radius,
                                CornerRadiusHandle::BottomRight => r.corner_radii.bottom_right = new_radius,
                                CornerRadiusHandle::BottomLeft => r.corner_radii.bottom_left = new_radius,
                            }
                            r.corner_radius = r.corner_radii.max_radius();
                        } else {
                            r.corner_radius = new_radius;
                            r.corner_radii = crate::core::CornerRadii::uniform(new_radius);
                        }
                    }
                }
                ctx.set_cursor("crosshair");
                ctx.request_redraw();
            }
            SelectState::Panning { last_screen } => {
                let dx = event.screen_pos.x - last_screen.x;
                let dy = event.screen_pos.y - last_screen.y;
                ctx.viewport.pan_by(dx, dy);
                *last_screen = event.screen_pos;
                ctx.request_redraw();
            }
            SelectState::Resizing {
                handle,
                initial_bounds,
                initial_elements,
                origin,
                ..
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
            SelectState::Rotating {
                center,
                start_angle,
                initial_elements,
            } => {
                let cur_angle = (event.world_pos.y - center.y).atan2(event.world_pos.x - center.x);
                let mut delta_angle = cur_angle - *start_angle;
                if event.shift_pressed {
                    let step = std::f32::consts::PI / 12.0; // 15 degrees snap
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
            SelectState::DraggingElements {
                start_world,
                initial_bounds,
                initial_elements,
                has_dragged,
                ..
            } => {
                let dist = event.world_pos.distance_to(*start_world);
                if dist > 1.0 / ctx.viewport.zoom {
                    *has_dragged = true;
                }

                let mut raw_dx = (event.world_pos.x - start_world.x).round();
                let mut raw_dy = (event.world_pos.y - start_world.y).round();

                if event.shift_pressed {
                    if raw_dx.abs() > 2.0 * raw_dy.abs() {
                        raw_dy = 0.0;
                    } else if raw_dy.abs() > 2.0 * raw_dx.abs() {
                        raw_dx = 0.0;
                    } else {
                        raw_dy = raw_dx.abs() * raw_dy.signum();
                    }
                }

                let proposed = Rect::new(
                    initial_bounds.x + raw_dx,
                    initial_bounds.y + raw_dy,
                    initial_bounds.width,
                    initial_bounds.height,
                );
                let selected_ids = ctx.document.selected_ids.clone();
                let (snapped_origin, _) = ctx.snap_rect(proposed, &selected_ids);
                let final_dx = (snapped_origin.x - initial_bounds.x).round();
                let final_dy = (snapped_origin.y - initial_bounds.y).round();

                for (id, initial_el) in initial_elements.iter() {
                    if let Some(el) = ctx.document.elements.iter_mut().find(|e| e.id() == *id) {
                        let mut modified = initial_el.clone();
                        modified.translate(final_dx, final_dy);
                        *el = modified;
                    }
                }

                ctx.set_cursor("move");
                ctx.request_redraw();
            }
            SelectState::BoxSelecting {
                start_world,
                current_world,
            } => {
                *current_world = event.world_pos;
                let select_rect = Rect::from_points(*start_world, *current_world).normalize();

                if select_rect.width > 2.0 || select_rect.height > 2.0 {
                    let mut matched_ids = Vec::new();
                    for el in &ctx.document.elements {
                        if el.bounds().intersects(select_rect) {
                            matched_ids.push(el.id());
                        }
                    }

                    if !event.shift_pressed {
                        ctx.document.deselect_all();
                    }
                    for id in matched_ids {
                        ctx.document.select(id, true);
                    }
                }
                ctx.request_redraw();
            }
            SelectState::Idle => {
                if ctx.document.selected_ids.len() == 1 {
                    let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
                    if let Some(Element::Rect(rect_el)) =
                        ctx.document.elements.iter().find(|e| e.id() == sel_id)
                    {
                        if hit_corner_radius_handle(
                            rect_el.rect,
                            rect_el.effective_radii(),
                            event.world_pos,
                            ctx.viewport.zoom,
                        )
                        .is_some()
                        {
                            ctx.set_cursor("crosshair");
                            return;
                        }
                    }
                }

                if let Some(bounds) = ctx.document.selection_bounds() {
                    if let Some(handle) = hit_transform_handle(bounds, event.world_pos, ctx.viewport.zoom) {
                        ctx.set_cursor(handle.cursor_name());
                        return;
                    }
                }

                if ctx.document.hit_test(event.world_pos).is_some() {
                    ctx.set_cursor("pointer");
                } else {
                    ctx.set_cursor("default");
                }
            }
        }
    }

    fn on_double_click(&mut self, ctx: &mut PluginContext, event: &PointerEvent) -> bool {
        let hit_text_handle_id = ctx.document.elements.iter().find_map(|el| {
            if let Element::Text(t) = el {
                if crate::plugins::features::text::hit_text_box_handle(
                    t.bounds(),
                    event.world_pos,
                    ctx.viewport.zoom,
                )
                .is_some()
                {
                    return Some(t.id);
                }
            }
            None
        });

        if let Some(id) = hit_text_handle_id {
            ctx.document.snapshot();
            if let Some(Element::Text(t)) =
                ctx.document.elements.iter_mut().find(|el| el.id() == id)
            {
                t.box_width = None;
                t.box_height = None;
            }
            ctx.request_redraw();
            return true;
        }

        let hit_id = ctx.document.hit_test(event.world_pos);
        if let Some(id) = hit_id {
            ctx.document.select(id, false);
            ctx.request_redraw();
            return true;
        }
        false
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if let SelectState::DraggingElements {
            hit_id,
            has_dragged: false,
            ..
        } = self.state
        {
            if !event.shift_pressed && ctx.document.selected_ids.len() > 1 {
                ctx.document.select(hit_id, false);
            }
        }
        self.state = SelectState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.state = SelectState::Idle;
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
        if let SelectState::BoxSelecting {
            start_world,
            current_world,
        } = self.state
        {
            let r = Rect::from_points(start_world, current_world).round();

            let mut fill_paint = skia::Paint::default();
            fill_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.12), None);
            fill_paint.set_style(skia::PaintStyle::Fill);

            let mut stroke_paint = skia::Paint::default();
            stroke_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.85), None);
            stroke_paint.set_style(skia::PaintStyle::Stroke);
            stroke_paint.set_stroke_width((1.0 / viewport.zoom).max(1.0));
            stroke_paint.set_anti_alias(true);

            canvas.draw_rect(r.to_skia(), &fill_paint);
            canvas.draw_rect(r.to_skia(), &stroke_paint);
        }

        // Render Corner Radius Handles if a single RectElement is selected
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Rect(rect_el)) =
                ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                let r = rect_el.rect.normalize();
                let handles = [
                    CornerRadiusHandle::TopLeft,
                    CornerRadiusHandle::TopRight,
                    CornerRadiusHandle::BottomRight,
                    CornerRadiusHandle::BottomLeft,
                ];
                let handle_radius = (4.5 / viewport.zoom).clamp(3.5, 6.5);
                let stroke_w = (1.5 / viewport.zoom).max(1.0);

                let mut fill_paint = skia::Paint::default();
                fill_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                fill_paint.set_style(skia::PaintStyle::Fill);
                fill_paint.set_anti_alias(true);

                let mut stroke_paint = skia::Paint::default();
                stroke_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 1.0), None);
                stroke_paint.set_style(skia::PaintStyle::Stroke);
                stroke_paint.set_stroke_width(stroke_w);
                stroke_paint.set_anti_alias(true);

                let radii = rect_el.effective_radii();
                for h in handles {
                    let rad = match h {
                        CornerRadiusHandle::TopLeft => radii.top_left,
                        CornerRadiusHandle::TopRight => radii.top_right,
                        CornerRadiusHandle::BottomRight => radii.bottom_right,
                        CornerRadiusHandle::BottomLeft => radii.bottom_left,
                    };
                    let pos = h.position(r, rad, viewport.zoom);
                    canvas.draw_circle(pos.to_skia(), handle_radius, &fill_paint);
                    canvas.draw_circle(pos.to_skia(), handle_radius, &stroke_paint);
                }
            }
        }
    }
}
