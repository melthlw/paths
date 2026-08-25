use super::CanvasWidget;
use crate::core::{Color, Element, Point};
use gtk4::prelude::*;

impl CanvasWidget {
    pub fn active_fill_color(&self) -> Color {
        self.state
            .try_borrow()
            .map(|s| s.active_fill_color)
            .unwrap_or_else(|_| Color::BLACK)
    }

    pub fn active_stroke_color(&self) -> Option<Color> {
        self.state
            .try_borrow()
            .ok()
            .and_then(|s| s.active_stroke_color)
    }

    pub fn active_stroke_width(&self) -> f32 {
        self.state
            .try_borrow()
            .map(|s| s.active_stroke_width)
            .unwrap_or(2.0)
    }

    pub fn set_fill_color(&self, color: Color) {
        let mut state = self.state.borrow_mut();
        state.active_fill_color = color;

        if !state.document.selected_ids.is_empty() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            let is_mesh_tool = state.plugin_manager.active_id() == "mesh_gradient";
            let active_mesh_node = if is_mesh_tool {
                state.plugin_manager.active_feature().and_then(|f| f.get_active_mesh_node())
            } else {
                None
            };
            let is_grad_tool = state.plugin_manager.active_id() == "gradient";
            let active_grad_stop = if is_grad_tool {
                state.plugin_manager.active_feature().and_then(|f| f.get_active_gradient_stop())
            } else {
                None
            };

            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    if let Some(node_idx) = active_mesh_node {
                        let mesh_mut = match el {
                            crate::core::Element::Rect(r) => &mut r.mesh_gradient,
                            crate::core::Element::Path(p) => &mut p.mesh_gradient,
                            _ => &mut None,
                        };
                        let mut updated = false;
                        if let Some(m) = mesh_mut {
                            if node_idx < m.nodes.len() {
                                m.nodes[node_idx].color = color;
                                updated = true;
                            }
                        }
                        let mut fills = el.fills();
                        if let Some(f0) = fills.first_mut() {
                            if let Some(m) = &mut f0.mesh {
                                if node_idx < m.nodes.len() {
                                    m.nodes[node_idx].color = color;
                                    updated = true;
                                }
                            }
                        }
                        if updated {
                            el.set_fills(fills);
                            continue;
                        }
                    }
                    if let Some(stop_idx) = active_grad_stop {
                        let grad_mut = match el {
                            crate::core::Element::Rect(r) => &mut r.gradient,
                            crate::core::Element::Path(p) => &mut p.gradient,
                            _ => &mut None,
                        };
                        let mut updated = false;
                        if let Some(g) = grad_mut {
                            if stop_idx < g.stops.len() {
                                g.stops[stop_idx].color = color;
                                updated = true;
                            }
                        }
                        let mut fills = el.fills();
                        if let Some(f0) = fills.first_mut() {
                            let mut eff = f0.effective_stops();
                            if stop_idx < eff.len() {
                                eff[stop_idx].color = color;
                                f0.stops = eff.clone();
                                if stop_idx == 0 {
                                    f0.color = color;
                                }
                                if stop_idx == eff.len() - 1 {
                                    f0.secondary_color = color;
                                }
                                updated = true;
                            }
                        }
                        if updated {
                            el.set_fills(fills);
                            continue;
                        }
                    }
                    el.set_fill_color(Some(color));
                }
            }
        }
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_stroke_color(&self, color: Option<Color>) {
        let mut state = self.state.borrow_mut();
        state.active_stroke_color = color;

        if !state.document.selected_ids.is_empty() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    el.set_stroke_color(color);
                }
            }
        }
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn get_selected_fills_and_strokes(
        &self,
    ) -> Option<(Vec<crate::core::FillLayer>, Vec<crate::core::StrokeLayer>)> {
        if let Ok(state) = self.state.try_borrow() {
            state.document.get_selected_fills_and_strokes()
        } else {
            None
        }
    }

    pub fn get_selected_mesh(&self) -> Option<crate::core::MeshGradient> {
        if let Ok(state) = self.state.try_borrow() {
            for el in &state.document.elements {
                if state.document.selected_ids.contains(&el.id()) {
                    let mesh_opt = match el {
                        crate::core::Element::Rect(r) => &r.mesh_gradient,
                        crate::core::Element::Path(p) => &p.mesh_gradient,
                        _ => &None,
                    };
                    if let Some(m) = mesh_opt {
                        return Some(m.clone());
                    }
                    if let Some(f0) = el.fills().first() {
                        if let Some(m) = &f0.mesh {
                            return Some(m.clone());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn reset_selected_mesh_grid(
        &self,
        rows: usize,
        cols: usize,
        c1: Option<Color>,
        c2: Option<Color>,
    ) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            let def_c1 = state.active_fill_color;
            let def_c2 = state
                .active_stroke_color
                .unwrap_or(Color::from_hex("#ff2a6d").unwrap());
            let c1 = c1.unwrap_or(def_c1);
            let c2 = c2.unwrap_or(def_c2);

            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let bounds = el.bounds();
                    let mesh = crate::core::MeshGradient::new_grid(bounds, rows, cols, c1, c2);
                    match el {
                        crate::core::Element::Rect(r) => r.mesh_gradient = Some(mesh.clone()),
                        crate::core::Element::Path(p) => p.mesh_gradient = Some(mesh.clone()),
                        _ => {}
                    }
                    let mut fills = el.fills();
                    if fills.is_empty() {
                        fills.push(crate::core::FillLayer::default());
                    }
                    if let Some(f0) = fills.first_mut() {
                        f0.style = crate::core::FillStyle::Mesh;
                        f0.mesh = Some(mesh.clone());
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_mesh_node_color(&self, idx: usize, color: Color) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mut updated_mesh = None;
                    let mesh_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.mesh_gradient,
                        crate::core::Element::Path(p) => &mut p.mesh_gradient,
                        _ => &mut None,
                    };
                    if let Some(m) = mesh_mut {
                        if idx < m.nodes.len() {
                            m.nodes[idx].color = color;
                            updated_mesh = Some(m.clone());
                        }
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        f0.style = crate::core::FillStyle::Mesh;
                        if let Some(um) = updated_mesh.clone() {
                            f0.mesh = Some(um);
                        } else if let Some(m) = &mut f0.mesh {
                            if idx < m.nodes.len() {
                                m.nodes[idx].color = color;
                                updated_mesh = Some(m.clone());
                            }
                        }
                    }
                    el.set_fills(fills);
                    if let Some(um) = updated_mesh {
                        match el {
                            crate::core::Element::Rect(r) => r.mesh_gradient = Some(um),
                            crate::core::Element::Path(p) => p.mesh_gradient = Some(um),
                            _ => {}
                        }
                    }
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn get_active_mesh_node(&self) -> Option<usize> {
        self.state
            .try_borrow()
            .ok()
            .and_then(|s| s.plugin_manager.active_feature().and_then(|f| f.get_active_mesh_node()))
    }

    pub fn get_active_mesh_info(&self) -> Option<(usize, usize, usize, Color, bool)> {
        let state = self.state.try_borrow().ok()?;
        let selected_id = state.document.selected_ids.iter().next()?;
        let el = state.document.elements.iter().find(|e| e.id() == *selected_id)?;
        let mesh = match el {
            crate::core::Element::Rect(r) => r.mesh_gradient.as_ref()?,
            crate::core::Element::Path(p) => p.mesh_gradient.as_ref()?,
            _ => return None,
        };
        let active_node = state
            .plugin_manager
            .active_feature()
            .and_then(|f| f.get_active_mesh_node())
            .unwrap_or(0)
            .min(mesh.nodes.len().saturating_sub(1));
        let col = mesh.nodes.get(active_node).map(|n| n.color).unwrap_or(Color::BLACK);
        Some((active_node, mesh.rows, mesh.cols, col, mesh.smooth_curves))
    }

    pub fn reset_selected_mesh_to_bounds(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let bounds = el.bounds();
                    let mesh_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.mesh_gradient,
                        crate::core::Element::Path(p) => &mut p.mesh_gradient,
                        _ => &mut None,
                    };
                    let mut updated_mesh = None;
                    if let Some(m) = mesh_mut {
                        m.reset_to_bounds(bounds);
                        updated_mesh = Some(m.clone());
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        if let Some(um) = updated_mesh.clone() {
                            f0.mesh = Some(um);
                        } else if let Some(m) = &mut f0.mesh {
                            m.reset_to_bounds(bounds);
                            updated_mesh = Some(m.clone());
                        }
                    }
                    el.set_fills(fills);
                    if let Some(um) = updated_mesh {
                        match el {
                            crate::core::Element::Rect(r) => r.mesh_gradient = Some(um),
                            crate::core::Element::Path(p) => p.mesh_gradient = Some(um),
                            _ => {}
                        }
                    }
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn split_selected_mesh_row(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            let active_node = state
                .plugin_manager
                .active_feature()
                .and_then(|f| f.get_active_mesh_node());
            let mut new_sel = None;
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mesh_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.mesh_gradient,
                        crate::core::Element::Path(p) => &mut p.mesh_gradient,
                        _ => &mut None,
                    };
                    let mut updated_mesh = None;
                    if let Some(m) = mesh_mut {
                        let r_idx = active_node.map(|idx| idx / m.cols);
                        let next_node = m.split_row(r_idx);
                        new_sel = Some(next_node);
                        updated_mesh = Some(m.clone());
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        if let Some(um) = updated_mesh.clone() {
                            f0.mesh = Some(um);
                        } else if let Some(m) = &mut f0.mesh {
                            let r_idx = active_node.map(|idx| idx / m.cols);
                            let next_node = m.split_row(r_idx);
                            new_sel = Some(next_node);
                            updated_mesh = Some(m.clone());
                        }
                    }
                    el.set_fills(fills);
                    if let Some(um) = updated_mesh {
                        match el {
                            crate::core::Element::Rect(r) => r.mesh_gradient = Some(um),
                            crate::core::Element::Path(p) => p.mesh_gradient = Some(um),
                            _ => {}
                        }
                    }
                }
            }
            if let Some(idx) = new_sel {
                if let Some(feat) = state.plugin_manager.active_feature_mut() {
                    feat.set_active_mesh_node(idx);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn split_selected_mesh_col(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            let active_node = state
                .plugin_manager
                .active_feature()
                .and_then(|f| f.get_active_mesh_node());
            let mut new_sel = None;
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mesh_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.mesh_gradient,
                        crate::core::Element::Path(p) => &mut p.mesh_gradient,
                        _ => &mut None,
                    };
                    let mut updated_mesh = None;
                    if let Some(m) = mesh_mut {
                        let c_idx = active_node.map(|idx| idx % m.cols);
                        let next_node = m.split_col(c_idx);
                        new_sel = Some(next_node);
                        updated_mesh = Some(m.clone());
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        if let Some(um) = updated_mesh.clone() {
                            f0.mesh = Some(um);
                        } else if let Some(m) = &mut f0.mesh {
                            let c_idx = active_node.map(|idx| idx % m.cols);
                            let next_node = m.split_col(c_idx);
                            new_sel = Some(next_node);
                            updated_mesh = Some(m.clone());
                        }
                    }
                    el.set_fills(fills);
                    if let Some(um) = updated_mesh {
                        match el {
                            crate::core::Element::Rect(r) => r.mesh_gradient = Some(um),
                            crate::core::Element::Path(p) => p.mesh_gradient = Some(um),
                            _ => {}
                        }
                    }
                }
            }
            if let Some(idx) = new_sel {
                if let Some(feat) = state.plugin_manager.active_feature_mut() {
                    feat.set_active_mesh_node(idx);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn toggle_selected_mesh_smooth_curves(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mesh_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.mesh_gradient,
                        crate::core::Element::Path(p) => &mut p.mesh_gradient,
                        _ => &mut None,
                    };
                    let mut updated_mesh = None;
                    if let Some(m) = mesh_mut {
                        m.smooth_curves = !m.smooth_curves;
                        updated_mesh = Some(m.clone());
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        if let Some(um) = updated_mesh.clone() {
                            f0.mesh = Some(um);
                        } else if let Some(m) = &mut f0.mesh {
                            m.smooth_curves = !m.smooth_curves;
                            updated_mesh = Some(m.clone());
                        }
                    }
                    el.set_fills(fills);
                    if let Some(um) = updated_mesh {
                        match el {
                            crate::core::Element::Rect(r) => r.mesh_gradient = Some(um),
                            crate::core::Element::Path(p) => p.mesh_gradient = Some(um),
                            _ => {}
                        }
                    }
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn smooth_selected_mesh_spacing(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mesh_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.mesh_gradient,
                        crate::core::Element::Path(p) => &mut p.mesh_gradient,
                        _ => &mut None,
                    };
                    let mut updated_mesh = None;
                    if let Some(m) = mesh_mut {
                        m.smooth_grid_spacing();
                        updated_mesh = Some(m.clone());
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        if let Some(um) = updated_mesh.clone() {
                            f0.mesh = Some(um);
                        } else if let Some(m) = &mut f0.mesh {
                            m.smooth_grid_spacing();
                            updated_mesh = Some(m.clone());
                        }
                    }
                    el.set_fills(fills);
                    if let Some(um) = updated_mesh {
                        match el {
                            crate::core::Element::Rect(r) => r.mesh_gradient = Some(um),
                            crate::core::Element::Path(p) => p.mesh_gradient = Some(um),
                            _ => {}
                        }
                    }
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn delete_selected_mesh_node(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            let active_node = state
                .plugin_manager
                .active_feature()
                .and_then(|f| f.get_active_mesh_node())
                .unwrap_or(0);
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mesh_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.mesh_gradient,
                        crate::core::Element::Path(p) => &mut p.mesh_gradient,
                        _ => &mut None,
                    };
                    let mut updated_mesh = None;
                    if let Some(m) = mesh_mut {
                        if m.delete_node(active_node) {
                            updated_mesh = Some(m.clone());
                        }
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        if let Some(um) = updated_mesh.clone() {
                            f0.mesh = Some(um);
                        } else if let Some(m) = &mut f0.mesh {
                            if m.delete_node(active_node) {
                                updated_mesh = Some(m.clone());
                            }
                        }
                    }
                    el.set_fills(fills);
                    if let Some(um) = updated_mesh {
                        match el {
                            crate::core::Element::Rect(r) => r.mesh_gradient = Some(um),
                            crate::core::Element::Path(p) => p.mesh_gradient = Some(um),
                            _ => {}
                        }
                    }
                }
            }
            if let Some(feat) = state.plugin_manager.active_feature_mut() {
                feat.set_active_mesh_node(0);
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn get_active_gradient_info(
        &self,
    ) -> Option<(usize, crate::core::GradientType, f32, Vec<crate::core::GradientStop>, Color)> {
        let state = self.state.try_borrow().ok()?;
        let selected_id = state.document.selected_ids.iter().next()?;
        let el = state.document.elements.iter().find(|e| e.id() == *selected_id)?;
        let fills = el.fills();
        let f0 = fills.first()?;
        let kind = match f0.style {
            crate::core::FillStyle::RadialGradient => crate::core::GradientType::Radial,
            _ => crate::core::GradientType::Linear,
        };
        let stops = f0.effective_stops();
        let active_stop = state
            .plugin_manager
            .active_feature()
            .and_then(|f| f.get_active_gradient_stop())
            .unwrap_or(0)
            .min(stops.len().saturating_sub(1));
        let col = stops.get(active_stop).map(|s| s.color).unwrap_or(Color::BLACK);
        Some((active_stop, kind, f0.angle, stops, col))
    }

    pub fn set_gradient_type(&self, kind: crate::core::GradientType) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let grad_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.gradient,
                        crate::core::Element::Path(p) => &mut p.gradient,
                        _ => &mut None,
                    };
                    if let Some(g) = grad_mut {
                        g.kind = kind;
                    }
                    let mut fills = el.fills();
                    if fills.is_empty() {
                        fills.push(crate::core::FillLayer::default());
                    }
                    if let Some(f0) = fills.first_mut() {
                        f0.style = match kind {
                            crate::core::GradientType::Radial => crate::core::FillStyle::RadialGradient,
                            _ => crate::core::FillStyle::LinearGradient,
                        };
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_gradient_angle(&self, angle: f32) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let b = el.bounds();
                    let cx = b.x + b.width * 0.5;
                    let cy = b.y + b.height * 0.5;
                    let rad = angle.to_radians();
                    let len = (b.width.hypot(b.height) * 0.5).max(1.0);
                    let dx = rad.cos() * len;
                    let dy = rad.sin() * len;

                    let grad_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.gradient,
                        crate::core::Element::Path(p) => &mut p.gradient,
                        _ => &mut None,
                    };
                    if let Some(g) = grad_mut {
                        g.start = crate::core::Point::new(cx - dx, cy - dy);
                        g.end = crate::core::Point::new(cx + dx, cy + dy);
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        f0.angle = angle;
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn reverse_selected_gradient_stops(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let grad_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.gradient,
                        crate::core::Element::Path(p) => &mut p.gradient,
                        _ => &mut None,
                    };
                    if let Some(g) = grad_mut {
                        for s in &mut g.stops {
                            s.offset = (1.0 - s.offset).clamp(0.0, 1.0);
                        }
                        g.stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        let mut eff = f0.effective_stops();
                        for s in &mut eff {
                            s.offset = (1.0 - s.offset).clamp(0.0, 1.0);
                        }
                        eff.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
                        f0.stops = eff.clone();
                        f0.color = eff.first().map(|s| s.color).unwrap_or(f0.color);
                        f0.secondary_color = eff.last().map(|s| s.color).unwrap_or(f0.secondary_color);
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn add_selected_gradient_stop(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            let mut new_sel = None;
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let grad_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.gradient,
                        crate::core::Element::Path(p) => &mut p.gradient,
                        _ => &mut None,
                    };
                    let mut new_stop = None;
                    if let Some(g) = grad_mut {
                        let c1 = g.stops.first().map(|s| s.color).unwrap_or(Color::BLACK);
                        let c2 = g.stops.last().map(|s| s.color).unwrap_or(Color::WHITE);
                        let mid_col = Color::new(
                            (c1.r + c2.r) * 0.5,
                            (c1.g + c2.g) * 0.5,
                            (c1.b + c2.b) * 0.5,
                            (c1.a + c2.a) * 0.5,
                        );
                        let s = crate::core::GradientStop::new(0.5, mid_col);
                        g.stops.push(s.clone());
                        g.stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
                        new_stop = Some(s);
                        if let Some(pos) = g.stops.iter().position(|st| (st.offset - 0.5).abs() < 0.001) {
                            new_sel = Some(pos);
                        }
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        let mut eff = f0.effective_stops();
                        let c1 = eff.first().map(|s| s.color).unwrap_or(Color::BLACK);
                        let c2 = eff.last().map(|s| s.color).unwrap_or(Color::WHITE);
                        let mid_col = Color::new(
                            (c1.r + c2.r) * 0.5,
                            (c1.g + c2.g) * 0.5,
                            (c1.b + c2.b) * 0.5,
                            (c1.a + c2.a) * 0.5,
                        );
                        let s = new_stop.unwrap_or_else(|| crate::core::GradientStop::new(0.5, mid_col));
                        eff.push(s);
                        eff.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
                        f0.stops = eff;
                    }
                    el.set_fills(fills);
                }
            }
            if let Some(idx) = new_sel {
                if let Some(feat) = state.plugin_manager.active_feature_mut() {
                    feat.set_active_gradient_stop(idx);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn delete_selected_gradient_stop(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            let active_stop = state
                .plugin_manager
                .active_feature()
                .and_then(|f| f.get_active_gradient_stop())
                .unwrap_or(0);
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let grad_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.gradient,
                        crate::core::Element::Path(p) => &mut p.gradient,
                        _ => &mut None,
                    };
                    if let Some(g) = grad_mut {
                        if g.stops.len() > 2 && active_stop < g.stops.len() {
                            g.stops.remove(active_stop);
                        }
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        let mut eff = f0.effective_stops();
                        if eff.len() > 2 && active_stop < eff.len() {
                            eff.remove(active_stop);
                            f0.stops = eff.clone();
                            f0.color = eff.first().map(|s| s.color).unwrap_or(f0.color);
                            f0.secondary_color = eff.last().map(|s| s.color).unwrap_or(f0.secondary_color);
                        }
                    }
                    el.set_fills(fills);
                }
            }
            if let Some(feat) = state.plugin_manager.active_feature_mut() {
                feat.set_active_gradient_stop(0);
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_gradient_stop_color(&self, stop_idx: usize, color: Color) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let grad_mut = match el {
                        crate::core::Element::Rect(r) => &mut r.gradient,
                        crate::core::Element::Path(p) => &mut p.gradient,
                        _ => &mut None,
                    };
                    if let Some(g) = grad_mut {
                        if stop_idx < g.stops.len() {
                            g.stops[stop_idx].color = color;
                        }
                    }
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        let mut eff = f0.effective_stops();
                        if stop_idx < eff.len() {
                            eff[stop_idx].color = color;
                            f0.stops = eff.clone();
                            if stop_idx == 0 {
                                f0.color = color;
                            }
                            if stop_idx == eff.len() - 1 {
                                f0.secondary_color = color;
                            }
                        }
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn get_active_pattern_info(
        &self,
    ) -> Option<(
        crate::core::element::PatternType,
        Color,
        Color,
        f32,
        f32,
        Point,
        Option<String>,
    )> {
        let state = self.state.try_borrow().ok()?;
        let selected_ids = state.document.selected_ids.clone();
        let first_id = selected_ids.iter().next()?;
        let elem = state.document.elements.iter().find(|e| e.id() == *first_id)?;
        let fills = elem.fills();
        let fill = fills
            .iter()
            .find(|f| f.style == crate::core::FillStyle::Pattern && f.enabled)
            .or_else(|| fills.iter().find(|f| f.style == crate::core::FillStyle::Pattern))?;
        Some((
            fill.pattern_type,
            fill.color,
            fill.secondary_color,
            fill.pattern_scale,
            fill.angle,
            fill.pattern_offset,
            fill.custom_pattern_path.clone(),
        ))
    }

    pub fn set_pattern_type_and_custom_path(
        &self,
        pt: crate::core::element::PatternType,
        custom_path: Option<String>,
    ) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mut fills = el.fills();
                    if fills.is_empty() {
                        fills.push(crate::core::FillLayer::default());
                    }
                    if let Some(f0) = fills.first_mut() {
                        f0.style = crate::core::FillStyle::Pattern;
                        f0.pattern_type = pt;
                        f0.custom_pattern_path = custom_path.clone();
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_pattern_primary_color(&self, color: Color) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        f0.color = color;
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_pattern_secondary_color(&self, color: Color) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        f0.secondary_color = color;
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_pattern_scale(&self, scale: f32) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        f0.pattern_scale = scale.clamp(4.0, 2048.0);
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_pattern_angle(&self, angle: f32) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        f0.angle = angle;
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_pattern_offset(&self, offset: Point) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        f0.pattern_offset = offset;
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn reset_pattern_transform(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        f0.pattern_scale = 24.0;
                        f0.angle = 0.0;
                        f0.pattern_offset = Point::ZERO;
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn swap_pattern_colors(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    let mut fills = el.fills();
                    if let Some(f0) = fills.first_mut() {
                        let c1 = f0.color;
                        let c2 = f0.secondary_color;
                        f0.color = c2;
                        f0.secondary_color = c1;
                    }
                    el.set_fills(fills);
                }
            }
            state.notify_status();
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_fills(&self, fills: Vec<crate::core::FillLayer>) {
        let mut state = self.state.borrow_mut();
        if let Some(first_fill) = fills.iter().find(|f| f.enabled) {
            state.active_fill_color = first_fill.color;
        }
        if !state.document.selected_ids.is_empty() {
            state.document.set_selected_fills(fills);
        }
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_strokes(&self, strokes: Vec<crate::core::StrokeLayer>) {
        let mut state = self.state.borrow_mut();
        if let Some(first_stroke) = strokes.iter().find(|s| s.enabled) {
            state.active_stroke_color = Some(first_stroke.color);
            state.active_stroke_width = first_stroke.width;
        }
        if !state.document.selected_ids.is_empty() {
            state.document.set_selected_strokes(strokes);
        }
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_opacity(&self, opacity: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_opacity(opacity);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_blend_mode(&self, mode: crate::core::BlendMode) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_blend_mode(mode);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_blur(&self, blur: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_blur(blur);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_rect_corner_radius(&self, radius: f32) {
        let r_val = radius.max(0.0);
        let radii = crate::core::CornerRadii::uniform(r_val);
        let mut state = self.state.borrow_mut();

        let current_style = state
            .document
            .selected_ids
            .iter()
            .find_map(|id| {
                state
                    .document
                    .elements
                    .iter()
                    .find(|e| e.id() == *id)
                    .and_then(|e| {
                        if let Element::Rect(r) = e {
                            Some(r.corner_style)
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or(crate::core::CornerStyle::Round);

        let origin = crate::core::ShapeOrigin::Rectangle {
            corner_radius: r_val,
            corner_radii: radii,
            corner_style: current_style,
        };
        if let Some(feat) = state.plugin_manager.active_feature_mut() {
            feat.set_shape_params(&origin);
        }

        let selected_ids = state.document.selected_ids.clone();
        if !selected_ids.is_empty() {
            state.document.snapshot();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    if let Element::Rect(r) = el {
                        r.corner_radius = r_val;
                        r.corner_radii = radii;
                    }
                }
            }
            self.drawing_area.queue_draw();
        }
        state.notify_status();
    }

    pub fn set_rect_corner_radii(&self, radii: crate::core::CornerRadii) {
        let mut state = self.state.borrow_mut();
        let max_r = radii.max_radius();

        let current_style = state
            .document
            .selected_ids
            .iter()
            .find_map(|id| {
                state
                    .document
                    .elements
                    .iter()
                    .find(|e| e.id() == *id)
                    .and_then(|e| {
                        if let Element::Rect(r) = e {
                            Some(r.corner_style)
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or(crate::core::CornerStyle::Round);

        let origin = crate::core::ShapeOrigin::Rectangle {
            corner_radius: max_r,
            corner_radii: radii,
            corner_style: current_style,
        };
        if let Some(feat) = state.plugin_manager.active_feature_mut() {
            feat.set_shape_params(&origin);
        }

        let selected_ids = state.document.selected_ids.clone();
        if !selected_ids.is_empty() {
            state.document.snapshot();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    if let Element::Rect(r) = el {
                        r.corner_radii = radii;
                        r.corner_radius = max_r;
                    }
                }
            }
            self.drawing_area.queue_draw();
        }
        state.notify_status();
    }

    pub fn set_rect_corner_style(&self, style: crate::core::CornerStyle) {
        let mut state = self.state.borrow_mut();

        let (cur_radius, cur_radii) = state
            .document
            .selected_ids
            .iter()
            .find_map(|id| {
                state
                    .document
                    .elements
                    .iter()
                    .find(|e| e.id() == *id)
                    .and_then(|e| {
                        if let Element::Rect(r) = e {
                            Some((r.corner_radius, r.corner_radii))
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or((0.0, crate::core::CornerRadii::default()));

        let origin = crate::core::ShapeOrigin::Rectangle {
            corner_radius: cur_radius,
            corner_radii: cur_radii,
            corner_style: style,
        };
        if let Some(feat) = state.plugin_manager.active_feature_mut() {
            feat.set_shape_params(&origin);
        }

        let selected_ids = state.document.selected_ids.clone();
        if !selected_ids.is_empty() {
            state.document.snapshot();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    if let Element::Rect(r) = el {
                        r.corner_style = style;
                    }
                }
            }
            self.drawing_area.queue_draw();
        }
        state.notify_status();
    }

    pub fn update_shape_origin(&self, new_origin: crate::core::ShapeOrigin) {
        let mut state = self.state.borrow_mut();
        if let Some(feat) = state.plugin_manager.active_feature_mut() {
            feat.set_shape_params(&new_origin);
        }

        let selected_ids = state.document.selected_ids.clone();
        if !selected_ids.is_empty() {
            let has_matching = state.document.elements.iter().any(|el| {
                if selected_ids.contains(&el.id()) {
                    if let Element::Rect(_) = el {
                        if matches!(&new_origin, crate::core::ShapeOrigin::Rectangle { .. }) {
                            return true;
                        }
                    }
                    if let Element::Path(p) = el {
                        return match (&p.shape_origin, &new_origin) {
                            (
                                Some(crate::core::ShapeOrigin::Star { .. }),
                                crate::core::ShapeOrigin::Star { .. },
                            ) => true,
                            (
                                Some(crate::core::ShapeOrigin::Triangle { .. }),
                                crate::core::ShapeOrigin::Triangle { .. },
                            ) => true,
                            (
                                Some(crate::core::ShapeOrigin::Circle { .. }),
                                crate::core::ShapeOrigin::Circle { .. },
                            ) => true,
                            (
                                Some(crate::core::ShapeOrigin::Spiral { .. }),
                                crate::core::ShapeOrigin::Spiral { .. },
                            ) => true,
                            (
                                Some(crate::core::ShapeOrigin::Rectangle { .. }),
                                crate::core::ShapeOrigin::Rectangle { .. },
                            ) => true,
                            _ => false,
                        };
                    }
                }
                false
            });

            if has_matching {
                state.document.snapshot();
                for el in &mut state.document.elements {
                    if selected_ids.contains(&el.id()) {
                        if let Element::Rect(r) = el {
                            if let crate::core::ShapeOrigin::Rectangle {
                                corner_radius,
                                corner_radii,
                                corner_style,
                            } = &new_origin
                            {
                                r.corner_radius = *corner_radius;
                                r.corner_radii = *corner_radii;
                                r.corner_style = *corner_style;
                            }
                        }
                        if let Element::Path(p) = el {
                            let is_matching_variant = match (&p.shape_origin, &new_origin) {
                                (
                                    Some(crate::core::ShapeOrigin::Star { .. }),
                                    crate::core::ShapeOrigin::Star { .. },
                                ) => true,
                                (
                                    Some(crate::core::ShapeOrigin::Triangle { .. }),
                                    crate::core::ShapeOrigin::Triangle { .. },
                                ) => true,
                                (
                                    Some(crate::core::ShapeOrigin::Circle { .. }),
                                    crate::core::ShapeOrigin::Circle { .. },
                                ) => true,
                                (
                                    Some(crate::core::ShapeOrigin::Spiral { .. }),
                                    crate::core::ShapeOrigin::Spiral { .. },
                                ) => true,
                                (
                                    Some(crate::core::ShapeOrigin::Rectangle { .. }),
                                    crate::core::ShapeOrigin::Rectangle { .. },
                                ) => true,
                                _ => false,
                            };
                            if !is_matching_variant {
                                continue;
                            }

                            let rect = p.shape_rect.unwrap_or_else(|| p.bounds());

                            let old_id = p.id;
                            let fill = p.fill_color;
                            let stroke = p.stroke_color;
                            let sw = p.stroke_width;

                            let mut new_elem = match &new_origin {
                                crate::core::ShapeOrigin::Star {
                                    corner_radius,
                                    points,
                                    inner_ratio,
                                } => crate::plugins::features::star::StarFeature::create_star_path(
                                    rect,
                                    fill,
                                    stroke,
                                    sw,
                                    *points,
                                    *inner_ratio,
                                    *corner_radius,
                                ),
                                crate::core::ShapeOrigin::Triangle {
                                    corner_radius,
                                    sides,
                                } => crate::plugins::features::triangle::TriangleFeature::create_triangle_path(
                                    rect,
                                    fill,
                                    stroke,
                                    sw,
                                    *sides,
                                    *corner_radius,
                                ),
                                crate::core::ShapeOrigin::Circle {
                                    arc_mode,
                                    start_angle,
                                    end_angle,
                                } => crate::plugins::features::circle::CircleFeature::create_ellipse_path(
                                    rect,
                                    fill,
                                    stroke,
                                    sw,
                                    *arc_mode,
                                    *start_angle,
                                    *end_angle,
                                ),
                                crate::core::ShapeOrigin::Spiral {
                                    turns,
                                    divergence,
                                    inner_radius,
                                } => crate::plugins::features::spiral::SpiralFeature::create_spiral_path(
                                    rect,
                                    stroke,
                                    sw,
                                    *turns,
                                    *divergence,
                                    *inner_radius,
                                ),
                                crate::core::ShapeOrigin::Rectangle {
                                    corner_radius: _,
                                    corner_radii,
                                    corner_style,
                                } => {
                                    let mut rect_el = crate::core::RectElement::new(rect, fill, stroke);
                                    rect_el.stroke_width = sw;
                                    rect_el.corner_radii = *corner_radii;
                                    rect_el.corner_radius = corner_radii.max_radius();
                                    rect_el.corner_style = *corner_style;
                                    rect_el.to_path_element()
                                }
                            };
                            new_elem.id = old_id;
                            new_elem.shape_rect = Some(rect);
                            *p = new_elem;
                        }
                    }
                }
                self.drawing_area.queue_draw();
            }
        }
        state.notify_status();
    }

    pub fn set_selected_font_family(&self, family: &str) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_font_family(family);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_font_weight(&self, weight: u32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_font_weight(weight);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_font_size(&self, size: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_font_size(size);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_line_height(&self, line_height: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_line_height(line_height);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_letter_spacing(&self, spacing: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_letter_spacing(spacing);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_word_spacing(&self, spacing: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_word_spacing(spacing);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_align(&self, align: crate::core::TextAlign) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_align(align);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn get_selected_text_element(&self) -> Option<crate::core::element::TextElement> {
        if let Ok(state) = self.state.try_borrow() {
            state.document.get_selected_text_element().cloned()
        } else {
            None
        }
    }

    pub fn set_selected_text_box_width(&self, width: Option<f32>) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_box_width(width);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_kerning_offset(&self, offset: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_kerning_offset(offset);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_opentype_features(&self, features: crate::core::element::OpenTypeFeatures) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_opentype_features(features);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_case(&self, text_case: crate::core::element::TextCase) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_case(text_case);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn attach_selected_text_to_path(&self) -> bool {
        let mut state = self.state.borrow_mut();
        let attached = state.document.attach_selected_text_to_path();
        if attached {
            state.notify_status();
            drop(state);
            self.drawing_area.queue_draw();
        }
        attached
    }

    pub fn detach_selected_text_from_path(&self) -> bool {
        let mut state = self.state.borrow_mut();
        let detached = state.document.detach_selected_text_from_path();
        if detached {
            state.notify_status();
            drop(state);
            self.drawing_area.queue_draw();
        }
        detached
    }

    pub fn set_selected_text_path_offset(&self, offset: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_path_offset(offset);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_path_inverted(&self, inverted: bool) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_path_inverted(inverted);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_path_orientation(&self, orientation: bool) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_path_orientation(orientation);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_path_repeat(&self, repeat: bool) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_path_repeat(repeat);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_path_spacing(&self, spacing: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_path_spacing(spacing);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_underline(&self, underline: bool) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_underline(underline);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_strikethrough(&self, strikethrough: bool) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_strikethrough(strikethrough);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_baseline(&self, baseline: crate::core::element::TextBaseline) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_baseline(baseline);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn insert_symbol_to_selected_text(&self, symbol: &str) {
        let mut state = self.state.borrow_mut();
        state.document.insert_symbol_to_selected_text(symbol);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_path_valign(&self, valign: crate::core::element::PathVerticalAlign) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_path_valign(valign);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_path_glyph_orientation(&self, orientation: crate::core::element::PathGlyphOrientation) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_path_glyph_orientation(orientation);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn copy_selected_style(&self) -> bool {
        let mut state = self.state.borrow_mut();
        let sel_ids = state.document.selected_ids.clone();
        if let Some(&first_id) = sel_ids.iter().next() {
            if let Some(el) = state.document.find_element(first_id) {
                let snapshot = el.extract_style_snapshot();
                state.copied_style = Some(snapshot);
                return true;
            }
        }
        false
    }

    pub fn paste_style_to_selected(&self) -> bool {
        let mut state = self.state.borrow_mut();
        if let Some(style) = state.copied_style.clone() {
            if state.document.selected_ids.is_empty() {
                return false;
            }
            state.document.snapshot();
            let ids = state.document.selected_ids.clone();
            for id in ids {
                if let Some(el) = state.document.find_element_mut(id) {
                    el.apply_style_snapshot(&style);
                }
            }
            state.notify_status();
            self.drawing_area.queue_draw();
            return true;
        }
        false
    }

    pub fn can_attach_text_to_path(&self) -> bool {
        if let Ok(state) = self.state.try_borrow() {
            let elems: Vec<&crate::core::Element> = state
                .document
                .selected_ids
                .iter()
                .filter_map(|id| state.document.find_element(*id))
                .collect();
            let has_text = elems.iter().any(|e| matches!(e, crate::core::Element::Text(_)));
            let has_path = elems.iter().any(|e| !matches!(e, crate::core::Element::Text(_)));
            has_text && has_path
        } else {
            false
        }
    }

    pub fn can_detach_text_from_path(&self) -> bool {
        if let Ok(state) = self.state.try_borrow() {
            let elems: Vec<&crate::core::Element> = state
                .document
                .selected_ids
                .iter()
                .filter_map(|id| state.document.find_element(*id))
                .collect();
            elems.iter().any(|e| {
                if let crate::core::Element::Text(t) = e {
                    t.path_id.is_some()
                } else {
                    false
                }
            })
        } else {
            false
        }
    }
}
