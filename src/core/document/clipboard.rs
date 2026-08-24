use crate::core::element::{BlendMode, CloneElement, Element, ElementId};
use crate::core::geometry::{Point, Rect};
use super::Document;

impl Document {
    pub fn copy_selected(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.clipboard = self
            .elements
            .iter()
            .filter(|e| self.selected_ids.contains(&e.id()))
            .cloned()
            .collect();
    }

    pub fn cut_selected(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.copy_selected();
        self.remove_selected();
    }

    pub fn paste(&mut self, offset: Option<Point>) -> Vec<ElementId> {
        if self.clipboard.is_empty() {
            return Vec::new();
        }
        self.snapshot();
        let d = offset.unwrap_or(Point::new(15.0, 15.0));
        let mut new_ids = Vec::new();
        self.selected_ids.clear();
        for el in &self.clipboard {
            let mut new_el = el.clone_with_new_id();
            new_el.translate(d.x, d.y);
            let new_id = new_el.id();
            self.elements.push(new_el);
            self.selected_ids.insert(new_id);
            new_ids.push(new_id);
        }
        new_ids
    }

    pub fn duplicate_selected(&mut self) -> Vec<ElementId> {
        if self.selected_ids.is_empty() {
            return Vec::new();
        }
        self.snapshot();
        let mut new_elements = Vec::new();
        let mut new_ids = Vec::new();
        for el in self
            .elements
            .iter()
            .filter(|e| self.selected_ids.contains(&e.id()))
        {
            let mut new_el = el.clone_with_new_id();
            new_el.translate(15.0, 15.0);
            new_ids.push(new_el.id());
            new_elements.push(new_el);
        }
        self.selected_ids.clear();
        for id in &new_ids {
            self.selected_ids.insert(*id);
        }
        self.elements.extend(new_elements);
        new_ids
    }

    pub fn has_clones_selected(&self) -> bool {
        self.elements
            .iter()
            .any(|e| self.selected_ids.contains(&e.id()) && matches!(e, Element::Clone(_)))
    }

    pub fn has_masters_selected(&self) -> bool {
        self.elements
            .iter()
            .any(|e| matches!(e, Element::Clone(c) if self.selected_ids.contains(&c.source_id)))
    }

    pub fn clone_selected(&mut self) -> Vec<ElementId> {
        if self.selected_ids.is_empty() {
            return Vec::new();
        }
        self.snapshot();
        let mut new_elements = Vec::new();
        let mut new_ids = Vec::new();

        for el in self
            .elements
            .iter()
            .filter(|e| self.selected_ids.contains(&e.id()))
        {
            let (
                source_id,
                initial_offset,
                initial_scale,
                initial_rotation,
                initial_opacity,
                initial_blend,
                initial_blur,
                name,
            ) = match el {
                Element::Clone(c) => {
                    // Clone of a clone references the original master
                    let offset = Point::new(c.offset.x + 20.0, c.offset.y + 20.0);
                    (
                        c.source_id,
                        offset,
                        c.scale,
                        c.rotation,
                        c.opacity,
                        c.blend_mode,
                        c.blur_radius,
                        c.name.clone(),
                    )
                }
                other => {
                    let offset = Point::new(20.0, 20.0);
                    let name = Some(crate::i18n!("Clone of {}", other.name()));
                    (
                        other.id(),
                        offset,
                        Point::new(1.0, 1.0),
                        0.0,
                        1.0,
                        BlendMode::Normal,
                        0.0,
                        name,
                    )
                }
            };

            let mut clone_elem = CloneElement::new(source_id, initial_offset);
            clone_elem.scale = initial_scale;
            clone_elem.rotation = initial_rotation;
            clone_elem.opacity = initial_opacity;
            clone_elem.blend_mode = initial_blend;
            clone_elem.blur_radius = initial_blur;
            clone_elem.name = name;

            if let Some(master) = Self::find_element_recursive(&self.elements, source_id) {
                let mb = master.bounds();
                clone_elem.cached_bounds = Some(Rect::new(
                    mb.x + clone_elem.offset.x,
                    mb.y + clone_elem.offset.y,
                    mb.width * clone_elem.scale.x.abs().max(0.001),
                    mb.height * clone_elem.scale.y.abs().max(0.001),
                ));
            }

            let new_id = clone_elem.id;
            new_ids.push(new_id);
            new_elements.push(Element::Clone(clone_elem));
        }

        self.selected_ids.clear();
        for id in &new_ids {
            self.selected_ids.insert(*id);
        }
        self.elements.extend(new_elements);
        new_ids
    }

    pub(crate) fn convert_clone_to_concrete(c: &CloneElement, master: &Element) -> Element {
        let mut concrete = master.clone_with_new_id();
        concrete.translate(c.offset.x, c.offset.y);
        if (c.scale.x - 1.0).abs() > 0.001 || (c.scale.y - 1.0).abs() > 0.001 {
            let b = concrete.bounds();
            concrete.scale(
                Point::new(b.x, b.y),
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
        concrete.set_opacity(c.opacity * concrete.opacity());
        if c.blend_mode != BlendMode::Normal {
            concrete.set_blend_mode(c.blend_mode);
        }
        if c.blur_radius > 0.001 {
            concrete.set_blur(c.blur_radius);
        }
        concrete
    }

    pub fn unlink_selected_clones(&mut self) -> Vec<ElementId> {
        if !self.has_clones_selected() {
            return Vec::new();
        }
        self.snapshot();
        let mut unlinked_ids = Vec::new();
        let old_elements = self.elements.clone();
        let mut new_elements = Vec::new();

        for el in self.elements.drain(..) {
            if self.selected_ids.contains(&el.id()) {
                if let Element::Clone(ref c) = el {
                    if let Some(master) = Self::find_element_recursive(&old_elements, c.source_id) {
                        let concrete = Self::convert_clone_to_concrete(c, master);
                        unlinked_ids.push(concrete.id());
                        new_elements.push(concrete);
                        continue;
                    }
                }
            }
            new_elements.push(el);
        }

        self.elements = new_elements;
        self.selected_ids.clear();
        for id in &unlinked_ids {
            self.selected_ids.insert(*id);
        }
        unlinked_ids
    }

    pub fn unlink_clone_by_id(&mut self, clone_id: ElementId) -> Option<ElementId> {
        self.snapshot();
        let old_elements = self.elements.clone();
        let mut unlinked_id = None;
        let mut new_elements = Vec::new();

        for el in self.elements.drain(..) {
            if el.id() == clone_id {
                if let Element::Clone(ref c) = el {
                    if let Some(master) = Self::find_element_recursive(&old_elements, c.source_id) {
                        let concrete = Self::convert_clone_to_concrete(c, master);
                        let nid = concrete.id();
                        unlinked_id = Some(nid);
                        new_elements.push(concrete);
                        continue;
                    }
                }
            }
            new_elements.push(el);
        }

        self.elements = new_elements;
        if let Some(nid) = unlinked_id {
            if self.selected_ids.contains(&clone_id) {
                self.selected_ids.remove(&clone_id);
                self.selected_ids.insert(nid);
            }
        }
        unlinked_id
    }

    pub fn unlink_all_clones_for_master(&mut self, master_id: ElementId) -> Vec<ElementId> {
        self.snapshot();
        let old_elements = self.elements.clone();
        let mut unlinked_ids = Vec::new();
        let mut new_elements = Vec::new();

        for el in self.elements.drain(..) {
            if let Element::Clone(ref c) = el {
                if c.source_id == master_id {
                    if let Some(master) = Self::find_element_recursive(&old_elements, c.source_id) {
                        let concrete = Self::convert_clone_to_concrete(c, master);
                        let nid = concrete.id();
                        unlinked_ids.push(nid);
                        if self.selected_ids.contains(&c.id) {
                            self.selected_ids.remove(&c.id);
                            self.selected_ids.insert(nid);
                        }
                        new_elements.push(concrete);
                        continue;
                    }
                }
            }
            new_elements.push(el);
        }

        self.elements = new_elements;
        unlinked_ids
    }

    pub fn get_clones_for_master(&self, master_id: ElementId) -> Vec<CloneElement> {
        let mut clones = Vec::new();
        for el in &self.elements {
            if let Element::Clone(c) = el {
                if c.source_id == master_id {
                    clones.push(c.clone());
                }
            }
        }
        clones
    }

    pub fn get_all_clone_relationships(&self) -> Vec<(ElementId, Vec<CloneElement>)> {
        let mut map: std::collections::BTreeMap<u64, (ElementId, Vec<CloneElement>)> = std::collections::BTreeMap::new();
        for el in &self.elements {
            if let Element::Clone(c) = el {
                map.entry(c.source_id.0)
                    .or_insert_with(|| (c.source_id, Vec::new()))
                    .1
                    .push(c.clone());
            }
        }
        map.into_values().collect()
    }

    pub fn get_master_for_clone(&self, clone_id: ElementId) -> Option<&Element> {
        for el in &self.elements {
            if let Element::Clone(c) = el {
                if c.id == clone_id {
                    return self.find_element(c.source_id);
                }
            }
        }
        None
    }

    pub fn select_original_element(&mut self) {
        for el in &self.elements {
            if self.selected_ids.contains(&el.id()) {
                if let Element::Clone(c) = el {
                    let source_id = c.source_id;
                    self.selected_ids.clear();
                    self.selected_ids.insert(source_id);
                    return;
                }
            }
        }
    }

    pub fn select_linked_clones(&mut self) {
        let master_ids: Vec<ElementId> = self.selected_ids.iter().cloned().collect();
        let mut clone_ids = Vec::new();
        for el in &self.elements {
            if let Element::Clone(c) = el {
                if master_ids.contains(&c.source_id) {
                    clone_ids.push(c.id);
                }
            }
        }
        if !clone_ids.is_empty() {
            self.selected_ids.clear();
            for id in clone_ids {
                self.selected_ids.insert(id);
            }
        }
    }
}
