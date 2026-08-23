use crate::core::color::Color;
use crate::core::element::{Element, TextAlign};
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
}
