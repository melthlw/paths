use skia_safe as skia;

use crate::core::{Point, PointerButton, PointerEvent, Rect, Viewport};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

pub struct ZoomFeature {
    is_dragging: bool,
    start_pos: Point,
    current_pos: Point,
}

impl Default for ZoomFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoomFeature {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            start_pos: Point::ZERO,
            current_pos: Point::ZERO,
        }
    }
}

impl FeaturePlugin for ZoomFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("zoom-in");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary)
            && event.button != Some(PointerButton::Secondary)
        {
            return;
        }
        self.is_dragging = true;
        self.start_pos = event.world_pos;
        self.current_pos = event.world_pos;
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if self.is_dragging {
            self.current_pos = event.world_pos;
            ctx.request_redraw();
        } else {
            let cursor = if event.shift_pressed || event.alt_pressed {
                "zoom-out"
            } else {
                "zoom-in"
            };
            ctx.set_cursor(cursor);
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if !self.is_dragging {
            return;
        }
        self.is_dragging = false;

        let dx = (self.current_pos.x - self.start_pos.x).abs();
        let dy = (self.current_pos.y - self.start_pos.y).abs();
        let zoom = ctx.viewport.zoom;

        if dx * zoom > 8.0 && dy * zoom > 8.0 {
            // Dragged a box: zoom directly into the selected box!
            let min_x = self.start_pos.x.min(self.current_pos.x);
            let min_y = self.start_pos.y.min(self.current_pos.y);
            let rect = Rect::new(min_x, min_y, dx, dy);
            ctx.viewport.zoom_to_rect(rect, ctx.widget_size);
        } else {
            // Single click: zoom in or zoom out centered at pointer position
            let screen_focus = event.screen_pos;
            let zoom_factor = if event.shift_pressed
                || event.alt_pressed
                || event.button == Some(PointerButton::Secondary)
            {
                1.0 / 1.5
            } else {
                1.5
            };
            ctx.viewport
                .zoom_at(screen_focus, zoom_factor, ctx.widget_size);
        }

        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.is_dragging = false;
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        _ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        if !self.is_dragging {
            return;
        }

        let zoom = viewport.zoom;
        let min_x = self.start_pos.x.min(self.current_pos.x);
        let min_y = self.start_pos.y.min(self.current_pos.y);
        let width = (self.current_pos.x - self.start_pos.x).abs();
        let height = (self.current_pos.y - self.start_pos.y).abs();
        let rect = skia::Rect::from_xywh(min_x, min_y, width, height);

        // Fill background
        let mut bg_paint = skia::Paint::default();
        bg_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 0.12), None);
        bg_paint.set_style(skia::PaintStyle::Fill);
        bg_paint.set_anti_alias(true);
        canvas.draw_rect(rect, &bg_paint);

        // Dashed border
        let mut stroke_paint = skia::Paint::default();
        stroke_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 0.9), None);
        stroke_paint.set_stroke_width(1.5 / zoom);
        stroke_paint.set_style(skia::PaintStyle::Stroke);
        stroke_paint.set_anti_alias(true);
        let intervals = [5.0 / zoom, 3.0 / zoom];
        stroke_paint.set_path_effect(skia::dash_path_effect::new(&intervals, 0.0));
        canvas.draw_rect(rect, &stroke_paint);

        // Center Magnifier HUD badge if dragged box is large enough
        if width * zoom > 30.0 && height * zoom > 30.0 {
            let scale_x = ctx_scale_factor(ctx_avail_w(viewport), width);
            let scale_y = ctx_scale_factor(ctx_avail_h(viewport), height);
            let target_zoom = scale_x
                .min(scale_y)
                .clamp(Viewport::MIN_ZOOM, Viewport::MAX_ZOOM);
            let zoom_pct = (target_zoom * 100.0).round() as i32;
            let hud_text = format!("{}%", zoom_pct);

            let font = skia::Font::new(crate::core::renderer::get_ui_typeface(), 11.0 / zoom);
            let (text_w, _) = font.measure_str(&hud_text, None);
            let badge_w = text_w + 12.0 / zoom;
            let badge_h = 18.0 / zoom;
            let badge_rect = skia::Rect::from_xywh(
                min_x + width / 2.0 - badge_w / 2.0,
                min_y + height / 2.0 - badge_h / 2.0,
                badge_w,
                badge_h,
            );

            let mut badge_bg = skia::Paint::default();
            badge_bg.set_color4f(skia::Color4f::new(0.1, 0.1, 0.14, 0.88), None);
            badge_bg.set_style(skia::PaintStyle::Fill);
            badge_bg.set_anti_alias(true);
            canvas.draw_round_rect(badge_rect, 4.0 / zoom, 4.0 / zoom, &badge_bg);

            let mut text_paint = skia::Paint::default();
            text_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            text_paint.set_anti_alias(true);
            canvas.draw_str(
                &hud_text,
                skia::Point::new(
                    min_x + width / 2.0 - text_w / 2.0,
                    min_y + height / 2.0 + 4.0 / zoom,
                ),
                &font,
                &text_paint,
            );
        }
    }
}

fn ctx_avail_w(viewport: &Viewport) -> f32 {
    let _ = viewport;
    800.0
}

fn ctx_avail_h(viewport: &Viewport) -> f32 {
    let _ = viewport;
    600.0
}

fn ctx_scale_factor(avail: f32, dimension: f32) -> f32 {
    if dimension <= 0.0 {
        1.0
    } else {
        avail / dimension
    }
}

// -------------------------------------------------------------
// Action Feature: Zoom to Selection
// -------------------------------------------------------------
pub struct ZoomSelectionFeature;
impl FeaturePlugin for ZoomSelectionFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        let target_rect = ctx
            .document
            .selection_bounds()
            .or_else(|| ctx.document.active_page().map(|p| p.rect))
            .unwrap_or_else(|| Rect::new(-400.0, -300.0, 800.0, 600.0));
        ctx.viewport.zoom_to_rect(target_rect, ctx.widget_size);
        ctx.request_redraw();
    }
    fn on_pointer_down(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
    fn on_pointer_move(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
    fn on_pointer_up(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
}

// -------------------------------------------------------------
// Action Feature: Zoom to Fit All
// -------------------------------------------------------------
pub struct ZoomFitAllFeature;
impl FeaturePlugin for ZoomFitAllFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        let mut all_bounds: Option<Rect> = None;
        for el in &ctx.document.elements {
            all_bounds = match all_bounds {
                Some(b) => Some(b.union(el.bounds())),
                None => Some(el.bounds()),
            };
        }
        for page in &ctx.document.pages {
            all_bounds = match all_bounds {
                Some(b) => Some(b.union(page.rect)),
                None => Some(page.rect),
            };
        }
        let target_rect = all_bounds.unwrap_or_else(|| Rect::new(-400.0, -300.0, 800.0, 600.0));
        ctx.viewport.zoom_to_rect(target_rect, ctx.widget_size);
        ctx.request_redraw();
    }
    fn on_pointer_down(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
    fn on_pointer_move(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
    fn on_pointer_up(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
}

// -------------------------------------------------------------
// Action Feature: Zoom 100%
// -------------------------------------------------------------
pub struct Zoom100Feature;
impl FeaturePlugin for Zoom100Feature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.viewport.reset();
        ctx.request_redraw();
    }
    fn on_pointer_down(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
    fn on_pointer_move(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
    fn on_pointer_up(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
}

// -------------------------------------------------------------
// Action Feature: Zoom to Page
// -------------------------------------------------------------
pub struct ZoomPageFeature;
impl FeaturePlugin for ZoomPageFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        let target_rect = ctx
            .document
            .active_page()
            .map(|p| p.rect)
            .unwrap_or_else(|| Rect::new(0.0, 0.0, 794.0, 1123.0));
        ctx.viewport.zoom_to_rect(target_rect, ctx.widget_size);
        ctx.request_redraw();
    }
    fn on_pointer_down(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
    fn on_pointer_move(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
    fn on_pointer_up(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}
}
