use gtk4::prelude::*;
use std::rc::Rc;

use crate::core::element::CornerStyle;
use crate::core::geometry::Rect;
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

    // 1. Sleek GNOME Adwaita Header Card
    let header_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .margin_bottom(4)
        .css_classes(["card"])
        .build();

    let header_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .margin_start(12)
        .margin_end(10)
        .margin_top(8)
        .margin_bottom(4)
        .build();

    let header_icon = gtk4::Image::from_icon_name("view-grid-symbolic");
    header_icon.set_pixel_size(16);

    let header_title = gtk4::Label::builder()
        .label(crate::core::gettext("Modifiers"))
        .css_classes(["heading"])
        .hexpand(true)
        .xalign(0.0)
        .build();

    // + Add Modifier Menu Button (ALWAYS ACTIVE & ENABLED)
    let add_mod_btn = gtk4::MenuButton::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(crate::core::gettext("Add Modifier"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .sensitive(true)
        .build();

    // Extensible Searchable Modifier Catalog Gallery Popover
    let popover = gtk4::Popover::builder()
        .position(gtk4::PositionType::Bottom)
        .has_arrow(true)
        .autohide(true)
        .css_classes(["menu"])
        .build();

    let pop_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .width_request(280)
        .build();

    // Search bar for filtering modifiers
    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text(crate::core::gettext("Search Modifiers..."))
        .margin_bottom(4)
        .build();
    pop_box.append(&search_entry);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .max_content_height(340)
        .propagate_natural_height(true)
        .build();

    let catalog_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_end(4)
        .build();

    // ── CATEGORY 1: GENERATE & DUPLICATE ──
    let cat_dup_title = gtk4::Label::builder()
        .label(crate::core::gettext("GENERATE & DUPLICATE"))
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .margin_start(4)
        .build();
    let cat_dup_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .build();

    let btn_add_array = create_menu_item_button(
        "view-grid-symbolic",
        &crate::core::gettext("Array Modifier"),
        &crate::core::gettext("Linear, Radial & Grid Duplication"),
    );
    cat_dup_box.append(&btn_add_array);

    // ── CATEGORY 2: DEFORM & WARP ──
    let cat_deform_title = gtk4::Label::builder()
        .label(crate::core::gettext("DEFORM & WARP"))
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .margin_start(4)
        .margin_top(4)
        .build();
    let cat_deform_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .build();

    let btn_add_env = create_menu_item_button(
        "transform-symbolic",
        &crate::core::gettext("Envelope Warp"),
        &crate::core::gettext("4-Point Mesh Distortion"),
    );
    cat_deform_box.append(&btn_add_env);

    // ── CATEGORY 3: PATH & CORNERS ──
    let cat_path_title = gtk4::Label::builder()
        .label(crate::core::gettext("PATH & CORNERS"))
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .margin_start(4)
        .margin_top(4)
        .build();
    let cat_path_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .build();

    let btn_add_chamfer = create_menu_item_button(
        "tool-node-symbolic",
        &crate::core::gettext("Dynamic Chamfer"),
        &crate::core::gettext("Corner Rounding & Bevels"),
    );
    cat_path_box.append(&btn_add_chamfer);

    catalog_box.append(&cat_dup_title);
    catalog_box.append(&cat_dup_box);
    catalog_box.append(&cat_deform_title);
    catalog_box.append(&cat_deform_box);
    catalog_box.append(&cat_path_title);
    catalog_box.append(&cat_path_box);

    scrolled.set_child(Some(&catalog_box));
    pop_box.append(&scrolled);

    popover.set_child(Some(&pop_box));
    add_mod_btn.set_popover(Some(&popover));

    // Search filter callback
    let b_array_c = btn_add_array.clone();
    let b_env_c = btn_add_env.clone();
    let b_chamf_c = btn_add_chamfer.clone();
    search_entry.connect_search_changed(move |se| {
        let q = se.text().to_lowercase();
        let match_array = q.is_empty() || "array modifier linear radial grid duplication".contains(&q);
        let match_env = q.is_empty() || "envelope warp distortion mesh 4-point".contains(&q);
        let match_chamf = q.is_empty() || "dynamic chamfer corner rounding bevels".contains(&q);

        b_array_c.set_visible(match_array);
        b_env_c.set_visible(match_env);
        b_chamf_c.set_visible(match_chamf);
    });

    header_row.append(&header_icon);
    header_row.append(&header_title);
    header_row.append(&add_mod_btn);

    let context_sub = gtk4::Label::builder()
        .label(crate::core::gettext("No object selected"))
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(8)
        .build();

    header_card.append(&header_row);
    header_card.append(&context_sub);
    container.append(&header_card);

    // Modifiers List Container
    let list_container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .build();
    container.append(&list_container);

    let list_c = list_container.clone();
    let ctx_sub_c = context_sub.clone();
    let header_icon_c = header_icon.clone();
    let canvas_c = canvas.clone();

    let update_fn_holder = Rc::new(std::cell::RefCell::new(None::<Rc<dyn Fn()>>));
    let update_fn_holder_c = update_fn_holder.clone();

    let update_fn = Rc::new(move || {
        crate::ui::inspector::appearance::clear_box(&list_c);

        let sel_ids = canvas_c.selected_element_ids();

        if sel_ids.is_empty() {
            ctx_sub_c.set_text(&crate::core::gettext("No object selected"));
            header_icon_c.set_icon_name(Some("view-grid-symbolic"));
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

            let mods = elem.modifiers();
            if mods.is_empty() {
                let empty_lbl = gtk4::Label::builder()
                    .label(crate::core::gettext("No active modifiers"))
                    .css_classes(["caption", "dim-label"])
                    .margin_top(4)
                    .margin_bottom(4)
                    .build();
                list_c.append(&empty_lbl);
            } else {
                let self_update = update_fn_holder_c.borrow().clone();
                let total_mods = mods.len();
                for (mod_idx, m) in mods.iter().enumerate() {
                    let card = build_modifier_card(
                        mod_idx,
                        total_mods,
                        m,
                        first_id,
                        &canvas_c,
                        self_update.clone(),
                    );
                    list_c.append(&card);
                }
            }
        }
    });

    *update_fn_holder.borrow_mut() = Some(update_fn.clone());

    // Helper to ensure target element exists (creates a shape if none selected)
    let ensure_target_element = |canvas_widget: &CanvasWidget, modifier: Modifier| {
        let sel_ids = canvas_widget.selected_element_ids();
        if !sel_ids.is_empty() {
            let mut state = canvas_widget.state.borrow_mut();
            if let Some(elem) = state.document.find_element_mut(sel_ids[0]) {
                if let Some(mods) = elem.modifiers_mut() {
                    mods.push(modifier);
                }
            }
            drop(state);
            if let Ok(st) = canvas_widget.state.try_borrow() {
                st.notify_status();
            }
        } else {
            // Create a default Rectangle element on active page and select it
            let rect = Rect::new(200.0, 200.0, 160.0, 160.0);
            let mut new_rect = crate::core::RectElement::new(rect, Some(crate::core::Color::new(0.2, 0.5, 0.9, 1.0)), None);
            new_rect.modifiers.push(modifier);
            let new_elem = crate::core::Element::Rect(new_rect);
            let new_id = new_elem.id();

            let mut state = canvas_widget.state.borrow_mut();
            state.document.elements.push(new_elem);
            state.document.selected_ids.clear();
            state.document.selected_ids.insert(new_id);
            drop(state);
            if let Ok(st) = canvas_widget.state.try_borrow() {
                st.notify_status();
            }
        }
    };

    // Wire Popover Actions
    let update_ref1 = update_fn.clone();
    let canvas_a = canvas.clone();
    let pop_a = popover.clone();
    btn_add_array.connect_clicked(move |_| {
        pop_a.popdown();
        ensure_target_element(&canvas_a, Modifier::Array(ArrayModifier::default()));
        canvas_a.queue_draw();
        update_ref1();
    });

    let update_ref2 = update_fn.clone();
    let canvas_e = canvas.clone();
    let pop_e = popover.clone();
    btn_add_env.connect_clicked(move |_| {
        pop_e.popdown();
        ensure_target_element(&canvas_e, Modifier::EnvelopeWarp(EnvelopeWarpModifier::default()));
        canvas_e.queue_draw();
        update_ref2();
    });

    let update_ref3 = update_fn.clone();
    let canvas_c_chamf = canvas.clone();
    let pop_c_chamf = popover.clone();
    btn_add_chamfer.connect_clicked(move |_| {
        pop_c_chamf.popdown();
        ensure_target_element(&canvas_c_chamf, Modifier::ChamferRounding(ChamferRoundingModifier::default()));
        canvas_c_chamf.queue_draw();
        update_ref3();
    });

    ModifiersSection {
        container,
        update_fn,
    }
}

fn create_menu_item_button(icon_name: &str, title: &str, subtitle: &str) -> gtk4::Button {
    let btn = gtk4::Button::builder()
        .css_classes(["flat"])
        .halign(gtk4::Align::Fill)
        .build();

    let box_item = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .margin_start(8)
        .margin_end(8)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    let img = gtk4::Image::from_icon_name(icon_name);
    img.set_pixel_size(18);

    let lbl_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .valign(gtk4::Align::Center)
        .build();

    let title_lbl = gtk4::Label::builder()
        .label(title)
        .css_classes(["body"])
        .xalign(0.0)
        .build();

    let sub_lbl = gtk4::Label::builder()
        .label(subtitle)
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .build();

    lbl_box.append(&title_lbl);
    lbl_box.append(&sub_lbl);

    box_item.append(&img);
    box_item.append(&lbl_box);

    btn.set_child(Some(&box_item));
    btn
}

fn build_modifier_card(
    mod_idx: usize,
    total_mods: usize,
    modifier: &Modifier,
    elem_id: crate::core::ElementId,
    canvas: &CanvasWidget,
    on_change: Option<Rc<dyn Fn()>>,
) -> gtk4::Box {
    let card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(0)
        .css_classes(["card"])
        .build();

    let header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_start(10)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let badge_lbl = gtk4::Label::builder()
        .label(&format!("#{}", mod_idx + 1))
        .css_classes(["caption", "dim-label"])
        .valign(gtk4::Align::Center)
        .build();

    let icon = gtk4::Image::from_icon_name(modifier.icon_name());
    icon.set_pixel_size(16);

    let name_lbl = gtk4::Label::builder()
        .label(modifier.name())
        .css_classes(["heading"])
        .hexpand(true)
        .xalign(0.0)
        .build();

    // Reorder Buttons: Up and Down
    let btn_up = gtk4::Button::builder()
        .icon_name("go-up-symbolic")
        .tooltip_text(crate::core::gettext("Move Up"))
        .css_classes(["flat", "circular"])
        .valign(gtk4::Align::Center)
        .sensitive(mod_idx > 0)
        .build();

    let btn_dn = gtk4::Button::builder()
        .icon_name("go-down-symbolic")
        .tooltip_text(crate::core::gettext("Move Down"))
        .css_classes(["flat", "circular"])
        .valign(gtk4::Align::Center)
        .sensitive(mod_idx + 1 < total_mods)
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

    header.append(&badge_lbl);
    header.append(&icon);
    header.append(&name_lbl);
    header.append(&btn_up);
    header.append(&btn_dn);
    header.append(&switch);
    header.append(&del_btn);
    card.append(&header);

    // Reorder Up Handler
    let canvas_up = canvas.clone();
    let on_up_update = on_change.clone();
    btn_up.connect_clicked(move |_| {
        let mut state = canvas_up.state.borrow_mut();
        if let Some(elem) = state.document.find_element_mut(elem_id) {
            if let Some(mods) = elem.modifiers_mut() {
                if mod_idx > 0 && mod_idx < mods.len() {
                    mods.swap(mod_idx, mod_idx - 1);
                }
            }
        }
        drop(state);
        if let Ok(st) = canvas_up.state.try_borrow() {
            st.notify_status();
        }
        canvas_up.queue_draw();
        if let Some(ref update_fn) = on_up_update {
            update_fn();
        }
    });

    // Reorder Down Handler
    let canvas_dn = canvas.clone();
    let on_dn_update = on_change.clone();
    btn_dn.connect_clicked(move |_| {
        let mut state = canvas_dn.state.borrow_mut();
        if let Some(elem) = state.document.find_element_mut(elem_id) {
            if let Some(mods) = elem.modifiers_mut() {
                if mod_idx + 1 < mods.len() {
                    mods.swap(mod_idx, mod_idx + 1);
                }
            }
        }
        drop(state);
        if let Ok(st) = canvas_dn.state.try_borrow() {
            st.notify_status();
        }
        canvas_dn.queue_draw();
        if let Some(ref update_fn) = on_dn_update {
            update_fn();
        }
    });

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
                .spacing(8)
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

            // Mode switcher callback
            let canvas_m = canvas.clone();
            let on_ch_mode = on_change.clone();
            mode_combo.connect_selected_notify(move |cb| {
                let sel = cb.selected();
                let mut state = canvas_m.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::Array(a)) = mods.get_mut(mod_idx) {
                            a.mode = match sel {
                                1 => ArrayMode::Radial {
                                    count: 6,
                                    radius: 80.0,
                                    start_angle_deg: 0.0,
                                    total_angle_deg: 360.0,
                                    rotate_copies: true,
                                },
                                2 => ArrayMode::Grid {
                                    rows: 3,
                                    cols: 3,
                                    spacing_x: 50.0,
                                    spacing_y: 50.0,
                                },
                                _ => ArrayMode::Linear {
                                    count: 4,
                                    offset_x: 40.0,
                                    offset_y: 0.0,
                                    scale_step: 1.0,
                                    rotate_step_deg: 0.0,
                                },
                            };
                        }
                    }
                }
                drop(state);
                if let Ok(st) = canvas_m.state.try_borrow() {
                    st.notify_status();
                }
                canvas_m.queue_draw();
                if let Some(ref cb_fn) = on_ch_mode {
                    cb_fn();
                }
            });

            match &arr.mode {
                ArrayMode::Linear {
                    count,
                    offset_x,
                    offset_y,
                    scale_step: _,
                    rotate_step_deg,
                } => {
                    let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

                    let lbl_count = gtk4::Label::new(Some(&crate::core::gettext("Count")));
                    lbl_count.set_css_classes(&["caption", "dim-label"]);
                    grid.attach(&lbl_count, 0, 0, 1, 1);

                    let spin_count = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
                    spin_count.set_value(*count as f64);
                    grid.attach(&spin_count, 1, 0, 1, 1);

                    let lbl_dx = gtk4::Label::new(Some(&crate::core::gettext("Offset X/Y")));
                    lbl_dx.set_css_classes(&["caption", "dim-label"]);
                    grid.attach(&lbl_dx, 0, 1, 1, 1);

                    let dx_dy_box = gtk4::Box::builder().spacing(4).build();
                    let spin_dx = gtk4::SpinButton::with_range(-2000.0, 2000.0, 5.0);
                    spin_dx.set_value(*offset_x as f64);
                    let spin_dy = gtk4::SpinButton::with_range(-2000.0, 2000.0, 5.0);
                    spin_dy.set_value(*offset_y as f64);
                    dx_dy_box.append(&spin_dx);
                    dx_dy_box.append(&spin_dy);
                    grid.attach(&dx_dy_box, 1, 1, 1, 1);

                    let lbl_rot = gtk4::Label::new(Some(&crate::core::gettext("Rotate (°)")));
                    lbl_rot.set_css_classes(&["caption", "dim-label"]);
                    grid.attach(&lbl_rot, 0, 2, 1, 1);

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
                        drop(state);
                        if let Ok(st) = canvas_cb.state.try_borrow() {
                            st.notify_status();
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

                    let lbl_count = gtk4::Label::new(Some(&crate::core::gettext("Count")));
                    lbl_count.set_css_classes(&["caption", "dim-label"]);
                    grid.attach(&lbl_count, 0, 0, 1, 1);

                    let spin_count = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
                    spin_count.set_value(*count as f64);
                    grid.attach(&spin_count, 1, 0, 1, 1);

                    let lbl_rad = gtk4::Label::new(Some(&crate::core::gettext("Radius")));
                    lbl_rad.set_css_classes(&["caption", "dim-label"]);
                    grid.attach(&lbl_rad, 0, 1, 1, 1);

                    let spin_rad = gtk4::SpinButton::with_range(0.0, 2000.0, 5.0);
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
                        drop(state);
                        if let Ok(st) = canvas_cb.state.try_borrow() {
                            st.notify_status();
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

                    let lbl_rc = gtk4::Label::new(Some(&crate::core::gettext("Rows / Cols")));
                    lbl_rc.set_css_classes(&["caption", "dim-label"]);
                    grid.attach(&lbl_rc, 0, 0, 1, 1);

                    let rc_box = gtk4::Box::builder().spacing(4).build();
                    let spin_r = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
                    spin_r.set_value(*rows as f64);
                    let spin_c = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
                    spin_c.set_value(*cols as f64);
                    rc_box.append(&spin_r);
                    rc_box.append(&spin_c);
                    grid.attach(&rc_box, 1, 0, 1, 1);

                    let lbl_sp = gtk4::Label::new(Some(&crate::core::gettext("Spacing X/Y")));
                    lbl_sp.set_css_classes(&["caption", "dim-label"]);
                    grid.attach(&lbl_sp, 0, 1, 1, 1);

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
                        drop(state);
                        if let Ok(st) = canvas_cb.state.try_borrow() {
                            st.notify_status();
                        }
                        canvas_cb.queue_draw();
                    };

                    let u_g1 = update_g.clone();
                    let s_c1 = spin_c.clone();
                    let s_sx1 = spin_sx.clone();
                    let s_sy1 = spin_sy.clone();
                    spin_r.connect_value_changed(move |s| {
                        u_g1(s.value() as u32, s_c1.value() as u32, s_sx1.value() as f32, s_sy1.value() as f32);
                    });

                    let u_g2 = update_g.clone();
                    let s_r2 = spin_r.clone();
                    let s_sx2 = spin_sx.clone();
                    let s_sy2 = spin_sy.clone();
                    spin_c.connect_value_changed(move |s| {
                        u_g2(s_r2.value() as u32, s.value() as u32, s_sx2.value() as f32, s_sy2.value() as f32);
                    });

                    let u_g3 = update_g.clone();
                    let s_r3 = spin_r.clone();
                    let s_c3 = spin_c.clone();
                    let s_sy3 = spin_sy.clone();
                    spin_sx.connect_value_changed(move |s| {
                        u_g3(s_r3.value() as u32, s_c3.value() as u32, s.value() as f32, s_sy3.value() as f32);
                    });

                    let u_g4 = update_g.clone();
                    let s_r4 = spin_r.clone();
                    let s_c4 = spin_c.clone();
                    let s_sx4 = spin_sx.clone();
                    spin_sy.connect_value_changed(move |s| {
                        u_g4(s_r4.value() as u32, s_c4.value() as u32, s_sx4.value() as f32, s.value() as f32);
                    });
                }
            }
        }
        Modifier::EnvelopeWarp(env) => {
            let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

            let lbl_tl = gtk4::Label::new(Some(&crate::core::gettext("Top-Left Offset")));
            lbl_tl.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_tl, 0, 0, 1, 1);

            let spin_tl_x = gtk4::SpinButton::with_range(-500.0, 500.0, 2.0);
            spin_tl_x.set_value(env.top_left_offset.x as f64);
            grid.attach(&spin_tl_x, 1, 0, 1, 1);

            let lbl_tr = gtk4::Label::new(Some(&crate::core::gettext("Top-Right Offset")));
            lbl_tr.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_tr, 0, 1, 1, 1);

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
                drop(state);
                if let Ok(st) = canvas_cb.state.try_borrow() {
                    st.notify_status();
                }
                canvas_cb.queue_draw();
            });
        }
        Modifier::ChamferRounding(ch) => {
            let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

            let lbl_style = gtk4::Label::new(Some(&crate::core::gettext("Corner Style")));
            lbl_style.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_style, 0, 0, 1, 1);

            let combo_style = gtk4::DropDown::from_strings(&[
                &crate::core::gettext("Round"),
                &crate::core::gettext("Chamfer"),
                &crate::core::gettext("Concave"),
            ]);
            let active_style_idx = match ch.style {
                CornerStyle::Round => 0,
                CornerStyle::Chamfer => 1,
                CornerStyle::Concave => 2,
            };
            combo_style.set_selected(active_style_idx);
            grid.attach(&combo_style, 1, 0, 1, 1);

            let lbl_r = gtk4::Label::new(Some(&crate::core::gettext("Radius (px)")));
            lbl_r.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_r, 0, 1, 1, 1);

            let spin_r = gtk4::SpinButton::with_range(0.0, 200.0, 1.0);
            spin_r.set_value(ch.radius as f64);
            grid.attach(&spin_r, 1, 1, 1, 1);

            body.append(&grid);

            let canvas_cb = canvas.clone();
            let spin_r_c = spin_r.clone();
            combo_style.connect_selected_notify(move |cb| {
                let sel_style = match cb.selected() {
                    1 => CornerStyle::Chamfer,
                    2 => CornerStyle::Concave,
                    _ => CornerStyle::Round,
                };
                let mut state = canvas_cb.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::ChamferRounding(c)) = mods.get_mut(mod_idx) {
                            c.style = sel_style;
                            c.radius = spin_r_c.value() as f32;
                        }
                    }
                }
                drop(state);
                if let Ok(st) = canvas_cb.state.try_borrow() {
                    st.notify_status();
                }
                canvas_cb.queue_draw();
            });

            let canvas_cb2 = canvas.clone();
            let combo_style_c = combo_style.clone();
            spin_r.connect_value_changed(move |s| {
                let sel_style = match combo_style_c.selected() {
                    1 => CornerStyle::Chamfer,
                    2 => CornerStyle::Concave,
                    _ => CornerStyle::Round,
                };
                let mut state = canvas_cb2.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::ChamferRounding(c)) = mods.get_mut(mod_idx) {
                            c.radius = s.value() as f32;
                            c.style = sel_style;
                        }
                    }
                }
                drop(state);
                if let Ok(st) = canvas_cb2.state.try_borrow() {
                    st.notify_status();
                }
                canvas_cb2.queue_draw();
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
        drop(state);
        if let Ok(st) = canvas_sw.state.try_borrow() {
            st.notify_status();
        }
        canvas_sw.queue_draw();
    });

    // Delete modifier handler
    let canvas_del = canvas.clone();
    let on_del_update = on_change;
    del_btn.connect_clicked(move |_| {
        let mut state = canvas_del.state.borrow_mut();
        if let Some(elem) = state.document.find_element_mut(elem_id) {
            if let Some(mods) = elem.modifiers_mut() {
                if mod_idx < mods.len() {
                    mods.remove(mod_idx);
                }
            }
        }
        drop(state);
        if let Ok(st) = canvas_del.state.try_borrow() {
            st.notify_status();
        }
        canvas_del.queue_draw();
        if let Some(ref update_fn) = on_del_update {
            update_fn();
        }
    });

    card
}
