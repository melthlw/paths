use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::super::swatch::create_swatch_button;
use crate::core::{Color, FillLayer, FillStyle, PatternType};
use crate::ui::canvas::CanvasWidget;
use crate::ui::color_picker::ColorPickerPopover;

pub struct FillRow {
    pub container: gtk4::Box,
}

impl FillRow {
    pub fn new(
        entry: &FillLayer,
        canvas: CanvasWidget,
        is_updating: Rc<Cell<bool>>,
        list_ref: Rc<RefCell<Vec<FillLayer>>>,
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

        let style_label = match entry.style {
            FillStyle::Solid => crate::core::gettext("Solid"),
            FillStyle::LinearGradient => crate::core::gettext("Linear"),
            FillStyle::RadialGradient => crate::core::gettext("Radial"),
            FillStyle::Mesh => crate::core::gettext("Mesh"),
            FillStyle::Pattern => crate::core::gettext("Pattern"),
        };
        let style_btn = gtk4::Button::builder()
            .label(style_label)
            .css_classes(["pill-btn"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Fill Style"))
            .build();
        line1.append(&style_btn);

        let style_popover = gtk4::Popover::builder().has_arrow(true).build();
        popovers_to_cleanup.borrow_mut().push(style_popover.clone());
        let style_menu = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .width_request(170)
            .build();

        let fill_styles = [
            (
                FillStyle::Solid,
                crate::core::gettext("Solid"),
                "media-record-symbolic",
            ),
            (
                FillStyle::LinearGradient,
                crate::core::gettext("Linear Gradient"),
                "media-playlist-consecutive-symbolic",
            ),
            (
                FillStyle::RadialGradient,
                crate::core::gettext("Radial Gradient"),
                "media-record-symbolic",
            ),
            (
                FillStyle::Mesh,
                crate::core::gettext("Mesh Gradient"),
                "action-unavailable-symbolic",
            ),
            (
                FillStyle::Pattern,
                crate::core::gettext("Geometric Pattern"),
                "view-grid-symbolic",
            ),
        ];

        for (st, st_lbl, st_icon) in fill_styles {
            let item_btn = gtk4::Button::builder().css_classes(["flat"]).build();
            let item_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(8)
                .margin_start(6)
                .margin_end(6)
                .margin_top(4)
                .margin_bottom(4)
                .valign(gtk4::Align::Center)
                .build();
            let item_img = gtk4::Image::from_icon_name(st_icon);
            let item_label = gtk4::Label::builder()
                .label(st_lbl)
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .build();
            item_box.append(&item_img);
            item_box.append(&item_label);
            item_btn.set_child(Some(&item_box));

            let list = list_ref.clone();
            let canvas_s = canvas.clone();
            let pop_c = style_popover.clone();
            let rebuild = rebuild_cb.clone();
            item_btn.connect_clicked(move |_| {
                let mut l = list.borrow_mut();
                if let Some(e) = l.get_mut(idx) {
                    e.style = st;
                }
                let cloned = l.clone();
                drop(l);
                canvas_s.set_selected_fills(cloned);
                pop_c.popdown();
                rebuild();
            });
            style_menu.append(&item_btn);
        }

        style_popover.set_child(Some(&style_menu));
        style_popover.set_parent(&style_btn);
        let pop_open = style_popover.clone();
        style_btn.connect_clicked(move |_| {
            pop_open.popup();
        });

        let (color_btn1, swatch1_area, col1_cell) = create_swatch_button(entry.color);
        let mode1_idx = match entry.style {
            FillStyle::Solid => 0,
            FillStyle::LinearGradient | FillStyle::RadialGradient => 1,
            FillStyle::Mesh => 2,
            FillStyle::Pattern => 3,
        };
        let picker1 = ColorPickerPopover::new(canvas.clone(), entry.color, mode1_idx);
        popovers_to_cleanup
            .borrow_mut()
            .push(picker1.popover().clone());
        picker1.attach_to(&color_btn1);
        let p1_open = picker1.clone();
        color_btn1.connect_clicked(move |_| {
            p1_open.popup();
        });
        line1.append(&color_btn1);

        let hex_entry1 = gtk4::Entry::builder()
            .text(entry.color.to_hex())
            .width_chars(8)
            .max_width_chars(9)
            .css_classes(["numeric", "pill-entry"])
            .valign(gtk4::Align::Center)
            .hexpand(entry.style == FillStyle::Solid)
            .build();

        let (
            _color_btn2_opt,
            hex_entry2_opt,
            col2_cell_opt,
            swatch2_area_opt,
            picker2_opt,
            _angle_btn_opt,
        ) = if entry.style != FillStyle::Solid {
            let (color_btn2, swatch2_area, col2_cell) = create_swatch_button(entry.secondary_color);
            let picker2 = ColorPickerPopover::new(canvas.clone(), entry.secondary_color, mode1_idx);
            popovers_to_cleanup
                .borrow_mut()
                .push(picker2.popover().clone());
            picker2.attach_to(&color_btn2);
            let p2_open = picker2.clone();
            color_btn2.connect_clicked(move |_| {
                p2_open.popup();
            });
            line1.append(&color_btn2);

            let hex_entry2 = gtk4::Entry::builder()
                .text(entry.secondary_color.to_hex())
                .width_chars(8)
                .max_width_chars(9)
                .css_classes(["numeric", "pill-entry"])
                .valign(gtk4::Align::Center)
                .hexpand(true)
                .build();

            if entry.style == FillStyle::Pattern {
                let pat_btn = gtk4::Button::builder()
                    .label(entry.pattern_type.label())
                    .css_classes(["pill-btn"])
                    .valign(gtk4::Align::Center)
                    .tooltip_text(crate::core::gettext("Pattern Type"))
                    .build();

                let pat_pop = gtk4::Popover::builder().has_arrow(true).build();
                popovers_to_cleanup.borrow_mut().push(pat_pop.clone());
                let pat_menu = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(4)
                    .margin_top(6)
                    .margin_bottom(6)
                    .margin_start(6)
                    .margin_end(6)
                    .width_request(130)
                    .build();

                let pat_options = [
                    (
                        PatternType::Checkerboard,
                        crate::core::gettext("Checkerboard"),
                    ),
                    (PatternType::Dots, crate::core::gettext("Dots")),
                    (PatternType::Stripes, crate::core::gettext("Stripes")),
                    (PatternType::Grid, crate::core::gettext("Grid")),
                    (PatternType::Hexagon, crate::core::gettext("Honeycomb")),
                ];

                for (pt, pt_lbl) in pat_options {
                    let p_item = gtk4::Button::builder()
                        .label(pt_lbl)
                        .css_classes(["flat"])
                        .halign(gtk4::Align::Fill)
                        .build();

                    let list = list_ref.clone();
                    let canvas_p = canvas.clone();
                    let pp_c = pat_pop.clone();
                    let rebuild = rebuild_cb.clone();
                    p_item.connect_clicked(move |_| {
                        let mut l = list.borrow_mut();
                        if let Some(e) = l.get_mut(idx) {
                            e.pattern_type = pt;
                        }
                        let cloned = l.clone();
                        drop(l);
                        canvas_p.set_selected_fills(cloned);
                        pp_c.popdown();
                        rebuild();
                    });
                    pat_menu.append(&p_item);
                }

                pat_pop.set_child(Some(&pat_menu));
                pat_pop.set_parent(&pat_btn);

                let pop_c = pat_pop.clone();
                pat_btn.connect_clicked(move |_| {
                    pop_c.popup();
                });
                line1.append(&pat_btn);
            }

            if entry.style == FillStyle::Mesh {
                let mesh_tool_btn = gtk4::Button::builder()
                    .label(crate::core::gettext("Mesh"))
                    .icon_name("action-unavailable-symbolic")
                    .css_classes(["pill-btn"])
                    .valign(gtk4::Align::Center)
                    .tooltip_text(crate::core::gettext(
                        "Edit Mesh Nodes on Canvas (Mesh Tool)",
                    ))
                    .build();

                let canvas_m = canvas.clone();
                mesh_tool_btn.connect_clicked(move |_| {
                    canvas_m.set_active_tool("mesh_gradient");
                });
                line1.append(&mesh_tool_btn);
            }

            let angle_btn =
                if entry.style == FillStyle::LinearGradient || entry.style == FillStyle::Pattern {
                    let abtn = gtk4::Button::builder()
                        .label(format!("{}°", entry.angle.round() as i32))
                        .css_classes(["pill-btn", "numeric"])
                        .valign(gtk4::Align::Center)
                        .tooltip_text(crate::core::gettext("Angle"))
                        .build();

                    let angle_popover = gtk4::Popover::builder().has_arrow(true).build();
                    popovers_to_cleanup.borrow_mut().push(angle_popover.clone());
                    let angle_menu = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Vertical)
                        .spacing(4)
                        .margin_top(6)
                        .margin_bottom(6)
                        .margin_start(6)
                        .margin_end(6)
                        .width_request(140)
                        .build();

                    let angle_presets = [
                        (0.0f32, crate::core::gettext("0° (Horizontal →)")),
                        (45.0f32, crate::core::gettext("45° (Diagonal ↘)")),
                        (90.0f32, crate::core::gettext("90° (Vertical ↓)")),
                        (135.0f32, crate::core::gettext("135° (Diagonal ↙)")),
                        (180.0f32, crate::core::gettext("180° (Horizontal ←)")),
                        (270.0f32, crate::core::gettext("270° (Vertical ↑)")),
                    ];

                    for (ang, ang_lbl) in angle_presets {
                        let p_btn = gtk4::Button::builder()
                            .label(ang_lbl)
                            .css_classes(["flat"])
                            .halign(gtk4::Align::Fill)
                            .build();

                        let list = list_ref.clone();
                        let canvas_a = canvas.clone();
                        let ap_c = angle_popover.clone();
                        let rebuild = rebuild_cb.clone();
                        p_btn.connect_clicked(move |_| {
                            let mut l = list.borrow_mut();
                            if let Some(e) = l.get_mut(idx) {
                                e.angle = ang;
                            }
                            let cloned = l.clone();
                            drop(l);
                            canvas_a.set_selected_fills(cloned);
                            ap_c.popdown();
                            rebuild();
                        });
                        angle_menu.append(&p_btn);
                    }

                    angle_popover.set_child(Some(&angle_menu));
                    angle_popover.set_parent(&abtn);

                    let ap_open = angle_popover.clone();
                    abtn.connect_clicked(move |_| {
                        ap_open.popup();
                    });
                    line1.append(&abtn);
                    Some(abtn)
                } else {
                    None
                };

            (
                Some(color_btn2),
                Some(hex_entry2),
                Some(col2_cell),
                Some(swatch2_area),
                Some(picker2),
                angle_btn,
            )
        } else {
            (None, None, None, None, None, None)
        };

        if let Some(hex2) = &hex_entry2_opt {
            line1.append(&hex_entry1);
            line1.append(hex2);
        } else {
            line1.append(&hex_entry1);
        }

        let op_pct = (entry.opacity * 100.0).round() as i32;
        let op_entry = gtk4::Entry::builder()
            .text(format!("{}%", op_pct))
            .width_chars(5)
            .max_width_chars(5)
            .css_classes(["numeric", "pill-entry"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Opacity"))
            .hexpand(false)
            .build();
        line1.append(&op_entry);

        let eye_icon_name = if entry.enabled {
            "view-reveal-symbolic"
        } else {
            "view-conceal-symbolic"
        };
        let fill_vis_tip = if entry.enabled {
            crate::core::gettext("Hide fill")
        } else {
            crate::core::gettext("Show fill")
        };
        let vis_btn = gtk4::Button::builder()
            .icon_name(eye_icon_name)
            .css_classes(["flat", "circular"])
            .valign(gtk4::Align::Center)
            .tooltip_text(fill_vis_tip)
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
                canvas.set_selected_fills(cloned);
                rebuild();
            });
        }
        line1.append(&vis_btn);

        let del_btn = gtk4::Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(["flat", "circular"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Remove fill"))
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
                canvas.set_selected_fills(cloned);
                rebuild();
            });
        }
        line1.append(&del_btn);

        container.append(&line1);

        // Wire controls for Color 1
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let u = is_updating.clone();
            let hex = hex_entry1.clone();
            let col1_c = col1_cell.clone();
            let swatch1_a = swatch1_area.clone();
            picker1.on_color_changed(move |col| {
                col1_c.set(col);
                swatch1_a.queue_draw();
                u.set(true);
                hex.set_text(&col.to_hex());
                let mut l = list.borrow_mut();
                if let Some(e) = l.get_mut(idx) {
                    e.color = col;
                }
                let cloned = l.clone();
                drop(l);
                canvas.set_selected_fills(cloned);
                u.set(false);
            });
        }
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let u = is_updating.clone();
            let pick1 = picker1.clone();
            let col1_c = col1_cell.clone();
            let swatch1_a = swatch1_area.clone();
            hex_entry1.connect_activate(move |entry| {
                if u.get() {
                    return;
                }
                let text = entry.text();
                let hex_str = text.trim();
                if let Some(col) = Color::from_hex(hex_str) {
                    u.set(true);
                    pick1.set_color(col);
                    col1_c.set(col);
                    swatch1_a.queue_draw();
                    let mut l = list.borrow_mut();
                    if let Some(e) = l.get_mut(idx) {
                        e.color = col;
                    }
                    let cloned = l.clone();
                    drop(l);
                    canvas.set_selected_fills(cloned);
                    u.set(false);
                }
            });
        }

        // Wire controls for Color 2 (if present)
        if let (Some(hex2), Some(col2_c), Some(swatch2_a), Some(picker2)) =
            (hex_entry2_opt, col2_cell_opt, swatch2_area_opt, picker2_opt)
        {
            {
                let list = list_ref.clone();
                let canvas = canvas.clone();
                let u = is_updating.clone();
                let hex = hex2.clone();
                let col2_cell_c = col2_c.clone();
                let swatch2_area_c = swatch2_a.clone();
                picker2.on_color_changed(move |col| {
                    col2_cell_c.set(col);
                    swatch2_area_c.queue_draw();
                    u.set(true);
                    hex.set_text(&col.to_hex());
                    let mut l = list.borrow_mut();
                    if let Some(e) = l.get_mut(idx) {
                        e.secondary_color = col;
                    }
                    let cloned = l.clone();
                    drop(l);
                    canvas.set_selected_fills(cloned);
                    u.set(false);
                });
            }
            {
                let list = list_ref.clone();
                let canvas = canvas.clone();
                let u = is_updating.clone();
                let pick2 = picker2.clone();
                let col2_cell_c = col2_c.clone();
                let swatch2_area_c = swatch2_a.clone();
                hex2.connect_activate(move |entry| {
                    if u.get() {
                        return;
                    }
                    let text = entry.text();
                    let hex_str = text.trim();
                    if let Some(col) = Color::from_hex(hex_str) {
                        u.set(true);
                        pick2.set_color(col);
                        col2_cell_c.set(col);
                        swatch2_area_c.queue_draw();
                        let mut l = list.borrow_mut();
                        if let Some(e) = l.get_mut(idx) {
                            e.secondary_color = col;
                        }
                        let cloned = l.clone();
                        drop(l);
                        canvas.set_selected_fills(cloned);
                        u.set(false);
                    }
                });
            }
        }

        // Wire opacity control
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let u = is_updating.clone();
            op_entry.connect_activate(move |entry| {
                if u.get() {
                    return;
                }
                let cur_op = list.borrow().get(idx).map(|e| e.opacity * 100.0).unwrap_or(100.0);
                if let Ok(val) = crate::core::eval_math_expression(
                    entry.text().as_str(),
                    crate::core::Unit::Px,
                    Some(cur_op),
                ) {
                    let alpha = (val / 100.0).clamp(0.0, 1.0);
                    u.set(true);
                    entry.set_text(&format!("{}%", (alpha * 100.0).round() as i32));
                    let mut l = list.borrow_mut();
                    if let Some(e) = l.get_mut(idx) {
                        e.opacity = alpha;
                    }
                    let cloned = l.clone();
                    drop(l);
                    canvas.set_selected_fills(cloned);
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
