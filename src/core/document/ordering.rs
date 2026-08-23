use crate::core::element::{BlendMode, Element, ElementId, FillLayer, GroupElement, StrokeLayer};
use super::Document;

impl Document {
    pub fn get_layers_info(&self) -> Vec<crate::core::layer::LayerItemInfo> {
        self.elements
            .iter()
            .enumerate()
            .rev()
            .map(|(idx, el)| self.create_layer_item_info(el, idx))
            .collect()
    }

    fn create_layer_item_info(
        &self,
        el: &Element,
        z_index: usize,
    ) -> crate::core::layer::LayerItemInfo {
        let (is_group, children, is_clip) = match el {
            Element::Group(g) => {
                let is_clip = g.clip_element.is_some();
                let ch: Vec<crate::core::layer::LayerItemInfo> = g
                    .children
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(cidx, cel)| self.create_layer_item_info(cel, cidx))
                    .collect();
                (true, ch, is_clip)
            }
            _ => (false, Vec::new(), false),
        };

        crate::core::layer::LayerItemInfo {
            id: el.id(),
            name: el.name(),
            icon_name: el.icon_name(),
            element_type: el.element_type_name(),
            visible: el.visible(),
            locked: el.locked(),
            is_selected: self.selected_ids.contains(&el.id()),
            opacity: el.opacity(),
            z_index,
            is_group,
            children,
            is_clip,
        }
    }

    pub fn find_element_recursive<'a>(
        elements: &'a [Element],
        id: ElementId,
    ) -> Option<&'a Element> {
        for el in elements {
            if el.id() == id {
                return Some(el);
            }
            if let Element::Group(g) = el {
                if let Some(found) = Self::find_element_recursive(&g.children, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn find_element_mut_recursive<'a>(
        elements: &'a mut [Element],
        id: ElementId,
    ) -> Option<&'a mut Element> {
        for el in elements {
            if el.id() == id {
                return Some(el);
            }
            if let Element::Group(g) = el {
                if let Some(found) = Self::find_element_mut_recursive(&mut g.children, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn set_element_visibility(&mut self, id: ElementId, visible: bool) {
        self.snapshot();
        if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, id) {
            el.set_visible(visible);
        }
    }

    pub fn set_element_locked(&mut self, id: ElementId, locked: bool) {
        self.snapshot();
        if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, id) {
            el.set_locked(locked);
        }
    }

    pub fn set_selected_opacity(&mut self, opacity: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for id in &self.selected_ids {
            if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, *id) {
                el.set_opacity(opacity);
            }
        }
    }

    pub fn set_selected_blend_mode(&mut self, mode: BlendMode) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for id in &self.selected_ids {
            if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, *id) {
                el.set_blend_mode(mode);
            }
        }
    }

    pub fn set_selected_blur(&mut self, blur: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for id in &self.selected_ids {
            if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, *id) {
                el.set_blur(blur);
            }
        }
    }

    pub fn get_selection_blend_info(&self) -> Option<(BlendMode, f32, f32)> {
        for id in &self.selected_ids {
            if let Some(el) = Self::find_element_recursive(&self.elements, *id) {
                return Some((el.blend_mode(), el.blur(), el.opacity()));
            }
        }
        None
    }

    pub fn set_selected_fills(&mut self, fills: Vec<FillLayer>) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for id in &self.selected_ids {
            if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, *id) {
                el.set_fills(fills.clone());
            }
        }
    }

    pub fn set_selected_strokes(&mut self, strokes: Vec<StrokeLayer>) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for id in &self.selected_ids {
            if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, *id) {
                el.set_strokes(strokes.clone());
            }
        }
    }

    pub fn get_selected_fills_and_strokes(&self) -> Option<(Vec<FillLayer>, Vec<StrokeLayer>)> {
        for id in &self.selected_ids {
            if let Some(el) = Self::find_element_recursive(&self.elements, *id) {
                return Some((el.fills(), el.strokes()));
            }
        }
        None
    }

    pub fn move_layer_up(&mut self, id: ElementId) {
        if let Some(idx) = self.elements.iter().position(|e| e.id() == id) {
            if idx + 1 < self.elements.len() {
                self.snapshot();
                self.elements.swap(idx, idx + 1);
            }
        }
    }

    pub fn move_layer_down(&mut self, id: ElementId) {
        if let Some(idx) = self.elements.iter().position(|e| e.id() == id) {
            if idx > 0 {
                self.snapshot();
                self.elements.swap(idx, idx - 1);
            }
        }
    }

    pub fn bring_selected_to_front(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        let mut unselected = Vec::new();
        let mut selected = Vec::new();
        for el in self.elements.drain(..) {
            if self.selected_ids.contains(&el.id()) {
                selected.push(el);
            } else {
                unselected.push(el);
            }
        }
        unselected.extend(selected);
        self.elements = unselected;
    }

    pub fn send_selected_to_back(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        let mut unselected = Vec::new();
        let mut selected = Vec::new();
        for el in self.elements.drain(..) {
            if self.selected_ids.contains(&el.id()) {
                selected.push(el);
            } else {
                unselected.push(el);
            }
        }
        selected.extend(unselected);
        self.elements = selected;
    }

    pub fn bring_selected_forward(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        let len = self.elements.len();
        if len <= 1 {
            return;
        }
        for i in (0..len - 1).rev() {
            if self.selected_ids.contains(&self.elements[i].id())
                && !self.selected_ids.contains(&self.elements[i + 1].id())
            {
                self.elements.swap(i, i + 1);
            }
        }
    }

    pub fn send_selected_backward(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        let len = self.elements.len();
        if len <= 1 {
            return;
        }
        for i in 1..len {
            if self.selected_ids.contains(&self.elements[i].id())
                && !self.selected_ids.contains(&self.elements[i - 1].id())
            {
                self.elements.swap(i, i - 1);
            }
        }
    }

    pub fn group_selected(&mut self) -> Option<ElementId> {
        if self.selected_ids.len() < 2 {
            return None;
        }
        self.snapshot();
        let mut grouped = Vec::new();
        let mut highest_idx = 0;
        let mut remaining = Vec::new();

        for el in self.elements.drain(..) {
            if self.selected_ids.contains(&el.id()) {
                grouped.push(el);
                highest_idx = remaining.len();
            } else {
                remaining.push(el);
            }
        }

        let group = GroupElement::new(grouped);
        let group_id = group.id;
        let group_el = Element::Group(group);

        if highest_idx <= remaining.len() {
            remaining.insert(highest_idx, group_el);
        } else {
            remaining.push(group_el);
        }

        self.elements = remaining;
        self.selected_ids.clear();
        self.selected_ids.insert(group_id);
        Some(group_id)
    }

    pub fn ungroup_selected(&mut self) -> Vec<ElementId> {
        if self.selected_ids.is_empty() {
            return Vec::new();
        }
        self.snapshot();
        let mut new_selection = Vec::new();
        let mut new_elements = Vec::new();

        for el in self.elements.drain(..) {
            if self.selected_ids.contains(&el.id()) {
                if let Element::Group(g) = el {
                    for child in g.children {
                        new_selection.push(child.id());
                        new_elements.push(child);
                    }
                    if let Some(clip) = g.clip_element {
                        new_selection.push(clip.id());
                        new_elements.push(*clip);
                    }
                } else {
                    new_elements.push(el);
                }
            } else {
                new_elements.push(el);
            }
        }

        self.elements = new_elements;
        self.selected_ids.clear();
        for id in &new_selection {
            self.selected_ids.insert(*id);
        }
        new_selection
    }

    pub fn set_clip_group_selected(&mut self) -> Option<ElementId> {
        if self.selected_ids.len() < 2 {
            return None;
        }
        self.snapshot();
        let mut grouped = Vec::new();
        let mut highest_idx = 0;
        let mut remaining = Vec::new();

        for el in self.elements.drain(..) {
            if self.selected_ids.contains(&el.id()) {
                grouped.push(el);
                highest_idx = remaining.len();
            } else {
                remaining.push(el);
            }
        }

        let clip_element = grouped.pop().map(Box::new);
        let mut group = GroupElement::new(grouped);
        group.clip_element = clip_element;
        let group_id = group.id;
        let group_el = Element::Group(group);

        if highest_idx <= remaining.len() {
            remaining.insert(highest_idx, group_el);
        } else {
            remaining.push(group_el);
        }

        self.elements = remaining;
        self.selected_ids.clear();
        self.selected_ids.insert(group_id);
        Some(group_id)
    }
}
