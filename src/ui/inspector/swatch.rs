use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::core::Color;

pub fn draw_rounded_rect(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
    let r = radius.min(width / 2.0).min(height / 2.0);
    cr.new_sub_path();
    cr.arc(x + width - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(
        x + width - r,
        y + height - r,
        r,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    cr.arc(
        x + r,
        y + height - r,
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

pub fn create_swatch_button(
    initial_color: Color,
) -> (gtk4::Button, gtk4::DrawingArea, Rc<Cell<Color>>) {
    let col_cell = Rc::new(Cell::new(initial_color));
    let area = gtk4::DrawingArea::builder()
        .content_width(34)
        .content_height(18)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .build();

    let col_draw = col_cell.clone();
    area.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;
        let c = col_draw.get();

        let pad_x = 2.0;
        let pad_y = 2.0;
        let sw = w - pad_x * 2.0;
        let sh = h - pad_y * 2.0;
        let radius = 4.5;

        // Clip rounded rectangle for the swatch
        cr.save().unwrap();
        draw_rounded_rect(cr, pad_x, pad_y, sw, sh, radius);
        cr.clip();

        if c.a < 0.99 {
            let cs = 4.0;
            let cols = (sw / cs).ceil() as usize;
            let rows = (sh / cs).ceil() as usize;
            for r in 0..rows {
                for col in 0..cols {
                    if (r + col) % 2 == 0 {
                        cr.set_source_rgb(0.85, 0.85, 0.85);
                    } else {
                        cr.set_source_rgb(0.65, 0.65, 0.65);
                    }
                    cr.rectangle(pad_x + col as f64 * cs, pad_y + r as f64 * cs, cs, cs);
                    let _ = cr.fill();
                }
            }
        }

        cr.set_source_rgba(c.r as f64, c.g as f64, c.b as f64, c.a as f64);
        let _ = cr.paint();
        cr.restore().unwrap();

        // Crisp border
        draw_rounded_rect(cr, pad_x + 0.5, pad_y + 0.5, sw - 1.0, sh - 1.0, radius);
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
        cr.set_line_width(1.0);
        let _ = cr.stroke();
    });

    let btn = gtk4::Button::builder()
        .child(&area)
        .css_classes(["pill-btn", "color-swatch-btn"])
        .valign(gtk4::Align::Center)
        .build();

    (btn, area, col_cell)
}

#[derive(Clone)]
pub struct PillSlider {
    pub container: gtk4::DrawingArea,
    pub value: Rc<Cell<f64>>,
    pub on_change: Rc<RefCell<Option<Box<dyn Fn(f64)>>>>,
    pub is_updating: Rc<Cell<bool>>,
}

impl PillSlider {
    pub fn new(label: &str, initial_val: f64) -> Self {
        let value = Rc::new(Cell::new(initial_val.clamp(0.0, 1.0)));
        let label_str = label.to_string();
        let on_change: Rc<RefCell<Option<Box<dyn Fn(f64)>>>> = Rc::new(RefCell::new(None));
        let is_updating = Rc::new(Cell::new(false));

        let drawing_area = gtk4::DrawingArea::builder()
            .content_height(34)
            .hexpand(true)
            .halign(gtk4::Align::Fill)
            .margin_start(10)
            .margin_end(10)
            .margin_bottom(4)
            .build();
        drawing_area.set_cursor_from_name(Some("ew-resize"));

        let val_draw = value.clone();
        let lbl_draw = label_str;
        drawing_area.set_draw_func(move |area, cr, width, height| {
            let w = width as f64;
            let h = height as f64;
            let r = 8.0; // Libadwaita pill radius
            let pct = val_draw.get();

            let fg = area.color();
            let is_dark = (fg.red() + fg.green() + fg.blue()) > 1.5;

            let draw_pill = |cr: &cairo::Context, pw: f64| {
                cr.new_sub_path();
                cr.arc(pw - r, r, r, -std::f64::consts::FRAC_PI_2, 0.0);
                cr.arc(pw - r, h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
                cr.arc(
                    r,
                    h - r,
                    r,
                    std::f64::consts::FRAC_PI_2,
                    std::f64::consts::PI,
                );
                cr.arc(
                    r,
                    r,
                    r,
                    std::f64::consts::PI,
                    3.0 * std::f64::consts::FRAC_PI_2,
                );
                cr.close_path();
            };

            // 1. Unfilled Trough (matches Libadwaita button/card background)
            draw_pill(cr, w);
            if is_dark {
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
            } else {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.06);
            }
            let _ = cr.fill_preserve();
            if is_dark {
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.05);
            } else {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.08);
            }
            cr.set_line_width(1.0);
            let _ = cr.stroke();

            // Text computation
            let text = format!("{} ({}%)", lbl_draw, (pct * 100.0).round() as i32);
            cr.select_font_face(
                "Cantarell",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Bold,
            );
            cr.set_font_size(13.0);

            let (tx, ty) = if let Ok(extents) = cr.text_extents(&text) {
                (
                    (w - extents.width()) / 2.0 - extents.x_bearing(),
                    (h - extents.height()) / 2.0 - extents.y_bearing(),
                )
            } else {
                (w / 4.0, h / 2.0)
            };

            // 2. Render Text for Unfilled portion (Theme FG)
            cr.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 0.85);
            cr.move_to(tx, ty);
            let _ = cr.show_text(&text);

            // 3. Filled Portion (Accent Blue) with White Text clipped
            if pct > 0.005 {
                let fill_w = (w * pct).clamp(0.0, w);

                cr.save().unwrap();
                // Clip to outer pill
                draw_pill(cr, w);
                cr.clip();

                // Clip to filled width
                cr.rectangle(0.0, 0.0, fill_w, h);
                cr.clip();

                // Fill Accent Blue
                draw_pill(cr, w);
                cr.set_source_rgb(0.208, 0.518, 0.894); // #3584e4 GNOME Blue
                let _ = cr.fill();

                // Render White Text over Blue Fill
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.move_to(tx, ty);
                let _ = cr.show_text(&text);

                cr.restore().unwrap();
            }
        });

        // Click & Drag Gestures
        let gesture = gtk4::GestureDrag::new();
        let val_drag = value.clone();
        let area_drag = drawing_area.clone();
        let cb_drag = on_change.clone();
        let updating_drag = is_updating.clone();

        gesture.connect_drag_begin(move |_g, start_x, _start_y| {
            let width = area_drag.width() as f64;
            if width > 0.0 {
                let new_val = (start_x / width).clamp(0.0, 1.0);
                val_drag.set(new_val);
                area_drag.queue_draw();
                if !updating_drag.get() {
                    if let Some(cb) = cb_drag.borrow().as_ref() {
                        cb(new_val);
                    }
                }
            }
        });

        let val_update = value.clone();
        let area_update = drawing_area.clone();
        let gesture_ref = gesture.clone();
        let cb_update = on_change.clone();
        let updating_update = is_updating.clone();

        gesture.connect_drag_update(move |_g, offset_x, _offset_y| {
            let width = area_update.width() as f64;
            if width > 0.0 {
                let start_x = gesture_ref.start_point().map(|(sx, _)| sx).unwrap_or(0.0);
                let current_x = start_x + offset_x;
                let new_val = (current_x / width).clamp(0.0, 1.0);
                val_update.set(new_val);
                area_update.queue_draw();
                if !updating_update.get() {
                    if let Some(cb) = cb_update.borrow().as_ref() {
                        cb(new_val);
                    }
                }
            }
        });

        drawing_area.add_controller(gesture);

        Self {
            container: drawing_area,
            value,
            on_change,
            is_updating,
        }
    }

    pub fn set_on_change<F: Fn(f64) + 'static>(&self, f: F) {
        *self.on_change.borrow_mut() = Some(Box::new(f));
    }

    pub fn value(&self) -> f64 {
        self.value.get()
    }

    pub fn set_value(&self, val: f64) {
        self.is_updating.set(true);
        self.value.set(val.clamp(0.0, 1.0));
        self.container.queue_draw();
        self.is_updating.set(false);
    }

    pub fn widget(&self) -> &gtk4::DrawingArea {
        &self.container
    }
}
