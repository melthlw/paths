use crate::core::color::Color;
use crate::core::element::{Element, OpenTypeFeatures, TextAlign, TextCase, TextElement};
use super::Document;

impl Document {
    pub fn get_selected_style(&self) -> Option<(Option<Color>, Option<Color>, f32)> {
        for elem in &self.elements {
            if self.selected_ids.contains(&elem.id()) {
                return Some((elem.fill_color(), elem.stroke_color(), elem.stroke_width()));
            }
        }
        None
    }

    pub fn get_selected_text_element(&self) -> Option<&TextElement> {
        for elem in &self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    return Some(t);
                }
            }
        }
        None
    }

    pub fn get_selected_text_info(
        &self,
    ) -> Option<(String, u32, f32, f32, TextAlign, f32, f32)> {
        for elem in &self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    return Some((
                        t.font_family.clone(),
                        t.font_weight,
                        t.font_size,
                        t.line_height,
                        t.alignment,
                        t.letter_spacing,
                        t.word_spacing,
                    ));
                }
            }
        }
        None
    }

    pub fn set_selected_font_family(&mut self, family: &str) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.font_family = family.to_string();
                }
            }
        }
    }

    pub fn set_selected_font_weight(&mut self, weight: u32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.font_weight = weight;
                }
            }
        }
    }

    pub fn set_selected_font_size(&mut self, size: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.font_size = size.max(6.0);
                }
            }
        }
    }

    pub fn set_selected_line_height(&mut self, line_height: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.line_height = line_height.max(0.5);
                }
            }
        }
    }

    pub fn set_selected_letter_spacing(&mut self, letter_spacing: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.letter_spacing = letter_spacing;
                }
            }
        }
    }

    pub fn set_selected_word_spacing(&mut self, word_spacing: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.word_spacing = word_spacing;
                }
            }
        }
    }

    pub fn set_selected_kerning_offset(&mut self, offset: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.kerning_offset = offset;
                }
            }
        }
    }

    pub fn set_selected_opentype_features(&mut self, features: OpenTypeFeatures) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.opentype_features = features;
                }
            }
        }
    }

    pub fn set_selected_text_case(&mut self, text_case: TextCase) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.text_case = text_case;
                }
            }
        }
    }

    pub fn set_selected_text_align(&mut self, align: TextAlign) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.alignment = align;
                }
            }
        }
    }

    pub fn set_selected_text_box_width(&mut self, width: Option<f32>) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.box_width = width.map(|w| w.max(20.0));
                    if width.is_none() {
                        t.box_height = None;
                    }
                }
            }
        }
    }

    pub fn attach_selected_text_to_path(&mut self) -> bool {
        let text_id = self.elements.iter().find_map(|e| {
            if self.selected_ids.contains(&e.id()) {
                if let Element::Text(t) = e {
                    return Some(t.id);
                }
            }
            None
        });

        let path_id = self.elements.iter().find_map(|e| {
            if self.selected_ids.contains(&e.id()) {
                match e {
                    Element::Path(p) => Some(p.id),
                    Element::Rect(r) => Some(r.id),
                    _ => None,
                }
            } else {
                None
            }
        });

        if let (Some(tid), Some(pid)) = (text_id, path_id) {
            self.snapshot();
            for elem in &mut self.elements {
                if elem.id() == tid {
                    if let Element::Text(t) = elem {
                        t.path_id = Some(pid);
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn detach_selected_text_from_path(&mut self) -> bool {
        if self.selected_ids.is_empty() {
            return false;
        }
        self.snapshot();
        let mut detached = false;
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    if t.path_id.is_some() {
                        t.path_id = None;
                        detached = true;
                    }
                }
            }
        }
        detached
    }

    pub fn set_selected_text_path_offset(&mut self, offset: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.path_offset = offset;
                }
            }
        }
    }

    pub fn set_selected_text_path_inverted(&mut self, inverted: bool) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.path_side_inverted = inverted;
                }
            }
        }
    }

    pub fn set_selected_text_path_orientation(&mut self, orientation: bool) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.path_align_orientation = orientation;
                }
            }
        }
    }

    pub fn set_selected_text_path_repeat(&mut self, repeat: bool) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.path_repeat = repeat;
                }
            }
        }
    }

    pub fn set_selected_text_path_spacing(&mut self, spacing: f32) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.path_spacing = spacing.max(0.0);
                }
            }
        }
    }

    pub fn set_selected_text_underline(&mut self, underline: bool) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.underline = underline;
                }
            }
        }
    }

    pub fn set_selected_text_strikethrough(&mut self, strikethrough: bool) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.strikethrough = strikethrough;
                }
            }
        }
    }

    pub fn set_selected_text_baseline(&mut self, baseline: crate::core::element::TextBaseline) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.baseline = baseline;
                }
            }
        }
    }

    pub fn insert_symbol_to_selected_text(&mut self, symbol: &str) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.text.push_str(symbol);
                }
            }
        }
    }

    pub fn set_selected_text_path_valign(&mut self, valign: crate::core::element::PathVerticalAlign) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.path_valign = valign;
                }
            }
        }
    }

    pub fn set_selected_text_path_glyph_orientation(&mut self, orientation: crate::core::element::PathGlyphOrientation) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        for elem in &mut self.elements {
            if self.selected_ids.contains(&elem.id()) {
                if let Element::Text(t) = elem {
                    t.path_glyph_orientation = orientation;
                }
            }
        }
    }
}

