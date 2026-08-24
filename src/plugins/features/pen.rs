use gtk4::gdk;
use skia_safe as skia;

use crate::core::{
    Element, ElementId, KeyEvent, PathElement, PathNode, Point, PointerButton, PointerEvent,
    Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PenMode {
    #[default]
    Bezier,
    Lines,
}

pub struct PenFeature {
    pub nodes: Vec<PathNode>,
    pub current_cursor: Option<Point>,
    pub is_dragging_handle: bool,
    pub mode: PenMode,

    #[allow(dead_code)]
    pub auto_close: bool,
    pub resuming_path_id: Option<ElementId>,
}

impl Default for PenFeature {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            current_cursor: None,
            is_dragging_handle: false,
            mode: PenMode::Bezier,
            auto_close: true,
            resuming_path_id: None,
        }
    }
}

impl PenFeature {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn undo_last_node(&mut self, ctx: &mut PluginContext) -> bool {
        if !self.nodes.is_empty() {
            self.nodes.pop();
            if self.nodes.is_empty() {
                self.resuming_path_id = None;
                self.current_cursor = None;
                self.is_dragging_handle = false;
            }
            ctx.clear_snap_guides();
            ctx.request_redraw();
            true
        } else {
            false
        }
    }

    pub fn finish_path(&mut self, ctx: &mut PluginContext, is_closed: bool) {
        if self.nodes.len() >= 2 {
            if let Some(resuming_id) = self.resuming_path_id.take() {
                // Update existing open path in document
                let mut found = false;
                for elem in &mut ctx.document.elements {
                    if elem.id() == resuming_id {
                        if let Element::Path(p) = elem {
                            p.nodes = self.nodes.clone();
                            p.is_closed = is_closed;
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    let path = PathElement::new(
                        self.nodes.clone(),
                        is_closed,
                        None,
                        Some(ctx.active_fill_color),
                        ctx.active_stroke_width.max(2.0),
                    );
                    ctx.document.add_element(Element::Path(path));
                }
                ctx.document.select(resuming_id, false);
            } else {
                let path = PathElement::new(
                    self.nodes.clone(),
                    is_closed,
                    None, // Outline by default
                    Some(ctx.active_fill_color),
                    ctx.active_stroke_width.max(2.0),
                );
                let id = path.id;
                ctx.document.add_element(Element::Path(path));
                ctx.document.select(id, false);
            }
        }
        self.nodes.clear();
        self.current_cursor = None;
        self.is_dragging_handle = false;
        self.resuming_path_id = None;
        ctx.clear_snap_guides();
        ctx.request_redraw();
    }

    #[allow(dead_code)]
    pub fn resume_selected_path(&mut self, ctx: &mut PluginContext) -> bool {
        if !self.nodes.is_empty() {
            return false;
        }

        // Find if any selected element is an open PathElement
        for elem in &ctx.document.elements {
            if ctx.document.selected_ids.contains(&elem.id()) {
                if let Element::Path(p) = elem {
                    if !p.is_closed && !p.nodes.is_empty() {
                        self.nodes = p.nodes.clone();
                        self.resuming_path_id = Some(p.id);
                        if let Some(last) = self.nodes.last() {
                            self.current_cursor = Some(last.point);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl FeaturePlugin for PenFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:pen");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        // Right click (Secondary button) finishes active path immediately
        if event.button == Some(PointerButton::Secondary) {
            if !self.nodes.is_empty() {
                self.finish_path(ctx, false);
            }
            return;
        }

        if event.button != Some(PointerButton::Primary) {
            return;
        }

        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude);
        let tolerance = 16.0 / ctx.viewport.zoom;

        // If not currently drawing, check if clicking near start or end of an existing open path
        if self.nodes.is_empty() {
            let mut resumed = false;
            for elem in &ctx.document.elements {
                if let Element::Path(p) = elem {
                    if !p.is_closed && !p.nodes.is_empty() {
                        if p.nodes.len() == 1 {
                            let pt = p.nodes[0].point;
                            if pt.distance_to(snapped) <= tolerance {
                                self.nodes = p.nodes.clone();
                                self.resuming_path_id = Some(p.id);
                                self.is_dragging_handle = self.mode == PenMode::Bezier;
                                ctx.set_cursor("tool:pen");
                                ctx.request_redraw();
                                resumed = true;
                                break;
                            }
                        } else {
                            let first = p.nodes.first().unwrap().point;
                            let last = p.nodes.last().unwrap().point;

                            if first.distance_to(snapped) <= tolerance {
                                // Resume from start: reverse nodes so we append from start point
                                let mut reversed = p.nodes.clone();
                                reversed.reverse();
                                for node in &mut reversed {
                                    std::mem::swap(&mut node.handle_in, &mut node.handle_out);
                                }
                                self.nodes = reversed;
                                self.resuming_path_id = Some(p.id);
                                self.is_dragging_handle = self.mode == PenMode::Bezier;
                                ctx.set_cursor("tool:pen");
                                ctx.request_redraw();
                                resumed = true;
                                break;
                            } else if last.distance_to(snapped) <= tolerance {
                                // Resume from end: continue appending
                                self.nodes = p.nodes.clone();
                                self.resuming_path_id = Some(p.id);
                                self.is_dragging_handle = self.mode == PenMode::Bezier;
                                ctx.set_cursor("tool:pen");
                                ctx.request_redraw();
                                resumed = true;
                                break;
                            }
                        }
                    }
                }
            }
            if resumed {
                return;
            }
        }

        // Check if clicking near start point to close path
        if let Some(first) = self.nodes.first() {
            if self.nodes.len() >= 3 && first.point.distance_to(snapped) <= tolerance {
                self.finish_path(ctx, true);
                return;
            }
        }

        let new_node = PathNode::new(snapped);
        self.nodes.push(new_node);
        self.is_dragging_handle = self.mode == PenMode::Bezier;
        ctx.set_cursor("tool:pen");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude);
        self.current_cursor = Some(snapped);

        let tolerance = 16.0 / ctx.viewport.zoom;
        if let Some(first) = self.nodes.first() {
            if self.nodes.len() >= 3 && first.point.distance_to(snapped) <= tolerance {
                ctx.set_cursor("tool:pen_close");
            } else {
                ctx.set_cursor("tool:pen");
            }
        } else {
            ctx.set_cursor("tool:pen");
        }

        if self.is_dragging_handle && self.mode == PenMode::Bezier {
            if let Some(last) = self.nodes.last_mut() {
                let anchor = last.point;
                let handle_out = snapped;
                let handle_in = Point::new(
                    anchor.x - (handle_out.x - anchor.x),
                    anchor.y - (handle_out.y - anchor.y),
                );
                last.handle_out = Some(handle_out);
                last.handle_in = Some(handle_in);
                ctx.request_redraw();
            }
        } else {
            ctx.request_redraw();
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        self.is_dragging_handle = false;
        ctx.request_redraw();
    }

    fn on_key_down(&mut self, ctx: &mut PluginContext, event: &KeyEvent) -> bool {
        // 1. Ctrl+Z or BackSpace / Delete: Undo last node
        if (event.ctrl_pressed && (event.key == gdk::Key::z || event.key == gdk::Key::Z))
            || event.key == gdk::Key::BackSpace
            || event.key == gdk::Key::Delete
        {
            if !self.nodes.is_empty() {
                return self.undo_last_node(ctx);
            }
        }

        // 2. Return / Enter: Finish open path
        if event.key == gdk::Key::Return || event.key == gdk::Key::KP_Enter {
            if !self.nodes.is_empty() {
                self.finish_path(ctx, false);
                return true;
            }
        }

        // 3. 'C' or 'c': Close & Finish path
        if event.key == gdk::Key::c || event.key == gdk::Key::C {
            if self.nodes.len() >= 3 {
                self.finish_path(ctx, true);
                return true;
            }
        }

        // 4. Escape: Finish or Cancel
        if event.key == gdk::Key::Escape {
            if !self.nodes.is_empty() {
                self.finish_path(ctx, false);
                return true;
            }
        }

        false
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.finish_path(ctx, false);
    }

    fn is_editing(&self) -> bool {
        !self.nodes.is_empty()
    }

    fn as_pen_feature(&self) -> Option<&PenFeature> {
        Some(self)
    }

    fn as_pen_feature_mut(&mut self) -> Option<&mut PenFeature> {
        Some(self)
    }

    fn render_overlay(
        &self,
        ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        let zoom = viewport.zoom.max(0.001);
        let node_size = 9.5 / zoom;
        let half = node_size / 2.0;
        let corner_radius = 2.0 / zoom;
        let handle_radius = 4.5 / zoom;
        let stroke_w = 1.5 / zoom;
        let shadow_offset = Point::new(0.8 / zoom, 1.2 / zoom);

        // Paints matching path_editor
        let mut shadow_paint = skia::Paint::default();
        shadow_paint.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.35), None);
        shadow_paint.set_style(skia::PaintStyle::Fill);
        shadow_paint.set_anti_alias(true);

        let mut path_outline_paint = skia::Paint::default();
        path_outline_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.90), None);
        path_outline_paint.set_style(skia::PaintStyle::Stroke);
        path_outline_paint.set_stroke_width(1.5 / zoom);
        path_outline_paint.set_anti_alias(true);

        let mut node_fill = skia::Paint::default();
        node_fill.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        node_fill.set_style(skia::PaintStyle::Fill);
        node_fill.set_anti_alias(true);

        let mut node_selected_fill = skia::Paint::default();
        node_selected_fill.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 1.0), None);
        node_selected_fill.set_style(skia::PaintStyle::Fill);
        node_selected_fill.set_anti_alias(true);

        let mut node_selected_stroke = skia::Paint::default();
        node_selected_stroke.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        node_selected_stroke.set_style(skia::PaintStyle::Stroke);
        node_selected_stroke.set_stroke_width(1.6 / zoom);
        node_selected_stroke.set_anti_alias(true);

        let mut node_stroke = skia::Paint::default();
        node_stroke.set_color4f(skia::Color4f::new(0.15, 0.45, 0.85, 1.0), None);
        node_stroke.set_style(skia::PaintStyle::Stroke);
        node_stroke.set_stroke_width(stroke_w);
        node_stroke.set_anti_alias(true);

        let mut start_node_badge = skia::Paint::default();
        start_node_badge.set_color4f(skia::Color4f::new(0.18, 0.80, 0.44, 0.95), None);
        start_node_badge.set_style(skia::PaintStyle::Fill);
        start_node_badge.set_anti_alias(true);

        let mut hover_ring = skia::Paint::default();
        hover_ring.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.40), None);
        hover_ring.set_style(skia::PaintStyle::Stroke);
        hover_ring.set_stroke_width(3.0 / zoom);
        hover_ring.set_anti_alias(true);

        let mut selected_glow = skia::Paint::default();
        selected_glow.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.28), None);
        selected_glow.set_style(skia::PaintStyle::Stroke);
        selected_glow.set_stroke_width(3.5 / zoom);
        selected_glow.set_anti_alias(true);

        let mut handle_line_halo = skia::Paint::default();
        handle_line_halo.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.40), None);
        handle_line_halo.set_style(skia::PaintStyle::Stroke);
        handle_line_halo.set_stroke_width(2.5 / zoom);
        handle_line_halo.set_anti_alias(true);

        let mut handle_line = skia::Paint::default();
        handle_line.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.95), None);
        handle_line.set_style(skia::PaintStyle::Stroke);
        handle_line.set_stroke_width(1.2 / zoom);
        handle_line.set_anti_alias(true);

        let mut handle_circle_fill = skia::Paint::default();
        handle_circle_fill.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        handle_circle_fill.set_style(skia::PaintStyle::Fill);
        handle_circle_fill.set_anti_alias(true);

        let mut handle_circle_stroke = skia::Paint::default();
        handle_circle_stroke.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 1.0), None);
        handle_circle_stroke.set_style(skia::PaintStyle::Stroke);
        handle_circle_stroke.set_stroke_width(1.6 / zoom);
        handle_circle_stroke.set_anti_alias(true);

        if !self.nodes.is_empty() {
            // 1. Draw temporary path connecting placed nodes + active cursor rubberband
            let mut temp_nodes = self.nodes.clone();
            if let Some(cur) = self.current_cursor {
                if !self.is_dragging_handle {
                    temp_nodes.push(PathNode::new(cur));
                }
            }

            let temp_path = PathElement::new(
                temp_nodes,
                false,
                None,
                Some(ctx.active_fill_color),
                ctx.active_stroke_width.max(1.5),
            );

            canvas.draw_path(&temp_path.to_skia_path(), &path_outline_paint);

            let node_count = self.nodes.len();
            let is_hovering_close =
                if let (Some(first), Some(cur)) = (self.nodes.first(), self.current_cursor) {
                    node_count >= 3 && first.point.distance_to(cur) <= (16.0 / zoom)
                } else {
                    false
                };

            // 2. Draw anchor points & Bézier handles
            for (i, node) in self.nodes.iter().enumerate() {
                let pt = node.point;
                let is_last = i == node_count - 1;
                let is_start = i == 0;
                let has_handles = node.handle_in.is_some() || node.handle_out.is_some();

                // Bézier handles
                if let Some(h_out) = node.handle_out {
                    canvas.draw_line(pt.to_skia(), h_out.to_skia(), &handle_line_halo);
                    canvas.draw_line(pt.to_skia(), h_out.to_skia(), &handle_line);

                    // Drop shadow under handle grip
                    canvas.draw_circle(
                        Point::new(h_out.x + shadow_offset.x, h_out.y + shadow_offset.y).to_skia(),
                        handle_radius,
                        &shadow_paint,
                    );
                    canvas.draw_circle(h_out.to_skia(), handle_radius, &handle_circle_fill);
                    canvas.draw_circle(h_out.to_skia(), handle_radius, &handle_circle_stroke);

                    if is_last && self.is_dragging_handle {
                        canvas.draw_circle(h_out.to_skia(), handle_radius * 1.7, &hover_ring);
                    }
                }

                if let Some(h_in) = node.handle_in {
                    canvas.draw_line(pt.to_skia(), h_in.to_skia(), &handle_line_halo);
                    canvas.draw_line(pt.to_skia(), h_in.to_skia(), &handle_line);

                    // Drop shadow under handle grip
                    canvas.draw_circle(
                        Point::new(h_in.x + shadow_offset.x, h_in.y + shadow_offset.y).to_skia(),
                        handle_radius,
                        &shadow_paint,
                    );
                    canvas.draw_circle(h_in.to_skia(), handle_radius, &handle_circle_fill);
                    canvas.draw_circle(h_in.to_skia(), handle_radius, &handle_circle_stroke);
                }

                // Selection glow halo on active/last placed node
                if is_last {
                    canvas.draw_circle(pt.to_skia(), half * 1.75, &selected_glow);
                }

                // Hover ring when closing loop on first node
                if is_start && is_hovering_close {
                    canvas.draw_circle(pt.to_skia(), half * 2.0, &hover_ring);
                }

                // Anchor Node Body
                let rect = skia::Rect::from_xywh(pt.x - half, pt.y - half, node_size, node_size);
                let shadow_rect = skia::Rect::from_xywh(
                    pt.x - half + shadow_offset.x,
                    pt.y - half + shadow_offset.y,
                    node_size,
                    node_size,
                );

                if has_handles {
                    // Smooth / Curve Node: Circular Anchor
                    canvas.draw_circle(
                        Point::new(pt.x + shadow_offset.x, pt.y + shadow_offset.y).to_skia(),
                        half * 1.05,
                        &shadow_paint,
                    );

                    if is_last {
                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_selected_fill);
                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_selected_stroke);
                    } else {
                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_fill);
                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_stroke);
                    }
                } else {
                    // Corner Node: Sleek Rounded Rectangle
                    canvas.draw_round_rect(
                        shadow_rect,
                        corner_radius,
                        corner_radius,
                        &shadow_paint,
                    );

                    if is_last {
                        canvas.draw_round_rect(
                            rect,
                            corner_radius,
                            corner_radius,
                            &node_selected_fill,
                        );
                        canvas.draw_round_rect(
                            rect,
                            corner_radius,
                            corner_radius,
                            &node_selected_stroke,
                        );
                    } else {
                        canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_fill);
                        canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_stroke);
                    }
                }

                // Start node emerald green indicator core
                if is_start {
                    canvas.draw_circle(pt.to_skia(), half * 0.45, &start_node_badge);
                }
            }
        }
    }
}
