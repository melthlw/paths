use skia_safe as skia;

use crate::core::{Point, PointerButton, PointerEvent, Viewport};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, PartialEq)]
enum MeasureState {
    Idle,
    Measuring {
        start_pos: Point,
        current_pos: Point,
    },
    Measured {
        start_pos: Point,
        end_pos: Point,
    },
}

pub struct MeasureFeature {
    state: MeasureState,
}

impl Default for MeasureFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl MeasureFeature {
    pub fn new() -> Self {
        Self {
            state: MeasureState::Idle,
        }
    }
}

impl FeaturePlugin for MeasureFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("crosshair");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }
        let empty_exclude = std::collections::HashSet::new();
        let start = ctx.snap_point(event.world_pos, &empty_exclude).round();
        self.state = MeasureState::Measuring {
            start_pos: start,
            current_pos: start,
        };
        ctx.set_cursor("crosshair");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match &mut self.state {
            MeasureState::Measuring { current_pos, .. } => {
                let empty_exclude = std::collections::HashSet::new();
                let snapped = ctx.snap_point(event.world_pos, &empty_exclude).round();
                *current_pos = snapped;
                ctx.set_cursor("crosshair");
                ctx.request_redraw();
            }
            _ => {
                ctx.set_cursor("crosshair");
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        match self.state {
            MeasureState::Measuring {
                start_pos,
                current_pos,
            } => {
                if start_pos.distance_to(current_pos) > 1.0 {
                    self.state = MeasureState::Measured {
                        start_pos,
                        end_pos: current_pos,
                    };
                } else {
                    self.state = MeasureState::Idle;
                }
            }
            _ => {}
        }
        ctx.clear_snap_guides();
        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.state = MeasureState::Idle;
        ctx.clear_snap_guides();
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        _ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        let (p1, p2) = match self.state {
            MeasureState::Measuring {
                start_pos,
                current_pos,
            } => (start_pos, current_pos),
            MeasureState::Measured { start_pos, end_pos } => (start_pos, end_pos),
            MeasureState::Idle => return,
        };

        let zoom = viewport.zoom;
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let angle_rad = dy.atan2(dx);
        let mut angle_deg = angle_rad.to_degrees();
        if angle_deg < 0.0 {
            angle_deg += 360.0;
        }

        // 1. Draw dashed measurement line
        let mut line_paint = skia::Paint::default();
        line_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 0.9), None);
        line_paint.set_stroke_width(1.5 / zoom);
        line_paint.set_style(skia::PaintStyle::Stroke);
        line_paint.set_anti_alias(true);
        let intervals = [6.0 / zoom, 4.0 / zoom];
        line_paint.set_path_effect(skia::dash_path_effect::new(&intervals, 0.0));
        canvas.draw_line(p1.to_skia(), p2.to_skia(), &line_paint);

        // 2. Draw perpendicular ticks at ends
        let normal = Point::new(-dy / dist.max(0.001), dx / dist.max(0.001));
        let tick_len = 8.0 / zoom;

        let mut tick_paint = skia::Paint::default();
        tick_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 1.0), None);
        tick_paint.set_stroke_width(2.0 / zoom);
        tick_paint.set_style(skia::PaintStyle::Stroke);
        tick_paint.set_anti_alias(true);

        canvas.draw_line(
            (p1 - normal * tick_len).to_skia(),
            (p1 + normal * tick_len).to_skia(),
            &tick_paint,
        );
        canvas.draw_line(
            (p2 - normal * tick_len).to_skia(),
            (p2 + normal * tick_len).to_skia(),
            &tick_paint,
        );

        // 3. Draw Endpoint Dots
        let mut dot_paint = skia::Paint::default();
        dot_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        dot_paint.set_style(skia::PaintStyle::Fill);
        dot_paint.set_anti_alias(true);
        canvas.draw_circle(p1.to_skia(), 3.5 / zoom, &dot_paint);
        canvas.draw_circle(p2.to_skia(), 3.5 / zoom, &dot_paint);
        dot_paint.set_style(skia::PaintStyle::Stroke);
        dot_paint.set_stroke_width(1.5 / zoom);
        dot_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 1.0), None);
        canvas.draw_circle(p1.to_skia(), 3.5 / zoom, &dot_paint);
        canvas.draw_circle(p2.to_skia(), 3.5 / zoom, &dot_paint);

        // 4. Draw Floating Pill HUD with measurement info
        let mid = Point::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
        let offset_hud = normal * (18.0 / zoom);
        let hud_center = mid + offset_hud;

        let info_text = format!(
            "L: {:.1} px | ∠ {:.1}° | ΔX: {:.1} | ΔY: {:.1}",
            dist,
            angle_deg,
            dx.abs(),
            dy.abs()
        );

        let font = skia::Font::new(crate::core::renderer::get_ui_typeface(), 11.0 / zoom);
        let (text_w, _) = font.measure_str(&info_text, None);
        let padding_h = 8.0 / zoom;
        let hud_w = text_w + padding_h * 2.0;
        let hud_h = 18.0 / zoom;

        let hud_rect = skia::Rect::from_xywh(
            hud_center.x - hud_w / 2.0,
            hud_center.y - hud_h / 2.0,
            hud_w,
            hud_h,
        );

        // Pill background
        let is_dark = libadwaita::StyleManager::default().is_dark();
        let mut bg_paint = skia::Paint::default();
        if is_dark {
            bg_paint.set_color4f(skia::Color4f::new(0.12, 0.12, 0.14, 0.92), None);
        } else {
            bg_paint.set_color4f(skia::Color4f::new(0.96, 0.96, 0.98, 0.92), None);
        }
        bg_paint.set_style(skia::PaintStyle::Fill);
        bg_paint.set_anti_alias(true);
        canvas.draw_round_rect(hud_rect, 4.0 / zoom, 4.0 / zoom, &bg_paint);

        let mut border_paint = skia::Paint::default();
        border_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 0.6), None);
        border_paint.set_style(skia::PaintStyle::Stroke);
        border_paint.set_stroke_width(1.0 / zoom);
        border_paint.set_anti_alias(true);
        canvas.draw_round_rect(hud_rect, 4.0 / zoom, 4.0 / zoom, &border_paint);

        // Text
        let mut text_paint = skia::Paint::default();
        if is_dark {
            text_paint.set_color4f(skia::Color4f::new(0.95, 0.95, 0.98, 1.0), None);
        } else {
            text_paint.set_color4f(skia::Color4f::new(0.1, 0.1, 0.12, 1.0), None);
        }
        text_paint.set_anti_alias(true);
        let text_x = hud_center.x - text_w / 2.0;
        let text_y = hud_center.y + 4.0 / zoom;
        canvas.draw_str(
            &info_text,
            skia::Point::new(text_x, text_y),
            &font,
            &text_paint,
        );
    }
}
