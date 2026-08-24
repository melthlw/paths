use gtk4::prelude::*;
use crate::core::{Color, Element};
use super::CanvasWidget;

impl CanvasWidget {
    pub fn active_fill_color(&self) -> Color {
        self.state
            .try_borrow()
            .map(|s| s.active_fill_color)
            .unwrap_or_else(|_| Color::BLACK)
    }

    pub fn active_stroke_color(&self) -> Option<Color> {
        self.state
            .try_borrow()
            .ok()
            .and_then(|s| s.active_stroke_color)
    }

    pub fn active_stroke_width(&self) -> f32 {
        self.state
            .try_borrow()
            .map(|s| s.active_stroke_width)
            .unwrap_or(2.0)
    }

    pub fn set_fill_color(&self, color: Color) {
        let mut state = self.state.borrow_mut();
        state.active_fill_color = color;

        if !state.document.selected_ids.is_empty() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    el.set_fill_color(Some(color));
                }
            }
        }
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_stroke_color(&self, color: Option<Color>) {
        let mut state = self.state.borrow_mut();
        state.active_stroke_color = color;

        if !state.document.selected_ids.is_empty() {
            state.document.snapshot();
            let selected_ids = state.document.selected_ids.clone();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    el.set_stroke_color(color);
                }
            }
        }
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_fills(&self, fills: Vec<crate::core::FillLayer>) {
        let mut state = self.state.borrow_mut();
        if let Some(first_fill) = fills.iter().find(|f| f.enabled) {
            state.active_fill_color = first_fill.color;
        }
        if !state.document.selected_ids.is_empty() {
            state.document.set_selected_fills(fills);
        }
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_strokes(&self, strokes: Vec<crate::core::StrokeLayer>) {
        let mut state = self.state.borrow_mut();
        if let Some(first_stroke) = strokes.iter().find(|s| s.enabled) {
            state.active_stroke_color = Some(first_stroke.color);
            state.active_stroke_width = first_stroke.width;
        }
        if !state.document.selected_ids.is_empty() {
            state.document.set_selected_strokes(strokes);
        }
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_opacity(&self, opacity: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_opacity(opacity);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_blend_mode(&self, mode: crate::core::BlendMode) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_blend_mode(mode);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_blur(&self, blur: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_blur(blur);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_rect_corner_radius(&self, radius: f32) {
        let r_val = radius.max(0.0);
        let radii = crate::core::CornerRadii::uniform(r_val);
        let mut state = self.state.borrow_mut();

        let current_style = state
            .document
            .selected_ids
            .iter()
            .find_map(|id| {
                state
                    .document
                    .elements
                    .iter()
                    .find(|e| e.id() == *id)
                    .and_then(|e| {
                        if let Element::Rect(r) = e {
                            Some(r.corner_style)
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or(crate::core::CornerStyle::Round);

        let origin = crate::core::ShapeOrigin::Rectangle {
            corner_radius: r_val,
            corner_radii: radii,
            corner_style: current_style,
        };
        if let Some(feat) = state.plugin_manager.active_feature_mut() {
            feat.set_shape_params(&origin);
        }

        let selected_ids = state.document.selected_ids.clone();
        if !selected_ids.is_empty() {
            state.document.snapshot();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    if let Element::Rect(r) = el {
                        r.corner_radius = r_val;
                        r.corner_radii = radii;
                    }
                }
            }
            self.drawing_area.queue_draw();
        }
        state.notify_status();
    }

    pub fn set_rect_corner_radii(&self, radii: crate::core::CornerRadii) {
        let mut state = self.state.borrow_mut();
        let max_r = radii.max_radius();

        let current_style = state
            .document
            .selected_ids
            .iter()
            .find_map(|id| {
                state
                    .document
                    .elements
                    .iter()
                    .find(|e| e.id() == *id)
                    .and_then(|e| {
                        if let Element::Rect(r) = e {
                            Some(r.corner_style)
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or(crate::core::CornerStyle::Round);

        let origin = crate::core::ShapeOrigin::Rectangle {
            corner_radius: max_r,
            corner_radii: radii,
            corner_style: current_style,
        };
        if let Some(feat) = state.plugin_manager.active_feature_mut() {
            feat.set_shape_params(&origin);
        }

        let selected_ids = state.document.selected_ids.clone();
        if !selected_ids.is_empty() {
            state.document.snapshot();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    if let Element::Rect(r) = el {
                        r.corner_radii = radii;
                        r.corner_radius = max_r;
                    }
                }
            }
            self.drawing_area.queue_draw();
        }
        state.notify_status();
    }

    pub fn set_rect_corner_style(&self, style: crate::core::CornerStyle) {
        let mut state = self.state.borrow_mut();

        let (cur_radius, cur_radii) = state
            .document
            .selected_ids
            .iter()
            .find_map(|id| {
                state
                    .document
                    .elements
                    .iter()
                    .find(|e| e.id() == *id)
                    .and_then(|e| {
                        if let Element::Rect(r) = e {
                            Some((r.corner_radius, r.corner_radii))
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or((0.0, crate::core::CornerRadii::default()));

        let origin = crate::core::ShapeOrigin::Rectangle {
            corner_radius: cur_radius,
            corner_radii: cur_radii,
            corner_style: style,
        };
        if let Some(feat) = state.plugin_manager.active_feature_mut() {
            feat.set_shape_params(&origin);
        }

        let selected_ids = state.document.selected_ids.clone();
        if !selected_ids.is_empty() {
            state.document.snapshot();
            for el in &mut state.document.elements {
                if selected_ids.contains(&el.id()) {
                    if let Element::Rect(r) = el {
                        r.corner_style = style;
                    }
                }
            }
            self.drawing_area.queue_draw();
        }
        state.notify_status();
    }

    pub fn update_shape_origin(&self, new_origin: crate::core::ShapeOrigin) {
        let mut state = self.state.borrow_mut();
        if let Some(feat) = state.plugin_manager.active_feature_mut() {
            feat.set_shape_params(&new_origin);
        }

        let selected_ids = state.document.selected_ids.clone();
        if !selected_ids.is_empty() {
            let has_matching = state.document.elements.iter().any(|el| {
                if selected_ids.contains(&el.id()) {
                    if let Element::Rect(_) = el {
                        if matches!(&new_origin, crate::core::ShapeOrigin::Rectangle { .. }) {
                            return true;
                        }
                    }
                    if let Element::Path(p) = el {
                        return match (&p.shape_origin, &new_origin) {
                            (
                                Some(crate::core::ShapeOrigin::Star { .. }),
                                crate::core::ShapeOrigin::Star { .. },
                            ) => true,
                            (
                                Some(crate::core::ShapeOrigin::Triangle { .. }),
                                crate::core::ShapeOrigin::Triangle { .. },
                            ) => true,
                            (
                                Some(crate::core::ShapeOrigin::Circle { .. }),
                                crate::core::ShapeOrigin::Circle { .. },
                            ) => true,
                            (
                                Some(crate::core::ShapeOrigin::Spiral { .. }),
                                crate::core::ShapeOrigin::Spiral { .. },
                            ) => true,
                            (
                                Some(crate::core::ShapeOrigin::Rectangle { .. }),
                                crate::core::ShapeOrigin::Rectangle { .. },
                            ) => true,
                            _ => false,
                        };
                    }
                }
                false
            });

            if has_matching {
                state.document.snapshot();
                for el in &mut state.document.elements {
                    if selected_ids.contains(&el.id()) {
                        if let Element::Rect(r) = el {
                            if let crate::core::ShapeOrigin::Rectangle {
                                corner_radius,
                                corner_radii,
                                corner_style,
                            } = &new_origin
                            {
                                r.corner_radius = *corner_radius;
                                r.corner_radii = *corner_radii;
                                r.corner_style = *corner_style;
                            }
                        }
                        if let Element::Path(p) = el {
                            let is_matching_variant = match (&p.shape_origin, &new_origin) {
                                (
                                    Some(crate::core::ShapeOrigin::Star { .. }),
                                    crate::core::ShapeOrigin::Star { .. },
                                ) => true,
                                (
                                    Some(crate::core::ShapeOrigin::Triangle { .. }),
                                    crate::core::ShapeOrigin::Triangle { .. },
                                ) => true,
                                (
                                    Some(crate::core::ShapeOrigin::Circle { .. }),
                                    crate::core::ShapeOrigin::Circle { .. },
                                ) => true,
                                (
                                    Some(crate::core::ShapeOrigin::Spiral { .. }),
                                    crate::core::ShapeOrigin::Spiral { .. },
                                ) => true,
                                (
                                    Some(crate::core::ShapeOrigin::Rectangle { .. }),
                                    crate::core::ShapeOrigin::Rectangle { .. },
                                ) => true,
                                _ => false,
                            };
                            if !is_matching_variant {
                                continue;
                            }

                            let rect = p.shape_rect.unwrap_or_else(|| p.bounds());

                            let old_id = p.id;
                            let fill = p.fill_color;
                            let stroke = p.stroke_color;
                            let sw = p.stroke_width;

                            let mut new_elem = match &new_origin {
                                crate::core::ShapeOrigin::Star {
                                    corner_radius,
                                    points,
                                    inner_ratio,
                                } => crate::plugins::features::star::StarFeature::create_star_path(
                                    rect,
                                    fill,
                                    stroke,
                                    sw,
                                    *points,
                                    *inner_ratio,
                                    *corner_radius,
                                ),
                                crate::core::ShapeOrigin::Triangle {
                                    corner_radius,
                                    sides,
                                } => crate::plugins::features::triangle::TriangleFeature::create_triangle_path(
                                    rect,
                                    fill,
                                    stroke,
                                    sw,
                                    *sides,
                                    *corner_radius,
                                ),
                                crate::core::ShapeOrigin::Circle {
                                    arc_mode,
                                    start_angle,
                                    end_angle,
                                } => crate::plugins::features::circle::CircleFeature::create_ellipse_path(
                                    rect,
                                    fill,
                                    stroke,
                                    sw,
                                    *arc_mode,
                                    *start_angle,
                                    *end_angle,
                                ),
                                crate::core::ShapeOrigin::Spiral {
                                    turns,
                                    divergence,
                                    inner_radius,
                                } => crate::plugins::features::spiral::SpiralFeature::create_spiral_path(
                                    rect,
                                    stroke,
                                    sw,
                                    *turns,
                                    *divergence,
                                    *inner_radius,
                                ),
                                crate::core::ShapeOrigin::Rectangle {
                                    corner_radius: _,
                                    corner_radii,
                                    corner_style,
                                } => {
                                    let mut rect_el = crate::core::RectElement::new(rect, fill, stroke);
                                    rect_el.stroke_width = sw;
                                    rect_el.corner_radii = *corner_radii;
                                    rect_el.corner_radius = corner_radii.max_radius();
                                    rect_el.corner_style = *corner_style;
                                    rect_el.to_path_element()
                                }
                            };
                            new_elem.id = old_id;
                            new_elem.shape_rect = Some(rect);
                            *p = new_elem;
                        }
                    }
                }
                self.drawing_area.queue_draw();
            }
        }
        state.notify_status();
    }

    pub fn set_selected_font_family(&self, family: &str) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_font_family(family);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_font_weight(&self, weight: u32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_font_weight(weight);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_font_size(&self, size: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_font_size(size);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_line_height(&self, line_height: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_line_height(line_height);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_letter_spacing(&self, spacing: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_letter_spacing(spacing);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_word_spacing(&self, spacing: f32) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_word_spacing(spacing);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_align(&self, align: crate::core::TextAlign) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_align(align);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_selected_text_box_width(&self, width: Option<f32>) {
        let mut state = self.state.borrow_mut();
        state.document.set_selected_text_box_width(width);
        state.notify_status();
        self.drawing_area.queue_draw();
    }
}
