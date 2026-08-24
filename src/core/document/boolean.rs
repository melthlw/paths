use std::collections::HashSet;
use crate::core::element::{Element, PathElement};
use crate::core::geometry::Point;
use super::{BooleanOperation, Document};

impl Document {
    pub fn apply_boolean_operation(&mut self, op: BooleanOperation) {
        if self.selected_ids.len() < 2 {
            return;
        }

        // Get selected elements in document order
        let mut selected_indices = Vec::new();
        for (i, el) in self.elements.iter().enumerate() {
            if self.selected_ids.contains(&el.id()) {
                selected_indices.push(i);
            }
        }

        if selected_indices.len() < 2 {
            return;
        }

        self.snapshot();

        let base_idx = selected_indices[0];
        let base_elem = &self.elements[base_idx];
        let fill_color = base_elem.fill_color();
        let stroke_color = base_elem.stroke_color();
        let stroke_width = base_elem.stroke_width();

        let base_path = base_elem.to_skia_path();
        let mut other_paths = Vec::new();
        for &idx in &selected_indices[1..] {
            other_paths.push(self.elements[idx].to_skia_path());
        }

        let mut new_elements: Vec<PathElement> = Vec::new();

        match op {
            BooleanOperation::Union => {
                let mut accum = base_path;
                for p in &other_paths {
                    if let Some(res) = accum.op(p, skia_safe::PathOp::Union) {
                        accum = res;
                    }
                }
                new_elements =
                    PathElement::from_skia_path(&accum, fill_color, stroke_color, stroke_width);
            }
            BooleanOperation::Difference => {
                let mut accum = base_path;
                for p in &other_paths {
                    if let Some(res) = accum.op(p, skia_safe::PathOp::Difference) {
                        accum = res;
                    }
                }
                new_elements =
                    PathElement::from_skia_path(&accum, fill_color, stroke_color, stroke_width);
            }
            BooleanOperation::Intersection => {
                let mut accum = base_path;
                for p in &other_paths {
                    if let Some(res) = accum.op(p, skia_safe::PathOp::Intersect) {
                        accum = res;
                    }
                }
                new_elements =
                    PathElement::from_skia_path(&accum, fill_color, stroke_color, stroke_width);
            }
            BooleanOperation::Exclusion => {
                let mut accum = base_path;
                for p in &other_paths {
                    if let Some(res) = accum.op(p, skia_safe::PathOp::XOR) {
                        accum = res;
                    }
                }
                new_elements =
                    PathElement::from_skia_path(&accum, fill_color, stroke_color, stroke_width);
            }
            BooleanOperation::Division => {
                let top_path = &other_paths[0];
                if let Some(diff) = base_path.op(top_path, skia_safe::PathOp::Difference) {
                    new_elements.extend(PathElement::from_skia_path(
                        &diff,
                        fill_color,
                        stroke_color,
                        stroke_width,
                    ));
                }
                if let Some(inter) = base_path.op(top_path, skia_safe::PathOp::Intersect) {
                    let other_fill = self.elements[selected_indices[1]]
                        .fill_color()
                        .or(fill_color);
                    new_elements.extend(PathElement::from_skia_path(
                        &inter,
                        other_fill,
                        stroke_color,
                        stroke_width,
                    ));
                }
            }
            BooleanOperation::Cut => {
                let top_path = &other_paths[0];
                if let Some(diff) = base_path.op(top_path, skia_safe::PathOp::Difference) {
                    let mut diff_elems = PathElement::from_skia_path(
                        &diff,
                        None,
                        stroke_color.or(fill_color),
                        stroke_width,
                    );
                    for el in &mut diff_elems {
                        el.is_closed = false;
                    }
                    new_elements.extend(diff_elems);
                }
            }
        }

        if new_elements.is_empty() {
            return;
        }

        // Remove old elements
        let sel_set = self.selected_ids.clone();
        self.elements.retain(|el| !sel_set.contains(&el.id()));

        // Insert new elements at insertion point
        let insert_idx = base_idx.min(self.elements.len());
        let mut new_ids = HashSet::new();

        for (offset, elem) in new_elements.into_iter().enumerate() {
            let id = elem.id;
            new_ids.insert(id);
            self.elements
                .insert(insert_idx + offset, Element::Path(elem));
        }

        self.selected_ids = new_ids;
    }

    pub fn convert_selected_to_path(&mut self) -> bool {
        let mut converted_any = false;
        let mut new_elements = Vec::with_capacity(self.elements.len());
        let mut updated_selected_ids = HashSet::new();

        for el in &self.elements {
            if self.selected_ids.contains(&el.id()) {
                match el {
                    Element::Rect(r) => {
                        let path = r.to_path_element();
                        let path_id = path.id;
                        new_elements.push(Element::Path(path));
                        updated_selected_ids.insert(path_id);
                        converted_any = true;
                    }
                    Element::Brush(b) => {
                        let path = b.to_path_element();
                        let path_id = path.id;
                        new_elements.push(Element::Path(path));
                        updated_selected_ids.insert(path_id);
                        converted_any = true;
                    }
                    Element::Path(p) => {
                        let mut path = p.clone();
                        if path.shape_origin.is_some() {
                            path.shape_origin = None;
                            path.shape_rect = None;
                            converted_any = true;
                        }
                        let path_id = path.id;
                        new_elements.push(Element::Path(path));
                        updated_selected_ids.insert(path_id);
                    }
                    Element::Text(t) => {
                        new_elements.push(Element::Text(t.clone()));
                        updated_selected_ids.insert(t.id);
                    }
                    Element::Group(g) => {
                        new_elements.push(Element::Group(g.clone()));
                        updated_selected_ids.insert(g.id);
                    }
                    Element::Image(i) => {
                        new_elements.push(Element::Image(i.clone()));
                        updated_selected_ids.insert(i.id);
                    }
                    Element::Clone(c) => {
                        if let Some(master) =
                            Self::find_element_recursive(&self.elements, c.source_id)
                        {
                            let mut concrete = master.clone_with_new_id();
                            concrete.translate(c.offset.x, c.offset.y);
                            if (c.scale.x - 1.0).abs() > 0.001 || (c.scale.y - 1.0).abs() > 0.001 {
                                concrete.scale(
                                    Point::new(concrete.bounds().x, concrete.bounds().y),
                                    c.scale.x,
                                    c.scale.y,
                                );
                            }
                            if c.rotation.abs() > 0.001 {
                                let b = concrete.bounds();
                                concrete.rotate(
                                    Point::new(b.x + b.width * 0.5, b.y + b.height * 0.5),
                                    c.rotation,
                                );
                            }
                            match concrete {
                                Element::Rect(r) => {
                                    let path = r.to_path_element();
                                    let path_id = path.id;
                                    new_elements.push(Element::Path(path));
                                    updated_selected_ids.insert(path_id);
                                    converted_any = true;
                                }
                                Element::Path(p) => {
                                    let path_id = p.id;
                                    new_elements.push(Element::Path(p));
                                    updated_selected_ids.insert(path_id);
                                    converted_any = true;
                                }
                                other => {
                                    let oid = other.id();
                                    new_elements.push(other);
                                    updated_selected_ids.insert(oid);
                                }
                            }
                        } else {
                            new_elements.push(Element::Clone(c.clone()));
                            updated_selected_ids.insert(c.id);
                        }
                    }
                }
            } else {
                new_elements.push(el.clone());
            }
        }

        if converted_any {
            self.snapshot();
            self.elements = new_elements;
            self.selected_ids = updated_selected_ids;
            true
        } else {
            false
        }
    }
}
