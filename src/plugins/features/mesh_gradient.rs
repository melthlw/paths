use skia_safe as skia;

use crate::core::{
    Color, Element, ElementId, MeshGradient, Point, PointerButton, PointerEvent, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

pub struct MeshGradientFeature {
    target_id: Option<ElementId>,
    selected_node_idx: Option<usize>,
    is_dragging: bool,
    last_click_time: Option<std::time::Instant>,
    last_click_pos: Option<Point>,
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
            last_click_time: None,
            last_click_pos: None,
        }
    }

    fn find_target_and_node(
        &mut self,
        ctx: &mut PluginContext,
        point: Point,
    ) -> (Option<ElementId>, Option<usize>) {
        let hit_radius = 16.0 / ctx.viewport.zoom;

        // 1. First priority: Check if clicking on ANY node of the currently selected element
        if let Some(&first) = ctx.document.selected_ids.iter().next() {
            if let Some(elem) = ctx.document.elements.iter().find(|e| e.id() == first) {
                let mesh_opt = match elem {
                    Element::Rect(r) => &r.mesh_gradient,
                    Element::Path(p) => &p.mesh_gradient,
                    _ => &None,
                };
                if let Some(mesh) = mesh_opt {
                    for (i, node) in mesh.nodes.iter().enumerate() {
                        if node.point.distance_to(point) <= hit_radius {
                            return (Some(first), Some(i));
                        }
                    }
                }
            }
        }

        // 2. Second priority: Check if clicking on ANY node of other elements
        for elem in ctx.document.elements.iter().rev() {
            let mesh_opt = match elem {
                Element::Rect(r) => &r.mesh_gradient,
                Element::Path(p) => &p.mesh_gradient,
                _ => &None,
            };
            if let Some(mesh) = mesh_opt {
                for (i, node) in mesh.nodes.iter().enumerate() {
                    if node.point.distance_to(point) <= hit_radius {
                        let id = elem.id();
                        ctx.document.select(id, false);
                        return (Some(id), Some(i));
                    }
                }
            }
        }

        // 3. Third priority: Check if clicking inside currently selected element
        if let Some(&first) = ctx.document.selected_ids.iter().next() {
            if let Some(elem) = ctx.document.elements.iter().find(|e| e.id() == first) {
                if elem.hit_test(point) {
                    return (Some(first), None);
                }
            }
        }

        // 4. Fourth priority: General element hit test
        for elem in ctx.document.elements.iter().rev() {
            if elem.hit_test(point) {
                let id = elem.id();
                ctx.document.select(id, false);
                return (Some(id), None);
            }
        }

        // 5. Fallback: Keep currently selected target if available
        if let Some(&first) = ctx.document.selected_ids.iter().next() {
            return (Some(first), None);
        }

        (None, None)
    }
}

impl FeaturePlugin for MeshGradientFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:node");
        if let Some(&first) = ctx.document.selected_ids.iter().next() {
            self.target_id = Some(first);
            self.selected_node_idx = Some(0);
        }
    }

    fn get_active_mesh_node(&self) -> Option<usize> {
        self.selected_node_idx
    }

    fn set_active_mesh_node(&mut self, idx: usize) {
        self.selected_node_idx = Some(idx);
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        let now = std::time::Instant::now();
        let is_double_click = if let (Some(last_t), Some(last_p)) = (self.last_click_time, self.last_click_pos) {
            now.duration_since(last_t).as_millis() < 350 && last_p.distance_to(event.world_pos) < (14.0 / ctx.viewport.zoom)
        } else {
            false
        };
        self.last_click_time = Some(now);
        self.last_click_pos = Some(event.world_pos);

        let (target_id_opt, hit_node_idx) = self.find_target_and_node(ctx, event.world_pos);
        let target_id = match target_id_opt {
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

        for el in &ctx.document.elements {
            if el.id() == target_id {
                bounds = el.bounds();
                let mesh_opt = match el {
                    Element::Rect(r) => &r.mesh_gradient,
                    Element::Path(p) => &p.mesh_gradient,
                    _ => &None,
                };

                if let Some(_mesh) = mesh_opt {
                    has_mesh = true;
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
                    let mut fills = el.fills();
                    if fills.is_empty() {
                        fills.push(crate::core::FillLayer::default());
                    }
                    if let Some(f0) = fills.first_mut() {
                        f0.style = crate::core::FillStyle::Mesh;
                        f0.mesh = Some(new_mesh.clone());
                    }
                    el.set_fills(fills);
                    break;
                }
            }
            self.selected_node_idx = Some(0);
        } else if let Some(node_idx) = hit_node_idx {
            // Direct node hit (even if dragged outside shape)
            self.selected_node_idx = Some(node_idx);
            self.is_dragging = true;

            for el in &ctx.document.elements {
                if el.id() == target_id {
                    let mesh_opt = match el {
                        Element::Rect(r) => &r.mesh_gradient,
                        Element::Path(p) => &p.mesh_gradient,
                        _ => &None,
                    };
                    if let Some(mesh) = mesh_opt {
                        if node_idx < mesh.nodes.len() {
                            ctx.active_fill_color = mesh.nodes[node_idx].color;
                        }
                    }
                    break;
                }
            }
        } else if is_double_click || event.alt_pressed {
            // Double click or Alt-click on mesh area: Subdivide mesh at click position
            ctx.document.snapshot();
            let mut new_node_idx = None;
            for el in &mut ctx.document.elements {
                if el.id() == target_id {
                    let mesh_mut = match el {
                        Element::Rect(r) => &mut r.mesh_gradient,
                        Element::Path(p) => &mut p.mesh_gradient,
                        _ => &mut None,
                    };
                    let mut updated_mesh = None;
                    if let Some(mesh) = mesh_mut {
                        let idx = mesh.subdivide_at(event.world_pos, Some(ctx.active_fill_color));
                        new_node_idx = Some(idx);
                        updated_mesh = Some(mesh.clone());
                    }
                    if let Some(um) = updated_mesh {
                        let mut fills = el.fills();
                        if let Some(f0) = fills.first_mut() {
                            f0.style = crate::core::FillStyle::Mesh;
                            f0.mesh = Some(um);
                        }
                        el.set_fills(fills);
                    }
                    break;
                }
            }
            self.selected_node_idx = new_node_idx;
            self.is_dragging = true;
        } else {
            // Single click inside mesh: Select closest node and enable dragging
            let mut closest_idx = 0;
            let mut min_dist = f32::MAX;
            for el in &ctx.document.elements {
                if el.id() == target_id {
                    let mesh_opt = match el {
                        Element::Rect(r) => &r.mesh_gradient,
                        Element::Path(p) => &p.mesh_gradient,
                        _ => &None,
                    };
                    if let Some(mesh) = mesh_opt {
                        for (i, node) in mesh.nodes.iter().enumerate() {
                            let d = node.point.distance_to(event.world_pos);
                            if d < min_dist {
                                min_dist = d;
                                closest_idx = i;
                            }
                        }
                        if closest_idx < mesh.nodes.len() {
                            ctx.active_fill_color = mesh.nodes[closest_idx].color;
                        }
                    }
                    break;
                }
            }

            self.selected_node_idx = Some(closest_idx);
            self.is_dragging = true;
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
                        let mut updated_mesh = None;
                        if let Some(mesh) = mesh_mut {
                            if idx < mesh.nodes.len() {
                                mesh.nodes[idx].point = event.world_pos;
                                updated_mesh = Some(mesh.clone());
                            }
                        }
                        if let Some(um) = updated_mesh {
                            let mut fills = el.fills();
                            if let Some(f0) = fills.first_mut() {
                                f0.style = crate::core::FillStyle::Mesh;
                                f0.mesh = Some(um);
                            }
                            el.set_fills(fills);
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
            None => {
                if let Some(&first) = _ctx.document.selected_ids.iter().next() {
                    first
                } else {
                    return;
                }
            }
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
        line_paint.set_color4f(skia::Color4f::new(0.2, 0.6, 1.0, 0.65), None);
        line_paint.set_stroke_width(1.2 / zoom);
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

        // 2. Draw Mesh Nodes with Clear Highlights & Colors
        for (i, node) in mesh.nodes.iter().enumerate() {
            let is_sel = self.selected_node_idx == Some(i);
            let radius = if is_sel { 7.5 / zoom } else { 5.0 / zoom };

            if is_sel {
                // Outer glow halo for selected node
                let mut halo = skia::Paint::default();
                halo.set_color4f(skia::Color4f::new(0.15, 0.55, 1.0, 0.4), None);
                halo.set_style(skia::PaintStyle::Fill);
                halo.set_anti_alias(true);
                canvas.draw_circle(node.point.to_skia(), radius + 3.5 / zoom, &halo);
            }

            let mut node_paint = skia::Paint::default();
            node_paint.set_color4f(node.color.to_skia(), None);
            node_paint.set_style(skia::PaintStyle::Fill);
            node_paint.set_anti_alias(true);
            canvas.draw_circle(node.point.to_skia(), radius, &node_paint);

            let mut border = skia::Paint::default();
            if is_sel {
                border.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                border.set_stroke_width(2.5 / zoom);
            } else {
                border.set_color4f(skia::Color4f::new(0.05, 0.05, 0.1, 0.85), None);
                border.set_stroke_width(1.5 / zoom);
            }
            border.set_style(skia::PaintStyle::Stroke);
            border.set_anti_alias(true);
            canvas.draw_circle(node.point.to_skia(), radius, &border);
        }
    }
}
