use gtk4::prelude::*;
use std::rc::Rc;

use crate::core::element::CornerStyle;
use crate::core::geometry::Rect;
use crate::core::modifier::{
    ArrayMode, ArrayModifier, ChamferRoundingModifier, EnvelopeWarpModifier, Extrude3DMode,
    Extrude3DModifier, Modifier, OffsetPathModifier, TwistModifier, WaveDeformModifier,
    ZigZagModifier,
};
use crate::core::modifier_store::{load_all_modifier_assets, save_modifier_asset};
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
        .width_request(290)
        .build();

    // Search bar for filtering modifiers
    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text(crate::core::gettext("Search Modifiers & Assets..."))
        .margin_bottom(4)
        .build();
    pop_box.append(&search_entry);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .max_content_height(360)
        .propagate_natural_height(true)
        .build();

    let catalog_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_end(4)
        .build();

    // ── Category 1: Generate & duplicate ──
    let cat_dup_title = gtk4::Label::builder()
        .label(crate::core::gettext("Generate & duplicate"))
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

    // ── Category 2: Deform & warp ──
    let cat_deform_title = gtk4::Label::builder()
        .label(crate::core::gettext("Deform & warp"))
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .margin_start(4)
        .margin_top(4)
        .build();
    let cat_deform_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .build();

    let btn_add_extrude = create_menu_item_button(
        "orientation-landscape-symbolic",
        &crate::core::gettext("3D Extrude & Lighting"),
        &crate::core::gettext("3D Extrusion & Shaded Projection"),
    );
    let btn_add_twist = create_menu_item_button(
        "emblem-synchronizing-symbolic",
        &crate::core::gettext("Twist & Swirl"),
        &crate::core::gettext("Rotational Swirl Distortion"),
    );
    let btn_add_env = create_menu_item_button(
        "transform-symbolic",
        &crate::core::gettext("Envelope Warp"),
        &crate::core::gettext("4-Point Mesh Distortion"),
    );
    let btn_add_wave = create_menu_item_button(
        "view-refresh-symbolic",
        &crate::core::gettext("Sine Wave Ripple"),
        &crate::core::gettext("Parametric Wave Distortion"),
    );
    let btn_add_zigzag = create_menu_item_button(
        "edit-cut-symbolic",
        &crate::core::gettext("ZigZag Distortion"),
        &crate::core::gettext("Serrated Sawtooth Contour"),
    );
    cat_deform_box.append(&btn_add_extrude);
    cat_deform_box.append(&btn_add_twist);
    cat_deform_box.append(&btn_add_env);
    cat_deform_box.append(&btn_add_wave);
    cat_deform_box.append(&btn_add_zigzag);

    // ── Category 3: Path & corners ──
    let cat_path_title = gtk4::Label::builder()
        .label(crate::core::gettext("Path & corners"))
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
    let btn_add_offset = create_menu_item_button(
        "zoom-out-symbolic",
        &crate::core::gettext("Offset Path / Outline"),
        &crate::core::gettext("Expand / Contract Vector Contour"),
    );
    cat_path_box.append(&btn_add_chamfer);
    cat_path_box.append(&btn_add_offset);

    // ── Category 4: Local assets (assets/modifiers/) ──
    let cat_assets_title = gtk4::Label::builder()
        .label(crate::core::gettext("Local assets (assets/modifiers/)"))
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .margin_start(4)
        .margin_top(6)
        .build();
    let cat_assets_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .build();

    let local_assets = load_all_modifier_assets();
    let mut asset_buttons = Vec::new();

    let update_fn_holder = Rc::new(std::cell::RefCell::new(None::<Rc<dyn Fn()>>));
    let update_fn_holder_c = update_fn_holder.clone();

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

    for asset in local_assets {
        let btn_asset = create_menu_item_button(
            asset.modifier.icon_name(),
            &asset.name,
            &asset.description,
        );
        let canvas_ast = canvas.clone();
        let pop_ast = popover.clone();
        let ast_mod = asset.modifier.clone();
        let update_ast = update_fn_holder_c.clone();
        btn_asset.connect_clicked(move |_| {
            pop_ast.popdown();
            ensure_target_element(&canvas_ast, ast_mod.clone());
            canvas_ast.queue_draw();
            if let Some(ref uf) = *update_ast.borrow() {
                uf();
            }
        });
        cat_assets_box.append(&btn_asset);
        asset_buttons.push((asset.name.to_lowercase(), btn_asset));
    }

    catalog_box.append(&cat_dup_title);
    catalog_box.append(&cat_dup_box);
    catalog_box.append(&cat_deform_title);
    catalog_box.append(&cat_deform_box);
    catalog_box.append(&cat_path_title);
    catalog_box.append(&cat_path_box);
    catalog_box.append(&cat_assets_title);
    catalog_box.append(&cat_assets_box);

    scrolled.set_child(Some(&catalog_box));
    pop_box.append(&scrolled);

    popover.set_child(Some(&pop_box));
    add_mod_btn.set_popover(Some(&popover));

    // Search filter callback
    let b_array_c = btn_add_array.clone();
    let b_ext_c = btn_add_extrude.clone();
    let b_tw_c = btn_add_twist.clone();
    let b_env_c = btn_add_env.clone();
    let b_wave_c = btn_add_wave.clone();
    let b_zz_c = btn_add_zigzag.clone();
    let b_chamf_c = btn_add_chamfer.clone();
    let b_off_c = btn_add_offset.clone();
    search_entry.connect_search_changed(move |se| {
        let q = se.text().to_lowercase();
        b_array_c.set_visible(q.is_empty() || "array modifier linear radial grid duplication".contains(&q));
        b_ext_c.set_visible(q.is_empty() || "3d extrude lighting projection isometric".contains(&q));
        b_tw_c.set_visible(q.is_empty() || "twist swirl distortion rotational".contains(&q));
        b_env_c.set_visible(q.is_empty() || "envelope warp distortion mesh 4-point".contains(&q));
        b_wave_c.set_visible(q.is_empty() || "sine wave ripple distortion wave".contains(&q));
        b_zz_c.set_visible(q.is_empty() || "zigzag distortion serrated sawtooth contour".contains(&q));
        b_chamf_c.set_visible(q.is_empty() || "dynamic chamfer corner rounding bevels".contains(&q));
        b_off_c.set_visible(q.is_empty() || "offset path outline expand contract contour".contains(&q));

        for (ast_name, btn) in &asset_buttons {
            btn.set_visible(q.is_empty() || ast_name.contains(&q));
        }
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

    let update_ref_ext = update_fn.clone();
    let canvas_ext = canvas.clone();
    let pop_ext = popover.clone();
    btn_add_extrude.connect_clicked(move |_| {
        pop_ext.popdown();
        ensure_target_element(&canvas_ext, Modifier::Extrude3D(Extrude3DModifier::default()));
        canvas_ext.queue_draw();
        update_ref_ext();
    });

    let update_ref_tw = update_fn.clone();
    let canvas_tw = canvas.clone();
    let pop_tw = popover.clone();
    btn_add_twist.connect_clicked(move |_| {
        pop_tw.popdown();
        ensure_target_element(&canvas_tw, Modifier::Twist(TwistModifier::default()));
        canvas_tw.queue_draw();
        update_ref_tw();
    });

    let update_ref_wave = update_fn.clone();
    let canvas_wave = canvas.clone();
    let pop_wave = popover.clone();
    btn_add_wave.connect_clicked(move |_| {
        pop_wave.popdown();
        ensure_target_element(&canvas_wave, Modifier::WaveDeform(WaveDeformModifier::default()));
        canvas_wave.queue_draw();
        update_ref_wave();
    });

    let update_ref_zz = update_fn.clone();
    let canvas_zz = canvas.clone();
    let pop_zz = popover.clone();
    btn_add_zigzag.connect_clicked(move |_| {
        pop_zz.popdown();
        ensure_target_element(&canvas_zz, Modifier::ZigZag(ZigZagModifier::default()));
        canvas_zz.queue_draw();
        update_ref_zz();
    });

    let update_ref_off = update_fn.clone();
    let canvas_off = canvas.clone();
    let pop_off_c = popover.clone();
    btn_add_offset.connect_clicked(move |_| {
        pop_off_c.popdown();
        ensure_target_element(&canvas_off, Modifier::OffsetPath(OffsetPathModifier::default()));
        canvas_off.queue_draw();
        update_ref_off();
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
        .spacing(6)
        .css_classes(["card"])
        .build();

    // ── Pixel-Perfect Header Bar ──
    let header_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_start(10)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(4)
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

    // Sleek linked action bar: [Eye] [Apply] [Save] [Up] [Down] [Trash]
    let linked_bar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["linked"])
        .valign(gtk4::Align::Center)
        .build();

    let is_enabled = modifier.enabled();
    let vis_icon_name = if is_enabled { "view-visible-symbolic" } else { "view-hidden-symbolic" };
    let img_vis = gtk4::Image::from_icon_name(vis_icon_name);
    img_vis.set_pixel_size(14);
    let btn_vis = gtk4::Button::builder()
        .child(&img_vis)
        .tooltip_text(if is_enabled { crate::core::gettext("Hide Modifier") } else { crate::core::gettext("Show Modifier") })
        .css_classes(["flat"])
        .build();

    let img_apply = gtk4::Image::from_icon_name("emblem-ok-symbolic");
    img_apply.set_pixel_size(14);
    let btn_apply = gtk4::Button::builder()
        .child(&img_apply)
        .tooltip_text(crate::core::gettext("Bake / Apply Geometry"))
        .css_classes(["flat"])
        .build();

    let img_save = gtk4::Image::from_icon_name("document-save-symbolic");
    img_save.set_pixel_size(14);
    let btn_save = gtk4::Button::builder()
        .child(&img_save)
        .tooltip_text(crate::core::gettext("Save Preset Asset"))
        .css_classes(["flat"])
        .build();

    let img_up = gtk4::Image::from_icon_name("go-up-symbolic");
    img_up.set_pixel_size(14);
    let btn_up = gtk4::Button::builder()
        .child(&img_up)
        .tooltip_text(crate::core::gettext("Move Up"))
        .css_classes(["flat"])
        .sensitive(mod_idx > 0)
        .build();

    let img_dn = gtk4::Image::from_icon_name("go-down-symbolic");
    img_dn.set_pixel_size(14);
    let btn_dn = gtk4::Button::builder()
        .child(&img_dn)
        .tooltip_text(crate::core::gettext("Move Down"))
        .css_classes(["flat"])
        .sensitive(mod_idx + 1 < total_mods)
        .build();

    let img_del = gtk4::Image::from_icon_name("user-trash-symbolic");
    img_del.set_pixel_size(14);
    let del_btn = gtk4::Button::builder()
        .child(&img_del)
        .tooltip_text(crate::core::gettext("Delete Modifier"))
        .css_classes(["flat"])
        .build();

    linked_bar.append(&btn_vis);
    linked_bar.append(&btn_apply);
    linked_bar.append(&btn_save);
    linked_bar.append(&btn_up);
    linked_bar.append(&btn_dn);
    linked_bar.append(&del_btn);

    header_box.append(&badge_lbl);
    header_box.append(&icon);
    header_box.append(&name_lbl);
    header_box.append(&linked_bar);
    card.append(&header_box);

    // ── HIDE / REVEAL VISIBILITY HANDLER ──
    let canvas_vis = canvas.clone();
    let on_vis_update = on_change.clone();
    btn_vis.connect_clicked(move |_| {
        let mut state = canvas_vis.state.borrow_mut();
        if let Some(elem) = state.document.find_element_mut(elem_id) {
            if let Some(mods) = elem.modifiers_mut() {
                if let Some(m) = mods.get_mut(mod_idx) {
                    let cur = m.enabled();
                    m.set_enabled(!cur);
                }
            }
        }
        drop(state);
        if let Ok(st) = canvas_vis.state.try_borrow() {
            st.notify_status();
        }
        canvas_vis.queue_draw();
        if let Some(ref update_fn) = on_vis_update {
            update_fn();
        }
    });

    // ── APPLY (BAKE GEOMETRY) HANDLER ──
    let canvas_apply = canvas.clone();
    let on_apply_update = on_change.clone();
    btn_apply.connect_clicked(move |_| {
        let mut state = canvas_apply.state.borrow_mut();
        if let Some(elem) = state.document.find_element_mut(elem_id) {
            elem.apply_modifier(mod_idx);
        }
        drop(state);
        if let Ok(st) = canvas_apply.state.try_borrow() {
            st.notify_status();
        }
        canvas_apply.queue_draw();
        if let Some(ref update_fn) = on_apply_update {
            update_fn();
        }
    });

    // ── SAVE AS ASSET HANDLER ──
    let modifier_to_save = modifier.clone();
    btn_save.connect_clicked(move |_| {
        let default_name = format!("Custom {}", modifier_to_save.name());
        let desc = "Saved user modifier preset";
        let _ = save_modifier_asset(&default_name, desc, &modifier_to_save);
    });

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
        Modifier::Extrude3D(ext) => {
            // Row 1: Projection Mode + Lighting Switch
            let row_top = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(8)
                .margin_bottom(6)
                .build();

            let combo_m = gtk4::DropDown::from_strings(&[
                &crate::core::gettext("Isometric"),
                &crate::core::gettext("Cabinet 45°"),
                &crate::core::gettext("Perspective"),
            ]);
            let active_m_idx = match ext.mode {
                Extrude3DMode::Isometric => 0,
                Extrude3DMode::Cabinet => 1,
                Extrude3DMode::Perspective => 2,
            };
            combo_m.set_selected(active_m_idx);
            combo_m.set_hexpand(true);

            let sw_sh = gtk4::Switch::builder()
                .active(ext.shading)
                .valign(gtk4::Align::Center)
                .tooltip_text(&crate::core::gettext("3D Lighting Shading"))
                .build();

            row_top.append(&combo_m);
            row_top.append(&sw_sh);
            body.append(&row_top);

            // Row 2: Visual 3D Dimmer Knob + Depth & Angle Quick Inputs
            let row_knobs = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(10)
                .margin_bottom(6)
                .build();

            // Interactive GTK Angle Dimmer Knob Widget
            let angle_cell = Rc::new(std::cell::RefCell::new(ext.angle_deg));
            let da_knob = gtk4::DrawingArea::builder()
                .width_request(46)
                .height_request(46)
                .halign(gtk4::Align::Center)
                .valign(gtk4::Align::Center)
                .tooltip_text(&crate::core::gettext("Drag angle dimmer knob to rotate 3D projection"))
                .build();

            let angle_draw = angle_cell.clone();
            da_knob.set_draw_func(move |_da, cr, width, height| {
                let cx = width as f64 * 0.5;
                let cy = height as f64 * 0.5;
                let r = (width.min(height) as f64 * 0.45) - 2.0;

                cr.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(0.15, 0.18, 0.25, 0.9);
                cr.fill_preserve().unwrap();
                cr.set_source_rgba(0.35, 0.45, 0.6, 0.6);
                cr.set_line_width(1.5);
                cr.stroke().unwrap();

                let a_rad = (*angle_draw.borrow() as f64).to_radians();
                let dot_x = cx + r * 0.65 * a_rad.cos();
                let dot_y = cy + r * 0.65 * a_rad.sin();

                cr.move_to(cx, cy);
                cr.line_to(dot_x, dot_y);
                cr.set_source_rgba(0.0, 0.8, 1.0, 0.9);
                cr.set_line_width(2.0);
                cr.stroke().unwrap();

                cr.arc(dot_x, dot_y, 3.5, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(0.0, 0.95, 1.0, 1.0);
                cr.fill().unwrap();
            });

            let box_numeric = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(4)
                .hexpand(true)
                .build();

            // Depth Spin Row
            let line_d = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .build();
            let lbl_d = gtk4::Label::builder()
                .label(crate::core::gettext("Depth"))
                .css_classes(["caption", "dim-label"])
                .hexpand(true)
                .xalign(0.0)
                .build();
            let spin_d = gtk4::SpinButton::with_range(0.0, 500.0, 2.0);
            spin_d.set_value(ext.depth as f64);
            line_d.append(&lbl_d);
            line_d.append(&spin_d);

            // Angle Spin Row
            let line_a = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .build();
            let lbl_a = gtk4::Label::builder()
                .label(crate::core::gettext("Angle °"))
                .css_classes(["caption", "dim-label"])
                .hexpand(true)
                .xalign(0.0)
                .build();
            let spin_a = gtk4::SpinButton::with_range(-360.0, 360.0, 5.0);
            spin_a.set_value(ext.angle_deg as f64);
            line_a.append(&lbl_a);
            line_a.append(&spin_a);

            box_numeric.append(&line_d);
            box_numeric.append(&line_a);

            row_knobs.append(&da_knob);
            row_knobs.append(&box_numeric);
            body.append(&row_knobs);

            // Row 3: Single Master Depth Slider
            let scale_d = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 200.0, 1.0);
            scale_d.set_value(ext.depth as f64);
            scale_d.set_hexpand(true);
            scale_d.set_margin_bottom(6);
            body.append(&scale_d);

            // Row 4: 3D Extrusion Side Color Swatches
            let row_col = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(6)
                .margin_bottom(4)
                .build();
            let lbl_c = gtk4::Label::builder()
                .label(crate::core::gettext("Extrusion Color"))
                .css_classes(["caption", "dim-label"])
                .hexpand(true)
                .xalign(0.0)
                .build();

            let color_palette = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .build();

            let colors = [
                ("Auto", None),
                ("Dark Metal", Some(crate::core::Color::new(0.12, 0.14, 0.18, 1.0))),
                ("Gold", Some(crate::core::Color::new(0.95, 0.75, 0.2, 1.0))),
                ("Ruby", Some(crate::core::Color::new(0.85, 0.2, 0.35, 1.0))),
                ("Cyan", Some(crate::core::Color::new(0.1, 0.8, 0.95, 1.0))),
            ];

            let active_color_cell = Rc::new(std::cell::RefCell::new(ext.custom_side_color));

            for (c_name, c_opt) in colors {
                let btn = gtk4::Button::builder()
                    .label(c_name)
                    .css_classes(["flat", "caption"])
                    .build();

                let canvas_col = canvas.clone();
                let active_cell = active_color_cell.clone();
                btn.connect_clicked(move |_| {
                    *active_cell.borrow_mut() = c_opt;
                    let mut state = canvas_col.state.borrow_mut();
                    if let Some(elem) = state.document.find_element_mut(elem_id) {
                        if let Some(mods) = elem.modifiers_mut() {
                            if let Some(Modifier::Extrude3D(e)) = mods.get_mut(mod_idx) {
                                e.custom_side_color = c_opt;
                            }
                        }
                    }
                    drop(state);
                    if let Ok(st) = canvas_col.state.try_borrow() {
                        st.notify_status();
                    }
                    canvas_col.queue_draw();
                });
                color_palette.append(&btn);
            }

            row_col.append(&lbl_c);
            row_col.append(&color_palette);
            body.append(&row_col);

            // Row 0: 1-Click 3D Presets Bar
            let row_presets = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .margin_bottom(6)
                .build();

            let preset_list = [
                ("Extrude", 1.0, 0.0, 0.0),
                ("Pyramid", 0.3, 0.0, 0.0),
                ("Twist 3D", 1.0, 45.0, 0.0),
                ("Bevel", 1.0, 0.0, 4.0),
            ];

            for (p_name, p_taper, p_twist, p_bevel) in preset_list {
                let p_btn = gtk4::Button::builder()
                    .label(p_name)
                    .css_classes(["flat", "caption"])
                    .build();

                let canvas_p = canvas.clone();
                p_btn.connect_clicked(move |_| {
                    let mut state = canvas_p.state.borrow_mut();
                    if let Some(elem) = state.document.find_element_mut(elem_id) {
                        if let Some(mods) = elem.modifiers_mut() {
                            if let Some(Modifier::Extrude3D(e)) = mods.get_mut(mod_idx) {
                                e.taper = p_taper;
                                e.twist_deg = p_twist;
                                e.bevel_radius = p_bevel;
                            }
                        }
                    }
                    drop(state);
                    if let Ok(st) = canvas_p.state.try_borrow() {
                        st.notify_status();
                    }
                    canvas_p.queue_draw();
                });
                row_presets.append(&p_btn);
            }
            body.append(&row_presets);

            // Wire Live Responsive Callbacks
            let canvas_cb = canvas.clone();
            let active_cell_sync = active_color_cell;
            let update_ext = move |m_idx: u32, d: f32, a: f32, sh: bool| {
                let sel_m = match m_idx {
                    1 => Extrude3DMode::Cabinet,
                    2 => Extrude3DMode::Perspective,
                    _ => Extrude3DMode::Isometric,
                };
                let side_col = *active_cell_sync.borrow();
                let mut state = canvas_cb.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::Extrude3D(e)) = mods.get_mut(mod_idx) {
                            e.mode = sel_m;
                            e.depth = d;
                            e.angle_deg = a;
                            e.custom_side_color = side_col;
                            e.shading = sh;
                        }
                    }
                }
                drop(state);
                if let Ok(st) = canvas_cb.state.try_borrow() {
                    st.notify_status();
                }
                canvas_cb.queue_draw();
            };

            let u_e = update_ext;
            let c_m1 = combo_m.clone();
            let s_d1 = spin_d.clone();
            let s_a1 = spin_a.clone();
            let sw_sh1 = sw_sh.clone();
            let da_k_draw = da_knob.clone();
            let angle_cell_sync = angle_cell.clone();

            let sync_update = move || {
                let a = s_a1.value() as f32;
                *angle_cell_sync.borrow_mut() = a;
                da_k_draw.queue_draw();
                u_e(
                    c_m1.selected(),
                    s_d1.value() as f32,
                    a,
                    sw_sh1.is_active(),
                );
            };

            let su1 = Rc::new(sync_update);

            // Drag Gesture on Dimmer Knob Widget
            let drag_knob = gtk4::GestureDrag::new();
            let da_k_drag = da_knob.clone();
            let spin_a_knob = spin_a.clone();

            let update_knob_drag = move |x: f64, y: f64| {
                let cx = da_k_drag.width() as f64 * 0.5;
                let cy = da_k_drag.height() as f64 * 0.5;
                let dx = x - cx;
                let dy = y - cy;
                let deg = dy.atan2(dx).to_degrees();
                spin_a_knob.set_value(deg);
            };

            let up_k_start = update_knob_drag.clone();
            drag_knob.connect_drag_begin(move |_, x, y| {
                up_k_start(x, y);
            });
            drag_knob.connect_drag_update(move |gesture, offset_x, offset_y| {
                if let Some((start_x, start_y)) = gesture.start_point() {
                    update_knob_drag(start_x + offset_x, start_y + offset_y);
                }
            });
            da_knob.add_controller(drag_knob);

            let su_m = su1.clone();
            combo_m.connect_selected_notify(move |_| su_m());

            let su_sd = su1.clone();
            let sc_d2 = scale_d.clone();
            spin_d.connect_value_changed(move |s| {
                sc_d2.set_value(s.value());
                su_sd();
            });

            let su_scd = su1.clone();
            let spin_d2 = spin_d.clone();
            scale_d.connect_value_changed(move |s| {
                spin_d2.set_value(s.value());
                su_scd();
            });

            let su_sa = su1.clone();
            spin_a.connect_value_changed(move |_| {
                su_sa();
            });

            let su_sh = su1;
            sw_sh.connect_active_notify(move |_| su_sh());
        }
        Modifier::Twist(tw) => {
            let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

            let lbl_a = gtk4::Label::new(Some(&crate::core::gettext("Twist Angle (°)")));
            lbl_a.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_a, 0, 0, 1, 1);

            let spin_a = gtk4::SpinButton::with_range(-720.0, 720.0, 10.0);
            spin_a.set_value(tw.angle_deg as f64);
            grid.attach(&spin_a, 1, 0, 1, 1);

            let lbl_r = gtk4::Label::new(Some(&crate::core::gettext("Effect Radius (px)")));
            lbl_r.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_r, 0, 1, 1, 1);

            let spin_r = gtk4::SpinButton::with_range(5.0, 2000.0, 10.0);
            spin_r.set_value(tw.radius as f64);
            grid.attach(&spin_r, 1, 1, 1, 1);

            body.append(&grid);

            let canvas_cb = canvas.clone();
            let update_tw = move |a: f32, r: f32| {
                let mut state = canvas_cb.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::Twist(t)) = mods.get_mut(mod_idx) {
                            t.angle_deg = a;
                            t.radius = r;
                        }
                    }
                }
                drop(state);
                if let Ok(st) = canvas_cb.state.try_borrow() {
                    st.notify_status();
                }
                canvas_cb.queue_draw();
            };

            let u_t1 = update_tw.clone();
            let s_r1 = spin_r.clone();
            spin_a.connect_value_changed(move |s| {
                u_t1(s.value() as f32, s_r1.value() as f32);
            });

            let u_t2 = update_tw;
            let s_a2 = spin_a;
            spin_r.connect_value_changed(move |s| {
                u_t2(s_a2.value() as f32, s.value() as f32);
            });
        }
        Modifier::OffsetPath(off) => {
            let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

            let lbl_off = gtk4::Label::new(Some(&crate::core::gettext("Offset Distance")));
            lbl_off.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_off, 0, 0, 1, 1);

            let spin_off = gtk4::SpinButton::with_range(-500.0, 500.0, 1.0);
            spin_off.set_value(off.offset as f64);
            grid.attach(&spin_off, 1, 0, 1, 1);

            body.append(&grid);

            let canvas_cb = canvas.clone();
            spin_off.connect_value_changed(move |s| {
                let val = s.value() as f32;
                let mut state = canvas_cb.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::OffsetPath(o)) = mods.get_mut(mod_idx) {
                            o.offset = val;
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
        Modifier::ZigZag(zz) => {
            let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

            let lbl_r = gtk4::Label::new(Some(&crate::core::gettext("Ridges per Segment")));
            lbl_r.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_r, 0, 0, 1, 1);

            let spin_r = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
            spin_r.set_value(zz.ridges as f64);
            grid.attach(&spin_r, 1, 0, 1, 1);

            let lbl_amp = gtk4::Label::new(Some(&crate::core::gettext("Amplitude")));
            lbl_amp.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_amp, 0, 1, 1, 1);

            let spin_amp = gtk4::SpinButton::with_range(0.0, 200.0, 1.0);
            spin_amp.set_value(zz.amplitude as f64);
            grid.attach(&spin_amp, 1, 1, 1, 1);

            body.append(&grid);

            let canvas_cb = canvas.clone();
            let update_zz = move |r: u32, a: f32| {
                let mut state = canvas_cb.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::ZigZag(z)) = mods.get_mut(mod_idx) {
                            z.ridges = r;
                            z.amplitude = a;
                        }
                    }
                }
                drop(state);
                if let Ok(st) = canvas_cb.state.try_borrow() {
                    st.notify_status();
                }
                canvas_cb.queue_draw();
            };

            let u_z1 = update_zz.clone();
            let s_amp1 = spin_amp.clone();
            spin_r.connect_value_changed(move |s| {
                u_z1(s.value() as u32, s_amp1.value() as f32);
            });

            let u_z2 = update_zz;
            let s_r2 = spin_r;
            spin_amp.connect_value_changed(move |s| {
                u_z2(s_r2.value() as u32, s.value() as f32);
            });
        }
        Modifier::WaveDeform(wave) => {
            let grid = gtk4::Grid::builder().column_spacing(8).row_spacing(6).build();

            let lbl_amp = gtk4::Label::new(Some(&crate::core::gettext("Amplitude")));
            lbl_amp.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_amp, 0, 0, 1, 1);

            let spin_amp = gtk4::SpinButton::with_range(0.0, 200.0, 2.0);
            spin_amp.set_value(wave.amplitude as f64);
            grid.attach(&spin_amp, 1, 0, 1, 1);

            let lbl_len = gtk4::Label::new(Some(&crate::core::gettext("Wavelength")));
            lbl_len.set_css_classes(&["caption", "dim-label"]);
            grid.attach(&lbl_len, 0, 1, 1, 1);

            let spin_len = gtk4::SpinButton::with_range(5.0, 500.0, 5.0);
            spin_len.set_value(wave.wavelength as f64);
            grid.attach(&spin_len, 1, 1, 1, 1);

            body.append(&grid);

            let canvas_cb = canvas.clone();
            let update_wave = move |a: f32, l: f32| {
                let mut state = canvas_cb.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::WaveDeform(w)) = mods.get_mut(mod_idx) {
                            w.amplitude = a;
                            w.wavelength = l;
                        }
                    }
                }
                drop(state);
                if let Ok(st) = canvas_cb.state.try_borrow() {
                    st.notify_status();
                }
                canvas_cb.queue_draw();
            };

            let u_w1 = update_wave.clone();
            let s_len1 = spin_len.clone();
            spin_amp.connect_value_changed(move |s| {
                u_w1(s.value() as f32, s_len1.value() as f32);
            });

            let u_w2 = update_wave;
            let s_amp2 = spin_amp;
            spin_len.connect_value_changed(move |s| {
                u_w2(s_amp2.value() as f32, s.value() as f32);
            });
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
