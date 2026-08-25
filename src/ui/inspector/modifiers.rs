use gtk4::prelude::*;
use std::rc::Rc;

use crate::core::modifier::{
    ArrayMode, ArrayModifier, ChamferRoundingModifier, EnvelopeWarpModifier, Modifier,
};
use crate::ui::canvas::CanvasWidget;

pub struct ModifiersSection {
    pub container: gtk4::Box,
    pub update_fn: Rc<dyn Fn()>,
}

pub fn build_modifiers_section(canvas: &CanvasWidget) -> ModifiersSection {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(12)
        .build();

    // 1. Selection Context Header Card
    let header_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(10)
        .margin_bottom(4)
        .css_classes(["card"])
        .build();
    let header_card_inner = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(10)
        .margin_start(12)
        .margin_end(12)
        .margin_top(10)
        .margin_bottom(10)
        .hexpand(true)
        .build();

    let context_icon = gtk4::Image::from_icon_name("builder-symbolic");
    context_icon.set_pixel_size(20);
    context_icon.set_opacity(0.8);

    let context_label_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let context_title = gtk4::Label::builder()
        .label(crate::core::gettext("Modifiers Stack"))
        .css_classes(["title-4"])
        .xalign(0.0)
        .build();

    let context_sub = gtk4::Label::builder()
        .label(crate::core::gettext("No selection"))
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .build();

    context_label_box.append(&context_title);
    context_label_box.append(&context_sub);

    // + Add Modifier Menu Button
    let add_mod_btn = gtk4::MenuButton::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(crate::core::gettext("Add Modifier"))
        .css_classes(["flat", "circular"])
        .valign(gtk4::Align::Center)
        .build();

    // Add Modifier Popover
    let popover = gtk4::Popover::builder()
        .position(gtk4::PositionType::Bottom)
        .has_arrow(true)
        .autohide(true)
        .css_classes(["menu"])
        .build();

    let pop_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(4)
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .width_request(200)
        .build();

    let pop_title = gtk4::Label::builder()
        .label(crate::core::gettext("Add Parametric Modifier"))
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .margin_start(6)
        .margin_bottom(4)
        .build();
    pop_box.append(&pop_title);

    let btn_add_array = gtk4::Button::builder()
        .label(crate::core::gettext("⚡ Array Modifier"))
        .css_classes(["flat"])
        .halign(gtk4::Align::Fill)
        .build();
    pop_box.append(&btn_add_array);

    let btn_add_env = gtk4::Button::builder()
        .label(crate::core::gettext("🌊 Envelope Warp"))
        .css_classes(["flat"])
        .halign(gtk4::Align::Fill)
        .build();
    pop_box.append(&btn_add_env);

    let btn_add_chamfer = gtk4::Button::builder()
        .label(crate::core::gettext("📐 Dynamic Chamfer"))
        .css_classes(["flat"])
        .halign(gtk4::Align::Fill)
        .build();
    pop_box.append(&btn_add_chamfer);

    popover.set_child(Some(&pop_box));
    add_mod_btn.set_popover(Some(&popover));

    header_card_inner.append(&context_icon);
    header_card_inner.append(&context_label_box);
    header_card_inner.append(&add_mod_btn);
    header_card.append(&header_card_inner);
    container.append(&header_card);

    // Modifiers List Container
    let list_container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .build();
    container.append(&list_container);

    let list_c = list_container.clone();
    let ctx_sub_c = context_sub.clone();
    let ctx_icon_c = context_icon.clone();
    let canvas_c = canvas.clone();

    let update_fn = Rc::new(move || {
        crate::ui::inspector::appearance::clear_box(&list_c);

        let sel_ids = canvas_c.selected_element_ids();

        if sel_ids.is_empty() {
            ctx_sub_c.set_text(&crate::core::gettext("Select an object to add modifiers"));
            ctx_icon_c.set_icon_name(Some("dialog-information-symbolic"));
            return;
        }

        let first_id = sel_ids[0];
        let state_ref = canvas_c.state.borrow();
        let elem_opt = state_ref.document.find_element(first_id);

        if let Some(elem) = elem_opt {
            let elem_type_str = match elem {
                crate::core::Element::Rect(_) => crate::core::gettext("Vector Shape Selected"),
                crate::core::Element::Path(_) => crate::core::gettext("Vector Path Selected"),
                crate::core::Element::Brush(_) => crate::core::gettext("Freehand Brush Selected"),
                crate::core::Element::Text(_) => crate::core::gettext("Text Element Selected"),
                crate::core::Element::Image(_) => crate::core::gettext("Bitmap Image Selected"),
                crate::core::Element::Group(_) => crate::core::gettext("Group Selected"),
                crate::core::Element::Clone(_) => crate::core::gettext("Linked Clone Selected"),
            };
            ctx_sub_c.set_text(&elem_type_str);

            let icon_name = match elem {
                crate::core::Element::Image(_) => "image-x-generic-symbolic",
                crate::core::Element::Text(_) => "tool-text-symbolic",
                crate::core::Element::Group(_) => "folder-symbolic",
                _ => "builder-symbolic",
            };
            ctx_icon_c.set_icon_name(Some(icon_name));

            let mods = elem.modifiers();
            if mods.is_empty() {
                let empty_lbl = gtk4::Label::builder()
                    .label(crate::core::gettext("No active modifiers. Click + to add one."))
                    .css_classes(["caption", "dim-label"])
                    .margin_top(12)
                    .margin_bottom(12)
                    .build();
                list_c.append(&empty_lbl);
            } else {
                for (mod_idx, m) in mods.iter().enumerate() {
                    let card = build_modifier_card(
                        mod_idx,
                        m,
                        first_id,
                        &canvas_c,
                    );
                    list_c.append(&card);
                }
            }
        }
    });

    // Wire Popover Actions
    let update_ref1 = update_fn.clone();
    let canvas_a = canvas.clone();
    let pop_a = popover.clone();
    btn_add_array.connect_clicked(move |_| {
        pop_a.popdown();
        let sel_ids = canvas_a.selected_element_ids();
        if !sel_ids.is_empty() {
            let mut state = canvas_a.state.borrow_mut();
            if let Some(elem) = state.document.find_element_mut(sel_ids[0]) {
                if let Some(mods) = elem.modifiers_mut() {
                    mods.push(Modifier::Array(ArrayModifier::default()));
                }
            }
        }
        canvas_a.queue_draw();
        update_ref1();
    });

    let update_ref2 = update_fn.clone();
    let canvas_e = canvas.clone();
    let pop_e = popover.clone();
    btn_add_env.connect_clicked(move |_| {
        pop_e.popdown();
        let sel_ids = canvas_e.selected_element_ids();
        if !sel_ids.is_empty() {
            let mut state = canvas_e.state.borrow_mut();
            if let Some(elem) = state.document.find_element_mut(sel_ids[0]) {
                if let Some(mods) = elem.modifiers_mut() {
                    mods.push(Modifier::EnvelopeWarp(EnvelopeWarpModifier::default()));
                }
            }
        }
        canvas_e.queue_draw();
        update_ref2();
    });

    let update_ref3 = update_fn.clone();
    let canvas_c_chamf = canvas.clone();
    let pop_c_chamf = popover.clone();
    btn_add_chamfer.connect_clicked(move |_| {
        pop_c_chamf.popdown();
        let sel_ids = canvas_c_chamf.selected_element_ids();
        if !sel_ids.is_empty() {
            let mut state = canvas_c_chamf.state.borrow_mut();
            if let Some(elem) = state.document.find_element_mut(sel_ids[0]) {
                if let Some(mods) = elem.modifiers_mut() {
                    mods.push(Modifier::ChamferRounding(ChamferRoundingModifier::default()));
                }
            }
        }
        canvas_c_chamf.queue_draw();
        update_ref3();
    });

    ModifiersSection {
        container,
        update_fn,
    }
}

fn build_modifier_card(
    mod_idx: usize,
    modifier: &Modifier,
    elem_id: crate::core::ElementId,
    canvas: &CanvasWidget,
) -> gtk4::Box {
    let card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(0)
        .css_classes(["card"])
        .build();

    let header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let icon = gtk4::Image::from_icon_name(modifier.icon_name());
    icon.set_pixel_size(16);

    let name_lbl = gtk4::Label::builder()
        .label(modifier.name())
        .css_classes(["heading"])
        .hexpand(true)
        .xalign(0.0)
        .build();

    let switch = gtk4::Switch::builder()
        .active(modifier.enabled())
        .valign(gtk4::Align::Center)
        .build();

    let del_btn = gtk4::Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text(crate::core::gettext("Delete Modifier"))
        .css_classes(["flat", "circular"])
        .valign(gtk4::Align::Center)
        .build();

    header.append(&icon);
    header.append(&name_lbl);
    header.append(&switch);
    header.append(&del_btn);
    card.append(&header);

    // Body Controls
    let body = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(10)
        .build();

    match modifier {
        Modifier::Array(arr) => {
            let mode_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(6)
                .build();
            let mode_lbl = gtk4::Label::builder()
                .label(crate::core::gettext("Mode"))
                .css_classes(["caption", "dim-label"])
                .build();
            let mode_combo = gtk4::DropDown::from_strings(&[
                &crate::core::gettext("Linear"),
                &crate::core::gettext("Radial"),
                &crate::core::gettext("Grid"),
            ]);
            let active_idx = match arr.mode {
                ArrayMode::Linear { .. } => 0,
                ArrayMode::Radial { .. } => 1,
                ArrayMode::Grid { .. } => 2,
            };
            mode_combo.set_selected(active_idx);
            mode_box.append(&mode_lbl);
            mode_box.append(&mode_combo);
            body.append(&mode_box);

            match &arr.mode {
                ArrayMode::Linear {
                    count,
                    offset_x,
                    offset_y,
                    scale_step: _,
                    rotate_step_deg,
                } => {
                    let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

                    grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("Count"))), 0, 0, 1, 1);
                    let spin_count = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
                    spin_count.set_value(*count as f64);
                    grid.attach(&spin_count, 1, 0, 1, 1);

                    grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("DX / DY"))), 0, 1, 1, 1);
                    let dx_dy_box = gtk4::Box::builder().spacing(4).build();
                    let spin_dx = gtk4::SpinButton::with_range(-2000.0, 2000.0, 5.0);
                    spin_dx.set_value(*offset_x as f64);
                    let spin_dy = gtk4::SpinButton::with_range(-2000.0, 2000.0, 5.0);
                    spin_dy.set_value(*offset_y as f64);
                    dx_dy_box.append(&spin_dx);
                    dx_dy_box.append(&spin_dy);
                    grid.attach(&dx_dy_box, 1, 1, 1, 1);

                    grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("Rotate (°)"))), 0, 2, 1, 1);
                    let spin_rot = gtk4::SpinButton::with_range(-360.0, 360.0, 5.0);
                    spin_rot.set_value(*rotate_step_deg as f64);
                    grid.attach(&spin_rot, 1, 2, 1, 1);

                    body.append(&grid);

                    let canvas_cb = canvas.clone();
                    let update_array = move |c: u32, dx: f32, dy: f32, rot: f32| {
                        let mut state = canvas_cb.state.borrow_mut();
                        if let Some(elem) = state.document.find_element_mut(elem_id) {
                            if let Some(mods) = elem.modifiers_mut() {
                                if let Some(Modifier::Array(a)) = mods.get_mut(mod_idx) {
                                    a.mode = ArrayMode::Linear {
                                        count: c,
                                        offset_x: dx,
                                        offset_y: dy,
                                        scale_step: 1.0,
                                        rotate_step_deg: rot,
                                    };
                                }
                            }
                        }
                        canvas_cb.queue_draw();
                    };

                    let u_sc = update_array.clone();
                    let s_dx = spin_dx.clone();
                    let s_dy = spin_dy.clone();
                    let s_rot = spin_rot.clone();
                    spin_count.connect_value_changed(move |s| {
                        u_sc(s.value() as u32, s_dx.value() as f32, s_dy.value() as f32, s_rot.value() as f32);
                    });

                    let u_dx = update_array.clone();
                    let s_cnt = spin_count.clone();
                    let s_dy2 = spin_dy.clone();
                    let s_rot2 = spin_rot.clone();
                    spin_dx.connect_value_changed(move |s| {
                        u_dx(s_cnt.value() as u32, s.value() as f32, s_dy2.value() as f32, s_rot2.value() as f32);
                    });

                    let u_dy = update_array.clone();
                    let s_cnt3 = spin_count.clone();
                    let s_dx3 = spin_dx.clone();
                    let s_rot3 = spin_rot.clone();
                    spin_dy.connect_value_changed(move |s| {
                        u_dy(s_cnt3.value() as u32, s_dx3.value() as f32, s.value() as f32, s_rot3.value() as f32);
                    });

                    let u_rot = update_array.clone();
                    let s_cnt4 = spin_count.clone();
                    let s_dx4 = spin_dx.clone();
                    let s_dy4 = spin_dy.clone();
                    spin_rot.connect_value_changed(move |s| {
                        u_rot(s_cnt4.value() as u32, s_dx4.value() as f32, s_dy4.value() as f32, s.value() as f32);
                    });
                }
                ArrayMode::Radial {
                    count,
                    radius,
                    start_angle_deg: _,
                    total_angle_deg: _,
                    rotate_copies: _,
                } => {
                    let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

                    grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("Count"))), 0, 0, 1, 1);
                    let spin_count = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
                    spin_count.set_value(*count as f64);
                    grid.attach(&spin_count, 1, 0, 1, 1);

                    grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("Radius"))), 0, 1, 1, 1);
                    let spin_rad = gtk4::SpinButton::with_range(0.0, 2000.0, 10.0);
                    spin_rad.set_value(*radius as f64);
                    grid.attach(&spin_rad, 1, 1, 1, 1);

                    body.append(&grid);

                    let canvas_cb = canvas.clone();
                    let update_radial = move |c: u32, r: f32| {
                        let mut state = canvas_cb.state.borrow_mut();
                        if let Some(elem) = state.document.find_element_mut(elem_id) {
                            if let Some(mods) = elem.modifiers_mut() {
                                if let Some(Modifier::Array(a)) = mods.get_mut(mod_idx) {
                                    a.mode = ArrayMode::Radial {
                                        count: c,
                                        radius: r,
                                        start_angle_deg: 0.0,
                                        total_angle_deg: 360.0,
                                        rotate_copies: true,
                                    };
                                }
                            }
                        }
                        canvas_cb.queue_draw();
                    };

                    let u_sc = update_radial.clone();
                    let s_r = spin_rad.clone();
                    spin_count.connect_value_changed(move |s| {
                        u_sc(s.value() as u32, s_r.value() as f32);
                    });

                    let u_r = update_radial.clone();
                    let s_c = spin_count.clone();
                    spin_rad.connect_value_changed(move |s| {
                        u_r(s_c.value() as u32, s.value() as f32);
                    });
                }
                ArrayMode::Grid {
                    rows,
                    cols,
                    spacing_x,
                    spacing_y,
                } => {
                    let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

                    grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("Rows / Cols"))), 0, 0, 1, 1);
                    let rc_box = gtk4::Box::builder().spacing(4).build();
                    let spin_r = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
                    spin_r.set_value(*rows as f64);
                    let spin_c = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
                    spin_c.set_value(*cols as f64);
                    rc_box.append(&spin_r);
                    rc_box.append(&spin_c);
                    grid.attach(&rc_box, 1, 0, 1, 1);

                    grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("Spacing X/Y"))), 0, 1, 1, 1);
                    let sp_box = gtk4::Box::builder().spacing(4).build();
                    let spin_sx = gtk4::SpinButton::with_range(0.0, 1000.0, 5.0);
                    spin_sx.set_value(*spacing_x as f64);
                    let spin_sy = gtk4::SpinButton::with_range(0.0, 1000.0, 5.0);
                    spin_sy.set_value(*spacing_y as f64);
                    sp_box.append(&spin_sx);
                    sp_box.append(&spin_sy);
                    grid.attach(&sp_box, 1, 1, 1, 1);

                    body.append(&grid);

                    let canvas_cb = canvas.clone();
                    let update_g = move |r: u32, c: u32, sx: f32, sy: f32| {
                        let mut state = canvas_cb.state.borrow_mut();
                        if let Some(elem) = state.document.find_element_mut(elem_id) {
                            if let Some(mods) = elem.modifiers_mut() {
                                if let Some(Modifier::Array(a)) = mods.get_mut(mod_idx) {
                                    a.mode = ArrayMode::Grid {
                                        rows: r,
                                        cols: c,
                                        spacing_x: sx,
                                        spacing_y: sy,
                                    };
                                }
                            }
                        }
                        canvas_cb.queue_draw();
                    };

                    let u_g = update_g.clone();
                    let s_c = spin_c.clone();
                    let s_sx = spin_sx.clone();
                    let s_sy = spin_sy.clone();
                    spin_r.connect_value_changed(move |s| {
                        u_g(s.value() as u32, s_c.value() as u32, s_sx.value() as f32, s_sy.value() as f32);
                    });
                }
            }
        }
        Modifier::EnvelopeWarp(env) => {
            let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

            grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("Top-Left Offset"))), 0, 0, 1, 1);
            let spin_tl_x = gtk4::SpinButton::with_range(-500.0, 500.0, 2.0);
            spin_tl_x.set_value(env.top_left_offset.x as f64);
            grid.attach(&spin_tl_x, 1, 0, 1, 1);

            grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("Top-Right Offset"))), 0, 1, 1, 1);
            let spin_tr_x = gtk4::SpinButton::with_range(-500.0, 500.0, 2.0);
            spin_tr_x.set_value(env.top_right_offset.x as f64);
            grid.attach(&spin_tr_x, 1, 1, 1, 1);

            body.append(&grid);

            let canvas_cb = canvas.clone();
            spin_tl_x.connect_value_changed(move |s| {
                let mut state = canvas_cb.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::EnvelopeWarp(e)) = mods.get_mut(mod_idx) {
                            e.top_left_offset.x = s.value() as f32;
                        }
                    }
                }
                canvas_cb.queue_draw();
            });
        }
        Modifier::ChamferRounding(ch) => {
            let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

            grid.attach(&gtk4::Label::new(Some(&crate::core::gettext("Radius"))), 0, 0, 1, 1);
            let spin_r = gtk4::SpinButton::with_range(0.0, 200.0, 1.0);
            spin_r.set_value(ch.radius as f64);
            grid.attach(&spin_r, 1, 0, 1, 1);

            body.append(&grid);

            let canvas_cb = canvas.clone();
            spin_r.connect_value_changed(move |s| {
                let mut state = canvas_cb.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::ChamferRounding(c)) = mods.get_mut(mod_idx) {
                            c.radius = s.value() as f32;
                        }
                    }
                }
                canvas_cb.queue_draw();
            });
        }
    }

    card.append(&body);

    // Switch enable/disable handler
    let canvas_sw = canvas.clone();
    switch.connect_active_notify(move |sw| {
        let active = sw.is_active();
        let mut state = canvas_sw.state.borrow_mut();
        if let Some(elem) = state.document.find_element_mut(elem_id) {
            if let Some(mods) = elem.modifiers_mut() {
                if let Some(m) = mods.get_mut(mod_idx) {
                    m.set_enabled(active);
                }
            }
        }
        canvas_sw.queue_draw();
    });

    // Delete modifier handler
    let canvas_del = canvas.clone();
    del_btn.connect_clicked(move |_| {
        let mut state = canvas_del.state.borrow_mut();
        if let Some(elem) = state.document.find_element_mut(elem_id) {
            if let Some(mods) = elem.modifiers_mut() {
                if mod_idx < mods.len() {
                    mods.remove(mod_idx);
                }
            }
        }
        canvas_del.queue_draw();
    });

    card
}
