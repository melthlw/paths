use crate::core::element::{Element, ElementId};
use crate::core::geometry::Point;
use crate::core::path_editor_config::NodeType;
use super::Document;

impl Document {
    /// Convert selected nodes to specified NodeType on selected paths
    pub fn set_selected_nodes_type(
        &mut self,
        selected_nodes: &[(ElementId, usize)],
        node_type: NodeType,
    ) {
        if selected_nodes.is_empty() {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if let Element::Path(p) = el {
                let indices: Vec<usize> = selected_nodes
                    .iter()
                    .filter(|(id, _)| *id == p.id)
                    .map(|(_, idx)| *idx)
                    .collect();
                if !indices.is_empty() {
                    p.convert_nodes_type(&indices, node_type);
                }
            }
        }
    }

    /// Convert segments for selected nodes to straight lines
    pub fn make_selected_segments_straight(&mut self, selected_nodes: &[(ElementId, usize)]) {
        if selected_nodes.is_empty() {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if let Element::Path(p) = el {
                let indices: Vec<usize> = selected_nodes
                    .iter()
                    .filter(|(id, _)| *id == p.id)
                    .map(|(_, idx)| *idx)
                    .collect();
                if !indices.is_empty() {
                    p.make_segments_straight(&indices);
                }
            }
        }
    }

    /// Convert segments for selected nodes to curves
    pub fn make_selected_segments_curve(&mut self, selected_nodes: &[(ElementId, usize)]) {
        if selected_nodes.is_empty() {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if let Element::Path(p) = el {
                let indices: Vec<usize> = selected_nodes
                    .iter()
                    .filter(|(id, _)| *id == p.id)
                    .map(|(_, idx)| *idx)
                    .collect();
                if !indices.is_empty() {
                    p.make_segments_curve(&indices);
                }
            }
        }
    }

    /// Insert nodes between selected nodes or at midpoint
    pub fn insert_nodes_between_selected(
        &mut self,
        selected_nodes: &[(ElementId, usize)],
    ) -> Vec<(ElementId, usize)> {
        let mut new_selected = Vec::new();
        if selected_nodes.is_empty() {
            return new_selected;
        }
        self.snapshot();

        for el in &mut self.elements {
            if let Element::Path(p) = el {
                let mut indices: Vec<usize> = selected_nodes
                    .iter()
                    .filter(|(id, _)| *id == p.id)
                    .map(|(_, idx)| *idx)
                    .collect();
                indices.sort_unstable_by(|a, b| b.cmp(a));

                for idx in indices {
                    if idx < p.nodes.len() {
                        let inserted = p.insert_node_at_segment(idx, 0.5);
                        new_selected.push((p.id, inserted));
                    }
                }
            }
        }
        new_selected
    }

    /// Delete selected nodes and clean up empty paths
    pub fn delete_selected_nodes_op(&mut self, selected_nodes: &[(ElementId, usize)]) {
        if selected_nodes.is_empty() {
            return;
        }
        self.snapshot();
        let mut to_remove_elements = Vec::new();

        for el in &mut self.elements {
            if let Element::Path(p) = el {
                let indices: Vec<usize> = selected_nodes
                    .iter()
                    .filter(|(id, _)| *id == p.id)
                    .map(|(_, idx)| *idx)
                    .collect();
                if !indices.is_empty() {
                    p.delete_nodes(&indices);
                    if p.nodes.is_empty() {
                        to_remove_elements.push(p.id);
                    }
                }
            }
        }

        for id in to_remove_elements {
            self.elements.retain(|e| e.id() != id);
            self.selected_ids.remove(&id);
        }
    }

    /// Toggle closed state on active/selected paths
    pub fn toggle_selected_paths_closed(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if let Element::Path(p) = el {
                if self.selected_ids.contains(&p.id) {
                    p.toggle_closed();
                }
            }
        }
    }

    /// Reverse direction of selected paths
    pub fn reverse_selected_paths_direction(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if let Element::Path(p) = el {
                if self.selected_ids.contains(&p.id) {
                    p.reverse_direction();
                }
            }
        }
    }

    /// Align selected nodes horizontally
    pub fn align_selected_nodes_horizontal(&mut self, selected_nodes: &[(ElementId, usize)]) {
        if selected_nodes.len() < 2 {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if let Element::Path(p) = el {
                let indices: Vec<usize> = selected_nodes
                    .iter()
                    .filter(|(id, _)| *id == p.id)
                    .map(|(_, idx)| *idx)
                    .collect();
                if indices.len() >= 2 {
                    p.align_nodes_horizontal(&indices);
                }
            }
        }
    }

    /// Align selected nodes vertically
    pub fn align_selected_nodes_vertical(&mut self, selected_nodes: &[(ElementId, usize)]) {
        if selected_nodes.len() < 2 {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if let Element::Path(p) = el {
                let indices: Vec<usize> = selected_nodes
                    .iter()
                    .filter(|(id, _)| *id == p.id)
                    .map(|(_, idx)| *idx)
                    .collect();
                if indices.len() >= 2 {
                    p.align_nodes_vertical(&indices);
                }
            }
        }
    }

    /// Distribute selected nodes horizontally
    pub fn distribute_selected_nodes_horizontal(&mut self, selected_nodes: &[(ElementId, usize)]) {
        if selected_nodes.len() < 3 {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if let Element::Path(p) = el {
                let indices: Vec<usize> = selected_nodes
                    .iter()
                    .filter(|(id, _)| *id == p.id)
                    .map(|(_, idx)| *idx)
                    .collect();
                if indices.len() >= 3 {
                    p.distribute_nodes_horizontal(&indices);
                }
            }
        }
    }

    /// Distribute selected nodes vertically
    pub fn distribute_selected_nodes_vertical(&mut self, selected_nodes: &[(ElementId, usize)]) {
        if selected_nodes.len() < 3 {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if let Element::Path(p) = el {
                let indices: Vec<usize> = selected_nodes
                    .iter()
                    .filter(|(id, _)| *id == p.id)
                    .map(|(_, idx)| *idx)
                    .collect();
                if indices.len() >= 3 {
                    p.distribute_nodes_vertical(&indices);
                }
            }
        }
    }

    /// Get position of the first selected node for coordinate display
    pub fn get_selected_node_coord(&self, selected_nodes: &[(ElementId, usize)]) -> Option<Point> {
        if let Some(&(el_id, idx)) = selected_nodes.first() {
            for el in &self.elements {
                if let Element::Path(p) = el {
                    if p.id == el_id {
                        return p.nodes.get(idx).map(|n| n.point);
                    }
                }
            }
        }
        None
    }

    /// Set position of the selected node
    pub fn set_selected_node_coord(
        &mut self,
        selected_nodes: &[(ElementId, usize)],
        new_pos: Point,
    ) {
        if let Some(&(el_id, idx)) = selected_nodes.first() {
            self.snapshot();
            for el in &mut self.elements {
                if let Element::Path(p) = el {
                    if p.id == el_id {
                        p.set_node_position(idx, new_pos);
                        break;
                    }
                }
            }
        }
    }
}
