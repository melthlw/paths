use skia_safe as skia;

use crate::core::{
    Color, Element, ElementId, FillLayer, FillStyle, PatternType, Point, PointerButton,
    PointerEvent, Rect, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternDragHandle {
    Offset,
    Scale,
    Rotate,
}

pub struct PatternFeature {
    target_id: Option<ElementId>,
    is_dragging: bool,
    drag_handle: Option<PatternDragHandle>,
    drag_start: Option<Point>,
    initial_offset: Point,
    initial_scale: f32,
    initial_angle: f32,
}

impl Default for PatternFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternFeature {
    pub fn new() -> Self {
        Self {
            target_id: None,
            is_dragging: false,
            drag_handle: None,
            drag_start: None,
            initial_offset: Point::ZERO,
            initial_scale: 16.0,
            initial_angle: 0.0,
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

    fn get_target_pattern_info<'a>(
        elem: &'a Element,
    ) -> Option<(Rect, &'a FillLayer)> {
        let bounds = elem.bounds();
        let fills = match elem {
            Element::Rect(r) => &r.fills,
            Element::Path(p) => &p.fills,
            _ => return None,
        };

        let fill = fills.iter().find(|f| f.style == FillStyle::Pattern && f.enabled)
            .or_else(|| fills.iter().find(|f| f.style == FillStyle::Pattern))?;

        Some((bounds, fill))
    }

    fn calculate_gizmo_positions(
        bounds: Rect,
        fill: &FillLayer,
    ) -> (Point, Point, Point, Point, Point, f32, f32) {
        let origin = Point::new(bounds.x + fill.pattern_offset.x, bounds.y + fill.pattern_offset.y);
        let sz = fill.pattern_scale.clamp(4.0, 2048.0);
        let (tile_w, tile_h) = match fill.pattern_type {
            PatternType::Hexagon => (sz, (sz * 1.7320508).max(4.0)),
            PatternType::Brick | PatternType::Scales => (sz, (sz * 0.5).max(4.0)),
            _ => (sz, sz),
        };

        let rad = fill.angle.to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        // 4 corners of tile rectangle rotated around origin
        let _p_top_left = origin;
        let p_top_right = Point::new(origin.x + tile_w * cos_a, origin.y + tile_w * sin_a);
        let p_bot_right = Point::new(
            origin.x + tile_w * cos_a - tile_h * sin_a,
            origin.y + tile_w * sin_a + tile_h * cos_a,
        );
        let p_bot_left = Point::new(origin.x - tile_h * sin_a, origin.y + tile_h * cos_a);

        // Scale handle: at top-right edge (p_top_right)
        let p_scale = p_top_right;

        // Rotate handle: along bottom-left edge (p_bot_left)
        let p_rotate = p_bot_left;

        (origin, p_scale, p_rotate, p_bot_right, p_bot_left, tile_w, tile_h)
    }
}

impl FeaturePlugin for PatternFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:pattern");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        let target_id = match self.find_or_init_target(ctx, event.world_pos) {
            Some(id) => id,
            None => {
                self.target_id = None;
                ctx.request_redraw();
                return;
            }
        };

        self.target_id = Some(target_id);

        let (bounds, existing_fill) = {
            let elem = match ctx.document.elements.iter().find(|e| e.id() == target_id) {
                Some(e) => e,
                None => return,
            };
            (elem.bounds(), Self::get_target_pattern_info(elem).map(|(_, f)| f.clone()))
        };

        let fill = match existing_fill {
            Some(f) => f,
            None => {
                // Initialize default Pattern Fill on target element!
                ctx.document.snapshot();
                let mut new_fill = FillLayer::default();
                new_fill.style = FillStyle::Pattern;
                new_fill.pattern_type = PatternType::Checkerboard;
                new_fill.pattern_scale = 24.0;
                new_fill.color = ctx.active_fill_color;
                new_fill.secondary_color = ctx.active_stroke_color.unwrap_or(Color::WHITE);

                for el in &mut ctx.document.elements {
                    if el.id() == target_id {
                        match el {
                            Element::Rect(r) => {
                                r.fills.push(new_fill.clone());
                            }
                            Element::Path(p) => {
                                p.fills.push(new_fill.clone());
                            }
                            _ => {}
                        }
                        break;
                    }
                }
                new_fill
            }
        };

        let zoom = ctx.viewport.zoom;
        let hit_radius = 12.0 / zoom;

        let (p_origin, p_scale, p_rotate, _, _, _, _) =
            Self::calculate_gizmo_positions(bounds, &fill);

        // Check handle hits:
        let handle_hit = if event.world_pos.distance_to(p_origin) <= hit_radius {
            Some(PatternDragHandle::Offset)
        } else if event.world_pos.distance_to(p_scale) <= hit_radius {
            Some(PatternDragHandle::Scale)
        } else if event.world_pos.distance_to(p_rotate) <= hit_radius {
            Some(PatternDragHandle::Rotate)
        } else {
            // Check if clicking inside pattern origin or element body to move offset
            Some(PatternDragHandle::Offset)
        };

        self.drag_handle = handle_hit;
        self.drag_start = Some(event.world_pos);
        self.initial_offset = fill.pattern_offset;
        self.initial_scale = fill.pattern_scale;
        self.initial_angle = fill.angle;
        self.is_dragging = true;

        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if !self.is_dragging {
            ctx.set_cursor("tool:pattern");
            return;
        }

        let start = match self.drag_start {
            Some(s) => s,
            None => return,
        };

        let target_id = match self.target_id {
            Some(id) => id,
            None => return,
        };

        let elem = match ctx.document.elements.iter().find(|e| e.id() == target_id) {
            Some(e) => e,
            None => return,
        };

        let bounds = elem.bounds();
        let origin_base = Point::new(bounds.x + self.initial_offset.x, bounds.y + self.initial_offset.y);

        match self.drag_handle {
            Some(PatternDragHandle::Offset) => {
                let dx = event.world_pos.x - start.x;
                let dy = event.world_pos.y - start.y;
                let new_offset = Point::new(self.initial_offset.x + dx, self.initial_offset.y + dy);

                for el in &mut ctx.document.elements {
                    if el.id() == target_id {
                        match el {
                            Element::Rect(r) => {
                                for f in &mut r.fills {
                                    if f.style == FillStyle::Pattern {
                                        f.pattern_offset = new_offset;
                                    }
                                }
                            }
                            Element::Path(p) => {
                                for f in &mut p.fills {
                                    if f.style == FillStyle::Pattern {
                                        f.pattern_offset = new_offset;
                                    }
                                }
                            }
                            _ => {}
                        }
                        break;
                    }
                }
            }
            Some(PatternDragHandle::Scale) => {
                let dist = origin_base.distance_to(event.world_pos);
                let new_scale = dist.clamp(4.0, 2048.0);

                for el in &mut ctx.document.elements {
                    if el.id() == target_id {
                        match el {
                            Element::Rect(r) => {
                                for f in &mut r.fills {
                                    if f.style == FillStyle::Pattern {
                                        f.pattern_scale = new_scale;
                                    }
                                }
                            }
                            Element::Path(p) => {
                                for f in &mut p.fills {
                                    if f.style == FillStyle::Pattern {
                                        f.pattern_scale = new_scale;
                                    }
                                }
                            }
                            _ => {}
                        }
                        break;
                    }
                }
            }
            Some(PatternDragHandle::Rotate) => {
                let dx = event.world_pos.x - origin_base.x;
                let dy = event.world_pos.y - origin_base.y;
                let mut angle_deg = dy.atan2(dx).to_degrees();
                // Because rotate handle is on the Y axis (p_bot_left), offset by -90°
                angle_deg -= 90.0;
                if angle_deg < 0.0 {
                    angle_deg += 360.0;
                }
                if angle_deg >= 360.0 {
                    angle_deg -= 360.0;
                }

                if event.shift_pressed {
                    angle_deg = (angle_deg / 15.0).round() * 15.0;
                }

                for el in &mut ctx.document.elements {
                    if el.id() == target_id {
                        match el {
                            Element::Rect(r) => {
                                for f in &mut r.fills {
                                    if f.style == FillStyle::Pattern {
                                        f.angle = angle_deg;
                                    }
                                }
                            }
                            Element::Path(p) => {
                                for f in &mut p.fills {
                                    if f.style == FillStyle::Pattern {
                                        f.angle = angle_deg;
                                    }
                                }
                            }
                            _ => {}
                        }
                        break;
                    }
                }
            }
            None => {}
        }

        ctx.request_redraw();
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        if self.is_dragging {
            self.is_dragging = false;
            self.drag_handle = None;
            ctx.document.snapshot();
            ctx.request_redraw();
        }
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.is_dragging = false;
        self.drag_handle = None;
        self.target_id = None;
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        let target_id = match self.target_id {
            Some(id) => id,
            None => return,
        };

        let elem = match ctx.document.elements.iter().find(|e| e.id() == target_id) {
            Some(e) => e,
            None => return,
        };

        let (bounds, fill) = match Self::get_target_pattern_info(elem) {
            Some((b, f)) => (b, f),
            None => return,
        };

        let zoom = viewport.zoom;
        let (p_origin, p_scale, p_rotate, p_bot_right, p_bot_left, _, _) =
            Self::calculate_gizmo_positions(bounds, fill);

        // 1. Draw Tile Bounding Box (Outer dark + inner dashed white)
        let mut outer_paint = skia::Paint::default();
        outer_paint.set_color4f(skia::Color4f::new(0.1, 0.12, 0.16, 0.8), None);
        outer_paint.set_stroke_width(2.0 / zoom);
        outer_paint.set_style(skia::PaintStyle::Stroke);
        outer_paint.set_anti_alias(true);

        let mut box_path = skia::PathBuilder::new();
        box_path.move_to(p_origin.to_skia());
        box_path.line_to(p_scale.to_skia());
        box_path.line_to(p_bot_right.to_skia());
        box_path.line_to(p_bot_left.to_skia());
        box_path.close();
        let box_path = box_path.detach();
        canvas.draw_path(&box_path, &outer_paint);

        let mut inner_paint = skia::Paint::default();
        inner_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 0.95), None);
        inner_paint.set_stroke_width(1.2 / zoom);
        inner_paint.set_style(skia::PaintStyle::Stroke);
        inner_paint.set_anti_alias(true);
        let intervals = [5.0 / zoom, 3.0 / zoom];
        inner_paint.set_path_effect(skia::dash_path_effect::new(&intervals, 0.0));
        canvas.draw_path(&box_path, &inner_paint);

        // 2. Draw Vector Axis Lines (Origin -> Scale, Origin -> Rotate)
        let mut axis_x_paint = skia::Paint::default();
        axis_x_paint.set_color4f(skia::Color4f::new(0.2, 0.6, 1.0, 0.85), None);
        axis_x_paint.set_stroke_width(1.8 / zoom);
        axis_x_paint.set_style(skia::PaintStyle::Stroke);
        axis_x_paint.set_anti_alias(true);
        canvas.draw_line(p_origin.to_skia(), p_scale.to_skia(), &axis_x_paint);

        let mut axis_y_paint = skia::Paint::default();
        axis_y_paint.set_color4f(skia::Color4f::new(0.9, 0.3, 0.3, 0.85), None);
        axis_y_paint.set_stroke_width(1.8 / zoom);
        axis_y_paint.set_style(skia::PaintStyle::Stroke);
        axis_y_paint.set_anti_alias(true);
        canvas.draw_line(p_origin.to_skia(), p_rotate.to_skia(), &axis_y_paint);

        // 3. Draw Origin Handle (Diamond: Offset)
        let handle_sz = 6.5 / zoom;
        let mut origin_fill = skia::Paint::default();
        origin_fill.set_color4f(skia::Color4f::new(0.2, 0.6, 1.0, 1.0), None);
        origin_fill.set_style(skia::PaintStyle::Fill);
        origin_fill.set_anti_alias(true);

        let mut diamond = skia::PathBuilder::new();
        diamond.move_to(skia::Point::new(p_origin.x, p_origin.y - handle_sz * 1.3));
        diamond.line_to(skia::Point::new(p_origin.x + handle_sz * 1.3, p_origin.y));
        diamond.line_to(skia::Point::new(p_origin.x, p_origin.y + handle_sz * 1.3));
        diamond.line_to(skia::Point::new(p_origin.x - handle_sz * 1.3, p_origin.y));
        diamond.close();
        let diamond_path = diamond.detach();
        canvas.draw_path(&diamond_path, &origin_fill);

        let mut handle_border = skia::Paint::default();
        handle_border.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        handle_border.set_stroke_width(1.8 / zoom);
        handle_border.set_style(skia::PaintStyle::Stroke);
        handle_border.set_anti_alias(true);
        canvas.draw_path(&diamond_path, &handle_border);

        // 4. Draw Scale Handle (Square: Scale)
        let s_rect = skia::Rect::from_xywh(
            p_scale.x - handle_sz,
            p_scale.y - handle_sz,
            handle_sz * 2.0,
            handle_sz * 2.0,
        );
        let mut scale_fill = skia::Paint::default();
        scale_fill.set_color4f(skia::Color4f::new(0.18, 0.8, 0.44, 1.0), None);
        scale_fill.set_style(skia::PaintStyle::Fill);
        scale_fill.set_anti_alias(true);
        canvas.draw_rect(s_rect, &scale_fill);
        canvas.draw_rect(s_rect, &handle_border);

        // 5. Draw Rotation Handle (Circle: Rotation)
        let mut rot_fill = skia::Paint::default();
        rot_fill.set_color4f(skia::Color4f::new(1.0, 0.55, 0.0, 1.0), None);
        rot_fill.set_style(skia::PaintStyle::Fill);
        rot_fill.set_anti_alias(true);
        canvas.draw_circle(p_rotate.to_skia(), handle_sz * 1.1, &rot_fill);
        canvas.draw_circle(p_rotate.to_skia(), handle_sz * 1.1, &handle_border);

        // 6. Draw HUD Badge Readout (Offset, Scale, Angle)
        let badge_x = p_origin.x;
        let badge_y = p_origin.y - 20.0 / zoom;
        let badge_text = format!(
            "Scale: {:.1}px  •  Angle: {:.0}°  •  Offset: ({:.0}, {:.0})",
            fill.pattern_scale, fill.angle, fill.pattern_offset.x, fill.pattern_offset.y
        );

        let font = skia::Font::default();
        let text_w = font.measure_text(&badge_text, None).0 / zoom * 0.9;
        let badge_rect = skia::Rect::from_xywh(
            badge_x - 6.0 / zoom,
            badge_y - 12.0 / zoom,
            text_w + 12.0 / zoom,
            16.0 / zoom,
        );

        let mut badge_bg = skia::Paint::default();
        badge_bg.set_color4f(skia::Color4f::new(0.1, 0.12, 0.16, 0.85), None);
        badge_bg.set_style(skia::PaintStyle::Fill);
        badge_bg.set_anti_alias(true);
        canvas.draw_round_rect(badge_rect, 4.0 / zoom, 4.0 / zoom, &badge_bg);

        let mut text_paint = skia::Paint::default();
        text_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 0.95), None);
        text_paint.set_anti_alias(true);

        canvas.save();
        canvas.scale((1.0 / zoom * 0.85, 1.0 / zoom * 0.85));
        let scaled_x = badge_x * zoom / 0.85;
        let scaled_y = badge_y * zoom / 0.85;
        canvas.draw_str(&badge_text, (scaled_x, scaled_y), &font, &text_paint);
        canvas.restore();
    }
}
