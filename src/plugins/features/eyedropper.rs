use skia_safe as skia;

use crate::core::{Color, Point, PointerButton, PointerEvent, Viewport};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

pub struct EyedropperFeature {
    hover_pos: Option<Point>,
    hover_color: Option<Color>,
}

impl Default for EyedropperFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl EyedropperFeature {
    pub fn new() -> Self {
        Self {
            hover_pos: None,
            hover_color: None,
        }
    }

    fn sample_color_at(ctx: &PluginContext, point: Point) -> Color {
        // Hit-test elements from top to bottom
        for elem in ctx.document.elements.iter().rev() {
            if elem.hit_test(point) {
                if let Some(fill) = elem.fill_color() {
                    return fill;
                }
                if let Some(stroke) = elem.stroke_color() {
                    return stroke;
                }
            }
        }
        // Default to white artboard or clear
        Color::WHITE
    }
}

impl FeaturePlugin for EyedropperFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:eyedropper");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        let sampled = Self::sample_color_at(ctx, event.world_pos);

        if event.shift_pressed {
            ctx.active_stroke_color = Some(sampled);
        } else {
            ctx.active_fill_color = sampled;
        }

        // If elements are currently selected in the document, apply to selection
        let selected_ids = ctx.document.selected_ids.clone();
        if !selected_ids.is_empty() {
            ctx.document.snapshot();
            for el in &mut ctx.document.elements {
                if selected_ids.contains(&el.id()) {
                    if event.shift_pressed {
                        el.set_stroke_color(Some(sampled));
                    } else {
                        el.set_fill_color(Some(sampled));
                    }
                }
            }
        }

        self.hover_color = Some(sampled);
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        self.hover_pos = Some(event.world_pos);
        self.hover_color = Some(Self::sample_color_at(ctx, event.world_pos));
        ctx.set_cursor("tool:eyedropper");
        ctx.request_redraw();
    }

    fn on_pointer_up(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.hover_pos = None;
        self.hover_color = None;
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        _ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        let (pos, color) = match (self.hover_pos, self.hover_color) {
            (Some(p), Some(c)) => (p, c),
            _ => return,
        };

        let zoom = viewport.zoom;
        let loupe_offset = Point::new(16.0 / zoom, -16.0 / zoom);
        let center = pos + loupe_offset;
        let radius = 12.0 / zoom;

        // 1. Outer circle shadow
        let mut shadow_paint = skia::Paint::default();
        shadow_paint.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.35), None);
        shadow_paint.set_style(skia::PaintStyle::Fill);
        shadow_paint.set_anti_alias(true);
        canvas.draw_circle(
            skia::Point::new(center.x + 1.5 / zoom, center.y + 1.5 / zoom),
            radius + 1.5 / zoom,
            &shadow_paint,
        );

        // 2. Swatch Fill Circle
        let mut fill_paint = skia::Paint::default();
        fill_paint.set_color4f(color.to_skia(), None);
        fill_paint.set_style(skia::PaintStyle::Fill);
        fill_paint.set_anti_alias(true);
        canvas.draw_circle(center.to_skia(), radius, &fill_paint);

        // 3. Crisp white and dark dual-ring border
        let mut border_paint = skia::Paint::default();
        border_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        border_paint.set_style(skia::PaintStyle::Stroke);
        border_paint.set_stroke_width(2.0 / zoom);
        border_paint.set_anti_alias(true);
        canvas.draw_circle(center.to_skia(), radius, &border_paint);

        let mut inner_border = skia::Paint::default();
        inner_border.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.25), None);
        inner_border.set_style(skia::PaintStyle::Stroke);
        inner_border.set_stroke_width(1.0 / zoom);
        inner_border.set_anti_alias(true);
        canvas.draw_circle(center.to_skia(), radius - 1.0 / zoom, &inner_border);

        // 4. Hex Label Below Swatch
        let hex_text = color.to_hex();
        let font = skia::Font::new(crate::core::renderer::get_ui_typeface(), 10.0 / zoom);
        let (text_w, _) = font.measure_str(&hex_text, None);
        let pill_rect = skia::Rect::from_xywh(
            center.x - text_w / 2.0 - 4.0 / zoom,
            center.y + radius + 3.0 / zoom,
            text_w + 8.0 / zoom,
            14.0 / zoom,
        );

        let mut label_bg = skia::Paint::default();
        label_bg.set_color4f(skia::Color4f::new(0.1, 0.1, 0.12, 0.9), None);
        label_bg.set_style(skia::PaintStyle::Fill);
        label_bg.set_anti_alias(true);
        canvas.draw_round_rect(pill_rect, 3.0 / zoom, 3.0 / zoom, &label_bg);

        let mut label_text = skia::Paint::default();
        label_text.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        label_text.set_anti_alias(true);
        canvas.draw_str(
            &hex_text,
            skia::Point::new(center.x - text_w / 2.0, center.y + radius + 13.0 / zoom),
            &font,
            &label_text,
        );
    }
}
