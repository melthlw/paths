use gtk4::gdk;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use super::canvas::CanvasWidget;
use super::color_picker::ColorPickerPopover;
use crate::core::Color;

#[derive(Clone)]
pub struct ColorControlBar {
    container: gtk4::Box,
    current_fill: Rc<Cell<Option<gdk::RGBA>>>,
    current_stroke: Rc<Cell<Option<gdk::RGBA>>>,
    fill_area: gtk4::DrawingArea,
    stroke_area: gtk4::DrawingArea,
    fill_picker: ColorPickerPopover,
    stroke_picker: ColorPickerPopover,
}

fn draw_rounded_rect(cr: &cairo::Context, x: f64, y: f64, size: f64, radius: f64) {
    let r = radius.min(size / 2.0);
    cr.new_sub_path();
    cr.arc(x + size - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(
        x + size - r,
        y + size - r,
        r,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    cr.arc(
        x + r,
        y + size - r,
        r,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    cr.arc(
        x + r,
        y + r,
        r,
        std::f64::consts::PI,
        3.0 * std::f64::consts::FRAC_PI_2,
    );
    cr.close_path();
}

impl ColorControlBar {
    pub fn new(canvas: CanvasWidget) -> Self {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .css_classes(["toolbar", "card"])
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        let current_fill = Rc::new(Cell::new(Some(gdk::RGBA::new(0.0, 0.0, 0.0, 1.0))));

        let fill_area = gtk4::DrawingArea::builder()
            .content_width(22)
            .content_height(22)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        let current_fill_draw = current_fill.clone();
        fill_area.set_draw_func(move |_area, cr, width, height| {
            let w = width as f64;
            let h = height as f64;
            let size = 18.0;
            let ox = (w - size) / 2.0;
            let oy = (h - size) / 2.0;
            let radius = 3.5;

            if let Some(rgba) = current_fill_draw.get() {
                if rgba.alpha() > 0.01 {
                    draw_rounded_rect(cr, ox, oy, size, radius);
                    cr.set_source_rgba(
                        rgba.red() as f64,
                        rgba.green() as f64,
                        rgba.blue() as f64,
                        rgba.alpha() as f64,
                    );
                    let _ = cr.fill_preserve();

                    // Subtle outline
                    cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
                    cr.set_line_width(1.0);
                    let _ = cr.stroke();
                    return;
                }
            }

            // No fill: hollow square with red diagonal slash
            draw_rounded_rect(cr, ox + 0.5, oy + 0.5, size - 1.0, radius);
            cr.set_source_rgba(0.5, 0.5, 0.5, 0.4);
            cr.set_line_width(1.2);
            let _ = cr.stroke();

            cr.move_to(ox + 2.0, oy + 2.0);
            cr.line_to(ox + size - 2.0, oy + size - 2.0);
            cr.set_source_rgba(0.85, 0.25, 0.25, 0.85);
            cr.set_line_width(1.6);
            let _ = cr.stroke();
        });

        let fill_btn = gtk4::Button::builder()
            .child(&fill_area)
            .tooltip_text(&crate::core::gettext("Fill"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .build();

        let fill_picker =
            ColorPickerPopover::new(canvas.clone(), Color::BLACK, 0);
        fill_picker.attach_to(&fill_btn);

        let current_fill_inner = current_fill.clone();
        let canvas_fill_inner = canvas.clone();
        let fill_area_inner = fill_area.clone();
        fill_picker.on_color_changed(move |col| {
            current_fill_inner.set(Some(col.to_gdk()));
            let fills_opt = canvas_fill_inner.get_selected_fills_and_strokes();
            let fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
            if fills.first().map(|f| f.style == crate::core::FillStyle::Solid).unwrap_or(true) {
                canvas_fill_inner.set_fill_color(col);
            }
            fill_area_inner.queue_draw();
        });

        let fill_picker_open = fill_picker.clone();
        let canvas_fill_open = canvas.clone();
        fill_btn.connect_clicked(move |_| {
            let active_tool = canvas_fill_open
                .state()
                .try_borrow()
                .map(|s| s.plugin_manager.active_id())
                .unwrap_or("select");
            if active_tool == "gradient" {
                fill_picker_open.set_mode(1);
            } else if active_tool == "mesh_gradient" || active_tool == "mesh" {
                fill_picker_open.set_mode(2);
            }
            fill_picker_open.popup();
        });

        let current_stroke: Rc<Cell<Option<gdk::RGBA>>> = Rc::new(Cell::new(None));

        let stroke_area = gtk4::DrawingArea::builder()
            .content_width(22)
            .content_height(22)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        let current_stroke_draw = current_stroke.clone();
        stroke_area.set_draw_func(move |_area, cr, width, height| {
            let w = width as f64;
            let h = height as f64;
            let size = 18.0;
            let ox = (w - size) / 2.0;
            let oy = (h - size) / 2.0;
            let radius = 3.5;

            if let Some(rgba) = current_stroke_draw.get() {
                if rgba.alpha() > 0.01 {
                    let r = rgba.red() as f64;
                    let g = rgba.green() as f64;
                    let b = rgba.blue() as f64;
                    let a = rgba.alpha() as f64;

                    // If stroke is white or light, draw subtle contrast border
                    if r + g + b > 2.2 {
                        draw_rounded_rect(cr, ox + 0.5, oy + 0.5, size - 1.0, radius);
                        cr.set_source_rgba(0.0, 0.0, 0.0, 0.35);
                        cr.set_line_width(3.6);
                        let _ = cr.stroke();
                    }

                    // Draw colored hollow square
                    draw_rounded_rect(cr, ox + 0.5, oy + 0.5, size - 1.0, radius);
                    cr.set_source_rgba(r, g, b, a);
                    cr.set_line_width(2.6);
                    let _ = cr.stroke();
                    return;
                }
            }

            // No stroke: Draw crisp hollow square with red diagonal slash
            draw_rounded_rect(cr, ox + 0.5, oy + 0.5, size - 1.0, radius);
            cr.set_source_rgba(0.5, 0.5, 0.5, 0.45);
            cr.set_line_width(2.0);
            let _ = cr.stroke();

            cr.move_to(ox + 2.0, oy + 2.0);
            cr.line_to(ox + size - 2.0, oy + size - 2.0);
            cr.set_source_rgba(0.85, 0.25, 0.25, 0.85);
            cr.set_line_width(1.6);
            let _ = cr.stroke();
        });

        let stroke_btn = gtk4::Button::builder()
            .child(&stroke_area)
            .tooltip_text(&crate::core::gettext("Stroke"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .build();

        let stroke_picker = ColorPickerPopover::new(canvas.clone(), Color::BLACK, 0);
        stroke_picker.attach_to(&stroke_btn);

        let current_stroke_inner = current_stroke.clone();
        let canvas_stroke_inner = canvas.clone();
        let stroke_area_inner = stroke_area.clone();
        stroke_picker.on_color_changed(move |col| {
            current_stroke_inner.set(Some(col.to_gdk()));
            canvas_stroke_inner.set_stroke_color(Some(col));
            stroke_area_inner.queue_draw();
        });

        let stroke_picker_open = stroke_picker.clone();
        stroke_btn.connect_clicked(move |_| {
            stroke_picker_open.popup();
        });

        container.append(&fill_btn);
        container.append(&stroke_btn);

        Self {
            container,
            current_fill,
            current_stroke,
            fill_area,
            stroke_area,
            fill_picker,
            stroke_picker,
        }
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.container
    }

    pub fn update_state(&self, _selected_count: usize, style: (Option<Color>, Option<Color>, f32)) {
        let new_fill = style.0.map(|c| c.to_gdk());
        let new_stroke = style.1.map(|c| c.to_gdk());

        let fill_changed = self.current_fill.get() != new_fill;
        let stroke_changed = self.current_stroke.get() != new_stroke;

        if fill_changed {
            if let Some(c) = style.0 {
                self.fill_picker.set_color(c);
            }
            self.current_fill.set(new_fill);
            self.fill_area.queue_draw();
        }

        if stroke_changed {
            if let Some(c) = style.1 {
                self.stroke_picker.set_color(c);
            }
            self.current_stroke.set(new_stroke);
            self.stroke_area.queue_draw();
        }
    }

    pub fn set_orientation(&self, orientation: gtk4::Orientation) {
        self.container.set_orientation(orientation);
    }
}
