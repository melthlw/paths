use skia_safe as skia;

use crate::core::{Element, PathElement, PathNode, Point, PointerButton, PointerEvent, Viewport};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

pub struct PenFeature {
    nodes: Vec<PathNode>,
    current_cursor: Option<Point>,
    is_dragging_handle: bool,
}

impl Default for PenFeature {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            current_cursor: None,
            is_dragging_handle: false,
        }
    }
}

impl PenFeature {
    pub fn new() -> Self {
        Self::default()
    }

    fn finish_path(&mut self, ctx: &mut PluginContext, is_closed: bool) {
        if self.nodes.len() >= 2 {
            let path = PathElement::new(
                self.nodes.clone(),
                is_closed,
                None, // Outline by default
                Some(ctx.active_fill_color),
                ctx.active_stroke_width.max(2.0),
            );
            ctx.document.add_element(Element::Path(path));
        }
        self.nodes.clear();
        self.current_cursor = None;
        self.is_dragging_handle = false;
        ctx.clear_snap_guides();
        ctx.request_redraw();
    }
}

impl FeaturePlugin for PenFeature {
    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude);

        // Check if clicking near start point to close path
        if let Some(first) = self.nodes.first() {
            if self.nodes.len() >= 3 && first.point.distance_to(snapped) < 14.0 / ctx.viewport.zoom
            {
                self.finish_path(ctx, true);
                return;
            }
        }

        let new_node = PathNode::new(snapped);
        self.nodes.push(new_node);
        self.is_dragging_handle = true;
        ctx.set_cursor("crosshair");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        let empty_exclude = std::collections::HashSet::new();
        let snapped = ctx.snap_point(event.world_pos, &empty_exclude);
        self.current_cursor = Some(snapped);

        if self.is_dragging_handle {
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

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.finish_path(ctx, false);
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

            let mut handle_line = skia::Paint::default();
            handle_line.set_color4f(skia::Color4f::new(0.4, 0.6, 0.9, 0.8), None);
            handle_line.set_style(skia::PaintStyle::Stroke);
            handle_line.set_stroke_width((1.0 / zoom).max(0.75));
            handle_line.set_anti_alias(true);

            for node in &self.nodes {
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

                // Square anchor point
                let rect = skia::Rect::from_xywh(p.x - half, p.y - half, node_size, node_size);
                canvas.draw_rect(rect, &node_fill);
                canvas.draw_rect(rect, &node_stroke);
            }
        }
    }
}
