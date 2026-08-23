use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::super::swatch::create_swatch_button;
use crate::core::{Color, StrokeLayer, StrokeStyle};
use crate::ui::canvas::CanvasWidget;
use crate::ui::color_picker::ColorPickerPopover;

pub struct StrokeRow {
    pub container: gtk4::Box,
}

impl StrokeRow {
    pub fn new(
        entry: &StrokeLayer,
        canvas: CanvasWidget,
        is_updating: Rc<Cell<bool>>,
        list_ref: Rc<RefCell<Vec<StrokeLayer>>>,
        idx: usize,
        rebuild_cb: Rc<dyn Fn()>,
    ) -> Self {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let popovers_to_cleanup: Rc<RefCell<Vec<gtk4::Popover>>> =
            Rc::new(RefCell::new(Vec::new()));

        let line1 = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .build();

        let drag_handle = gtk4::Image::builder()
            .icon_name("list-drag-handle-symbolic")
            .opacity(0.35)
            .valign(gtk4::Align::Center)
            .build();
        line1.append(&drag_handle);

        let (color_btn, swatch_area, col_cell) = create_swatch_button(entry.color);
        let picker = ColorPickerPopover::new(canvas.clone(), entry.color, 0);
        popovers_to_cleanup
            .borrow_mut()
            .push(picker.popover().clone());
        picker.attach_to(&color_btn);
        let p_open = picker.clone();
        color_btn.connect_clicked(move |_| {
            p_open.popup();
        });
        line1.append(&color_btn);

        let hex_entry = gtk4::Entry::builder()
            .text(entry.color.to_hex())
            .width_chars(8)
            .max_width_chars(9)
            .css_classes(["numeric", "pill-entry"])
            .valign(gtk4::Align::Center)
            .hexpand(true)
            .build();
        line1.append(&hex_entry);

        let width_entry = gtk4::Entry::builder()
            .text(format!("{:.1} pt", entry.width))
            .width_chars(6)
            .max_width_chars(8)
            .css_classes(["numeric", "pill-entry"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Width"))
            .hexpand(false)
            .build();
        line1.append(&width_entry);

        let op_pct = (entry.opacity * 100.0).round() as i32;
        let op_entry = gtk4::Entry::builder()
            .text(format!("{}%", op_pct))
            .width_chars(5)
            .max_width_chars(5)
            .css_classes(["numeric", "pill-entry"])
            .valign(gtk4::Align::Center)
            .hexpand(false)
            .build();
        line1.append(&op_entry);

        let eye_icon_name = if entry.enabled {
            "view-reveal-symbolic"
        } else {
            "view-conceal-symbolic"
        };
        let stroke_vis_tip = if entry.enabled {
            crate::core::gettext("Hide stroke")
        } else {
            crate::core::gettext("Show stroke")
        };
        let vis_btn = gtk4::Button::builder()
            .icon_name(eye_icon_name)
            .css_classes(["flat", "circular"])
            .valign(gtk4::Align::Center)
            .tooltip_text(stroke_vis_tip)
            .opacity(if entry.enabled { 1.0 } else { 0.5 })
            .build();
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let rebuild = rebuild_cb.clone();
            vis_btn.connect_clicked(move |_| {
                let mut l = list.borrow_mut();
                if let Some(e) = l.get_mut(idx) {
                    e.enabled = !e.enabled;
                }
                let cloned = l.clone();
                drop(l);
                canvas.set_selected_strokes(cloned);
                rebuild();
            });
        }
        line1.append(&vis_btn);

        let del_btn = gtk4::Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(["flat", "circular"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Remove stroke"))
            .build();
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let rebuild = rebuild_cb.clone();
            del_btn.connect_clicked(move |_| {
                let mut l = list.borrow_mut();
                if idx < l.len() {
                    l.remove(idx);
                }
                let cloned = l.clone();
                drop(l);
                canvas.set_selected_strokes(cloned);
                rebuild();
            });
        }
        line1.append(&del_btn);

        container.append(&line1);

        let line2 = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .build();

        let current_style = Rc::new(Cell::new(entry.style));

        let line_btn = gtk4::Button::builder()
            .css_classes(["pill-btn"])
            .valign(gtk4::Align::Center)
            .hexpand(true)
            .build();

        let line_btn_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .margin_start(4)
            .margin_end(4)
            .valign(gtk4::Align::Center)
            .build();

        let line_preview = gtk4::DrawingArea::builder()
            .content_width(48)
            .content_height(12)
            .valign(gtk4::Align::Center)
            .hexpand(true)
            .build();

        let cur_st = current_style.clone();
        line_preview.set_draw_func(move |_area, cr, width, height| {
            cr.set_source_rgb(0.5, 0.5, 0.5);
            cr.set_line_width(2.0);
            let y = height as f64 / 2.0;
            match cur_st.get() {
                StrokeStyle::Solid => cr.set_dash(&[], 0.0),
                StrokeStyle::Dashed => cr.set_dash(&[6.0, 3.0], 0.0),
                StrokeStyle::Dotted => cr.set_dash(&[2.0, 3.0], 0.0),
            }
            cr.move_to(2.0, y);
            cr.line_to((width - 2) as f64, y);
            let _ = cr.stroke();
        });

        let chevron = gtk4::Image::builder()
            .icon_name("pan-down-symbolic")
            .valign(gtk4::Align::Center)
            .build();

        line_btn_box.append(&line_preview);
        line_btn_box.append(&chevron);
        line_btn.set_child(Some(&line_btn_box));
        line2.append(&line_btn);

        let dash_entry = gtk4::Entry::builder()
            .text(match entry.style {
                StrokeStyle::Solid => "—",
                StrokeStyle::Dashed => "8, 4",
                StrokeStyle::Dotted => "2, 4",
            })
            .width_chars(6)
            .max_width_chars(8)
            .css_classes(["numeric", "pill-entry"])
            .valign(gtk4::Align::Center)
            .hexpand(true)
            .editable(false)
            .can_focus(false)
            .build();
        line2.append(&dash_entry);

        let popover = gtk4::Popover::builder().has_arrow(true).build();
        popovers_to_cleanup.borrow_mut().push(popover.clone());

        let popover_menu = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .width_request(170)
            .build();

        let styles = [
            (StrokeStyle::Solid, crate::core::gettext("Solid"), "—"),
            (StrokeStyle::Dashed, crate::core::gettext("Dashed"), "8, 4"),
            (StrokeStyle::Dotted, crate::core::gettext("Dotted"), "2, 4"),
        ];

        for (st, label_text, dash_text) in styles {
            let item_btn = gtk4::Button::builder().css_classes(["flat"]).build();

            let item_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(12)
                .margin_start(8)
                .margin_end(8)
                .margin_top(6)
                .margin_bottom(6)
                .valign(gtk4::Align::Center)
                .build();

            let item_preview = gtk4::DrawingArea::builder()
                .content_width(54)
                .content_height(14)
                .valign(gtk4::Align::Center)
                .build();

            item_preview.set_draw_func(move |_area, cr, width, height| {
                cr.set_source_rgb(0.55, 0.55, 0.55);
                cr.set_line_width(2.5);
                let y = height as f64 / 2.0;
                match st {
                    StrokeStyle::Solid => cr.set_dash(&[], 0.0),
                    StrokeStyle::Dashed => cr.set_dash(&[6.0, 3.0], 0.0),
                    StrokeStyle::Dotted => cr.set_dash(&[2.0, 3.0], 0.0),
                }
                cr.move_to(2.0, y);
                cr.line_to((width - 2) as f64, y);
                let _ = cr.stroke();
            });

            let item_lbl = gtk4::Label::builder()
                .label(label_text)
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .build();

            item_box.append(&item_preview);
            item_box.append(&item_lbl);
            item_btn.set_child(Some(&item_box));

            let cur_style = current_style.clone();
            let preview = line_preview.clone();
            let dash_e = dash_entry.clone();
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let u = is_updating.clone();
            let pop_clone = popover.clone();

            item_btn.connect_clicked(move |_| {
                if u.get() {
                    return;
                }
                cur_style.set(st);
                u.set(true);
                dash_e.set_text(dash_text);
                u.set(false);
                preview.queue_draw();

                let mut l = list.borrow_mut();
                if let Some(e) = l.get_mut(idx) {
                    e.style = st;
                    let cloned = l.clone();
                    drop(l);
                    canvas.set_selected_strokes(cloned);
                }
                pop_clone.popdown();
            });

            popover_menu.append(&item_btn);
        }

        popover.set_child(Some(&popover_menu));
        popover.set_parent(&line_btn);

        let popover_click = popover.clone();
        line_btn.connect_clicked(move |_| {
            popover_click.popup();
        });

        container.append(&line2);

        // Wire controls
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let u = is_updating.clone();
            let hex = hex_entry.clone();
            let col_c = col_cell.clone();
            let swatch_a = swatch_area.clone();
            picker.on_color_changed(move |col| {
                col_c.set(col);
                swatch_a.queue_draw();
                u.set(true);
                hex.set_text(&col.to_hex());
                let mut l = list.borrow_mut();
                if let Some(e) = l.get_mut(idx) {
                    e.color = col;
                }
                let cloned = l.clone();
                drop(l);
                canvas.set_selected_strokes(cloned);
                u.set(false);
            });
        }
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let u = is_updating.clone();
            let pick = picker.clone();
            let col_c = col_cell.clone();
            let swatch_a = swatch_area.clone();
            hex_entry.connect_activate(move |entry| {
                if u.get() {
                    return;
                }
                let text = entry.text();
                let hex_str = text.trim();
                if let Some(col) = Color::from_hex(hex_str) {
                    u.set(true);
                    pick.set_color(col);
                    col_c.set(col);
                    swatch_a.queue_draw();
                    let mut l = list.borrow_mut();
                    if let Some(e) = l.get_mut(idx) {
                        e.color = col;
                    }
                    let cloned = l.clone();
                    drop(l);
                    canvas.set_selected_strokes(cloned);
                    u.set(false);
                }
            });
        }
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let u = is_updating.clone();
            width_entry.connect_activate(move |entry| {
                if u.get() {
                    return;
                }
                let txt = entry
                    .text()
                    .trim()
                    .trim_end_matches("pt")
                    .trim()
                    .to_string();
                if let Ok(w) = txt.parse::<f32>() {
                    let width = w.clamp(0.1, 200.0);
                    u.set(true);
                    entry.set_text(&format!("{:.1} pt", width));
                    let mut l = list.borrow_mut();
                    if let Some(e) = l.get_mut(idx) {
                        e.width = width;
                        let cloned = l.clone();
                        drop(l);
                        canvas.set_selected_strokes(cloned);
                    }
                    u.set(false);
                }
            });
        }
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let u = is_updating.clone();
            op_entry.connect_activate(move |entry| {
                if u.get() {
                    return;
                }
                let txt = entry.text().trim().trim_end_matches('%').to_string();
                if let Ok(val) = txt.parse::<f32>() {
                    let alpha = (val / 100.0).clamp(0.0, 1.0);
                    u.set(true);
                    entry.set_text(&format!("{}%", (alpha * 100.0).round() as i32));
                    let mut l = list.borrow_mut();
                    if let Some(e) = l.get_mut(idx) {
                        e.opacity = alpha;
                    }
                    let cloned = l.clone();
                    drop(l);
                    canvas.set_selected_strokes(cloned);
                    u.set(false);
                }
            });
        }

        {
            let pop_clean = popovers_to_cleanup.clone();
            container.connect_destroy(move |_| {
                for p in pop_clean.borrow().iter() {
                    if p.parent().is_some() {
                        p.unparent();
                    }
                }
            });
        }

        Self { container }
    }
}
