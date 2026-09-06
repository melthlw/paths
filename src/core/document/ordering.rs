use crate::core::element::{BlendMode, Element, ElementId, FillLayer, GroupElement, StrokeLayer};
use crate::core::Color;
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

    pub fn set_element_name(&mut self, id: ElementId, name: Option<String>) {
        self.snapshot();
        if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, id) {
            el.set_name(name);
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

    pub fn set_selected_fill_color(&mut self, color: Option<Color>) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for id in &self.selected_ids {
            if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, *id) {
                el.set_fill_color(color);
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

    pub fn set_selected_stroke_color(&mut self, color: Option<Color>) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for id in &self.selected_ids {
            if let Some(el) = Self::find_element_mut_recursive(&mut self.elements, *id) {
                el.set_stroke_color(color);
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

    pub fn move_layer_relative(
        &mut self,
        src_id: ElementId,
        target_id: ElementId,
        insert_above_in_ui: bool,
    ) {
        if src_id == target_id {
            return;
        }

        self.snapshot();

        let src_elem = match Self::remove_element_from_vec(&mut self.elements, src_id) {
            Some(el) => el,
            None => return,
        };

        if let Some(uninserted) = Self::insert_element_relative_in_vec(
            &mut self.elements,
            src_elem,
            target_id,
            insert_above_in_ui,
        ) {
            self.elements.push(uninserted);
        }
    }

    fn remove_element_from_vec(vec: &mut Vec<Element>, id: ElementId) -> Option<Element> {
        if let Some(pos) = vec.iter().position(|e| e.id() == id) {
            return Some(vec.remove(pos));
        }
        for el in vec.iter_mut() {
            if let Element::Group(g) = el {
                if let Some(found) = Self::remove_element_from_vec(&mut g.children, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn insert_element_relative_in_vec(
        vec: &mut Vec<Element>,
        to_insert: Element,
        target_id: ElementId,
        insert_above_in_ui: bool,
    ) -> Option<Element> {
        if let Some(pos) = vec.iter().position(|e| e.id() == target_id) {
            let insert_idx = if insert_above_in_ui { pos + 1 } else { pos };
            vec.insert(insert_idx, to_insert);
            return None;
        }
        let mut current = to_insert;
        for el in vec.iter_mut() {
            if let Element::Group(g) = el {
                if let Some(returned) = Self::insert_element_relative_in_vec(
                    &mut g.children,
                    current,
                    target_id,
                    insert_above_in_ui,
                ) {
                    current = returned;
                } else {
                    return None;
                }
            }
        }
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::element::PathElement;

    #[test]
    fn test_move_layer_relative_above_and_below() {
        let mut doc = Document::new();
        let e1 = Element::Path(PathElement::new(Vec::new(), false, None, None, 1.0));
        let e2 = Element::Path(PathElement::new(Vec::new(), false, None, None, 1.0));
        let e3 = Element::Path(PathElement::new(Vec::new(), false, None, None, 1.0));
        let id1 = e1.id();
        let id2 = e2.id();
        let id3 = e3.id();

        doc.elements = vec![e1, e2, e3]; // elements: [e1 (index 0), e2 (index 1), e3 (index 2)]

        // Move e1 above e3 in UI (highest z-index / end of elements)
        doc.move_layer_relative(id1, id3, true);
        assert_eq!(doc.elements[0].id(), id2);
        assert_eq!(doc.elements[1].id(), id3);
        assert_eq!(doc.elements[2].id(), id1);

        // Move e1 below e2 in UI (lowest z-index / start of elements)
        doc.move_layer_relative(id1, id2, false);
        assert_eq!(doc.elements[0].id(), id1);
        assert_eq!(doc.elements[1].id(), id2);
        assert_eq!(doc.elements[2].id(), id3);
    }

    #[test]
    fn test_set_element_name_and_get_layers_info() {
        let mut doc = Document::new();
        let e1 = Element::Path(PathElement::new(Vec::new(), false, None, None, 1.0));
        let id1 = e1.id();
        doc.elements = vec![e1];

        // Default name in layer info
        let layers = doc.get_layers_info();
        assert_eq!(layers[0].id, id1);
        let default_name = layers[0].name.clone();

        // Rename element
        doc.set_element_name(id1, Some("Custom Vector Path".to_string()));
        let layers_renamed = doc.get_layers_info();
        assert_eq!(layers_renamed[0].name, "Custom Vector Path");

        // Undo restores original name
        assert!(doc.undo());
        let layers_undone = doc.get_layers_info();
        assert_eq!(layers_undone[0].name, default_name);

        // Setting empty name normalizes to None and resets to default name
        doc.set_element_name(id1, Some("   ".to_string()));
        let layers_reset = doc.get_layers_info();
        assert_eq!(layers_reset[0].name, default_name);
    }
}
