use skia_safe as skia;
use std::collections::HashSet;

use crate::core::{
    dist_to_segment, Element, ElementId, HandleDisplayMode, KeyEvent, NodeType, PathElement, Point,
    PointerButton, PointerEvent, Rect, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditTarget {
    Node {
        element_id: ElementId,
        node_idx: usize,
    },
    HandleIn {
        element_id: ElementId,
        node_idx: usize,
    },
    HandleOut {
        element_id: ElementId,
        node_idx: usize,
    },
    Segment {
        element_id: ElementId,
        seg_idx: usize,
        insert_pos: Point,
        t: f32,
    },
    SegmentDrag {
        element_id: ElementId,
        seg_idx: usize,
        start_world: Point,
        t: f32,
    },
}

pub struct PathEditorFeature {
    active_target: Option<EditTarget>,
    selected_nodes: HashSet<(ElementId, usize)>,
    hover_target: Option<EditTarget>,
    last_drag_pos: Option<Point>,
    box_select_start: Option<Point>,
    box_select_current: Option<Point>,
    panning_last_screen: Option<Point>,
    is_dragging_segment: bool,
}

impl Default for PathEditorFeature {
    fn default() -> Self {
        Self {
            active_target: None,
            selected_nodes: HashSet::new(),
            hover_target: None,
            last_drag_pos: None,
            box_select_start: None,
            box_select_current: None,
            panning_last_screen: None,
            is_dragging_segment: false,
        }
    }
}

/// Computes distance to a cubic Bézier curve segment and returns (min_dist, best_t)
fn dist_to_bezier_segment_with_t(
    pt: Point,
    p0: Point,
    h0: Option<Point>,
    h1: Option<Point>,
    p3: Point,
) -> (f32, f32) {
    let p1 = h0.unwrap_or(p0);
    let p2 = h1.unwrap_or(p3);

    if p1 == p0 && p2 == p3 {
        let d = dist_to_segment(pt, p0, p3);
        let len_sq = (p3.x - p0.x) * (p3.x - p0.x) + (p3.y - p0.y) * (p3.y - p0.y);
        let t = if len_sq > 0.001 {
            let dot = (pt.x - p0.x) * (p3.x - p0.x) + (pt.y - p0.y) * (p3.y - p0.y);
            (dot / len_sq).clamp(0.0, 1.0)
        } else {
            0.5
        };
        return (d, t);
    }

    let mut min_dist = f32::MAX;
    let mut best_t = 0.5;
    let mut prev = p0;
    for step in 1..=24 {
        let t = step as f32 / 24.0;
        let it = 1.0 - t;
        let curr = Point::new(
            it * it * it * p0.x
                + 3.0 * it * it * t * p1.x
                + 3.0 * it * t * t * p2.x
                + t * t * t * p3.x,
            it * it * it * p0.y
                + 3.0 * it * it * t * p1.y
                + 3.0 * it * t * t * p2.y
                + t * t * t * p3.y,
        );
        let d = dist_to_segment(pt, prev, curr);
        if d < min_dist {
            min_dist = d;
            best_t = t - 0.5 / 24.0;
        }
        prev = curr;
    }
    (min_dist, best_t.clamp(0.0, 1.0))
}

impl PathEditorFeature {
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes effective or corner-projected handles for a node so handles are always available
    fn compute_node_handles(p: &PathElement, node_idx: usize) -> (Point, Point) {
        let count = p.nodes.len();
        let pt = p.nodes[node_idx].point;
        if count < 2 {
            return (Point::new(pt.x - 20.0, pt.y), Point::new(pt.x + 20.0, pt.y));
        }

        let node = &p.nodes[node_idx];

        let h_in = if let Some(h) = node.handle_in {
            h
        } else {
            let prev_pt = if node_idx > 0 {
                p.nodes[node_idx - 1].point
            } else if p.is_closed {
                p.nodes[count - 1].point
            } else {
                Point::new(
                    pt.x - (p.nodes[1].point.x - pt.x),
                    pt.y - (p.nodes[1].point.y - pt.y),
                )
            };
            Point::new(
                pt.x + (prev_pt.x - pt.x) * 0.3,
                pt.y + (prev_pt.y - pt.y) * 0.3,
            )
        };

        let h_out = if let Some(h) = node.handle_out {
            h
        } else {
            let next_pt = if node_idx + 1 < count {
                p.nodes[node_idx + 1].point
            } else if p.is_closed {
                p.nodes[0].point
            } else {
                Point::new(
                    pt.x - (p.nodes[count - 2].point.x - pt.x),
                    pt.y - (p.nodes[count - 2].point.y - pt.y),
                )
            };
            Point::new(
                pt.x + (next_pt.x - pt.x) * 0.3,
                pt.y + (next_pt.y - pt.y) * 0.3,
            )
        };

        (h_in, h_out)
    }

    fn should_show_handles(
        &self,
        handle_mode: HandleDisplayMode,
        is_node_selected: bool,
        is_path_selected: bool,
    ) -> bool {
        match handle_mode {
            HandleDisplayMode::SelectedOnly => is_node_selected,
            HandleDisplayMode::AllInSelectedPath => is_path_selected,
            HandleDisplayMode::Always => true,
        }
    }

    fn find_hit_target(&self, ctx: &PluginContext, world_pos: Point) -> Option<EditTarget> {
        let zoom = ctx.viewport.zoom.max(0.001);
        let tolerance = ctx.path_editor_config.hit_tolerance;
        let handle_threshold = (tolerance * 1.2) / zoom;
        let node_threshold = (tolerance * 1.3) / zoom;
        let segment_threshold = tolerance / zoom;
        let handle_mode = ctx.path_editor_config.handle_display_mode;

        // 1. Check Handles of candidate nodes
        for el in &ctx.document.elements {
            if let Element::Path(p) = el {
                let is_path_selected = ctx.document.is_selected(p.id);
                for (i, node) in p.nodes.iter().enumerate() {
                    let is_node_selected = self.selected_nodes.contains(&(p.id, i));
                    let show_handles = self.should_show_handles(handle_mode, is_node_selected, is_path_selected);

                    if show_handles || node.handle_in.is_some() || node.handle_out.is_some() {
                        let (h_in, h_out) = Self::compute_node_handles(p, i);
                        if let Some(hin) = node.handle_in.or(if show_handles { Some(h_in) } else { None }) {
                            if hin.distance_to(world_pos) <= handle_threshold {
                                return Some(EditTarget::HandleIn {
                                    element_id: p.id,
                                    node_idx: i,
                                });
                            }
                        }
                        if let Some(hout) = node.handle_out.or(if show_handles { Some(h_out) } else { None }) {
                            if hout.distance_to(world_pos) <= handle_threshold {
                                return Some(EditTarget::HandleOut {
                                    element_id: p.id,
                                    node_idx: i,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2. Check Anchor Nodes of selected paths
        for el in &ctx.document.elements {
            if let Element::Path(p) = el {
                if ctx.document.is_selected(p.id) {
                    for (i, node) in p.nodes.iter().enumerate() {
                        if node.point.distance_to(world_pos) <= node_threshold {
                            return Some(EditTarget::Node {
                                element_id: p.id,
                                node_idx: i,
                            });
                        }
                    }
                }
            }
        }

        // 3. Check Anchor Nodes of unselected paths
        for el in &ctx.document.elements {
            if let Element::Path(p) = el {
                if !ctx.document.is_selected(p.id) {
                    for (i, node) in p.nodes.iter().enumerate() {
                        if node.point.distance_to(world_pos) <= node_threshold {
                            return Some(EditTarget::Node {
                                element_id: p.id,
                                node_idx: i,
                            });
                        }
                    }
                }
            }
        }

        // 4. Check Segments / Curves of selected paths
        for el in &ctx.document.elements {
            if let Element::Path(p) = el {
                if ctx.document.is_selected(p.id) && p.nodes.len() >= 2 {
                    let count = p.nodes.len();
                    let loop_len = if p.is_closed { count } else { count - 1 };
                    for i in 0..loop_len {
                        let n0 = &p.nodes[i];
                        let n1 = &p.nodes[(i + 1) % count];
                        let (_h_in_0, h_out_0) = Self::compute_node_handles(p, i);
                        let (h_in_1, _h_out_1) = Self::compute_node_handles(p, (i + 1) % count);
                        let h0 = n0.handle_out.or(if n0.handle_in.is_some() { Some(h_out_0) } else { None });
                        let h1 = n1.handle_in.or(if n1.handle_out.is_some() { Some(h_in_1) } else { None });
                        let (dist, t) = dist_to_bezier_segment_with_t(world_pos, n0.point, h0, h1, n1.point);
                        if dist <= segment_threshold {
                            return Some(EditTarget::Segment {
                                element_id: p.id,
                                seg_idx: i,
                                insert_pos: world_pos,
                                t,
                            });
                        }
                    }
                }
            }
        }

        // 5. Check Segments / Curves of unselected paths
        for el in &ctx.document.elements {
            if let Element::Path(p) = el {
                if !ctx.document.is_selected(p.id) && p.nodes.len() >= 2 {
                    let count = p.nodes.len();
                    let loop_len = if p.is_closed { count } else { count - 1 };
                    for i in 0..loop_len {
                        let n0 = &p.nodes[i];
                        let n1 = &p.nodes[(i + 1) % count];
                        let (_h_in_0, h_out_0) = Self::compute_node_handles(p, i);
                        let (h_in_1, _h_out_1) = Self::compute_node_handles(p, (i + 1) % count);
                        let h0 = n0.handle_out.or(if n0.handle_in.is_some() { Some(h_out_0) } else { None });
                        let h1 = n1.handle_in.or(if n1.handle_out.is_some() { Some(h_in_1) } else { None });
                        let (dist, t) = dist_to_bezier_segment_with_t(world_pos, n0.point, h0, h1, n1.point);
                        if dist <= segment_threshold {
                            return Some(EditTarget::Segment {
                                element_id: p.id,
                                seg_idx: i,
                                insert_pos: world_pos,
                                t,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Snap handle angle to discrete increments if angle_snapping_step > 0
    fn snap_handle_angle(anchor: Point, handle: Point, step_deg: f32) -> Point {
        if step_deg <= 0.0 {
            return handle;
        }

        let dx = handle.x - anchor.x;
        let dy = handle.y - anchor.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return handle;
        }

        let angle_rad = dy.atan2(dx);
        let step_rad = step_deg.to_radians();
        let snapped_angle = (angle_rad / step_rad).round() * step_rad;

        Point::new(
            anchor.x + snapped_angle.cos() * len,
            anchor.y + snapped_angle.sin() * len,
        )
    }
}

impl FeaturePlugin for PathEditorFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        self.selected_nodes.clear();
        self.active_target = None;
        self.hover_target = None;
        ctx.set_cursor("tool:node");
        ctx.request_redraw();
    }

    fn get_selected_nodes(&self) -> Vec<(ElementId, usize)> {
        self.selected_nodes.iter().copied().collect()
    }

    fn set_selected_nodes(&mut self, nodes: Vec<(ElementId, usize)>) {
        self.selected_nodes = nodes.into_iter().collect();
    }

    fn clear_selected_nodes(&mut self) {
        self.selected_nodes.clear();
    }

    fn select_all_nodes(&mut self, ctx: &mut PluginContext) {
        self.selected_nodes.clear();
        for el in &ctx.document.elements {
            if let Element::Path(p) = el {
                if ctx.document.is_selected(p.id) {
                    for i in 0..p.nodes.len() {
                        self.selected_nodes.insert((p.id, i));
                    }
                }
            }
        }
        ctx.request_redraw();
    }

    fn on_double_click(&mut self, ctx: &mut PluginContext, event: &PointerEvent) -> bool {
        if event.button != Some(PointerButton::Primary) {
            return false;
        }

        let hit = self.find_hit_target(ctx, event.world_pos);
        match hit {
            Some(EditTarget::Node {
                element_id,
                node_idx,
            }) => {
                // Double click on node: Toggle between Corner and Smooth!
                ctx.document.snapshot();
                for el in &mut ctx.document.elements {
                    if let Element::Path(p) = el {
                        if p.id == element_id && !p.nodes.is_empty() {
                            p.toggle_node_smooth_corner(node_idx);
                            ctx.request_redraw();
                            return true;
                        }
                    }
                }
            }
            Some(EditTarget::Segment {
                element_id,
                seg_idx,
                t,
                ..
            }) => {
                // Double click on segment: Insert new node using De Casteljau subdivision!
                ctx.document.snapshot();
                for el in &mut ctx.document.elements {
                    if let Element::Path(p) = el {
                        if p.id == element_id {
                            let inserted_idx = p.insert_node_at_segment(seg_idx, t);
                            self.selected_nodes.clear();
                            self.selected_nodes.insert((element_id, inserted_idx));
                            ctx.request_redraw();
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }

        false
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button == Some(PointerButton::Middle) {
            self.panning_last_screen = Some(event.screen_pos);
            ctx.set_cursor("grabbing");
            ctx.request_redraw();
            return;
        }

        if event.button != Some(PointerButton::Primary) {
            return;
        }

        if let Some(target) = self.find_hit_target(ctx, event.world_pos) {
            self.last_drag_pos = Some(event.world_pos);

            match target {
                EditTarget::Node {
                    element_id,
                    node_idx,
                } => {
                    self.active_target = Some(target);
                    ctx.document.select(element_id, event.shift_pressed);
                    if event.shift_pressed {
                        if self.selected_nodes.contains(&(element_id, node_idx)) {
                            self.selected_nodes.remove(&(element_id, node_idx));
                        } else {
                            self.selected_nodes.insert((element_id, node_idx));
                        }
                    } else if !self.selected_nodes.contains(&(element_id, node_idx)) {
                        self.selected_nodes.clear();
                        self.selected_nodes.insert((element_id, node_idx));
                    }
                    ctx.document.snapshot();
                    ctx.set_cursor("move");
                }
                EditTarget::HandleIn {
                    element_id,
                    node_idx,
                }
                | EditTarget::HandleOut {
                    element_id,
                    node_idx,
                } => {
                    self.active_target = Some(target);
                    self.selected_nodes.clear();
                    self.selected_nodes.insert((element_id, node_idx));

                    // Materialize handles if currently None
                    for el in &mut ctx.document.elements {
                        if let Element::Path(p) = el {
                            if p.id == element_id {
                                let (h_in, h_out) = Self::compute_node_handles(p, node_idx);
                                if let Some(node) = p.nodes.get_mut(node_idx) {
                                    if node.handle_in.is_none() {
                                        node.handle_in = Some(h_in);
                                    }
                                    if node.handle_out.is_none() {
                                        node.handle_out = Some(h_out);
                                    }
                                }
                            }
                        }
                    }
                    ctx.document.snapshot();
                    ctx.set_cursor("grab");
                }
                EditTarget::Segment {
                    element_id,
                    seg_idx,
                    insert_pos,
                    t,
                } => {
                    ctx.document.select(element_id, false);

                    if ctx.path_editor_config.enable_direct_segment_drag {
                        // Start direct Bézier curve segment dragging
                        self.is_dragging_segment = true;
                        self.active_target = Some(EditTarget::SegmentDrag {
                            element_id,
                            seg_idx,
                            start_world: insert_pos,
                            t,
                        });
                        ctx.document.snapshot();
                        ctx.set_cursor("grabbing");
                    } else {
                        // Click adds a point and selects it
                        ctx.document.snapshot();
                        for el in &mut ctx.document.elements {
                            if let Element::Path(p) = el {
                                if p.id == element_id {
                                    let inserted = p.insert_node_at_segment(seg_idx, t);
                                    self.selected_nodes.clear();
                                    self.selected_nodes.insert((element_id, inserted));
                                    self.active_target = Some(EditTarget::Node {
                                        element_id,
                                        node_idx: inserted,
                                    });
                                    break;
                                }
                            }
                        }
                        ctx.set_cursor("move");
                    }
                }
                _ => {}
            }

            ctx.request_redraw();
            return;
        }

        // Element hit test or start box selection
        if let Some(hit_id) = ctx.document.hit_test(event.world_pos) {
            ctx.document.select(hit_id, event.shift_pressed);
            self.selected_nodes.clear();
            self.active_target = None;
            self.last_drag_pos = None;
        } else {
            if !event.shift_pressed {
                ctx.document.deselect_all();
                self.selected_nodes.clear();
            }
            self.box_select_start = Some(event.world_pos);
            self.box_select_current = Some(event.world_pos);
            self.active_target = None;
            self.last_drag_pos = None;
        }

        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        // Panning
        if let Some(ref mut last_screen) = self.panning_last_screen {
            let dx = event.screen_pos.x - last_screen.x;
            let dy = event.screen_pos.y - last_screen.y;
            ctx.viewport.pan_by(dx, dy);
            *last_screen = event.screen_pos;
            ctx.request_redraw();
            return;
        }

        // Box selecting nodes
        if let (Some(start), Some(cur)) =
            (self.box_select_start, &mut self.box_select_current)
        {
            *cur = event.world_pos;
            let marquee = Rect::from_points(start, *cur).normalize();

            for el in &ctx.document.elements {
                if let Element::Path(p) = el {
                    if ctx.document.is_selected(p.id) {
                        for (i, node) in p.nodes.iter().enumerate() {
                            if marquee.contains(node.point) {
                                self.selected_nodes.insert((p.id, i));
                            }
                        }
                    }
                }
            }

            ctx.set_cursor("crosshair");
            ctx.request_redraw();
            return;
        }

        // Direct Segment Dragging (Adjusting Bézier handles on the fly)
        if let (Some(EditTarget::SegmentDrag { element_id, seg_idx, t, .. }), Some(last_pos)) =
            (self.active_target, self.last_drag_pos)
        {
            let dx = event.world_pos.x - last_pos.x;
            let dy = event.world_pos.y - last_pos.y;

            for el in &mut ctx.document.elements {
                if let Element::Path(p) = el {
                    if p.id == element_id && seg_idx < p.nodes.len() {
                        p.bend_segment(seg_idx, Point::new(dx, dy), t);
                        break;
                    }
                }
            }

            self.last_drag_pos = Some(event.world_pos);
            ctx.set_cursor("grabbing");
            ctx.request_redraw();
            return;
        }

        // Dragging active target (Nodes or Handles)
        if let (Some(target), Some(last_pos)) = (self.active_target, self.last_drag_pos) {
            let mut dx = event.world_pos.x - last_pos.x;
            let mut dy = event.world_pos.y - last_pos.y;

            if matches!(target, EditTarget::Node { .. }) {
                let empty_exclude = HashSet::new();
                let snapped = ctx.snap_point(event.world_pos, &empty_exclude);
                dx = snapped.x - last_pos.x;
                dy = snapped.y - last_pos.y;
            }

            let angle_step = if event.shift_pressed {
                ctx.path_editor_config.angle_snapping_step
            } else {
                0.0
            };

            for el in &mut ctx.document.elements {
                if let Element::Path(p) = el {
                    match target {
                        EditTarget::Node { element_id, .. } if p.id == element_id => {
                            for &(sel_id, sel_idx) in &self.selected_nodes {
                                if sel_id == p.id {
                                    if let Some(node) = p.nodes.get_mut(sel_idx) {
                                        node.translate(dx, dy);
                                    }
                                }
                            }
                        }
                        EditTarget::HandleIn {
                            element_id,
                            node_idx,
                        } if p.id == element_id => {
                            if let Some(node) = p.nodes.get_mut(node_idx) {
                                let target_pos = Self::snap_handle_angle(node.point, event.world_pos, angle_step);
                                node.handle_in = Some(target_pos);

                                if event.alt_pressed {
                                    node.node_type = NodeType::Corner;
                                } else if node.is_symmetric() {
                                    let v_in = Point::new(node.point.x - target_pos.x, node.point.y - target_pos.y);
                                    node.handle_out = Some(Point::new(node.point.x + v_in.x, node.point.y + v_in.y));
                                } else if node.is_smooth() || node.is_auto() {
                                    let v_in = Point::new(node.point.x - target_pos.x, node.point.y - target_pos.y);
                                    let len_out = node.handle_out.map_or(target_pos.distance_to(node.point), |h| h.distance_to(node.point));
                                    let len_in = (v_in.x * v_in.x + v_in.y * v_in.y).sqrt().max(0.001);
                                    let unit = Point::new(v_in.x / len_in, v_in.y / len_in);
                                    node.handle_out = Some(Point::new(node.point.x + unit.x * len_out, node.point.y + unit.y * len_out));
                                }
                            }
                        }
                        EditTarget::HandleOut {
                            element_id,
                            node_idx,
                        } if p.id == element_id => {
                            if let Some(node) = p.nodes.get_mut(node_idx) {
                                let target_pos = Self::snap_handle_angle(node.point, event.world_pos, angle_step);
                                node.handle_out = Some(target_pos);

                                if event.alt_pressed {
                                    node.node_type = NodeType::Corner;
                                } else if node.is_symmetric() {
                                    let v_out = Point::new(node.point.x - target_pos.x, node.point.y - target_pos.y);
                                    node.handle_in = Some(Point::new(node.point.x + v_out.x, node.point.y + v_out.y));
                                } else if node.is_smooth() || node.is_auto() {
                                    let v_out = Point::new(node.point.x - target_pos.x, node.point.y - target_pos.y);
                                    let len_in = node.handle_in.map_or(target_pos.distance_to(node.point), |h| h.distance_to(node.point));
                                    let len_out = (v_out.x * v_out.x + v_out.y * v_out.y).sqrt().max(0.001);
                                    let unit = Point::new(v_out.x / len_out, v_out.y / len_out);
                                    node.handle_in = Some(Point::new(node.point.x + unit.x * len_in, node.point.y + unit.y * len_in));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            self.last_drag_pos = Some(event.world_pos);
            ctx.set_cursor("move");
            ctx.request_redraw();
            return;
        }

        // Hover tracking
        let hit = self.find_hit_target(ctx, event.world_pos);
        self.hover_target = hit;

        match hit {
            Some(EditTarget::HandleIn { .. }) | Some(EditTarget::HandleOut { .. }) => {
                ctx.set_cursor("grab");
            }
            Some(EditTarget::Node { .. }) => {
                ctx.set_cursor("tool:node");
            }
            Some(EditTarget::Segment { .. }) => {
                if ctx.path_editor_config.enable_direct_segment_drag {
                    ctx.set_cursor("tool:node_curve");
                } else {
                    ctx.set_cursor("tool:node_add");
                }
            }
            _ => {
                if ctx.document.hit_test(event.world_pos).is_some() {
                    ctx.set_cursor("pointer");
                } else {
                    ctx.set_cursor("tool:node");
                }
            }
        }
        ctx.request_redraw();
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        self.active_target = None;
        self.last_drag_pos = None;
        self.box_select_start = None;
        self.box_select_current = None;
        self.panning_last_screen = None;
        self.is_dragging_segment = false;
        ctx.clear_snap_guides();
        ctx.set_cursor("tool:node");
        ctx.request_redraw();
    }

    fn on_key_down(&mut self, ctx: &mut PluginContext, event: &KeyEvent) -> bool {
        // Delete / Backspace
        if (event.key == gtk4::gdk::Key::Delete || event.key == gtk4::gdk::Key::BackSpace)
            && !self.selected_nodes.is_empty()
        {
            let nodes_vec: Vec<(ElementId, usize)> = self.selected_nodes.iter().copied().collect();
            ctx.document.delete_selected_nodes_op(&nodes_vec);
            self.selected_nodes.clear();
            ctx.request_redraw();
            return true;
        }

        // Tab / Shift+Tab cycling
        if event.key == gtk4::gdk::Key::Tab {
            for el in &ctx.document.elements {
                if let Element::Path(p) = el {
                    if ctx.document.is_selected(p.id) && !p.nodes.is_empty() {
                        let cur_idx = self
                            .selected_nodes
                            .iter()
                            .find(|(id, _)| *id == p.id)
                            .map(|(_, idx)| *idx)
                            .unwrap_or(0);
                        let next_idx = if event.shift_pressed {
                            (cur_idx + p.nodes.len() - 1) % p.nodes.len()
                        } else {
                            (cur_idx + 1) % p.nodes.len()
                        };
                        self.selected_nodes.clear();
                        self.selected_nodes.insert((p.id, next_idx));
                        ctx.request_redraw();
                        return true;
                    }
                }
            }
        }

        // Ctrl+A select all nodes in active path
        if event.ctrl_pressed && (event.key == gtk4::gdk::Key::a || event.key == gtk4::gdk::Key::A) {
            self.select_all_nodes(ctx);
            return true;
        }

        // Arrow keys nudge
        let (dx, dy) = match event.key {
            gtk4::gdk::Key::Left => (-1.0, 0.0),
            gtk4::gdk::Key::Right => (1.0, 0.0),
            gtk4::gdk::Key::Up => (0.0, -1.0),
            gtk4::gdk::Key::Down => (0.0, 1.0),
            _ => (0.0, 0.0),
        };

        if (dx != 0.0 || dy != 0.0) && !self.selected_nodes.is_empty() {
            let mult = if event.shift_pressed { 10.0 } else { 1.0 };
            ctx.document.snapshot();
            for el in &mut ctx.document.elements {
                if let Element::Path(p) = el {
                    for &(sel_id, sel_idx) in &self.selected_nodes {
                        if sel_id == p.id {
                            if let Some(node) = p.nodes.get_mut(sel_idx) {
                                node.translate(dx * mult, dy * mult);
                            }
                        }
                    }
                }
            }
            ctx.request_redraw();
            return true;
        }

        if event.key == gtk4::gdk::Key::Escape {
            self.selected_nodes.clear();
            ctx.request_redraw();
            return true;
        }

        false
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.active_target = None;
        self.selected_nodes.clear();
        self.hover_target = None;
        self.last_drag_pos = None;
        self.box_select_start = None;
        self.box_select_current = None;
        self.panning_last_screen = None;
        self.is_dragging_segment = false;
        ctx.clear_snap_guides();
        ctx.set_cursor("tool:node");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        let zoom = viewport.zoom.max(0.001);
        let config = ctx.path_editor_config;

        let node_size = config.node_size / zoom;
        let half = node_size / 2.0;
        let corner_radius = 2.0 / zoom;
        let handle_radius = (config.handle_size * 0.95) / zoom;
        let stroke_w = 1.5 / zoom;

        // Paints
        let mut shadow_paint = skia::Paint::default();
        shadow_paint.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.35), None);
        shadow_paint.set_style(skia::PaintStyle::Fill);
        shadow_paint.set_anti_alias(true);

        let mut path_outline_paint = skia::Paint::default();
        path_outline_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.85), None);
        path_outline_paint.set_style(skia::PaintStyle::Stroke);
        path_outline_paint.set_stroke_width(1.2 / zoom);
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

        let shadow_offset = Point::new(0.8 / zoom, 1.2 / zoom);

        // 1. Draw Paths, Nodes, and Handles for selected objects
        for el in &ctx.document.elements {
            if let Element::Path(p) = el {
                let is_path_selected = ctx.document.is_selected(p.id);
                if is_path_selected {
                    if config.show_path_outline {
                        let sk_path = p.to_skia_path();
                        canvas.draw_path(&sk_path, &path_outline_paint);
                    }

                    // Highlight hovered segment if applicable
                    if let Some(EditTarget::Segment { element_id, seg_idx, .. }) = self.hover_target {
                        if element_id == p.id && seg_idx < p.nodes.len() && config.highlight_hovered_segment {
                            let count = p.nodes.len();
                            let next_idx = (seg_idx + 1) % count;
                            let n0 = &p.nodes[seg_idx];
                            let n1 = &p.nodes[next_idx];
                            let (_h_in_0, h_out_0) = Self::compute_node_handles(p, seg_idx);
                            let (h_in_1, _h_out_1) = Self::compute_node_handles(p, next_idx);
                            let h0 = n0.handle_out.or(if n0.handle_in.is_some() { Some(h_out_0) } else { None }).unwrap_or(n0.point);
                            let h1 = n1.handle_in.or(if n1.handle_out.is_some() { Some(h_in_1) } else { None }).unwrap_or(n1.point);

                            let mut seg_path = skia::PathBuilder::new();
                            seg_path.move_to(n0.point.to_skia());
                            seg_path.cubic_to(h0.to_skia(), h1.to_skia(), n1.point.to_skia());

                            let mut seg_glow = skia::Paint::default();
                            seg_glow.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.5), None);
                            seg_glow.set_style(skia::PaintStyle::Stroke);
                            seg_glow.set_stroke_width(3.5 / zoom);
                            seg_glow.set_anti_alias(true);
                            canvas.draw_path(&seg_path.detach(), &seg_glow);
                        }
                    }

                    for (i, node) in p.nodes.iter().enumerate() {
                        let pt = node.point;
                        let is_node_selected = self.selected_nodes.contains(&(p.id, i));
                        let show_handles = self.should_show_handles(config.handle_display_mode, is_node_selected, is_path_selected);

                        // Draw Bézier handles
                        if show_handles
                            || matches!(self.hover_target, Some(EditTarget::Node { element_id, node_idx }) if element_id == p.id && node_idx == i)
                        {
                            let (h_in, h_out) = Self::compute_node_handles(p, i);

                            // Handle Out
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

                            if matches!(self.hover_target, Some(EditTarget::HandleOut { element_id, node_idx }) if element_id == p.id && node_idx == i)
                            {
                                canvas.draw_circle(h_out.to_skia(), handle_radius * 1.7, &hover_ring);
                            }

                            // Handle In
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

                            if matches!(self.hover_target, Some(EditTarget::HandleIn { element_id, node_idx }) if element_id == p.id && node_idx == i)
                            {
                                canvas.draw_circle(h_in.to_skia(), handle_radius * 1.7, &hover_ring);
                            }
                        }

                        // Draw Anchor Node
                        let rect = skia::Rect::from_xywh(pt.x - half, pt.y - half, node_size, node_size);
                        let shadow_rect = skia::Rect::from_xywh(
                            pt.x - half + shadow_offset.x,
                            pt.y - half + shadow_offset.y,
                            node_size,
                            node_size,
                        );

                        // Selection glow halo
                        if is_node_selected {
                            canvas.draw_circle(pt.to_skia(), half * 1.75, &selected_glow);
                        }

                        if config.show_distinct_node_shapes {
                            match node.node_type {
                                NodeType::Corner => {
                                    // Sleek Rounded Rectangle for Corner nodes
                                    canvas.draw_round_rect(shadow_rect, corner_radius, corner_radius, &shadow_paint);

                                    if is_node_selected {
                                        canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_selected_fill);
                                        canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_selected_stroke);
                                    } else {
                                        canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_fill);
                                        canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_stroke);
                                    }
                                }
                                NodeType::Smooth => {
                                    // Sleek Circular Anchor for Smooth nodes
                                    canvas.draw_circle(
                                        Point::new(pt.x + shadow_offset.x, pt.y + shadow_offset.y).to_skia(),
                                        half * 1.05,
                                        &shadow_paint,
                                    );

                                    if is_node_selected {
                                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_selected_fill);
                                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_selected_stroke);
                                    } else {
                                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_fill);
                                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_stroke);
                                    }
                                }
                                NodeType::Symmetric | NodeType::Auto => {
                                    // Circle with center accent core for Symmetric & Auto nodes
                                    canvas.draw_circle(
                                        Point::new(pt.x + shadow_offset.x, pt.y + shadow_offset.y).to_skia(),
                                        half * 1.05,
                                        &shadow_paint,
                                    );

                                    if is_node_selected {
                                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_selected_fill);
                                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_selected_stroke);
                                        canvas.draw_circle(pt.to_skia(), half * 0.40, &node_fill);
                                    } else {
                                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_fill);
                                        canvas.draw_circle(pt.to_skia(), half * 1.05, &node_stroke);
                                        canvas.draw_circle(pt.to_skia(), half * 0.40, &node_stroke);
                                    }
                                }
                            }
                        } else {
                            // Uniform Rounded Rectangle
                            canvas.draw_round_rect(shadow_rect, corner_radius, corner_radius, &shadow_paint);

                            if is_node_selected {
                                canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_selected_fill);
                                canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_selected_stroke);
                            } else {
                                canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_fill);
                                canvas.draw_round_rect(rect, corner_radius, corner_radius, &node_stroke);
                            }
                        }

                        // Distinct start indicator for open paths
                        if !p.is_closed && i == 0 {
                            let mut start_badge = skia::Paint::default();
                            start_badge.set_color4f(skia::Color4f::new(0.18, 0.80, 0.44, 0.9), None);
                            start_badge.set_style(skia::PaintStyle::Fill);
                            start_badge.set_anti_alias(true);
                            canvas.draw_circle(pt.to_skia(), half * 0.4, &start_badge);
                        }

                        // Hover ring on node
                        if matches!(self.hover_target, Some(EditTarget::Node { element_id, node_idx }) if element_id == p.id && node_idx == i)
                        {
                            canvas.draw_circle(pt.to_skia(), half * 1.7, &hover_ring);
                        }
                    }

                    // Optional: Draw path direction arrows
                    if config.show_path_direction && p.nodes.len() >= 2 {
                        let count = p.nodes.len();
                        let loop_len = if p.is_closed { count } else { count - 1 };
                        for i in 0..loop_len {
                            let n0 = p.nodes[i].point;
                            let n1 = p.nodes[(i + 1) % count].point;
                            let mid = Point::new((n0.x + n1.x) / 2.0, (n0.y + n1.y) / 2.0);
                            let v = Point::new(n1.x - n0.x, n1.y - n0.y);
                            let len = (v.x * v.x + v.y * v.y).sqrt();
                            if len > 20.0 / zoom {
                                let u = Point::new(v.x / len, v.y / len);
                                let perp = Point::new(-u.y, u.x);
                                let arrow_len = 5.0 / zoom;
                                let mut arrow = skia::PathBuilder::new();
                                arrow.move_to(Point::new(mid.x + u.x * arrow_len, mid.y + u.y * arrow_len).to_skia());
                                arrow.line_to(Point::new(mid.x - u.x * arrow_len + perp.x * arrow_len * 0.6, mid.y - u.y * arrow_len + perp.y * arrow_len * 0.6).to_skia());
                                arrow.line_to(Point::new(mid.x - u.x * arrow_len - perp.x * arrow_len * 0.6, mid.y - u.y * arrow_len - perp.y * arrow_len * 0.6).to_skia());
                                arrow.close();
                                canvas.draw_path(&arrow.detach(), &node_selected_fill);
                            }
                        }
                    }
                }
            }
        }

        // 2. Draw add-point indicator (+) when hovering over a curve segment
        if let Some(EditTarget::Segment { insert_pos, .. }) = self.hover_target {
            if config.highlight_hovered_segment {
                let mut insert_preview_fill = skia::Paint::default();
                insert_preview_fill.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.95), None);
                insert_preview_fill.set_style(skia::PaintStyle::Fill);
                insert_preview_fill.set_anti_alias(true);

                canvas.draw_circle(
                    Point::new(insert_pos.x + shadow_offset.x, insert_pos.y + shadow_offset.y).to_skia(),
                    half * 1.1,
                    &shadow_paint,
                );
                canvas.draw_circle(insert_pos.to_skia(), half * 1.1, &insert_preview_fill);
                canvas.draw_circle(insert_pos.to_skia(), half * 1.7, &hover_ring);

                let mut cross_paint = skia::Paint::default();
                cross_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                cross_paint.set_style(skia::PaintStyle::Stroke);
                cross_paint.set_stroke_width(1.5 / zoom);
                cross_paint.set_anti_alias(true);
                let cross_len = half * 0.55;
                canvas.draw_line(
                    Point::new(insert_pos.x - cross_len, insert_pos.y).to_skia(),
                    Point::new(insert_pos.x + cross_len, insert_pos.y).to_skia(),
                    &cross_paint,
                );
                canvas.draw_line(
                    Point::new(insert_pos.x, insert_pos.y - cross_len).to_skia(),
                    Point::new(insert_pos.x, insert_pos.y + cross_len).to_skia(),
                    &cross_paint,
                );
            }
        }

        // 3. Draw Marquee Box for multi-node selection
        if let (Some(start), Some(cur)) = (self.box_select_start, self.box_select_current) {
            let r = Rect::from_points(start, cur).round();

            let mut fill_paint = skia::Paint::default();
            fill_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.12), None);
            fill_paint.set_style(skia::PaintStyle::Fill);

            let mut stroke_paint = skia::Paint::default();
            stroke_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.85), None);
            stroke_paint.set_style(skia::PaintStyle::Stroke);
            stroke_paint.set_stroke_width(1.0 / zoom);
            stroke_paint.set_anti_alias(true);

            canvas.draw_rect(r.to_skia(), &fill_paint);
            canvas.draw_rect(r.to_skia(), &stroke_paint);
        }
    }
}
