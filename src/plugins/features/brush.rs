use skia_safe as skia;

use crate::core::{BrushStroke, Element, Point, PointerButton, PointerEvent, Viewport};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

pub struct BrushFeature {
    current_points: Vec<Point>,
}

impl Default for BrushFeature {
    fn default() -> Self {
        Self {
            current_points: Vec::new(),
        }
    }
}

impl BrushFeature {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FeaturePlugin for BrushFeature {
    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }
        self.current_points.clear();
        self.current_points.push(event.world_pos);
        ctx.set_cursor("crosshair");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if !self.current_points.is_empty() {
            let last = *self.current_points.last().unwrap();
            if last.distance_to(event.world_pos) > 1.5 {
                self.current_points.push(event.world_pos);
                ctx.request_redraw();
            }
        } else {
            ctx.set_cursor("crosshair");
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if !self.current_points.is_empty() {
            self.current_points.push(event.world_pos);

            if self.current_points.len() >= 2 {
                let stroke = BrushStroke::new(
                    self.current_points.clone(),
                    ctx.active_fill_color,
                    (ctx.active_stroke_width * 2.0).max(2.0),
                );
                ctx.document.add_element(Element::Brush(stroke));
            }

            self.current_points.clear();
            ctx.set_cursor("crosshair");
            ctx.request_redraw();
        }
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.current_points.clear();
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
            let stroke = BrushStroke::new(
                self.current_points.clone(),
                ctx.active_fill_color,
                (ctx.active_stroke_width * 2.0).max(2.0),
            );
            stroke.render(canvas);
        }
    }
}
