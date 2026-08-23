use crate::core::element::{Element, ElementId};
use crate::core::geometry::{Point, Rect};
use super::Document;

impl Document {
    pub fn translate_selected(&mut self, dx: f32, dy: f32) {
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                el.translate(dx, dy);
            }
        }
    }

    pub fn flip_horizontal_selected(&mut self) {
        let bounds = match self.selection_bounds() {
            Some(b) => b,
            None => return,
        };
        self.snapshot();
        let center_x = bounds.x + bounds.width / 2.0;

        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                match el {
                    Element::Rect(r) => {
                        let cur_x = r.rect.x;
                        let w = r.rect.width;
                        r.rect.x = 2.0 * center_x - (cur_x + w);
                    }
                    Element::Brush(b) => {
                        for p in &mut b.points {
                            p.x = 2.0 * center_x - p.x;
                        }
                    }
                    Element::Path(path) => {
                        for node in &mut path.nodes {
                            node.point.x = 2.0 * center_x - node.point.x;
                            if let Some(ref mut h) = node.handle_in {
                                h.x = 2.0 * center_x - h.x;
                            }
                            if let Some(ref mut h) = node.handle_out {
                                h.x = 2.0 * center_x - h.x;
                            }
                            std::mem::swap(&mut node.handle_in, &mut node.handle_out);
                        }
                    }
                    Element::Text(t) => {
                        let b = t.bounds();
                        t.position.x = 2.0 * center_x - (b.x + b.width);
                    }
                    Element::Group(g) => {
                        g.scale(Point::new(center_x, bounds.y), -1.0, 1.0);
                    }
                    Element::Image(i) => {
                        let cur_x = i.rect.x;
                        let w = i.rect.width;
                        i.rect.x = 2.0 * center_x - (cur_x + w);
                    }
                    Element::Clone(c) => {
                        let b = c.bounds();
                        let target_x = 2.0 * center_x - (b.x + b.width);
                        c.translate(target_x - b.x, 0.0);
                        c.scale.x = -c.scale.x;
                    }
                }
            }
        }
    }

    pub fn flip_vertical_selected(&mut self) {
        let bounds = match self.selection_bounds() {
            Some(b) => b,
            None => return,
        };
        self.snapshot();
        let center_y = bounds.y + bounds.height / 2.0;

        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                match el {
                    Element::Rect(r) => {
                        let cur_y = r.rect.y;
                        let h = r.rect.height;
                        r.rect.y = 2.0 * center_y - (cur_y + h);
                    }
                    Element::Brush(b) => {
                        for p in &mut b.points {
                            p.y = 2.0 * center_y - p.y;
                        }
                    }
                    Element::Path(path) => {
                        for node in &mut path.nodes {
                            node.point.y = 2.0 * center_y - node.point.y;
                            if let Some(ref mut h) = node.handle_in {
                                h.y = 2.0 * center_y - h.y;
                            }
                            if let Some(ref mut h) = node.handle_out {
                                h.y = 2.0 * center_y - h.y;
                            }
                            std::mem::swap(&mut node.handle_in, &mut node.handle_out);
                        }
                    }
                    Element::Text(t) => {
                        let b = t.bounds();
                        t.position.y = 2.0 * center_y - (b.y + b.height);
                    }
                    Element::Group(g) => {
                        g.scale(Point::new(bounds.x, center_y), 1.0, -1.0);
                    }
                    Element::Image(i) => {
                        let cur_y = i.rect.y;
                        let h = i.rect.height;
                        i.rect.y = 2.0 * center_y - (cur_y + h);
                    }
                    Element::Clone(c) => {
                        let b = c.bounds();
                        let target_y = 2.0 * center_y - (b.y + b.height);
                        c.translate(0.0, target_y - b.y);
                        c.scale.y = -c.scale.y;
                    }
                }
            }
        }
    }

    pub fn align_selected_left(&mut self) {
        let bounds = match self.selection_bounds() {
            Some(b) => b,
            None => return,
        };
        self.snapshot();
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                let dx = bounds.x - el.bounds().x;
                el.translate(dx, 0.0);
            }
        }
    }

    pub fn align_selected_center_h(&mut self) {
        let bounds = match self.selection_bounds() {
            Some(b) => b,
            None => return,
        };
        self.snapshot();
        let target_center_x = bounds.x + bounds.width / 2.0;
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                let el_center_x = el.bounds().x + el.bounds().width / 2.0;
                let dx = target_center_x - el_center_x;
                el.translate(dx, 0.0);
            }
        }
    }

    pub fn align_selected_right(&mut self) {
        let bounds = match self.selection_bounds() {
            Some(b) => b,
            None => return,
        };
        self.snapshot();
        let target_right = bounds.x + bounds.width;
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                let el_right = el.bounds().x + el.bounds().width;
                let dx = target_right - el_right;
                el.translate(dx, 0.0);
            }
        }
    }

    pub fn align_selected_top(&mut self) {
        let bounds = match self.selection_bounds() {
            Some(b) => b,
            None => return,
        };
        self.snapshot();
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                let dy = bounds.y - el.bounds().y;
                el.translate(0.0, dy);
            }
        }
    }

    pub fn align_selected_middle(&mut self) {
        let bounds = match self.selection_bounds() {
            Some(b) => b,
            None => return,
        };
        self.snapshot();
        let target_center_y = bounds.y + bounds.height / 2.0;
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                let el_center_y = el.bounds().y + el.bounds().height / 2.0;
                let dy = target_center_y - el_center_y;
                el.translate(0.0, dy);
            }
        }
    }

    pub fn align_selected_bottom(&mut self) {
        let bounds = match self.selection_bounds() {
            Some(b) => b,
            None => return,
        };
        self.snapshot();
        let target_bottom = bounds.y + bounds.height;
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                let el_bottom = el.bounds().y + el.bounds().height;
                let dy = target_bottom - el_bottom;
                el.translate(0.0, dy);
            }
        }
    }

    pub fn align_selected_center_v(&mut self) {
        self.align_selected_middle();
    }

    pub fn distribute_selected_horizontally(&mut self) {
        let selected: Vec<usize> = self
            .elements
            .iter()
            .enumerate()
            .filter(|(_, el)| self.selected_ids.contains(&el.id()))
            .map(|(i, _)| i)
            .collect();
        if selected.len() < 3 {
            return;
        }
        self.snapshot();

        let mut indexed: Vec<(usize, f32, f32)> = selected
            .iter()
            .map(|&i| {
                let b = self.elements[i].bounds();
                (i, b.x, b.width)
            })
            .collect();
        indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let first_x = indexed.first().unwrap().1;
        let last = indexed.last().unwrap();
        let last_right = last.1 + last.2;
        let total_width: f32 = indexed.iter().map(|e| e.2).sum();
        let gap = (last_right - first_x - total_width) / (indexed.len() as f32 - 1.0);

        let mut cursor = first_x + indexed[0].2 + gap;
        for item in indexed.iter().skip(1).take(indexed.len() - 2) {
            let dx = cursor - item.1;
            self.elements[item.0].translate(dx, 0.0);
            cursor += item.2 + gap;
        }
    }

    pub fn distribute_selected_vertically(&mut self) {
        let selected: Vec<usize> = self
            .elements
            .iter()
            .enumerate()
            .filter(|(_, el)| self.selected_ids.contains(&el.id()))
            .map(|(i, _)| i)
            .collect();
        if selected.len() < 3 {
            return;
        }
        self.snapshot();

        let mut indexed: Vec<(usize, f32, f32)> = selected
            .iter()
            .map(|&i| {
                let b = self.elements[i].bounds();
                (i, b.y, b.height)
            })
            .collect();
        indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let first_y = indexed.first().unwrap().1;
        let last = indexed.last().unwrap();
        let last_bottom = last.1 + last.2;
        let total_height: f32 = indexed.iter().map(|e| e.2).sum();
        let gap = (last_bottom - first_y - total_height) / (indexed.len() as f32 - 1.0);

        let mut cursor = first_y + indexed[0].2 + gap;
        for item in indexed.iter().skip(1).take(indexed.len() - 2) {
            let dy = cursor - item.1;
            self.elements[item.0].translate(0.0, dy);
            cursor += item.2 + gap;
        }
    }

    pub fn scale_selected(&mut self, origin: Point, sx: f32, sy: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                el.scale(origin, sx, sy);
            }
        }
    }

    pub fn rotate_selected(&mut self, center: Point, angle_rad: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                el.rotate(center, angle_rad);
            }
        }
    }

    pub fn arrange_selected_in_grid(&mut self, rows: usize, cols: usize, gap_x: f32, gap_y: f32) {
        if self.selected_ids.len() < 2 || rows == 0 || cols == 0 {
            return;
        }

        self.snapshot();

        // Collect selected elements and their current bounds
        let mut items: Vec<(ElementId, Rect)> = self
            .elements
            .iter()
            .filter(|el| self.selected_ids.contains(&el.id()))
            .map(|el| (el.id(), el.bounds()))
            .collect();

        if items.is_empty() {
            return;
        }

        // Sort items spatially (top-to-bottom, left-to-right)
        items.sort_by(|a, b| {
            let y_diff = a.1.y - b.1.y;
            if y_diff.abs() > 10.0 {
                y_diff
                    .partial_cmp(&0.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                a.1.x
                    .partial_cmp(&b.1.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        let min_x = items.iter().map(|(_, r)| r.x).fold(f32::INFINITY, f32::min);
        let min_y = items.iter().map(|(_, r)| r.y).fold(f32::INFINITY, f32::min);

        let max_w = items.iter().map(|(_, r)| r.width).fold(0.0_f32, f32::max);
        let max_h = items.iter().map(|(_, r)| r.height).fold(0.0_f32, f32::max);

        for (index, (id, current_rect)) in items.iter().enumerate() {
            let row = index / cols;
            let col = index % cols;

            let target_x = min_x + col as f32 * (max_w + gap_x);
            let target_y = min_y + row as f32 * (max_h + gap_y);

            let dx = target_x - current_rect.x;
            let dy = target_y - current_rect.y;

            if let Some(el) = self.elements.iter_mut().find(|e| e.id() == *id) {
                el.translate(dx, dy);
            }
        }
    }

    pub fn arrange_selected_circular(&mut self, radius: f32, start_angle_deg: f32) {
        if self.selected_ids.len() < 2 {
            return;
        }

        self.snapshot();

        let items: Vec<(ElementId, Rect)> = self
            .elements
            .iter()
            .filter(|el| self.selected_ids.contains(&el.id()))
            .map(|el| (el.id(), el.bounds()))
            .collect();

        let count = items.len();
        if count == 0 {
            return;
        }

        let min_x = items.iter().map(|(_, r)| r.x).fold(f32::INFINITY, f32::min);
        let max_x = items
            .iter()
            .map(|(_, r)| r.x + r.width)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = items.iter().map(|(_, r)| r.y).fold(f32::INFINITY, f32::min);
        let max_y = items
            .iter()
            .map(|(_, r)| r.y + r.height)
            .fold(f32::NEG_INFINITY, f32::max);

        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;

        let angle_step = std::f32::consts::TAU / count as f32;
        let start_rad = start_angle_deg.to_radians();

        for (i, (id, current_rect)) in items.iter().enumerate() {
            let angle = start_rad + i as f32 * angle_step;
            let target_center_x = center_x + radius * angle.cos();
            let target_center_y = center_y + radius * angle.sin();

            let target_x = target_center_x - current_rect.width / 2.0;
            let target_y = target_center_y - current_rect.height / 2.0;

            let dx = target_x - current_rect.x;
            let dy = target_y - current_rect.y;

            if let Some(el) = self.elements.iter_mut().find(|e| e.id() == *id) {
                el.translate(dx, dy);
            }
        }
    }
}
