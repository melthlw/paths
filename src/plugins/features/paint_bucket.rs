use skia_safe as skia;

use crate::core::{ElementId, PointerButton, PointerEvent, Viewport};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

pub struct PaintBucketFeature {
    hover_id: Option<ElementId>,
}

impl Default for PaintBucketFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl PaintBucketFeature {
    pub fn new() -> Self {
        Self { hover_id: None }
    }
}

impl FeaturePlugin for PaintBucketFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:eyedropper");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        // Find clicked element (from topmost to bottom)
        let mut target_id = None;
        for elem in ctx.document.elements.iter().rev() {
            if elem.hit_test(event.world_pos) {
                target_id = Some(elem.id());
                break;
            }
        }

        if let Some(id) = target_id {
            ctx.document.snapshot();
            for el in &mut ctx.document.elements {
                if el.id() == id {
                    if event.shift_pressed {
                        let stroke_c = ctx.active_stroke_color.unwrap_or(ctx.active_fill_color);
                        el.set_stroke_color(Some(stroke_c));
                    } else {
                        el.set_fill_color(Some(ctx.active_fill_color));
                    }
                    break;
                }
            }
            ctx.document.select(id, false);
            ctx.request_redraw();
        }
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        let mut hit = None;
        for elem in ctx.document.elements.iter().rev() {
            if elem.hit_test(event.world_pos) {
                hit = Some(elem.id());
                break;
            }
        }

        if self.hover_id != hit {
            self.hover_id = hit;
            ctx.request_redraw();
        }
        ctx.set_cursor("crosshair");
    }

    fn on_pointer_up(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.hover_id = None;
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        _ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        let id = match self.hover_id {
            Some(i) => i,
            None => return,
        };

        // Highlight element with dashed cyan/blue stroke outline
        let doc = &_ctx.document;
        if let Some(elem) = doc.elements.iter().find(|e| e.id() == id) {
            let sk_path = elem.to_skia_path();
            let zoom = viewport.zoom;

            let mut paint = skia::Paint::default();
            paint.set_color4f(skia::Color4f::new(0.2, 0.65, 1.0, 0.8), None);
            paint.set_stroke_width(2.0 / zoom);
            paint.set_style(skia::PaintStyle::Stroke);
            paint.set_anti_alias(true);
            let intervals = [4.0 / zoom, 3.0 / zoom];
            paint.set_path_effect(skia::dash_path_effect::new(&intervals, 0.0));
            canvas.draw_path(&sk_path, &paint);
        }
    }
}
