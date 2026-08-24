use gtk4::gdk;
use skia_safe as skia;

use crate::core::{
    Element, ElementId, KeyEvent, PathElement, PathNode, Point, PointerButton, PointerEvent, Viewport,
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
    fn on_activate(&mut self, _ctx: &mut PluginContext) {
        // Do NOT automatically resume drawing on tool activation.
        // Drawing should only start or resume when the user clicks on canvas.
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
                                ctx.set_cursor("crosshair");
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
                                ctx.set_cursor("crosshair");
                                ctx.request_redraw();
                                resumed = true;
                                break;
                            } else if last.distance_to(snapped) <= tolerance {
                                // Resume from end: continue appending
                                self.nodes = p.nodes.clone();
                                self.resuming_path_id = Some(p.id);
                                self.is_dragging_handle = self.mode == PenMode::Bezier;
                                ctx.set_cursor("crosshair");
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
        ctx.set_cursor("crosshair");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude);
        self.current_cursor = Some(snapped);

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
        let zoom = viewport.zoom;
        let stroke_w = (2.0 / zoom).max(1.0);
        let node_size = (8.0 / zoom).max(4.0);

        if !self.nodes.is_empty() {
            // Draw temporary path connecting placed nodes
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
                ctx.active_stroke_width.max(2.0),
            );

            let mut path_paint = skia::Paint::default();
            path_paint.set_color4f(ctx.active_fill_color.to_skia(), None);
            path_paint.set_style(skia::PaintStyle::Stroke);
            path_paint.set_stroke_width(stroke_w);
            path_paint.set_anti_alias(true);
            canvas.draw_path(&temp_path.to_skia_path(), &path_paint);

            // Draw anchor points & handles
            let mut node_fill = skia::Paint::default();
            node_fill.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            node_fill.set_style(skia::PaintStyle::Fill);
            node_fill.set_anti_alias(true);

            let mut node_stroke = skia::Paint::default();
            node_stroke.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 1.0), None);
            node_stroke.set_style(skia::PaintStyle::Stroke);
            node_stroke.set_stroke_width(stroke_w);
            node_stroke.set_anti_alias(true);

            let mut start_node_stroke = skia::Paint::default();
            start_node_stroke.set_color4f(skia::Color4f::new(0.15, 0.75, 0.35, 1.0), None);
            start_node_stroke.set_style(skia::PaintStyle::Stroke);
            start_node_stroke.set_stroke_width(stroke_w * 1.5);
            start_node_stroke.set_anti_alias(true);

            let mut handle_line = skia::Paint::default();
            handle_line.set_color4f(skia::Color4f::new(0.4, 0.6, 0.9, 0.8), None);
            handle_line.set_style(skia::PaintStyle::Stroke);
            handle_line.set_stroke_width((1.0 / zoom).max(0.75));
            handle_line.set_anti_alias(true);

            for (i, node) in self.nodes.iter().enumerate() {
                let p = node.point;
                let half = node_size / 2.0;

                // Handles
                if let Some(h_out) = node.handle_out {
                    canvas.draw_line(p.to_skia(), h_out.to_skia(), &handle_line);
                    canvas.draw_circle(h_out.to_skia(), half * 0.8, &node_fill);
                    canvas.draw_circle(h_out.to_skia(), half * 0.8, &node_stroke);
                }
                if let Some(h_in) = node.handle_in {
                    canvas.draw_line(p.to_skia(), h_in.to_skia(), &handle_line);
                    canvas.draw_circle(h_in.to_skia(), half * 0.8, &node_fill);
                    canvas.draw_circle(h_in.to_skia(), half * 0.8, &node_stroke);
                }

                // Square anchor point (highlight first node in green if closeable)
                let rect = skia::Rect::from_xywh(p.x - half, p.y - half, node_size, node_size);
                canvas.draw_rect(rect, &node_fill);
                if i == 0 && self.nodes.len() >= 3 {
                    canvas.draw_rect(rect, &start_node_stroke);
                } else {
                    canvas.draw_rect(rect, &node_stroke);
                }
            }
        }
    }
}
