use super::CanvasWidget;
use crate::core::Point;
use gtk4::prelude::*;

impl CanvasWidget {
    pub fn convert_selected_to_path(&self) {
        let mut state = self.state.borrow_mut();
        if state.document.convert_selected_to_path() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn pen_undo_node(&self) {
        let mut state = self.state.borrow_mut();
        let key_event = crate::core::KeyEvent {
            key: gtk4::gdk::Key::BackSpace,
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        };
        let (handled, redraw) = state.on_key_pressed(&key_event);
        if handled || redraw {
            state.notify_status();
            drop(state);
            self.drawing_area.queue_draw();
        }
    }

    pub fn pen_finish_path(&self) {
        let mut state = self.state.borrow_mut();
        let key_event = crate::core::KeyEvent {
            key: gtk4::gdk::Key::Return,
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        };
        let (handled, redraw) = state.on_key_pressed(&key_event);
        if handled || redraw {
            state.notify_status();
            drop(state);
            self.drawing_area.queue_draw();
        }
    }

    pub fn pen_close_path(&self) {
        let mut state = self.state.borrow_mut();
        let key_event = crate::core::KeyEvent {
            key: gtk4::gdk::Key::c,
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        };
        let (handled, redraw) = state.on_key_pressed(&key_event);
        if handled || redraw {
            state.notify_status();
            drop(state);
            self.drawing_area.queue_draw();
        }
    }

    pub fn pen_resume_path(&self) {
        let mut state = self.state.borrow_mut();
        let mut open_path_info = None;
        for elem in &state.document.elements {
            if state.document.selected_ids.contains(&elem.id()) {
                if let crate::core::Element::Path(p) = elem {
                    if !p.is_closed && !p.nodes.is_empty() {
                        open_path_info = Some((p.id, p.nodes.clone()));
                        break;
                    }
                }
            }
        }

        if let Some((id, nodes)) = open_path_info {
            if let Some(feat) = state.plugin_manager.feature_by_id_mut("pen") {
                if let Some(pen) = feat.as_pen_feature_mut() {
                    pen.nodes = nodes;
                    pen.resuming_path_id = Some(id);
                    if let Some(last) = pen.nodes.last() {
                        pen.current_cursor = Some(last.point);
                    }
                }
            }
            state.notify_status();
            drop(state);
            self.drawing_area.queue_draw();
        }
    }

    pub fn flip_horizontal(&self) {
        let mut state = self.state.borrow_mut();
        state.document.flip_horizontal_selected();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn flip_vertical(&self) {
        let mut state = self.state.borrow_mut();
        state.document.flip_vertical_selected();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn rotate_selected_deg(&self, angle_deg: f32) {
        let mut state = self.state.borrow_mut();
        if let Some(bounds) = state.document.selection_bounds() {
            state.document.snapshot();
            let center = bounds.center();
            let angle_rad = angle_deg.to_radians();
            state.document.rotate_selected(center, angle_rad);
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn align_left(&self) {
        let mut state = self.state.borrow_mut();
        state.document.align_selected_left();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn align_right(&self) {
        let mut state = self.state.borrow_mut();
        state.document.align_selected_right();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn align_top(&self) {
        let mut state = self.state.borrow_mut();
        state.document.align_selected_top();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn align_bottom(&self) {
        let mut state = self.state.borrow_mut();
        state.document.align_selected_bottom();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn align_center_h(&self) {
        let mut state = self.state.borrow_mut();
        state.document.align_selected_center_h();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn align_center_v(&self) {
        let mut state = self.state.borrow_mut();
        state.document.align_selected_center_v();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn distribute_h(&self) {
        let mut state = self.state.borrow_mut();
        state.document.distribute_selected_horizontally();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn distribute_v(&self) {
        let mut state = self.state.borrow_mut();
        state.document.distribute_selected_vertically();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_x(&self, new_x: f32) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        if !selected_nodes.is_empty() {
            if let Some(pt) = state.document.get_selected_node_coord(&selected_nodes) {
                state
                    .document
                    .set_selected_node_coord(&selected_nodes, Point::new(new_x, pt.y));
                drop(state);
                self.notify_status();
                self.drawing_area.queue_draw();
                return;
            }
        }
        if let Some(bounds) = state.document.selection_bounds() {
            let dx = new_x - bounds.x;
            state.document.snapshot();
            state.document.translate_selected(dx, 0.0);
            drop(state);
            self.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn set_selected_y(&self, new_y: f32) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        if !selected_nodes.is_empty() {
            if let Some(pt) = state.document.get_selected_node_coord(&selected_nodes) {
                state
                    .document
                    .set_selected_node_coord(&selected_nodes, Point::new(pt.x, new_y));
                drop(state);
                self.notify_status();
                self.drawing_area.queue_draw();
                return;
            }
        }
        if let Some(bounds) = state.document.selection_bounds() {
            let dy = new_y - bounds.y;
            state.document.snapshot();
            state.document.translate_selected(0.0, dy);
            drop(state);
            self.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn set_selected_width(&self, new_w: f32, keep_aspect: bool) {
        let mut state = self.state.borrow_mut();
        if let Some(bounds) = state.document.selection_bounds() {
            if bounds.width > 0.0 {
                let sx = (new_w / bounds.width).max(0.001);
                let sy = if keep_aspect { sx } else { 1.0 };
                let origin = Point::new(bounds.x, bounds.y);
                state.document.snapshot();
                state.document.scale_selected(origin, sx, sy);
                state.notify_status();
                self.drawing_area.queue_draw();
            }
        }
    }

    pub fn set_selected_height(&self, new_h: f32, keep_aspect: bool) {
        let mut state = self.state.borrow_mut();
        if let Some(bounds) = state.document.selection_bounds() {
            if bounds.height > 0.0 {
                let sy = (new_h / bounds.height).max(0.001);
                let sx = if keep_aspect { sy } else { 1.0 };
                let origin = Point::new(bounds.x, bounds.y);
                state.document.snapshot();
                state.document.scale_selected(origin, sx, sy);
                state.notify_status();
                self.drawing_area.queue_draw();
            }
        }
    }

    pub fn arrange_selected_in_grid(&self, rows: usize, cols: usize, gap_x: f32, gap_y: f32) {
        let mut state = self.state.borrow_mut();
        state
            .document
            .arrange_selected_in_grid(rows, cols, gap_x, gap_y);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn arrange_selected_circular(&self, radius: f32, start_angle_deg: f32) {
        let mut state = self.state.borrow_mut();
        state
            .document
            .arrange_selected_circular(radius, start_angle_deg);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn apply_boolean_operation(&self, op: crate::core::document::BooleanOperation) {
        let mut state = self.state.borrow_mut();
        state.document.apply_boolean_operation(op);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_nodes_type(&self, node_type: crate::core::NodeType) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        state
            .document
            .set_selected_nodes_type(&selected_nodes, node_type);
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn make_selected_segments_straight(&self) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        state
            .document
            .make_selected_segments_straight(&selected_nodes);
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn make_selected_segments_curve(&self) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        state.document.make_selected_segments_curve(&selected_nodes);
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn insert_nodes_between_selected(&self) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        let new_sel = state
            .document
            .insert_nodes_between_selected(&selected_nodes);
        if !new_sel.is_empty() {
            if let Some(feat) = state.plugin_manager.feature_by_id_mut("path_editor") {
                feat.set_selected_nodes(new_sel);
            }
        }
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn delete_selected_nodes(&self) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        state.document.delete_selected_nodes_op(&selected_nodes);
        if let Some(feat) = state.plugin_manager.feature_by_id_mut("path_editor") {
            feat.clear_selected_nodes();
        }
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn toggle_selected_paths_closed(&self) {
        self.state
            .borrow_mut()
            .document
            .toggle_selected_paths_closed();
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn reverse_selected_paths_direction(&self) {
        self.state
            .borrow_mut()
            .document
            .reverse_selected_paths_direction();
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn align_selected_nodes_horizontal(&self) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        state
            .document
            .align_selected_nodes_horizontal(&selected_nodes);
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn align_selected_nodes_vertical(&self) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        state
            .document
            .align_selected_nodes_vertical(&selected_nodes);
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn distribute_selected_nodes_horizontal(&self) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        state
            .document
            .distribute_selected_nodes_horizontal(&selected_nodes);
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn distribute_selected_nodes_vertical(&self) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        state
            .document
            .distribute_selected_nodes_vertical(&selected_nodes);
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    #[allow(dead_code)]
    pub fn get_selected_node_coord(&self) -> Option<Point> {
        if let Ok(state) = self.state.try_borrow() {
            if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
                let nodes = feat.get_selected_nodes();
                state.document.get_selected_node_coord(&nodes)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn set_selected_node_coord(&self, new_pos: Point) {
        let mut state = self.state.borrow_mut();
        let selected_nodes = if let Some(feat) = state.plugin_manager.feature_by_id("path_editor") {
            feat.get_selected_nodes()
        } else {
            Vec::new()
        };
        state
            .document
            .set_selected_node_coord(&selected_nodes, new_pos);
        drop(state);
        self.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn transform_options(&self) -> crate::core::TransformOptions {
        self.state
            .try_borrow()
            .map(|s| s.transform_options)
            .unwrap_or_default()
    }

    pub fn set_scale_stroke_width(&self, enabled: bool) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.transform_options.scale_stroke_width = enabled;
        }
    }

    pub fn set_scale_corner_radii(&self, enabled: bool) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.transform_options.scale_corner_radii = enabled;
        }
    }

    pub fn set_move_gradients(&self, enabled: bool) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.transform_options.move_gradients = enabled;
        }
    }

    pub fn set_move_patterns(&self, enabled: bool) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.transform_options.move_patterns = enabled;
        }
    }
}
