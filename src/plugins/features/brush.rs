use skia_safe as skia;

use crate::core::{
    hit_transform_handle, BrushMode, BrushStroke, BrushStyle, Element, ElementId, Point,
    PointerButton, PointerEvent, Rect, StrokeCap, StrokeJoin, TransformHandle, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, PartialEq)]
enum BrushToolState {
    Idle,
    Drawing,
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

pub struct BrushFeature {
    state: BrushToolState,
    current_points: Vec<Point>,
    raw_points: Vec<Point>,
    pub mode: BrushMode,
    pub style: BrushStyle,
    pub width: f32,
    pub smoothing: f32,
    pub pressure_dynamics: bool,
    pub taper_start: bool,
    pub taper_end: bool,
    pub calligraphy_angle: f32,
    pub auto_close: bool,
    pub cap_style: StrokeCap,
    pub join_style: StrokeJoin,
}

impl Default for BrushFeature {
    fn default() -> Self {
        Self {
            state: BrushToolState::Idle,
            current_points: Vec::new(),
            raw_points: Vec::new(),
            mode: BrushMode::Brush,
            style: BrushStyle::Round,
            width: 6.0,
            smoothing: 0.5,
            pressure_dynamics: true,
            taper_start: false,
            taper_end: false,
            calligraphy_angle: 45.0,
            auto_close: false,
            cap_style: StrokeCap::Round,
            join_style: StrokeJoin::Round,
        }
    }
}

impl BrushFeature {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn point_count(&self) -> usize {
        self.current_points.len()
    }

    /// Real-time smoothing filter / stabilizer using exponential moving average and Chaikin corner smoothing
    fn smooth_points(raw: &[Point], smoothing_factor: f32) -> Vec<Point> {
        if raw.len() <= 2 || smoothing_factor <= 0.01 {
            return raw.to_vec();
        }

        let alpha = (1.0 - smoothing_factor.clamp(0.0, 0.95)).max(0.05);
        let mut filtered = Vec::with_capacity(raw.len());
        filtered.push(raw[0]);

        for i in 1..raw.len() {
            let prev = *filtered.last().unwrap();
            let curr = raw[i];
            let smoothed = Point::new(
                prev.x + alpha * (curr.x - prev.x),
                prev.y + alpha * (curr.y - prev.y),
            );
            filtered.push(smoothed);
        }

        // Apply Chaikin corner refinement if high smoothing requested
        if smoothing_factor > 0.4 && filtered.len() > 3 {
            let mut refined = Vec::with_capacity(filtered.len() * 2);
            refined.push(filtered[0]);
            for window in filtered.windows(2) {
                let p0 = window[0];
                let p1 = window[1];
                let q = Point::new(0.75 * p0.x + 0.25 * p1.x, 0.75 * p0.y + 0.25 * p1.y);
                let r = Point::new(0.25 * p0.x + 0.75 * p1.x, 0.25 * p0.y + 0.75 * p1.y);
                refined.push(q);
                refined.push(r);
            }
            if let Some(&last) = filtered.last() {
                refined.push(last);
            }
            refined
        } else {
            filtered
        }
    }
}

impl FeaturePlugin for BrushFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:brush");
    }

    fn as_brush_feature(&self) -> Option<&BrushFeature> {
        Some(self)
    }

    fn as_brush_feature_mut(&mut self) -> Option<&mut BrushFeature> {
        Some(self)
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
                    self.state = BrushToolState::Rotating {
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
                    self.state = BrushToolState::Resizing {
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

        // 1. Universal Selection: Check if clicking on ANY existing element with Ctrl / Shift or when already selected
        if event.ctrl_pressed || event.shift_pressed || (ctx.document.selected_ids.len() > 0 && ctx.document.hit_test(event.world_pos).is_some()) {
            if let Some(hit_id) = ctx.document.hit_test(event.world_pos) {
                ctx.document.select(hit_id, event.shift_pressed);
                self.state = BrushToolState::MovingElement {
                    start_world: event.world_pos,
                    elem_id: hit_id,
                    has_dragged: false,
                };
                ctx.set_cursor("move");
                ctx.request_redraw();
                return;
            }
        }

        // 2. Start Drawing Freehand Stroke
        if !event.shift_pressed && !event.ctrl_pressed {
            ctx.document.deselect_all();
        }

        self.state = BrushToolState::Drawing;
        self.raw_points.clear();
        self.current_points.clear();
        self.raw_points.push(event.world_pos);
        self.current_points.push(event.world_pos);
        ctx.set_cursor("tool:brush");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match &mut self.state {
            BrushToolState::Idle => {
                ctx.set_cursor("tool:brush");
            }
            BrushToolState::Drawing => {
                if let Some(last) = self.raw_points.last() {
                    let min_dist = (1.5 / ctx.viewport.zoom).max(0.5);
                    if last.distance_to(event.world_pos) >= min_dist {
                        self.raw_points.push(event.world_pos);
                        self.current_points = Self::smooth_points(&self.raw_points, self.smoothing);
                        ctx.request_redraw();
                    }
                }
            }
            BrushToolState::MovingElement {
                start_world,
                elem_id: _,
                has_dragged,
            } => {
                let dx = event.world_pos.x - start_world.x;
                let dy = event.world_pos.y - start_world.y;
                if dx.hypot(dy) > 2.0 {
                    if !*has_dragged {
                        ctx.document.snapshot();
                        *has_dragged = true;
                    }
                    ctx.document.translate_selected(dx, dy);
                    *start_world = event.world_pos;
                    ctx.set_cursor("move");
                    ctx.request_redraw();
                }
            }
            BrushToolState::Resizing {
                handle,
                initial_bounds,
                initial_elements,
                origin,
            } => {
                let empty_exclude = std::collections::HashSet::new();
                let snapped = ctx.snap_point(event.world_pos, &empty_exclude);
                let (sx, sy) = crate::core::calculate_resize_scales(
                    *handle,
                    snapped,
                    *initial_bounds,
                    *origin,
                    event.shift_pressed,
                );

                for (id, original_elem) in initial_elements {
                    if let Some(elem) = ctx.document.find_element_mut(*id) {
                        *elem = original_elem.clone();
                        elem.scale(*origin, sx, sy);
                    }
                }
                ctx.request_redraw();
            }
            BrushToolState::Rotating {
                center,
                start_angle,
                initial_elements,
            } => {
                let current_angle =
                    (event.world_pos.y - center.y).atan2(event.world_pos.x - center.x);
                let mut delta = current_angle - *start_angle;
                if event.shift_pressed {
                    let snap_15 = 15.0_f32.to_radians();
                    delta = (delta / snap_15).round() * snap_15;
                }

                for (id, original_elem) in initial_elements {
                    if let Some(elem) = ctx.document.find_element_mut(*id) {
                        *elem = original_elem.clone();
                        elem.rotate(*center, delta);
                    }
                }
                ctx.request_redraw();
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match self.state {
            BrushToolState::Drawing => {
                self.raw_points.push(event.world_pos);
                let smoothed = Self::smooth_points(&self.raw_points, self.smoothing);

                if smoothed.len() >= 2 {
                    ctx.document.snapshot();
                    let stroke_w = self.width.max(1.0);
                    let color = ctx.active_stroke_color.unwrap_or(ctx.active_fill_color);

                    let is_closed = self.auto_close || (smoothed.len() > 3 && smoothed[0].distance_to(*smoothed.last().unwrap()) < (stroke_w * 2.0).max(12.0));

                    match self.mode {
                        BrushMode::Pencil => {
                            // Pencil Mode: generates editable native vector PathElement
                            let mut stroke = BrushStroke::new(smoothed, color, stroke_w);
                            stroke.auto_close = is_closed;
                            stroke.cap_style = self.cap_style;
                            stroke.join_style = self.join_style;
                            let path_elem = stroke.to_path_element();
                            let elem_id = path_elem.id;
                            ctx.document.add_element(Element::Path(path_elem));
                            ctx.document.select(elem_id, false);
                        }
                        BrushMode::Brush => {
                            // Brush Mode: generates rich styled BrushStroke
                            let mut stroke = BrushStroke::new(smoothed, color, stroke_w);
                            stroke.style = self.style;
                            stroke.smoothing = self.smoothing;
                            stroke.calligraphy_angle = self.calligraphy_angle;
                            stroke.auto_close = is_closed;
                            stroke.cap_style = self.cap_style;
                            stroke.join_style = self.join_style;
                            stroke.taper_start = self.taper_start;
                            stroke.taper_end = self.taper_end;
                            let stroke_id = stroke.id;
                            ctx.document.add_element(Element::Brush(stroke));
                            ctx.document.select(stroke_id, false);
                        }
                    }
                }

                self.raw_points.clear();
                self.current_points.clear();
                self.state = BrushToolState::Idle;
                ctx.set_cursor("tool:brush");
                ctx.request_redraw();
            }
            _ => {
                self.state = BrushToolState::Idle;
                ctx.set_cursor("tool:brush");
                ctx.request_redraw();
            }
        }
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.raw_points.clear();
        self.current_points.clear();
        self.state = BrushToolState::Idle;
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        _viewport: &Viewport,
    ) {
        if self.current_points.len() >= 2 {
            let color = ctx.active_stroke_color.unwrap_or(ctx.active_fill_color);
            let mut stroke = BrushStroke::new(self.current_points.clone(), color, self.width.max(1.0));
            stroke.style = self.style;
            stroke.calligraphy_angle = self.calligraphy_angle;
            stroke.auto_close = self.auto_close;
            stroke.cap_style = self.cap_style;
            stroke.join_style = self.join_style;
            stroke.render(canvas);
        }
    }
}

