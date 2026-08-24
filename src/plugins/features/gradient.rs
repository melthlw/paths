use skia_safe as skia;

use crate::core::{
    Color, Element, ElementId, Gradient, GradientType, Point, PointerButton, PointerEvent, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
enum DraggingHandle {
    Start,
    End,
    Stop(usize),
}

pub struct GradientFeature {
    target_id: Option<ElementId>,
    is_dragging: bool,
    drag_handle: Option<DraggingHandle>,
    drag_start: Option<Point>,
    live_gradient: Option<Gradient>,
}

impl Default for GradientFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl GradientFeature {
    pub fn new() -> Self {
        Self {
            target_id: None,
            is_dragging: false,
            drag_handle: None,
            drag_start: None,
            live_gradient: None,
        }
    }

    fn find_or_init_target(&mut self, ctx: &mut PluginContext, point: Point) -> Option<ElementId> {
        // First check currently selected elements
        if let Some(&first_selected) = ctx.document.selected_ids.iter().next() {
            if let Some(elem) = ctx
                .document
                .elements
                .iter()
                .find(|e| e.id() == first_selected)
            {
                if elem.hit_test(point) || ctx.document.selected_ids.len() == 1 {
                    return Some(first_selected);
                }
            }
        }

        // Otherwise hit-test top-to-bottom
        for elem in ctx.document.elements.iter().rev() {
            if elem.hit_test(point) {
                let id = elem.id();
                ctx.document.select(id, false);
                return Some(id);
            }
        }
        None
    }
}

impl FeaturePlugin for GradientFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:gradient");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        let target_id = match self.find_or_init_target(ctx, event.world_pos) {
            Some(id) => id,
            None => {
                self.target_id = None;
                self.live_gradient = None;
                ctx.request_redraw();
                return;
            }
        };

        self.target_id = Some(target_id);

        if self.live_gradient.is_none() {
            for el in &ctx.document.elements {
                if el.id() == target_id {
                    let grad_opt = match el {
                        Element::Rect(r) => r.gradient.clone(),
                        Element::Path(p) => p.gradient.clone(),
                        _ => None,
                    };
                    if let Some(g) = grad_opt {
                        self.live_gradient = Some(g);
                    } else if let Some(f0) = el.fills().first() {
                        if f0.style == crate::core::FillStyle::LinearGradient
                            || f0.style == crate::core::FillStyle::RadialGradient
                        {
                            let b = el.bounds();
                            let start = Point::new(b.x, b.y + b.height * 0.5);
                            let end = Point::new(b.x + b.width, b.y + b.height * 0.5);
                            let mut g =
                                Gradient::new_linear(start, end, f0.color, f0.secondary_color);
                            if f0.style == crate::core::FillStyle::RadialGradient {
                                g.kind = GradientType::Radial;
                            }
                            self.live_gradient = Some(g);
                        }
                    }
                    break;
                }
            }
        }

        // Check if clicked near an existing handle
        let mut handle_hit = None;
        if let Some(grad) = &self.live_gradient {
            let hit_radius = 8.0 / ctx.viewport.zoom;
            if event.world_pos.distance_to(grad.start) <= hit_radius {
                handle_hit = Some(DraggingHandle::Start);
            } else if event.world_pos.distance_to(grad.end) <= hit_radius {
                handle_hit = Some(DraggingHandle::End);
            } else {
                for (idx, stop) in grad.stops.iter().enumerate() {
                    let stop_pt = grad.start + (grad.end - grad.start) * stop.offset;
                    if event.world_pos.distance_to(stop_pt) <= hit_radius {
                        handle_hit = Some(DraggingHandle::Stop(idx));
                        break;
                    }
                }
            }
        }

        if let Some(handle) = handle_hit {
            self.drag_handle = Some(handle);
            self.is_dragging = true;

            if let Some(grad) = &self.live_gradient {
                match handle {
                    DraggingHandle::Start => {
                        if let Some(s0) = grad.stops.first() {
                            ctx.active_fill_color = s0.color;
                        }
                    }
                    DraggingHandle::End => {
                        if let Some(s_end) = grad.stops.last() {
                            ctx.active_fill_color = s_end.color;
                        }
                    }
                    DraggingHandle::Stop(idx) => {
                        if let Some(s) = grad.stops.get(idx) {
                            ctx.active_fill_color = s.color;
                        }
                    }
                }
            }
        } else {
            // Start a new gradient drag line
            self.drag_start = Some(event.world_pos);
            self.is_dragging = true;
            self.drag_handle = Some(DraggingHandle::End);

            let primary = ctx.active_fill_color;
            let secondary = ctx.active_stroke_color.unwrap_or(Color::WHITE);
            let kind = if event.shift_pressed {
                GradientType::Radial
            } else {
                GradientType::Linear
            };

            let mut grad = Gradient::new_linear(
                event.world_pos,
                event.world_pos + Point::new(1.0, 1.0),
                primary,
                secondary,
            );
            if kind == GradientType::Radial {
                grad.kind = GradientType::Radial;
            }
            self.live_gradient = Some(grad);
        }

        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if !self.is_dragging {
            ctx.set_cursor("tool:gradient");
            return;
        }

        let _zoom = ctx.viewport.zoom;
        if let Some(grad) = &mut self.live_gradient {
            match self.drag_handle {
                Some(DraggingHandle::Start) => {
                    grad.start = event.world_pos;
                }
                Some(DraggingHandle::End) => {
                    grad.end = event.world_pos;
                }
                Some(DraggingHandle::Stop(idx)) => {
                    if idx < grad.stops.len() {
                        let seg_len = grad.start.distance_to(grad.end).max(1.0);
                        let v = grad.end - grad.start;
                        let p = event.world_pos - grad.start;
                        let proj = (p.x * v.x + p.y * v.y) / (seg_len * seg_len);
                        grad.stops[idx].offset = proj.clamp(0.0, 1.0);
                    }
                }
                None => {}
            }

            // Live update the element
            if let Some(id) = self.target_id {
                for el in &mut ctx.document.elements {
                    if el.id() == id {
                        match el {
                            Element::Rect(r) => r.gradient = Some(grad.clone()),
                            Element::Path(p) => p.gradient = Some(grad.clone()),
                            _ => {}
                        }
                        let mut fills = el.fills();
                        if fills.is_empty() {
                            fills.push(crate::core::FillLayer::default());
                        }
                        if let Some(f0) = fills.first_mut() {
                            f0.style = match grad.kind {
                                GradientType::Radial => crate::core::FillStyle::RadialGradient,
                                _ => crate::core::FillStyle::LinearGradient,
                            };
                            f0.color = grad.stops.first().map(|s| s.color).unwrap_or(Color::BLACK);
                            f0.secondary_color =
                                grad.stops.last().map(|s| s.color).unwrap_or(Color::WHITE);
                        }
                        el.set_fills(fills);
                        break;
                    }
                }
            }
        }

        ctx.request_redraw();
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        if self.is_dragging {
            self.is_dragging = false;
            self.drag_handle = None;

            if let (Some(id), Some(grad)) = (self.target_id, &self.live_gradient) {
                ctx.document.snapshot();
                for el in &mut ctx.document.elements {
                    if el.id() == id {
                        match el {
                            Element::Rect(r) => r.gradient = Some(grad.clone()),
                            Element::Path(p) => p.gradient = Some(grad.clone()),
                            _ => {}
                        }
                        let mut fills = el.fills();
                        if fills.is_empty() {
                            fills.push(crate::core::FillLayer::default());
                        }
                        if let Some(f0) = fills.first_mut() {
                            f0.style = match grad.kind {
                                GradientType::Radial => crate::core::FillStyle::RadialGradient,
                                _ => crate::core::FillStyle::LinearGradient,
                            };
                            f0.color = grad.stops.first().map(|s| s.color).unwrap_or(Color::BLACK);
                            f0.secondary_color =
                                grad.stops.last().map(|s| s.color).unwrap_or(Color::WHITE);
                        }
                        el.set_fills(fills);
                        break;
                    }
                }
            }
        }
        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.is_dragging = false;
        self.drag_handle = None;
        self.live_gradient = None;
        self.target_id = None;
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        _ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        let grad = match &self.live_gradient {
            Some(g) => g,
            None => return,
        };

        let zoom = viewport.zoom;
        let p1 = grad.start;
        let p2 = grad.end;

        // 1. Draw gradient vector axis
        let mut axis_paint = skia::Paint::default();
        axis_paint.set_color4f(skia::Color4f::new(0.1, 0.1, 0.15, 0.9), None);
        axis_paint.set_stroke_width(2.0 / zoom);
        axis_paint.set_style(skia::PaintStyle::Stroke);
        axis_paint.set_anti_alias(true);
        canvas.draw_line(p1.to_skia(), p2.to_skia(), &axis_paint);

        let mut axis_inner = skia::Paint::default();
        axis_inner.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 0.9), None);
        axis_inner.set_stroke_width(1.0 / zoom);
        axis_inner.set_style(skia::PaintStyle::Stroke);
        axis_inner.set_anti_alias(true);
        let intervals = [4.0 / zoom, 4.0 / zoom];
        axis_inner.set_path_effect(skia::dash_path_effect::new(&intervals, 0.0));
        canvas.draw_line(p1.to_skia(), p2.to_skia(), &axis_inner);

        // 2. If radial, draw guide circle
        if grad.kind == GradientType::Radial {
            let radius = p1.distance_to(p2);
            let mut circle_paint = skia::Paint::default();
            circle_paint.set_color4f(skia::Color4f::new(0.2, 0.6, 1.0, 0.5), None);
            circle_paint.set_stroke_width(1.5 / zoom);
            circle_paint.set_style(skia::PaintStyle::Stroke);
            circle_paint.set_anti_alias(true);
            let circle_intervals = [6.0 / zoom, 4.0 / zoom];
            circle_paint.set_path_effect(skia::dash_path_effect::new(&circle_intervals, 0.0));
            canvas.draw_circle(p1.to_skia(), radius, &circle_paint);
        }

        // 3. Draw intermediate color stops
        for stop in &grad.stops {
            let stop_pt = p1 + (p2 - p1) * stop.offset;
            let mut stop_fill = skia::Paint::default();
            stop_fill.set_color4f(stop.color.to_skia(), None);
            stop_fill.set_style(skia::PaintStyle::Fill);
            stop_fill.set_anti_alias(true);
            canvas.draw_circle(stop_pt.to_skia(), 4.5 / zoom, &stop_fill);

            let mut stop_border = skia::Paint::default();
            stop_border.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            stop_border.set_style(skia::PaintStyle::Stroke);
            stop_border.set_stroke_width(1.5 / zoom);
            stop_border.set_anti_alias(true);
            canvas.draw_circle(stop_pt.to_skia(), 4.5 / zoom, &stop_border);
        }

        // 4. Draw Start Handle (Square)
        let s_rect = skia::Rect::from_xywh(
            p1.x - 5.0 / zoom,
            p1.y - 5.0 / zoom,
            10.0 / zoom,
            10.0 / zoom,
        );
        let mut handle_paint = skia::Paint::default();
        if let Some(first) = grad.stops.first() {
            handle_paint.set_color4f(first.color.to_skia(), None);
        } else {
            handle_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        }
        handle_paint.set_style(skia::PaintStyle::Fill);
        handle_paint.set_anti_alias(true);
        canvas.draw_rect(s_rect, &handle_paint);

        let mut handle_border = skia::Paint::default();
        handle_border.set_color4f(skia::Color4f::new(0.1, 0.1, 0.1, 1.0), None);
        handle_border.set_style(skia::PaintStyle::Stroke);
        handle_border.set_stroke_width(1.5 / zoom);
        handle_border.set_anti_alias(true);
        canvas.draw_rect(s_rect, &handle_border);

        // 5. Draw End Handle (Circle)
        let mut end_paint = skia::Paint::default();
        if let Some(last) = grad.stops.last() {
            end_paint.set_color4f(last.color.to_skia(), None);
        } else {
            end_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        }
        end_paint.set_style(skia::PaintStyle::Fill);
        end_paint.set_anti_alias(true);
        canvas.draw_circle(p2.to_skia(), 5.0 / zoom, &end_paint);

        let mut end_border = skia::Paint::default();
        end_border.set_color4f(skia::Color4f::new(0.1, 0.1, 0.1, 1.0), None);
        end_border.set_style(skia::PaintStyle::Stroke);
        end_border.set_stroke_width(1.5 / zoom);
        end_border.set_anti_alias(true);
        canvas.draw_circle(p2.to_skia(), 5.0 / zoom, &end_border);
    }
}
