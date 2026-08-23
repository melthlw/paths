use crate::core::element::{Element, ElementId, ShapeOrigin};
use crate::core::geometry::{Point, Rect};
use super::Document;

impl Document {
    pub fn select(&mut self, id: ElementId, additive: bool) {
        if !additive {
            self.selected_ids.clear();
        }
        self.selected_ids.insert(id);
    }

    pub fn deselect_all(&mut self) {
        self.selected_ids.clear();
    }

    pub fn is_selected(&self, id: ElementId) -> bool {
        self.selected_ids.contains(&id)
    }

    pub fn selection_bounds(&self) -> Option<Rect> {
        let mut bounds: Option<Rect> = None;
        for el in &self.elements {
            if self.selected_ids.contains(&el.id()) {
                let el_bounds = self.element_bounds(el);
                bounds = match bounds {
                    Some(b) => Some(b.union(el_bounds)),
                    None => Some(el_bounds),
                };
            }
        }
        bounds
    }

    pub fn bounds(&self) -> Option<Rect> {
        let mut bounds: Option<Rect> = None;
        for el in &self.elements {
            let el_bounds = self.element_bounds(el);
            bounds = match bounds {
                Some(b) => Some(b.union(el_bounds)),
                None => Some(el_bounds),
            };
        }
        bounds
    }

    pub fn hit_test(&self, p: Point) -> Option<ElementId> {
        for el in self.elements.iter().rev() {
            if el.visible() && !el.locked() && self.element_hit_test(el, p) {
                return Some(el.id());
            }
        }
        None
    }

    pub fn select_same_fill(&mut self) {
        let target_fill = self
            .elements
            .iter()
            .find(|e| self.selected_ids.contains(&e.id()))
            .and_then(|e| e.fill_color());
        self.selected_ids.clear();
        for el in &self.elements {
            if el.visible() && !el.locked() && el.fill_color() == target_fill {
                self.selected_ids.insert(el.id());
            }
        }
    }

    pub fn select_same_stroke(&mut self) {
        let target_stroke = self
            .elements
            .iter()
            .find(|e| self.selected_ids.contains(&e.id()))
            .and_then(|e| e.stroke_color());
        self.selected_ids.clear();
        for el in &self.elements {
            if el.visible() && !el.locked() && el.stroke_color() == target_stroke {
                self.selected_ids.insert(el.id());
            }
        }
    }

    pub fn select_same_type(&mut self) {
        let target_type = self
            .elements
            .iter()
            .find(|e| self.selected_ids.contains(&e.id()))
            .map(|e| e.type_id());
        if let Some(t) = target_type {
            self.selected_ids.clear();
            for el in &self.elements {
                if el.visible() && !el.locked() && el.type_id() == t {
                    self.selected_ids.insert(el.id());
                }
            }
        }
    }

    pub fn hide_selected(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                el.set_visible(false);
            }
        }
        self.selected_ids.clear();
    }

    pub fn lock_selected(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for el in &mut self.elements {
            if self.selected_ids.contains(&el.id()) {
                el.set_locked(true);
            }
        }
        self.selected_ids.clear();
    }

    pub fn select_all(&mut self) {
        self.selected_ids.clear();
        for el in &self.elements {
            if el.visible() && !el.locked() {
                self.selected_ids.insert(el.id());
            }
        }
    }

    pub fn has_non_path_selected(&self) -> bool {
        if self.selected_ids.is_empty() {
            return false;
        }
        self.selected_ids.iter().any(|id| {
            self.elements.iter().any(|el| {
                el.id() == *id
                    && match el {
                        Element::Path(p) => p.shape_origin.is_some(),
                        _ => true,
                    }
            })
        })
    }

    /// Get the ShapeOrigin of the first selected element (if it's a Path with shape_origin or a Rect)
    pub fn get_selected_shape_origin(&self) -> Option<ShapeOrigin> {
        for id in &self.selected_ids {
            for el in &self.elements {
                if el.id() == *id {
                    match el {
                        Element::Path(p) => {
                            if let Some(ref origin) = p.shape_origin {
                                return Some(origin.clone());
                            }
                        }
                        Element::Rect(r) => {
                            return Some(ShapeOrigin::Rectangle {
                                corner_radius: r.corner_radius,
                                corner_radii: r.effective_radii(),
                                corner_style: r.corner_style,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }
}
