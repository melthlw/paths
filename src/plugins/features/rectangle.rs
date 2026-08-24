use skia_safe as skia;

use crate::core::{
    calculate_resize_scales, hit_transform_handle, Element, ElementId, Point, PointerButton,
    PointerEvent, Rect, RectElement, ShapeOrigin, TransformHandle, Viewport,
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

pub fn hit_corner_radius_handle(
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
enum RectToolState {
    Idle,
    Creating {
        start_pos: Point,
        current_pos: Point,
        is_square_locked: bool,
        is_center_anchored: bool,
    },
    DraggingCornerRadius {
        handle: CornerRadiusHandle,
        rect: Rect,
        elem_id: ElementId,
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

pub struct RectangleFeature {
    state: RectToolState,
    pub corner_radius: f32,
    pub corner_radii: crate::core::CornerRadii,
    pub corner_style: crate::core::CornerStyle,
}

impl Default for RectangleFeature {
    fn default() -> Self {
        Self {
            state: RectToolState::Idle,
            corner_radius: 0.0,
            corner_radii: crate::core::CornerRadii::default(),
            corner_style: crate::core::CornerStyle::default(),
        }
    }
}

impl RectangleFeature {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_rect(&self) -> Option<Rect> {
        if let RectToolState::Creating {
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

impl FeaturePlugin for RectangleFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:rectangle");
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
                    self.state = RectToolState::Rotating {
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
                    self.state = RectToolState::Resizing {
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

        // 1. Check if clicking on Corner Radius handle of a selected rectangle
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
                self.state = RectToolState::DraggingCornerRadius {
                    handle: ch,
                    rect: norm_rect,
                    elem_id,
                };
                ctx.set_cursor("tool:rectangle");
                ctx.request_redraw();
                return;
            }
        }

        // 2. Check if clicking on an existing rectangle element
        let hit_rect = ctx.document.elements.iter().rev().find_map(|e| {
            if let Element::Rect(r) = e {
                if r.rect.normalize().contains(event.world_pos) {
                    return Some((e.id(), r.corner_radius, r.corner_radii, r.corner_style, r.rect.normalize()));
                }
            }
            None
        });

        if let Some((id, r_radius, r_radii, r_style, r_rect)) = hit_rect {
            ctx.document.select(id, false);
            self.corner_radius = r_radius;
            self.corner_radii = r_radii;
            self.corner_style = r_style;
            self.state = RectToolState::MovingElement {
                start_world: event.world_pos,
                initial_rect: r_rect,
                elem_id: id,
                has_dragged: false,
            };
            ctx.set_cursor("move");
            ctx.request_redraw();
            return;
        }

        // 3. Otherwise, click on empty canvas -> deselect and start creating new rectangle
        if !event.shift_pressed {
            ctx.document.deselect_all();
        }
        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude).round();
        self.state = RectToolState::Creating {
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
            RectToolState::DraggingCornerRadius {
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

                self.corner_radius = new_radius;
                self.corner_radii = crate::core::CornerRadii::uniform(new_radius);

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
            RectToolState::MovingElement {
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
                        if let Element::Rect(r) = el {
                            r.rect = Rect::new(
                                initial_rect.x + dx,
                                initial_rect.y + dy,
                                initial_rect.width,
                                initial_rect.height,
                            );
                        }
                    }
                    ctx.set_cursor("move");
                    ctx.request_redraw();
                }
            }
            RectToolState::Creating {
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
            RectToolState::Resizing {
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
            RectToolState::Rotating {
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
            RectToolState::Idle => {
                if let Some(bounds) = ctx.document.selection_bounds() {
                    if let Some(handle) = hit_transform_handle(bounds, event.world_pos, ctx.viewport.zoom) {
                        ctx.set_cursor(handle.cursor_name());
                        return;
                    }
                }

                // Check if hovering corner radius handle of a selected rectangle
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

                // Check if hovering over any existing rectangle
                let is_hovering_rect = ctx.document.elements.iter().any(|e| {
                    if let Element::Rect(r) = e {
                        r.rect.normalize().contains(event.world_pos)
                    } else {
                        false
                    }
                });

                if is_hovering_rect {
                    ctx.set_cursor("pointer");
                } else {
                    ctx.set_cursor("tool:rectangle");
                }
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match self.state {
            RectToolState::Creating { .. } => {
                if let Some(rect) = self.current_rect() {
                    if rect.width > 2.0 && rect.height > 2.0 {
                        let mut elem = RectElement::new(
                            rect,
                            Some(ctx.active_fill_color),
                            ctx.active_stroke_color,
                        );
                        elem.stroke_width = ctx.active_stroke_width;
                        elem.corner_radius = self.corner_radius;
                        elem.corner_radii = self.corner_radii;
                        elem.corner_style = self.corner_style;
                        let new_id = elem.id;
                        ctx.document.add_element(Element::Rect(elem));
                        ctx.document.select(new_id, false);
                    }
                }
            }
            RectToolState::Idle => {
                let hit = ctx.document.elements.iter().rev().find_map(|e| {
                    if let Element::Rect(r) = e {
                        if r.rect.normalize().contains(event.world_pos) {
                            return Some(r.id);
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
            RectToolState::DraggingCornerRadius {
                elem_id,
                ..
            } => {
                if !event.shift_pressed {
                    ctx.document.select(elem_id, false);
                }
            }
            _ => {}
        }

        self.state = RectToolState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("tool:rectangle");
        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.state = RectToolState::Idle;
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
        // 1. Draw creation preview if creating
        if let Some(rect) = self.current_rect() {
            let mut temp_rect = RectElement::new(
                rect,
                Some(ctx.active_fill_color),
                ctx.active_stroke_color,
            );
            temp_rect.stroke_width = ctx.active_stroke_width;
            temp_rect.corner_radius = self.corner_radius;
            temp_rect.corner_radii = self.corner_radii;
            temp_rect.corner_style = self.corner_style;
            temp_rect.render(canvas);
        }

        // 2. Draw selection box and Corner Radius handles for selected RectElement(s)
        if ctx.document.selected_ids.len() == 1 {
            let sel_id = *ctx.document.selected_ids.iter().next().unwrap();
            if let Some(Element::Rect(rect_el)) =
                ctx.document.elements.iter().find(|e| e.id() == sel_id)
            {
                let r = rect_el.rect.normalize();

                // Corner radius handles
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

                let mut handle_stroke = skia::Paint::default();
                handle_stroke.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 1.0), None);
                handle_stroke.set_style(skia::PaintStyle::Stroke);
                handle_stroke.set_stroke_width(stroke_w);
                handle_stroke.set_anti_alias(true);

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
                    canvas.draw_circle(pos.to_skia(), handle_radius, &handle_stroke);
                }
            }
        }
    }

    fn get_shape_params(&self) -> Option<ShapeOrigin> {
        Some(ShapeOrigin::Rectangle {
            corner_radius: self.corner_radius,
            corner_radii: self.corner_radii,
            corner_style: self.corner_style,
        })
    }

    fn set_shape_params(&mut self, origin: &ShapeOrigin) {
        if let ShapeOrigin::Rectangle {
            corner_radius,
            corner_radii,
            corner_style,
        } = origin
        {
            self.corner_radius = *corner_radius;
            self.corner_radii = *corner_radii;
            self.corner_style = *corner_style;
        }
    }
}
