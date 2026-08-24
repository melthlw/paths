use skia_safe as skia;

use super::rect::{CornerRadii, CornerStyle};
use super::style::{
    create_fill_paint, create_skia_gradient_shader, render_mesh_gradient, BlendMode, FillLayer,
    FillStyle, Gradient, MeshGradient, StrokeLayer, StrokeStyle,
};
use super::ElementId;
use crate::core::color::Color;
use crate::core::geometry::{Point, Rect};
use crate::core::path_editor_config::NodeType;

/// Vector Anchor Node with optional Bezier control handles
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PathNode {
    pub point: Point,
    pub handle_in: Option<Point>,
    pub handle_out: Option<Point>,
    #[serde(default)]
    pub node_type: NodeType,
}

impl PathNode {
    pub fn new(point: Point) -> Self {
        Self {
            point,
            handle_in: None,
            handle_out: None,
            node_type: NodeType::Corner,
        }
    }

    pub fn with_handles(point: Point, handle_in: Option<Point>, handle_out: Option<Point>) -> Self {
        let node_type = if handle_in.is_some() || handle_out.is_some() {
            NodeType::Smooth
        } else {
            NodeType::Corner
        };
        Self {
            point,
            handle_in,
            handle_out,
            node_type,
        }
    }


    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.point.x += dx;
        self.point.y += dy;
        if let Some(ref mut h) = self.handle_in {
            h.x += dx;
            h.y += dy;
        }
        if let Some(ref mut h) = self.handle_out {
            h.x += dx;
            h.y += dy;
        }
    }

    pub fn is_smooth(&self) -> bool {
        self.node_type == NodeType::Smooth
    }

    pub fn is_symmetric(&self) -> bool {
        self.node_type == NodeType::Symmetric
    }



    pub fn is_auto(&self) -> bool {
        self.node_type == NodeType::Auto
    }
}

/// Arc rendering mode for circle/ellipse shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ArcMode {
    /// Full circle/ellipse
    Full,
    /// Open arc (no connecting lines)
    Arc,
    /// Pie/wedge segment (lines from arc ends to center)
    Segment,
    /// Chord (straight line connecting arc endpoints)
    Chord,
}

impl Default for ArcMode {
    fn default() -> Self {
        Self::Full
    }
}

/// Parametric shape origin — preserves creation parameters for re-generation
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ShapeOrigin {
    Rectangle {
        corner_radius: f32,
        corner_radii: CornerRadii,
        corner_style: CornerStyle,
    },
    Triangle {
        corner_radius: f32,
        sides: u32,
    },
    Star {
        corner_radius: f32,
        points: u32,
        inner_ratio: f32,
    },
    Circle {
        arc_mode: ArcMode,
        start_angle: f32,
        end_angle: f32,
    },
    Spiral {
        turns: f32,
        divergence: f32,
        inner_radius: f32,
    },
}

/// Full Bezier Vector Path Element
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PathElement {
    pub id: ElementId,
    pub nodes: Vec<PathNode>,
    pub is_closed: bool,
    #[serde(default)]
    pub subpath_lengths: Vec<usize>,
    pub fills: Vec<FillLayer>,
    pub strokes: Vec<StrokeLayer>,
    pub fill_color: Option<Color>,
    pub stroke_color: Option<Color>,
    pub stroke_width: f32,
    pub gradient: Option<Gradient>,
    pub mesh_gradient: Option<MeshGradient>,
    /// Parametric shape origin for re-generation when parameters change
    pub shape_origin: Option<ShapeOrigin>,
    /// Base bounding box used when regenerating parametric shapes
    pub shape_rect: Option<Rect>,
    pub name: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur: f32,
}

impl PathElement {
    pub fn new(
        nodes: Vec<PathNode>,
        is_closed: bool,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
        stroke_width: f32,
    ) -> Self {
        let mut fills = Vec::new();
        if let Some(f) = fill_color {
            fills.push(FillLayer::new(f));
        }
        let mut strokes = Vec::new();
        if let Some(s) = stroke_color {
            strokes.push(StrokeLayer::new(s, stroke_width));
        }
        Self {
            id: ElementId::new(),
            nodes,
            is_closed,
            subpath_lengths: Vec::new(),
            fills,
            strokes,
            fill_color,
            stroke_color,
            stroke_width: stroke_width.max(1.0),
            gradient: None,
            mesh_gradient: None,
            shape_origin: None,
            shape_rect: None,
            name: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            blur: 0.0,
        }
    }

    pub fn new_compound(
        nodes: Vec<PathNode>,
        subpath_lengths: Vec<usize>,
        is_closed: bool,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
        stroke_width: f32,
    ) -> Self {
        let mut elem = Self::new(nodes, is_closed, fill_color, stroke_color, stroke_width);
        elem.subpath_lengths = subpath_lengths;
        elem
    }



    pub fn to_skia_path(&self) -> skia::Path {
        if self.nodes.is_empty() {
            return skia::PathBuilder::new().detach();
        }

        let mut builder = skia::PathBuilder::new();

        if !self.subpath_lengths.is_empty() {
            let mut offset = 0;
            for &len in &self.subpath_lengths {
                if len == 0 || offset >= self.nodes.len() {
                    continue;
                }
                let end = (offset + len).min(self.nodes.len());
                let sub = &self.nodes[offset..end];
                offset = end;

                builder.move_to(sub[0].point.to_skia());
                let count = sub.len();
                let loop_count = if self.is_closed {
                    count
                } else {
                    count.saturating_sub(1)
                };
                for i in 0..loop_count {
                    let n1 = &sub[i];
                    let n2 = &sub[(i + 1) % count];
                    match (n1.handle_out, n2.handle_in) {
                        (Some(h1), Some(h2)) => {
                            builder.cubic_to(h1.to_skia(), h2.to_skia(), n2.point.to_skia());
                        }
                        (Some(h1), None) => {
                            builder.quad_to(h1.to_skia(), n2.point.to_skia());
                        }
                        (None, Some(h2)) => {
                            builder.quad_to(h2.to_skia(), n2.point.to_skia());
                        }
                        (None, None) => {
                            builder.line_to(n2.point.to_skia());
                        }
                    }
                }
                if self.is_closed {
                    builder.close();
                }
            }
        } else {
            builder.move_to(self.nodes[0].point.to_skia());
            let count = self.nodes.len();
            let loop_count = if self.is_closed {
                count
            } else {
                count.saturating_sub(1)
            };

            for i in 0..loop_count {
                let n1 = &self.nodes[i];
                let n2 = &self.nodes[(i + 1) % count];

                match (n1.handle_out, n2.handle_in) {
                    (Some(h1), Some(h2)) => {
                        builder.cubic_to(h1.to_skia(), h2.to_skia(), n2.point.to_skia());
                    }
                    (Some(h1), None) => {
                        builder.quad_to(h1.to_skia(), n2.point.to_skia());
                    }
                    (None, Some(h2)) => {
                        builder.quad_to(h2.to_skia(), n2.point.to_skia());
                    }
                    (None, None) => {
                        builder.line_to(n2.point.to_skia());
                    }
                }
            }

            if self.is_closed {
                builder.close();
            }
        }

        builder.detach()
    }

    pub fn bounds(&self) -> Rect {
        if self.nodes.is_empty() {
            return Rect::ZERO;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for n in &self.nodes {
            min_x = min_x.min(n.point.x);
            min_y = min_y.min(n.point.y);
            max_x = max_x.max(n.point.x);
            max_y = max_y.max(n.point.y);

            if let Some(h) = n.handle_in {
                min_x = min_x.min(h.x);
                min_y = min_y.min(h.y);
                max_x = max_x.max(h.x);
                max_y = max_y.max(h.y);
            }
            if let Some(h) = n.handle_out {
                min_x = min_x.min(h.x);
                min_y = min_y.min(h.y);
                max_x = max_x.max(h.x);
                max_y = max_y.max(h.y);
            }
        }

        let margin = self.stroke_width / 2.0 + 2.0;
        Rect::new(
            min_x - margin,
            min_y - margin,
            (max_x - min_x) + margin * 2.0,
            (max_y - min_y) + margin * 2.0,
        )
    }

    pub fn hit_test(&self, p: Point) -> bool {
        let threshold = (self.stroke_width / 2.0 + 8.0).max(10.0);
        if !self.bounds().expand(threshold).contains(p) {
            return false;
        }

        let sk_path = self.to_skia_path();
        if self.is_closed || self.fill_color.is_some() {
            if sk_path.contains(p.to_skia()) {
                return true;
            }
        }

        for n in &self.nodes {
            if n.point.distance_to(p) <= threshold {
                return true;
            }
        }

        for i in 0..self.nodes.len().saturating_sub(1) {
            let p1 = self.nodes[i].point;
            let p2 = self.nodes[i + 1].point;
            if dist_to_segment(p, p1, p2) <= threshold {
                return true;
            }
        }

        if self.is_closed && self.nodes.len() > 2 {
            let p1 = self.nodes.last().unwrap().point;
            let p2 = self.nodes.first().unwrap().point;
            if dist_to_segment(p, p1, p2) <= threshold {
                return true;
            }
        }

        false
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        if let Some(r) = &mut self.shape_rect {
            r.x = (r.x + dx).round();
            r.y = (r.y + dy).round();
        }
        for n in &mut self.nodes {
            n.translate(dx, dy);
        }
        if let Some(g) = &mut self.gradient {
            g.translate(dx, dy);
        }
        if let Some(m) = &mut self.mesh_gradient {
            m.translate(dx, dy);
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        if let Some(r) = &mut self.shape_rect {
            let new_x = origin.x + (r.x - origin.x) * sx;
            let new_y = origin.y + (r.y - origin.y) * sy;
            let new_w = (r.width * sx.abs()).max(1.0);
            let new_h = (r.height * sy.abs()).max(1.0);
            *r = Rect::new(new_x, new_y, new_w, new_h).round();
        }
        for n in &mut self.nodes {
            n.point.x = origin.x + (n.point.x - origin.x) * sx;
            n.point.y = origin.y + (n.point.y - origin.y) * sy;
            if let Some(h) = &mut n.handle_in {
                h.x = origin.x + (h.x - origin.x) * sx;
                h.y = origin.y + (h.y - origin.y) * sy;
            }
            if let Some(h) = &mut n.handle_out {
                h.x = origin.x + (h.x - origin.x) * sx;
                h.y = origin.y + (h.y - origin.y) * sy;
            }
        }
        if let Some(g) = &mut self.gradient {
            g.scale(origin, sx, sy);
        }
        if let Some(m) = &mut self.mesh_gradient {
            m.scale(origin, sx, sy);
        }
    }

    pub fn rotate(&mut self, center: Point, angle_rad: f32) {
        for n in &mut self.nodes {
            n.point = crate::core::geometry::rotate_point(n.point, center, angle_rad);
            if let Some(h) = n.handle_in {
                n.handle_in = Some(crate::core::geometry::rotate_point(h, center, angle_rad));
            }
            if let Some(h) = n.handle_out {
                n.handle_out = Some(crate::core::geometry::rotate_point(h, center, angle_rad));
            }
        }
    }

    /// Set node type for specified node indices and adjust handles accordingly
    pub fn convert_nodes_type(&mut self, indices: &[usize], new_type: NodeType) {
        let count = self.nodes.len();
        if count == 0 {
            return;
        }

        for &idx in indices {
            if idx >= count {
                continue;
            }

            self.nodes[idx].node_type = new_type;

            match new_type {
                NodeType::Corner => {
                    // Handles become independent; existing positions are kept
                }
                NodeType::Smooth => {
                    let pt = self.nodes[idx].point;
                    let (h_in, h_out) = (self.nodes[idx].handle_in, self.nodes[idx].handle_out);
                    match (h_in, h_out) {
                        (Some(hin), Some(hout)) => {
                            let d_in = hin.distance_to(pt).max(0.1);
                            let d_out = hout.distance_to(pt).max(0.1);
                            let v_out = Point::new(hout.x - pt.x, hout.y - pt.y);
                            let len_out = (v_out.x * v_out.x + v_out.y * v_out.y).sqrt().max(0.001);
                            let u_out = Point::new(v_out.x / len_out, v_out.y / len_out);
                            self.nodes[idx].handle_out =
                                Some(Point::new(pt.x + u_out.x * d_out, pt.y + u_out.y * d_out));
                            self.nodes[idx].handle_in =
                                Some(Point::new(pt.x - u_out.x * d_in, pt.y - u_out.y * d_in));
                        }
                        (Some(hin), None) => {
                            let d_in = hin.distance_to(pt).max(0.1);
                            let v_in = Point::new(hin.x - pt.x, hin.y - pt.y);
                            let len_in = (v_in.x * v_in.x + v_in.y * v_in.y).sqrt().max(0.001);
                            let u_in = Point::new(v_in.x / len_in, v_in.y / len_in);
                            self.nodes[idx].handle_out =
                                Some(Point::new(pt.x - u_in.x * d_in, pt.y - u_in.y * d_in));
                        }
                        (None, Some(hout)) => {
                            let d_out = hout.distance_to(pt).max(0.1);
                            let v_out = Point::new(hout.x - pt.x, hout.y - pt.y);
                            let len_out = (v_out.x * v_out.x + v_out.y * v_out.y).sqrt().max(0.001);
                            let u_out = Point::new(v_out.x / len_out, v_out.y / len_out);
                            self.nodes[idx].handle_in =
                                Some(Point::new(pt.x - u_out.x * d_out, pt.y - u_out.y * d_out));
                        }
                        (None, None) => {
                            let prev_idx = if idx > 0 {
                                idx - 1
                            } else if self.is_closed {
                                count - 1
                            } else {
                                0
                            };
                            let next_idx = if idx + 1 < count {
                                idx + 1
                            } else if self.is_closed {
                                0
                            } else {
                                count - 1
                            };
                            let prev_p = self.nodes[prev_idx].point;
                            let next_p = self.nodes[next_idx].point;
                            let tangent = Point::new(
                                (next_p.x - prev_p.x) * 0.25,
                                (next_p.y - prev_p.y) * 0.25,
                            );
                            self.nodes[idx].handle_out =
                                Some(Point::new(pt.x + tangent.x, pt.y + tangent.y));
                            self.nodes[idx].handle_in =
                                Some(Point::new(pt.x - tangent.x, pt.y - tangent.y));
                        }
                    }
                }
                NodeType::Symmetric => {
                    let pt = self.nodes[idx].point;
                    let (h_in, h_out) = (self.nodes[idx].handle_in, self.nodes[idx].handle_out);
                    let avg_len = match (h_in, h_out) {
                        (Some(hin), Some(hout)) => {
                            (hin.distance_to(pt) + hout.distance_to(pt)) / 2.0
                        }
                        (Some(hin), None) => hin.distance_to(pt),
                        (None, Some(hout)) => hout.distance_to(pt),
                        (None, None) => {
                            let prev_idx = if idx > 0 {
                                idx - 1
                            } else if self.is_closed {
                                count - 1
                            } else {
                                0
                            };
                            let next_idx = if idx + 1 < count {
                                idx + 1
                            } else if self.is_closed {
                                0
                            } else {
                                count - 1
                            };
                            let prev_p = self.nodes[prev_idx].point;
                            let next_p = self.nodes[next_idx].point;
                            (prev_p.distance_to(next_p) * 0.25).max(15.0)
                        }
                    };

                    let u_dir = if let Some(hout) = h_out {
                        let v = Point::new(hout.x - pt.x, hout.y - pt.y);
                        let len = (v.x * v.x + v.y * v.y).sqrt().max(0.001);
                        Point::new(v.x / len, v.y / len)
                    } else if let Some(hin) = h_in {
                        let v = Point::new(pt.x - hin.x, pt.y - hin.y);
                        let len = (v.x * v.x + v.y * v.y).sqrt().max(0.001);
                        Point::new(v.x / len, v.y / len)
                    } else {
                        let prev_idx = if idx > 0 {
                            idx - 1
                        } else if self.is_closed {
                            count - 1
                        } else {
                            0
                        };
                        let next_idx = if idx + 1 < count {
                            idx + 1
                        } else if self.is_closed {
                            0
                        } else {
                            count - 1
                        };
                        let prev_p = self.nodes[prev_idx].point;
                        let next_p = self.nodes[next_idx].point;
                        let v = Point::new(next_p.x - prev_p.x, next_p.y - prev_p.y);
                        let len = (v.x * v.x + v.y * v.y).sqrt().max(0.001);
                        Point::new(v.x / len, v.y / len)
                    };

                    self.nodes[idx].handle_out = Some(Point::new(
                        pt.x + u_dir.x * avg_len,
                        pt.y + u_dir.y * avg_len,
                    ));
                    self.nodes[idx].handle_in = Some(Point::new(
                        pt.x - u_dir.x * avg_len,
                        pt.y - u_dir.y * avg_len,
                    ));
                }
                NodeType::Auto => {
                    self.recalculate_auto_handles(idx);
                }
            }
        }
    }

    /// Recalculates auto-smooth handles based on neighbor nodes
    pub fn recalculate_auto_handles(&mut self, idx: usize) {
        let count = self.nodes.len();
        if count < 2 || idx >= count {
            return;
        }

        let pt = self.nodes[idx].point;
        let prev_idx = if idx > 0 {
            idx - 1
        } else if self.is_closed {
            count - 1
        } else {
            0
        };
        let next_idx = if idx + 1 < count {
            idx + 1
        } else if self.is_closed {
            0
        } else {
            count - 1
        };

        let prev_p = self.nodes[prev_idx].point;
        let next_p = self.nodes[next_idx].point;

        let d_prev = pt.distance_to(prev_p);
        let d_next = pt.distance_to(next_p);

        let v = Point::new(next_p.x - prev_p.x, next_p.y - prev_p.y);
        let len = (v.x * v.x + v.y * v.y).sqrt().max(0.001);
        let u = Point::new(v.x / len, v.y / len);

        let scale = 0.3;
        let len_in = d_prev * scale;
        let len_out = d_next * scale;

        if idx > 0 || self.is_closed {
            self.nodes[idx].handle_in = Some(Point::new(pt.x - u.x * len_in, pt.y - u.y * len_in));
        } else {
            self.nodes[idx].handle_in = None;
        }

        if idx + 1 < count || self.is_closed {
            self.nodes[idx].handle_out =
                Some(Point::new(pt.x + u.x * len_out, pt.y + u.y * len_out));
        } else {
            self.nodes[idx].handle_out = None;
        }
    }

    /// Convert segments connecting selected nodes to straight lines (retract handles)
    pub fn make_segments_straight(&mut self, indices: &[usize]) {
        for &idx in indices {
            if idx < self.nodes.len() {
                self.nodes[idx].handle_out = None;
                self.nodes[idx].handle_in = None;
                self.nodes[idx].node_type = NodeType::Corner;
            }
        }
    }

    /// Convert segments connecting selected nodes to smooth curves
    pub fn make_segments_curve(&mut self, indices: &[usize]) {
        self.convert_nodes_type(indices, NodeType::Smooth);
    }

    /// Insert a new node at parameter t (0.0 .. 1.0) along segment seg_idx using De Casteljau subdivision
    pub fn insert_node_at_segment(&mut self, seg_idx: usize, t: f32) -> usize {
        let count = self.nodes.len();
        if count == 0 || seg_idx >= count {
            return 0;
        }

        let next_idx = (seg_idx + 1) % count;
        let p0 = self.nodes[seg_idx].point;
        let p3 = self.nodes[next_idx].point;
        let h0 = self.nodes[seg_idx].handle_out.unwrap_or(p0);
        let h1 = self.nodes[next_idx].handle_in.unwrap_or(p3);

        let clamp_t = t.clamp(0.05, 0.95);
        let it = 1.0 - clamp_t;

        let q0 = Point::new(it * p0.x + clamp_t * h0.x, it * p0.y + clamp_t * h0.y);
        let q1 = Point::new(it * h0.x + clamp_t * h1.x, it * h0.y + clamp_t * h1.y);
        let q2 = Point::new(it * h1.x + clamp_t * p3.x, it * h1.y + clamp_t * p3.y);

        let r0 = Point::new(it * q0.x + clamp_t * q1.x, it * q0.y + clamp_t * q1.y);
        let r1 = Point::new(it * q1.x + clamp_t * q2.x, it * q1.y + clamp_t * q2.y);

        let s = Point::new(it * r0.x + clamp_t * r1.x, it * r0.y + clamp_t * r1.y);

        self.nodes[seg_idx].handle_out = Some(q0);
        self.nodes[next_idx].handle_in = Some(q2);

        let mut new_node = PathNode::with_handles(s, Some(r0), Some(r1));
        new_node.node_type = NodeType::Smooth;

        let insert_idx = seg_idx + 1;
        self.nodes.insert(insert_idx, new_node);
        insert_idx
    }

    /// Toggle node between Smooth (extended tangent handles) and Corner (retracted handles)
    pub fn toggle_node_smooth_corner(&mut self, idx: usize) {
        let count = self.nodes.len();
        if count == 0 || idx >= count {
            return;
        }

        let is_currently_corner = self.nodes[idx].node_type == NodeType::Corner
            && self.nodes[idx].handle_in.is_none()
            && self.nodes[idx].handle_out.is_none();

        if is_currently_corner {
            self.convert_nodes_type(&[idx], NodeType::Smooth);
        } else {
            self.nodes[idx].handle_in = None;
            self.nodes[idx].handle_out = None;
            self.nodes[idx].node_type = NodeType::Corner;
        }
    }

    /// Directly deform/bend a segment by dragging along it
    pub fn bend_segment(&mut self, seg_idx: usize, delta: Point, t: f32) {
        let count = self.nodes.len();
        if count < 2 || seg_idx >= count {
            return;
        }

        let next_idx = (seg_idx + 1) % count;
        let p0 = self.nodes[seg_idx].point;
        let p3 = self.nodes[next_idx].point;

        let clamp_t = t.clamp(0.1, 0.9);
        let weight_h0 = (1.0 - clamp_t).max(0.2);
        let weight_h1 = clamp_t.max(0.2);

        let cur_h0 = self.nodes[seg_idx].handle_out.unwrap_or(Point::new(
            p0.x + (p3.x - p0.x) * 0.33,
            p0.y + (p3.y - p0.y) * 0.33,
        ));
        let cur_h1 = self.nodes[next_idx].handle_in.unwrap_or(Point::new(
            p3.x - (p3.x - p0.x) * 0.33,
            p3.y - (p3.y - p0.y) * 0.33,
        ));

        self.nodes[seg_idx].handle_out = Some(Point::new(
            cur_h0.x + delta.x * weight_h0 * 1.3,
            cur_h0.y + delta.y * weight_h0 * 1.3,
        ));
        self.nodes[next_idx].handle_in = Some(Point::new(
            cur_h1.x + delta.x * weight_h1 * 1.3,
            cur_h1.y + delta.y * weight_h1 * 1.3,
        ));
    }

    /// Delete nodes at specified indices
    pub fn delete_nodes(&mut self, indices: &[usize]) {
        let mut sorted: Vec<usize> = indices.to_vec();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        sorted.dedup();

        for idx in sorted {
            if idx < self.nodes.len() {
                self.nodes.remove(idx);
            }
        }
    }

    /// Toggle closed state of the path
    pub fn toggle_closed(&mut self) {
        self.is_closed = !self.is_closed;
    }

    /// Reverse direction of the path
    pub fn reverse_direction(&mut self) {
        self.nodes.reverse();
        for node in &mut self.nodes {
            std::mem::swap(&mut node.handle_in, &mut node.handle_out);
        }
    }

    /// Align selected nodes horizontally (set Y to average Y)
    pub fn align_nodes_horizontal(&mut self, indices: &[usize]) {
        let valid_indices: Vec<usize> = indices
            .iter()
            .copied()
            .filter(|&i| i < self.nodes.len())
            .collect();
        if valid_indices.len() < 2 {
            return;
        }

        let sum_y: f32 = valid_indices.iter().map(|&i| self.nodes[i].point.y).sum();
        let avg_y = (sum_y / valid_indices.len() as f32).round();

        for &i in &valid_indices {
            let dy = avg_y - self.nodes[i].point.y;
            self.nodes[i].translate(0.0, dy);
        }
    }

    /// Align selected nodes vertically (set X to average X)
    pub fn align_nodes_vertical(&mut self, indices: &[usize]) {
        let valid_indices: Vec<usize> = indices
            .iter()
            .copied()
            .filter(|&i| i < self.nodes.len())
            .collect();
        if valid_indices.len() < 2 {
            return;
        }

        let sum_x: f32 = valid_indices.iter().map(|&i| self.nodes[i].point.x).sum();
        let avg_x = (sum_x / valid_indices.len() as f32).round();

        for &i in &valid_indices {
            let dx = avg_x - self.nodes[i].point.x;
            self.nodes[i].translate(dx, 0.0);
        }
    }

    /// Distribute selected nodes evenly along horizontal axis
    pub fn distribute_nodes_horizontal(&mut self, indices: &[usize]) {
        let mut valid_indices: Vec<usize> = indices
            .iter()
            .copied()
            .filter(|&i| i < self.nodes.len())
            .collect();
        if valid_indices.len() < 3 {
            return;
        }

        valid_indices.sort_by(|&a, &b| {
            self.nodes[a]
                .point
                .x
                .partial_cmp(&self.nodes[b].point.x)
                .unwrap()
        });

        let min_x = self.nodes[*valid_indices.first().unwrap()].point.x;
        let max_x = self.nodes[*valid_indices.last().unwrap()].point.x;
        let step = (max_x - min_x) / (valid_indices.len() - 1) as f32;

        for (order, &idx) in valid_indices.iter().enumerate() {
            let target_x = (min_x + order as f32 * step).round();
            let dx = target_x - self.nodes[idx].point.x;
            self.nodes[idx].translate(dx, 0.0);
        }
    }

    /// Distribute selected nodes evenly along vertical axis
    pub fn distribute_nodes_vertical(&mut self, indices: &[usize]) {
        let mut valid_indices: Vec<usize> = indices
            .iter()
            .copied()
            .filter(|&i| i < self.nodes.len())
            .collect();
        if valid_indices.len() < 3 {
            return;
        }

        valid_indices.sort_by(|&a, &b| {
            self.nodes[a]
                .point
                .y
                .partial_cmp(&self.nodes[b].point.y)
                .unwrap()
        });

        let min_y = self.nodes[*valid_indices.first().unwrap()].point.y;
        let max_y = self.nodes[*valid_indices.last().unwrap()].point.y;
        let step = (max_y - min_y) / (valid_indices.len() - 1) as f32;

        for (order, &idx) in valid_indices.iter().enumerate() {
            let target_y = (min_y + order as f32 * step).round();
            let dy = target_y - self.nodes[idx].point.y;
            self.nodes[idx].translate(0.0, dy);
        }
    }

    /// Set coordinate for a single node
    pub fn set_node_position(&mut self, idx: usize, pt: Point) {
        if let Some(node) = self.nodes.get_mut(idx) {
            let dx = pt.x - node.point.x;
            let dy = pt.y - node.point.y;
            node.translate(dx, dy);
        }
    }

    pub fn from_skia_path(
        path: &skia::Path,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
        stroke_width: f32,
    ) -> Vec<PathElement> {
        let mut subpaths: Vec<Vec<PathNode>> = Vec::new();
        let mut current_nodes = Vec::new();
        let mut is_closed = false;

        let iter = skia::path::Iter::new(path, false);

        for (verb, points) in iter {
            match verb {
                skia::path::Verb::Move => {
                    if !current_nodes.is_empty() {
                        subpaths.push(std::mem::take(&mut current_nodes));
                    }
                    if !points.is_empty() {
                        current_nodes.push(PathNode::new(Point::new(points[0].x, points[0].y)));
                    }
                }
                skia::path::Verb::Line => {
                    if points.len() >= 2 {
                        current_nodes.push(PathNode::new(Point::new(points[1].x, points[1].y)));
                    }
                }
                skia::path::Verb::Quad => {
                    if points.len() >= 3 {
                        let p0 = points[0];
                        let p1 = points[1];
                        let p2 = points[2];
                        let cp1 = Point::new(
                            p0.x + (2.0 / 3.0) * (p1.x - p0.x),
                            p0.y + (2.0 / 3.0) * (p1.y - p0.y),
                        );
                        let cp2 = Point::new(
                            p2.x + (2.0 / 3.0) * (p1.x - p2.x),
                            p2.y + (2.0 / 3.0) * (p1.y - p2.y),
                        );

                        if let Some(last_node) = current_nodes.last_mut() {
                            last_node.handle_out = Some(cp1);
                        }
                        let mut new_node = PathNode::new(Point::new(p2.x, p2.y));
                        new_node.handle_in = Some(cp2);
                        current_nodes.push(new_node);
                    }
                }
                skia::path::Verb::Conic => {
                    if points.len() >= 3 {
                        let p0 = points[0];
                        let p1 = points[1];
                        let p2 = points[2];
                        let cp1 = Point::new(
                            p0.x + (2.0 / 3.0) * (p1.x - p0.x),
                            p0.y + (2.0 / 3.0) * (p1.y - p0.y),
                        );
                        let cp2 = Point::new(
                            p2.x + (2.0 / 3.0) * (p1.x - p2.x),
                            p2.y + (2.0 / 3.0) * (p1.y - p2.y),
                        );
                        if let Some(last_node) = current_nodes.last_mut() {
                            last_node.handle_out = Some(cp1);
                        }
                        let mut new_node = PathNode::new(Point::new(p2.x, p2.y));
                        new_node.handle_in = Some(cp2);
                        current_nodes.push(new_node);
                    }
                }
                skia::path::Verb::Cubic => {
                    if points.len() >= 4 {
                        let cp1 = Point::new(points[1].x, points[1].y);
                        let cp2 = Point::new(points[2].x, points[2].y);
                        let end_pt = Point::new(points[3].x, points[3].y);

                        if let Some(last_node) = current_nodes.last_mut() {
                            last_node.handle_out = Some(cp1);
                        }
                        let mut new_node = PathNode::new(end_pt);
                        new_node.handle_in = Some(cp2);
                        current_nodes.push(new_node);
                    }
                }
                skia::path::Verb::Close => {
                    is_closed = true;
                    if current_nodes.len() > 1 {
                        let first = current_nodes.first().unwrap().point;
                        let last = current_nodes.last().unwrap().point;
                        if (first.x - last.x).abs() < 0.001 && (first.y - last.y).abs() < 0.001 {
                            if current_nodes.len() > 2 {
                                let popped = current_nodes.pop().unwrap();
                                if let Some(first_node) = current_nodes.first_mut() {
                                    if first_node.handle_in.is_none() {
                                        first_node.handle_in = popped.handle_in;
                                    }
                                }
                            }
                        }
                    }
                    if !current_nodes.is_empty() {
                        subpaths.push(std::mem::take(&mut current_nodes));
                    }
                }
                skia::path::Verb::Done => break,
            }
        }

        if !current_nodes.is_empty() {
            subpaths.push(current_nodes);
        }

        if subpaths.is_empty() {
            return Vec::new();
        }

        if subpaths.len() == 1 {
            vec![PathElement::new(
                subpaths.into_iter().next().unwrap(),
                is_closed,
                fill_color,
                stroke_color,
                stroke_width,
            )]
        } else {
            let subpath_lengths = subpaths.iter().map(|s| s.len()).collect();
            let nodes = subpaths.into_iter().flatten().collect();
            vec![PathElement::new_compound(
                nodes,
                subpath_lengths,
                is_closed,
                fill_color,
                stroke_color,
                stroke_width,
            )]
        }
    }

    pub fn render(&self, canvas: &skia::Canvas) {
        let path = self.to_skia_path();

        // 1. Fills
        if let Some(mesh) = &self.mesh_gradient {
            render_mesh_gradient(canvas, &path, mesh);
        } else if let Some(grad) = &self.gradient {
            let mut paint = skia::Paint::default();
            paint.set_style(skia::PaintStyle::Fill);
            paint.set_anti_alias(true);
            if let Some(shader) = create_skia_gradient_shader(grad) {
                paint.set_shader(shader);
            } else if let Some(fill) = self.fill_color {
                paint.set_color4f(fill.to_skia(), None);
            }
            canvas.draw_path(&path, &paint);
        } else if !self.fills.is_empty() {
            let bounds = self.bounds();
            for fill in self.fills.iter().rev() {
                if !fill.enabled {
                    continue;
                }
                if fill.style == FillStyle::Mesh {
                    let mesh = fill
                        .mesh
                        .as_ref()
                        .or(self.mesh_gradient.as_ref())
                        .cloned()
                        .unwrap_or_else(|| {
                            MeshGradient::new_grid(bounds, 3, 3, fill.color, fill.secondary_color)
                        });
                    render_mesh_gradient(canvas, &path, &mesh);
                } else {
                    let paint = create_fill_paint(fill, bounds);
                    canvas.draw_path(&path, &paint);
                }
            }
        } else if let Some(fill) = self.fill_color {
            let mut paint = skia::Paint::default();
            paint.set_color4f(fill.to_skia(), None);
            paint.set_style(skia::PaintStyle::Fill);
            paint.set_anti_alias(true);
            canvas.draw_path(&path, &paint);
        }

        // 2. Strokes
        if !self.strokes.is_empty() {
            for stroke in self.strokes.iter().rev() {
                if !stroke.enabled || stroke.width <= 0.0 {
                    continue;
                }
                let mut paint = skia::Paint::default();
                let col = stroke.color.with_alpha(stroke.color.a * stroke.opacity);
                paint.set_color4f(col.to_skia(), None);
                paint.set_style(skia::PaintStyle::Stroke);
                paint.set_stroke_width(stroke.width);
                paint.set_stroke_cap(skia::PaintCap::Round);
                paint.set_stroke_join(skia::PaintJoin::Round);
                paint.set_anti_alias(true);
                match stroke.style {
                    StrokeStyle::Solid => {}
                    StrokeStyle::Dashed => {
                        if let Some(pe) =
                            skia::PathEffect::dash(&[stroke.width * 3.5, stroke.width * 2.0], 0.0)
                        {
                            paint.set_path_effect(pe);
                        }
                    }
                    StrokeStyle::Dotted => {
                        if let Some(pe) = skia::PathEffect::dash(&[0.1, stroke.width * 2.0], 0.0) {
                            paint.set_path_effect(pe);
                        }
                    }
                }
                canvas.draw_path(&path, &paint);
            }
        } else if let Some(stroke) = self.stroke_color {
            if self.stroke_width > 0.0 {
                let mut paint = skia::Paint::default();
                paint.set_color4f(stroke.to_skia(), None);
                paint.set_style(skia::PaintStyle::Stroke);
                paint.set_stroke_width(self.stroke_width);
                paint.set_stroke_cap(skia::PaintCap::Round);
                paint.set_stroke_join(skia::PaintJoin::Round);
                paint.set_anti_alias(true);
                canvas.draw_path(&path, &paint);
            }
        }
    }
}

pub fn dist_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let l2 = (a.x - b.x).powi(2) + (a.y - b.y).powi(2);
    if l2 == 0.0 {
        return p.distance_to(a);
    }
    let t = (((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2).clamp(0.0, 1.0);
    let projection = Point::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y));
    p.distance_to(projection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compound_path_to_skia_and_bounds() {
        // Outer square 0..100, inner cutout 25..75
        let outer = vec![
            PathNode::new(Point::new(0.0, 0.0)),
            PathNode::new(Point::new(100.0, 0.0)),
            PathNode::new(Point::new(100.0, 100.0)),
            PathNode::new(Point::new(0.0, 100.0)),
        ];
        let inner = vec![
            PathNode::new(Point::new(25.0, 25.0)),
            PathNode::new(Point::new(75.0, 25.0)),
            PathNode::new(Point::new(75.0, 75.0)),
            PathNode::new(Point::new(25.0, 75.0)),
        ];

        let compound =
            PathElement::new_from_subpaths(vec![outer, inner], true, Some(Color::BLACK), None, 1.0);

        assert_eq!(compound.subpath_lengths, vec![4, 4]);
        assert_eq!(compound.nodes.len(), 8);

        let sk_path = compound.to_skia_path();
        assert!(!sk_path.is_empty());

        let b = compound.bounds();
        assert!(b.width >= 100.0);
        assert!(b.height >= 100.0);
    }
}
