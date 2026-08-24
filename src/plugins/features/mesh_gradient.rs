use skia_safe as skia;

use crate::core::{
    Color, Element, ElementId, MeshGradient, Point, PointerButton, PointerEvent, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

pub struct MeshGradientFeature {
    target_id: Option<ElementId>,
    selected_node_idx: Option<usize>,
    is_dragging: bool,
}

impl Default for MeshGradientFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshGradientFeature {
    pub fn new() -> Self {
        Self {
            target_id: None,
            selected_node_idx: None,
            is_dragging: false,
        }
    }

    fn find_target(&mut self, ctx: &mut PluginContext, point: Point) -> Option<ElementId> {
        if let Some(&first) = ctx.document.selected_ids.iter().next() {
            if let Some(elem) = ctx.document.elements.iter().find(|e| e.id() == first) {
                if elem.hit_test(point) {
                    return Some(first);
                }
            }
        }

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

impl FeaturePlugin for MeshGradientFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:node");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        let target_id = match self.find_target(ctx, event.world_pos) {
            Some(id) => id,
            None => {
                self.target_id = None;
                self.selected_node_idx = None;
                ctx.request_redraw();
                return;
            }
        };

        self.target_id = Some(target_id);

        // Check if target already has mesh gradient
        let mut has_mesh = false;
        let mut bounds = crate::core::Rect::ZERO;
        let mut hit_idx = None;

        for el in &ctx.document.elements {
            if el.id() == target_id {
                bounds = el.bounds();
                let mesh_opt = match el {
                    Element::Rect(r) => &r.mesh_gradient,
                    Element::Path(p) => &p.mesh_gradient,
                    _ => &None,
                };

                if let Some(mesh) = mesh_opt {
                    has_mesh = true;
                    let hit_dist = 10.0 / ctx.viewport.zoom;
                    for (i, node) in mesh.nodes.iter().enumerate() {
                        if node.point.distance_to(event.world_pos) <= hit_dist {
                            hit_idx = Some(i);
                            break;
                        }
                    }
                }
                break;
            }
        }

        if !has_mesh {
            // Initialize 3x3 Mesh Grid
            ctx.document.snapshot();
            let c1 = ctx.active_fill_color;
            let c2 = ctx.active_stroke_color.unwrap_or(Color::BLUE);
            let new_mesh = MeshGradient::new_grid(bounds, 3, 3, c1, c2);

            for el in &mut ctx.document.elements {
                if el.id() == target_id {
                    match el {
                        Element::Rect(r) => r.mesh_gradient = Some(new_mesh.clone()),
                        Element::Path(p) => p.mesh_gradient = Some(new_mesh.clone()),
                        _ => {}
                    }
                    break;
                }
            }
            self.selected_node_idx = Some(0);
        } else if let Some(idx) = hit_idx {
            self.selected_node_idx = Some(idx);
            self.is_dragging = true;

            // If Shift is pressed, apply active fill color to this node!
            if event.shift_pressed {
                ctx.document.snapshot();
                for el in &mut ctx.document.elements {
                    if el.id() == target_id {
                        let mesh_mut = match el {
                            Element::Rect(r) => &mut r.mesh_gradient,
                            Element::Path(p) => &mut p.mesh_gradient,
                            _ => &mut None,
                        };
                        if let Some(mesh) = mesh_mut {
                            if idx < mesh.nodes.len() {
                                mesh.nodes[idx].color = ctx.active_fill_color;
                            }
                        }
                        break;
                    }
                }
            }
        } else {
            self.selected_node_idx = None;
        }

        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if self.is_dragging {
            if let (Some(id), Some(idx)) = (self.target_id, self.selected_node_idx) {
                for el in &mut ctx.document.elements {
                    if el.id() == id {
                        let mesh_mut = match el {
                            Element::Rect(r) => &mut r.mesh_gradient,
                            Element::Path(p) => &mut p.mesh_gradient,
                            _ => &mut None,
                        };
                        if let Some(mesh) = mesh_mut {
                            if idx < mesh.nodes.len() {
                                mesh.nodes[idx].point = event.world_pos;
                            }
                        }
                        break;
                    }
                }
                ctx.request_redraw();
            }
        }
        ctx.set_cursor("crosshair");
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        if self.is_dragging {
            self.is_dragging = false;
            ctx.document.snapshot();
            ctx.request_redraw();
        }
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.is_dragging = false;
        self.selected_node_idx = None;
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
        let id = match self.target_id {
            Some(i) => i,
            None => return,
        };

        let doc = &_ctx.document;
        let elem = match doc.elements.iter().find(|e| e.id() == id) {
            Some(e) => e,
            None => return,
        };

        let mesh = match elem {
            Element::Rect(r) => match &r.mesh_gradient {
                Some(m) => m,
                None => return,
            },
            Element::Path(p) => match &p.mesh_gradient {
                Some(m) => m,
                None => return,
            },
            _ => return,
        };

        let zoom = viewport.zoom;

        // 1. Draw Mesh Grid Connecting Lines
        let mut line_paint = skia::Paint::default();
        line_paint.set_color4f(skia::Color4f::new(0.2, 0.6, 1.0, 0.6), None);
        line_paint.set_stroke_width(1.0 / zoom);
        line_paint.set_style(skia::PaintStyle::Stroke);
        line_paint.set_anti_alias(true);

        for r in 0..mesh.rows {
            for c in 0..mesh.cols {
                let idx = r * mesh.cols + c;
                let pt = mesh.nodes[idx].point;

                // Horizontal segment
                if c + 1 < mesh.cols {
                    let next_h = mesh.nodes[r * mesh.cols + (c + 1)].point;
                    canvas.draw_line(pt.to_skia(), next_h.to_skia(), &line_paint);
                }

                // Vertical segment
                if r + 1 < mesh.rows {
                    let next_v = mesh.nodes[(r + 1) * mesh.cols + c].point;
                    canvas.draw_line(pt.to_skia(), next_v.to_skia(), &line_paint);
                }
            }
        }

        // 2. Draw Mesh Nodes
        for (i, node) in mesh.nodes.iter().enumerate() {
            let is_sel = self.selected_node_idx == Some(i);
            let radius = if is_sel { 6.0 / zoom } else { 4.5 / zoom };

            let mut node_paint = skia::Paint::default();
            node_paint.set_color4f(node.color.to_skia(), None);
            node_paint.set_style(skia::PaintStyle::Fill);
            node_paint.set_anti_alias(true);
            canvas.draw_circle(node.point.to_skia(), radius, &node_paint);

            let mut border = skia::Paint::default();
            if is_sel {
                border.set_color4f(skia::Color4f::new(1.0, 0.8, 0.2, 1.0), None);
                border.set_stroke_width(2.0 / zoom);
            } else {
                border.set_color4f(skia::Color4f::new(0.1, 0.1, 0.15, 0.9), None);
                border.set_stroke_width(1.2 / zoom);
            }
            border.set_style(skia::PaintStyle::Stroke);
            border.set_anti_alias(true);
            canvas.draw_circle(node.point.to_skia(), radius, &border);
        }
    }
}
