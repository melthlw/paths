use gtk4::prelude::*;
use std::rc::Rc;

use crate::core::element::CornerStyle;
use crate::core::geometry::Rect;
use crate::core::modifier::{
    ArrayMode, ArrayModifier, Bevel3DStyle, ChamferRoundingModifier, EnvelopeWarpModifier, Extrude3DMode,
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
        &crate::core::gettext("3D Extrude"),
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
                if (matches!(elem, crate::core::Element::Image(_)) || matches!(elem, crate::core::Element::Text(_)))
                    && !matches!(modifier, Modifier::Array(_))
                {
                    // Photos and live Text/Fonts only support Array modifier
                    return;
                }
                if let Some(mods) = elem.modifiers_mut() {
                    mods.push(modifier);
                }
            }
            state.mark_dirty();
            drop(state);
            if let Ok(st) = canvas_widget.state.try_borrow() {
                st.notify_status();
            }
        } else {
            let rect = Rect::new(200.0, 200.0, 160.0, 160.0);
            let mut new_rect = crate::core::RectElement::new(
                rect,
                Some(crate::core::Color::new(0.2, 0.5, 0.9, 1.0)),
                None,
            );
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
        let btn_asset =
            create_menu_item_button(asset.modifier.icon_name(), &asset.name, &asset.description);
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
        let is_arr = matches!(asset.modifier, Modifier::Array(_));
        asset_buttons.push((asset.name.to_lowercase(), is_arr, btn_asset));
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

    // Dynamic catalog visibility filter by selected element type and query
    let update_catalog_visibility = {
        let b_array = btn_add_array.clone();
        let b_ext = btn_add_extrude.clone();
        let b_tw = btn_add_twist.clone();
        let b_env = btn_add_env.clone();
        let b_wave = btn_add_wave.clone();
        let b_zz = btn_add_zigzag.clone();
        let b_chamf = btn_add_chamfer.clone();
        let b_off = btn_add_offset.clone();

        let c_dup_t = cat_dup_title.clone();
        let c_dup_b = cat_dup_box.clone();
        let c_def_t = cat_deform_title.clone();
        let c_def_b = cat_deform_box.clone();
        let c_pth_t = cat_path_title.clone();
        let c_pth_b = cat_path_box.clone();
        let c_ast_t = cat_assets_title.clone();
        let c_ast_b = cat_assets_box.clone();

        let canvas_cat = canvas.clone();
        let asset_buttons_cat = asset_buttons.clone();

        Rc::new(move |query: &str| {
            let q = query.to_lowercase();

            let (is_image, is_text, is_vector) = if let Ok(st) = canvas_cat.state.try_borrow() {
                if let Some(&sel_id) = st.document.selected_ids.iter().next() {
                    if let Some(el) = st.document.find_element(sel_id) {
                        match el {
                            crate::core::Element::Image(_) => (true, false, false),
                            crate::core::Element::Text(_) => (false, true, false),
                            _ => (false, false, true),
                        }
                    } else {
                        (false, false, true)
                    }
                } else {
                    (false, false, true)
                }
            } else {
                (false, false, true)
            };

            // Array is supported for all elements (Image, Text, Vector)
            let array_match = q.is_empty() || "array modifier linear radial grid duplication".contains(&q);
            b_array.set_visible(array_match);
            c_dup_t.set_visible(array_match);
            c_dup_b.set_visible(array_match);

            // Deform & Warp modifiers (3D Extrude, Twist, Envelope, Wave, Zigzag)
            // NEVER shown on raster photos/images OR live Text/fonts!
            let allow_deform = is_vector;
            let ext_match = allow_deform && (q.is_empty() || "3d extrude lighting projection isometric".contains(&q));
            let tw_match = allow_deform && (q.is_empty() || "twist swirl distortion rotational".contains(&q));
            let env_match = allow_deform && (q.is_empty() || "envelope warp distortion mesh 4-point".contains(&q));
            let wave_match = allow_deform && (q.is_empty() || "sine wave ripple distortion wave".contains(&q));
            let zz_match = allow_deform && (q.is_empty() || "zigzag distortion serrated sawtooth contour".contains(&q));

            b_ext.set_visible(ext_match);
            b_tw.set_visible(tw_match);
            b_env.set_visible(env_match);
            b_wave.set_visible(wave_match);
            b_zz.set_visible(zz_match);

            let has_deform = ext_match || tw_match || env_match || wave_match || zz_match;
            c_def_t.set_visible(has_deform);
            c_def_b.set_visible(has_deform);

            // Path & Corners (Chamfer, Offset Path)
            let allow_path = is_vector;
            let chamf_match = allow_path && (q.is_empty() || "dynamic chamfer corner rounding bevels".contains(&q));
            let off_match = allow_path && (q.is_empty() || "offset path outline expand contract contour".contains(&q));

            b_chamf.set_visible(chamf_match);
            b_off.set_visible(off_match);

            let has_path = chamf_match || off_match;
            c_pth_t.set_visible(has_path);
            c_pth_b.set_visible(has_path);

            // Local Assets
            let mut any_ast = false;
            for (ast_name, is_arr, btn) in &asset_buttons_cat {
                let allow_ast = if is_image || is_text {
                    *is_arr
                } else {
                    true
                };
                let v = allow_ast && (q.is_empty() || ast_name.contains(&q));
                btn.set_visible(v);
                if v {
                    any_ast = true;
                }
            }
            c_ast_t.set_visible(any_ast);
            c_ast_b.set_visible(any_ast);
        })
    };

    let upd_cat_se = update_catalog_visibility.clone();
    search_entry.connect_search_changed(move |se| {
        upd_cat_se(&se.text());
    });

    let upd_cat_pop = update_catalog_visibility.clone();
    let se_pop = search_entry.clone();
    popover.connect_show(move |_| {
        upd_cat_pop(&se_pop.text());
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

    let last_state_cache = Rc::new(std::cell::RefCell::new((None::<crate::core::element::ElementId>, Vec::<crate::core::modifier::Modifier>::new())));
    let last_cache_c = last_state_cache.clone();

    let update_fn = Rc::new(move || {
        let sel_ids = canvas_c.selected_element_ids();

        if sel_ids.is_empty() {
            let mut cache = last_cache_c.borrow_mut();
            if cache.0.is_none() && cache.1.is_empty() {
                return;
            }
            cache.0 = None;
            cache.1.clear();
            crate::ui::inspector::appearance::clear_box(&list_c);
            ctx_sub_c.set_text(&crate::core::gettext("No object selected"));
            header_icon_c.set_icon_name(Some("view-grid-symbolic"));
            return;
        }

        let first_id = sel_ids[0];
        let state_ref = canvas_c.state.borrow();
        let elem_opt = state_ref.document.find_element(first_id);

        if let Some(elem) = elem_opt {
            let mods = elem.modifiers().to_vec();
            {
                let mut cache = last_cache_c.borrow_mut();
                if cache.0 == Some(first_id) && cache.1 == mods {
                    return; // Fast-path: Same element & modifiers, skip expensive GTK reconstruction
                }
                cache.0 = Some(first_id);
                cache.1 = mods.clone();
            }

            crate::ui::inspector::appearance::clear_box(&list_c);

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
        ensure_target_element(
            &canvas_ext,
            Modifier::Extrude3D(Extrude3DModifier::default()),
        );
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
        ensure_target_element(
            &canvas_wave,
            Modifier::WaveDeform(WaveDeformModifier::default()),
        );
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
        ensure_target_element(
            &canvas_off,
            Modifier::OffsetPath(OffsetPathModifier::default()),
        );
        canvas_off.queue_draw();
        update_ref_off();
    });

    let update_ref2 = update_fn.clone();
    let canvas_e = canvas.clone();
    let pop_e = popover.clone();
    btn_add_env.connect_clicked(move |_| {
        pop_e.popdown();
        ensure_target_element(
            &canvas_e,
            Modifier::EnvelopeWarp(EnvelopeWarpModifier::default()),
        );
        canvas_e.queue_draw();
        update_ref2();
    });

    let update_ref3 = update_fn.clone();
    let canvas_c_chamf = canvas.clone();
    let pop_c_chamf = popover.clone();
    btn_add_chamfer.connect_clicked(move |_| {
        pop_c_chamf.popdown();
        ensure_target_element(
            &canvas_c_chamf,
            Modifier::ChamferRounding(ChamferRoundingModifier::default()),
        );
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
    let vis_icon_name = if is_enabled {
        "view-visible-symbolic"
    } else {
        "view-hidden-symbolic"
    };
    let img_vis = gtk4::Image::from_icon_name(vis_icon_name);
    img_vis.set_pixel_size(14);
    let btn_vis = gtk4::Button::builder()
        .child(&img_vis)
        .tooltip_text(if is_enabled {
            crate::core::gettext("Hide Modifier")
        } else {
            crate::core::gettext("Show Modifier")
        })
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
        state.mark_dirty();
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
        state.mark_dirty();
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
        state.mark_dirty();
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
        state.mark_dirty();
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
                    let grid = gtk4::Grid::builder()
                        .column_spacing(8)
                        .row_spacing(6)
                        .build();

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
                        u_sc(
                            s.value() as u32,
                            s_dx.value() as f32,
                            s_dy.value() as f32,
                            s_rot.value() as f32,
                        );
                    });

                    let u_dx = update_array.clone();
                    let s_cnt = spin_count.clone();
                    let s_dy2 = spin_dy.clone();
                    let s_rot2 = spin_rot.clone();
                    spin_dx.connect_value_changed(move |s| {
                        u_dx(
                            s_cnt.value() as u32,
                            s.value() as f32,
                            s_dy2.value() as f32,
                            s_rot2.value() as f32,
                        );
                    });

                    let u_dy = update_array.clone();
                    let s_cnt3 = spin_count.clone();
                    let s_dx3 = spin_dx.clone();
                    let s_rot3 = spin_rot.clone();
                    spin_dy.connect_value_changed(move |s| {
                        u_dy(
                            s_cnt3.value() as u32,
                            s_dx3.value() as f32,
                            s.value() as f32,
                            s_rot3.value() as f32,
                        );
                    });

                    let u_rot = update_array.clone();
                    let s_cnt4 = spin_count.clone();
                    let s_dx4 = spin_dx.clone();
                    let s_dy4 = spin_dy.clone();
                    spin_rot.connect_value_changed(move |s| {
                        u_rot(
                            s_cnt4.value() as u32,
                            s_dx4.value() as f32,
                            s_dy4.value() as f32,
                            s.value() as f32,
                        );
                    });
                }
                ArrayMode::Radial {
                    count,
                    radius,
                    start_angle_deg: _,
                    total_angle_deg: _,
                    rotate_copies: _,
                } => {
                    let grid = gtk4::Grid::builder()
                        .column_spacing(8)
                        .row_spacing(6)
                        .build();

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
                    let grid = gtk4::Grid::builder()
                        .column_spacing(8)
                        .row_spacing(6)
                        .build();

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
                        u_g1(
                            s.value() as u32,
                            s_c1.value() as u32,
                            s_sx1.value() as f32,
                            s_sy1.value() as f32,
                        );
                    });

                    let u_g2 = update_g.clone();
                    let s_r2 = spin_r.clone();
                    let s_sx2 = spin_sx.clone();
                    let s_sy2 = spin_sy.clone();
                    spin_c.connect_value_changed(move |s| {
                        u_g2(
                            s_r2.value() as u32,
                            s.value() as u32,
                            s_sx2.value() as f32,
                            s_sy2.value() as f32,
                        );
                    });

                    let u_g3 = update_g.clone();
                    let s_r3 = spin_r.clone();
                    let s_c3 = spin_c.clone();
                    let s_sy3 = spin_sy.clone();
                    spin_sx.connect_value_changed(move |s| {
                        u_g3(
                            s_r3.value() as u32,
                            s_c3.value() as u32,
                            s.value() as f32,
                            s_sy3.value() as f32,
                        );
                    });

                    let u_g4 = update_g.clone();
                    let s_r4 = spin_r.clone();
                    let s_c4 = spin_c.clone();
                    let s_sx4 = spin_sx.clone();
                    spin_sy.connect_value_changed(move |s| {
                        u_g4(
                            s_r4.value() as u32,
                            s_c4.value() as u32,
                            s_sx4.value() as f32,
                            s.value() as f32,
                        );
                    });
                }
            }
        }
        Modifier::Extrude3D(ext) => {
            let container_3d = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(8)
                .build();

            // Row 1: Presets DropDown + Shading Toggle Switch
            let row_top = gtk4::Box::builder().spacing(6).build();

            let combo_pres = gtk4::DropDown::from_strings(&[
                &crate::core::gettext("Custom 3D"),
                &crate::core::gettext("Isometric Top"),
                &crate::core::gettext("Isometric Left"),
                &crate::core::gettext("Front View"),
                &crate::core::gettext("Top View"),
                &crate::core::gettext("Tilt 3D"),
                &crate::core::gettext("Dramatic 3D"),
                &crate::core::gettext("Coin Spin"),
                &crate::core::gettext("Cabinet 45°"),
            ]);
            combo_pres.set_hexpand(true);

            let active_pres_idx = match ext.mode {
                Extrude3DMode::Isometric => 1,
                Extrude3DMode::Cabinet => 8,
                _ => 0,
            };
            combo_pres.set_selected(active_pres_idx);

            let sw_sh = gtk4::Switch::builder()
                .active(ext.shading)
                .valign(gtk4::Align::Center)
                .tooltip_text(&crate::core::gettext("Toggle 3D Lighting"))
                .build();

            row_top.append(&combo_pres);
            row_top.append(&sw_sh);
            container_3d.append(&row_top);

            // State Cells
            let rx_cell = Rc::new(std::cell::RefCell::new(ext.rot_x));
            let ry_cell = Rc::new(std::cell::RefCell::new(ext.rot_y));
            let rz_cell = Rc::new(std::cell::RefCell::new(ext.rot_z));
            let light_az_cell = Rc::new(std::cell::RefCell::new(ext.light_angle_deg));
            let light_el_cell = Rc::new(std::cell::RefCell::new(ext.light_elevation_deg));
            let active_color_cell = Rc::new(std::cell::RefCell::new(ext.custom_side_color));
            let active_preset_cell = Rc::new(std::cell::RefCell::new(ext.material_preset));

            // Row 2: Dual Interactive Gizmos (Trackball on Left + Light Gizmo on Right)
            let row_gizmos = gtk4::Box::builder().spacing(10).homogeneous(true).build();

            // Trackball Box
            let box_tb = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(3).halign(gtk4::Align::Center).build();
            let da_trackball = gtk4::DrawingArea::builder()
                .width_request(68)
                .height_request(68)
                .halign(gtk4::Align::Center)
                .valign(gtk4::Align::Center)
                .tooltip_text(&crate::core::gettext("Drag to rotate 3D object\nDouble-click to reset"))
                .build();

            let lbl_tb_title = gtk4::Label::builder()
                .label(&crate::core::gettext("Rotation 3D"))
                .css_classes(["caption", "dim-label"])
                .halign(gtk4::Align::Center)
                .build();

            box_tb.append(&da_trackball);
            box_tb.append(&lbl_tb_title);

            let rx_draw = rx_cell.clone();
            let ry_draw = ry_cell.clone();
            let rz_draw = rz_cell.clone();

            da_trackball.set_draw_func(move |_da, cr, width, height| {
                let cx = width as f64 * 0.5;
                let cy = height as f64 * 0.5;
                let r = (width.min(height) as f64 * 0.46) - 2.0;

                cr.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(0.12, 0.14, 0.20, 0.95);
                cr.fill_preserve().unwrap();
                cr.set_source_rgba(0.32, 0.42, 0.58, 0.8);
                cr.set_line_width(1.5);
                cr.stroke().unwrap();

                let rx = *rx_draw.borrow();
                let ry = *ry_draw.borrow();
                let rz = *rz_draw.borrow();

                let cube_sz = r * 0.50;
                let corners = [
                    (-1.0, -1.0, -1.0), (1.0, -1.0, -1.0), (1.0, 1.0, -1.0), (-1.0, 1.0, -1.0),
                    (-1.0, -1.0, 1.0), (1.0, -1.0, 1.0), (1.0, 1.0, 1.0), (-1.0, 1.0, 1.0),
                ];

                let proj_corner = |(x, y, z): (f64, f64, f64)| -> (f64, f64, f64) {
                    let v = crate::core::modifier::Vec3::new(
                        (x * cube_sz) as f32, (y * cube_sz) as f32, (z * cube_sz) as f32,
                    ).rotate_euler(rx, ry, rz);
                    (cx + v.x as f64, cy + v.y as f64, v.z as f64)
                };

                let projected: Vec<(f64, f64, f64)> = corners.iter().map(|&c| proj_corner(c)).collect();
                let edges = [
                    (0, 1), (1, 2), (2, 3), (3, 0),
                    (4, 5), (5, 6), (6, 7), (7, 4),
                    (0, 4), (1, 5), (2, 6), (3, 7),
                ];

                cr.set_line_width(1.2);
                for (i1, i2) in edges {
                    let p1 = projected[i1];
                    let p2 = projected[i2];
                    let avg_z = (p1.2 + p2.2) * 0.5;
                    if avg_z >= 0.0 {
                        cr.set_source_rgba(0.0, 0.9, 1.0, 0.9);
                    } else {
                        cr.set_source_rgba(0.3, 0.45, 0.6, 0.35);
                    }
                    cr.move_to(p1.0, p1.1);
                    cr.line_to(p2.0, p2.1);
                    cr.stroke().unwrap();
                }

                let axis_len = r * 0.78;
                let x_axis = crate::core::modifier::Vec3::new(axis_len as f32, 0.0, 0.0).rotate_euler(rx, ry, rz);
                let y_axis = crate::core::modifier::Vec3::new(0.0, axis_len as f32, 0.0).rotate_euler(rx, ry, rz);
                let z_axis = crate::core::modifier::Vec3::new(0.0, 0.0, axis_len as f32).rotate_euler(rx, ry, rz);

                cr.move_to(cx, cy);
                cr.line_to(cx + x_axis.x as f64, cy + x_axis.y as f64);
                cr.set_source_rgba(1.0, 0.35, 0.35, 0.95);
                cr.set_line_width(2.0);
                cr.stroke().unwrap();

                cr.move_to(cx, cy);
                cr.line_to(cx + y_axis.x as f64, cy + y_axis.y as f64);
                cr.set_source_rgba(0.35, 0.9, 0.45, 0.95);
                cr.set_line_width(2.0);
                cr.stroke().unwrap();

                cr.move_to(cx, cy);
                cr.line_to(cx + z_axis.x as f64, cy + z_axis.y as f64);
                cr.set_source_rgba(0.2, 0.8, 1.0, 0.95);
                cr.set_line_width(2.0);
                cr.stroke().unwrap();
            });

            // Light Gizmo Box
            let box_lg = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(3).halign(gtk4::Align::Center).build();
            let da_light = gtk4::DrawingArea::builder()
                .width_request(68)
                .height_request(68)
                .halign(gtk4::Align::Center)
                .valign(gtk4::Align::Center)
                .tooltip_text(&crate::core::gettext("Drag to position Light source in 3D\nDouble-click to reset"))
                .build();

            let lbl_lg_title = gtk4::Label::builder()
                .label(&crate::core::gettext("Lighting Gizmo"))
                .css_classes(["caption", "dim-label"])
                .halign(gtk4::Align::Center)
                .build();

            box_lg.append(&da_light);
            box_lg.append(&lbl_lg_title);

            let laz_draw = light_az_cell.clone();
            let lel_draw = light_el_cell.clone();

            da_light.set_draw_func(move |_da, cr, width, height| {
                let cx = width as f64 * 0.5;
                let cy = height as f64 * 0.5;
                let r = (width.min(height) as f64 * 0.46) - 2.0;

                // Base Dial
                cr.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(0.10, 0.12, 0.17, 0.95);
                cr.fill_preserve().unwrap();
                cr.set_source_rgba(0.28, 0.36, 0.48, 0.8);
                cr.set_line_width(1.5);
                cr.stroke().unwrap();

                // Crosshairs
                cr.set_source_rgba(0.3, 0.4, 0.5, 0.35);
                cr.set_line_width(1.0);
                cr.move_to(cx - r * 0.8, cy);
                cr.line_to(cx + r * 0.8, cy);
                cr.move_to(cx, cy - r * 0.8);
                cr.line_to(cx, cy + r * 0.8);
                cr.stroke().unwrap();

                // Elevation ring
                cr.arc(cx, cy, r * 0.5, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(0.3, 0.4, 0.5, 0.25);
                cr.stroke().unwrap();

                let az_rad = (*laz_draw.borrow() as f64).to_radians();
                let el_val = *lel_draw.borrow() as f64; // 0 (edge) to 90 (center)
                let dist_from_center = r * 0.78 * (1.0 - (el_val / 90.0).clamp(0.0, 1.0));

                let sun_x = cx + dist_from_center * az_rad.cos();
                let sun_y = cy - dist_from_center * az_rad.sin();

                // Ray from center to sun
                cr.move_to(cx, cy);
                cr.line_to(sun_x, sun_y);
                cr.set_source_rgba(1.0, 0.8, 0.2, 0.6);
                cr.set_line_width(1.5);
                cr.stroke().unwrap();

                // Sun puck
                cr.arc(sun_x, sun_y, 6.0, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(1.0, 0.88, 0.2, 0.98);
                cr.fill_preserve().unwrap();
                cr.set_source_rgba(1.0, 0.5, 0.0, 0.9);
                cr.set_line_width(1.5);
                cr.stroke().unwrap();

                // Center origin dot
                cr.arc(cx, cy, 2.5, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(0.8, 0.8, 0.9, 0.7);
                cr.fill().unwrap();
            });

            row_gizmos.append(&box_tb);
            row_gizmos.append(&box_lg);
            container_3d.append(&row_gizmos);

            // Row 3: Core Geometry Sliders (Depth & Bevel)
            let box_geo = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).spacing(6).build();

            // Depth Slider
            let row_depth = gtk4::Box::builder().spacing(6).build();
            let lbl_d = gtk4::Label::builder().label("Depth:").css_classes(["caption", "dim-label"]).width_request(45).xalign(0.0).build();
            let adj_dep = gtk4::Adjustment::new(ext.depth as f64, 0.0, 300.0, 2.0, 10.0, 0.0);
            let scale_dep = gtk4::Scale::builder()
                .adjustment(&adj_dep)
                .hexpand(true)
                .draw_value(false)
                .valign(gtk4::Align::Center)
                .build();
            let lbl_dep_val = gtk4::Label::builder()
                .label(&format!("{:.0}px", ext.depth))
                .css_classes(["caption", "numeric"])
                .width_request(42)
                .xalign(1.0)
                .build();
            let lbl_dep_c = lbl_dep_val.clone();
            scale_dep.connect_value_changed(move |s| {
                lbl_dep_c.set_label(&format!("{:.0}px", s.value()));
            });

            row_depth.append(&lbl_d);
            row_depth.append(&scale_dep);
            row_depth.append(&lbl_dep_val);
            box_geo.append(&row_depth);

            // Bevel / Corner Rounding Slider & Style
            let row_bevel = gtk4::Box::builder().spacing(4).build();
            let lbl_b = gtk4::Label::builder().label("Bevel:").css_classes(["caption", "dim-label"]).width_request(45).xalign(0.0).build();
            let combo_bstyle = gtk4::DropDown::from_strings(&[
                &crate::core::gettext("Round"),
                &crate::core::gettext("Chamfer"),
                &crate::core::gettext("Concave"),
            ]);
            let active_bstyle_idx = match ext.bevel_style {
                Bevel3DStyle::Chamfer => 1,
                Bevel3DStyle::Convex => 2,
                _ => 0,
            };
            combo_bstyle.set_selected(active_bstyle_idx);
            combo_bstyle.set_width_request(85);

            let adj_bev = gtk4::Adjustment::new(ext.bevel_radius as f64, 0.0, 60.0, 1.0, 5.0, 0.0);
            let scale_bev = gtk4::Scale::builder()
                .adjustment(&adj_bev)
                .hexpand(true)
                .draw_value(false)
                .valign(gtk4::Align::Center)
                .build();
            let lbl_bev_val = gtk4::Label::builder()
                .label(&format!("{:.0}px", ext.bevel_radius))
                .css_classes(["caption", "numeric"])
                .width_request(38)
                .xalign(1.0)
                .build();
            let lbl_bev_c = lbl_bev_val.clone();
            scale_bev.connect_value_changed(move |s| {
                lbl_bev_c.set_label(&format!("{:.0}px", s.value()));
            });

            row_bevel.append(&lbl_b);
            row_bevel.append(&combo_bstyle);
            row_bevel.append(&scale_bev);
            row_bevel.append(&lbl_bev_val);
            box_geo.append(&row_bevel);

            container_3d.append(&box_geo);

            // Row 5: Collapsible "Advanced 3D & Materials" Expander
            let exp_adv = gtk4::Expander::builder()
                .label(&crate::core::gettext("Advanced 3D & Materials"))
                .expanded(false)
                .build();

            let box_adv = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(6)
                .margin_top(6)
                .margin_bottom(4)
                .build();

            let (row_pers, scale_pers, _lbl_pers_v) = {
                let row = gtk4::Box::builder().spacing(6).build();
                let lbl = gtk4::Label::builder().label("Camera:").css_classes(["caption", "dim-label"]).width_request(65).xalign(0.0).build();
                let adj = gtk4::Adjustment::new(ext.perspective as f64, 0.0, 1500.0, 10.0, 50.0, 0.0);
                let scale = gtk4::Scale::builder().adjustment(&adj).hexpand(true).draw_value(false).valign(gtk4::Align::Center).build();
                let init_str = if ext.perspective > 0.0 { format!("{:.0}", ext.perspective) } else { "Ortho".to_string() };
                let lbl_val = gtk4::Label::builder().label(&init_str).css_classes(["caption", "numeric"]).width_request(45).xalign(1.0).build();
                let lbl_v_c = lbl_val.clone();
                scale.connect_value_changed(move |s| {
                    let v = s.value();
                    if v <= 5.0 {
                        lbl_v_c.set_label("Ortho");
                    } else {
                        lbl_v_c.set_label(&format!("{:.0}", v));
                    }
                });
                row.append(&lbl);
                row.append(&scale);
                row.append(&lbl_val);
                (row, scale, lbl_val)
            };

            let (row_metal, scale_metal, _lbl_metal_v) = {
                let row = gtk4::Box::builder().spacing(6).build();
                let lbl = gtk4::Label::builder().label("Metallic:").css_classes(["caption", "dim-label"]).width_request(65).xalign(0.0).build();
                let adj = gtk4::Adjustment::new(ext.metallic as f64 * 100.0, 0.0, 100.0, 1.0, 10.0, 0.0);
                let scale = gtk4::Scale::builder().adjustment(&adj).hexpand(true).draw_value(false).valign(gtk4::Align::Center).build();
                let lbl_val = gtk4::Label::builder().label(&format!("{:.0}%", ext.metallic * 100.0)).css_classes(["caption", "numeric"]).width_request(45).xalign(1.0).build();
                let lbl_v_c = lbl_val.clone();
                scale.connect_value_changed(move |s| {
                    lbl_v_c.set_label(&format!("{:.0}%", s.value()));
                });
                row.append(&lbl);
                row.append(&scale);
                row.append(&lbl_val);
                (row, scale, lbl_val)
            };

            let (row_rough, scale_rough, _lbl_rough_v) = {
                let row = gtk4::Box::builder().spacing(6).build();
                let lbl = gtk4::Label::builder().label("Roughness:").css_classes(["caption", "dim-label"]).width_request(65).xalign(0.0).build();
                let adj = gtk4::Adjustment::new(ext.roughness as f64 * 100.0, 0.0, 100.0, 1.0, 10.0, 0.0);
                let scale = gtk4::Scale::builder().adjustment(&adj).hexpand(true).draw_value(false).valign(gtk4::Align::Center).build();
                let lbl_val = gtk4::Label::builder().label(&format!("{:.0}%", ext.roughness * 100.0)).css_classes(["caption", "numeric"]).width_request(45).xalign(1.0).build();
                let lbl_v_c = lbl_val.clone();
                scale.connect_value_changed(move |s| {
                    lbl_v_c.set_label(&format!("{:.0}%", s.value()));
                });
                row.append(&lbl);
                row.append(&scale);
                row.append(&lbl_val);
                (row, scale, lbl_val)
            };

            let (row_rim, scale_rim, _lbl_rim_v) = {
                let row = gtk4::Box::builder().spacing(6).build();
                let lbl = gtk4::Label::builder().label("Rim Light:").css_classes(["caption", "dim-label"]).width_request(65).xalign(0.0).build();
                let adj = gtk4::Adjustment::new(ext.rim_light as f64 * 100.0, 0.0, 100.0, 1.0, 10.0, 0.0);
                let scale = gtk4::Scale::builder().adjustment(&adj).hexpand(true).draw_value(false).valign(gtk4::Align::Center).build();
                let lbl_val = gtk4::Label::builder().label(&format!("{:.0}%", ext.rim_light * 100.0)).css_classes(["caption", "numeric"]).width_request(45).xalign(1.0).build();
                let lbl_v_c = lbl_val.clone();
                scale.connect_value_changed(move |s| {
                    lbl_v_c.set_label(&format!("{:.0}%", s.value()));
                });
                row.append(&lbl);
                row.append(&scale);
                row.append(&lbl_val);
                (row, scale, lbl_val)
            };

            let (row_spec, scale_spec, _lbl_spec_v) = {
                let row = gtk4::Box::builder().spacing(6).build();
                let lbl = gtk4::Label::builder().label("Gloss:").css_classes(["caption", "dim-label"]).width_request(65).xalign(0.0).build();
                let adj = gtk4::Adjustment::new(ext.gloss_specular as f64 * 100.0, 0.0, 100.0, 1.0, 10.0, 0.0);
                let scale = gtk4::Scale::builder().adjustment(&adj).hexpand(true).draw_value(false).valign(gtk4::Align::Center).build();
                let lbl_val = gtk4::Label::builder().label(&format!("{:.0}%", ext.gloss_specular * 100.0)).css_classes(["caption", "numeric"]).width_request(45).xalign(1.0).build();
                let lbl_v_c = lbl_val.clone();
                scale.connect_value_changed(move |s| {
                    lbl_v_c.set_label(&format!("{:.0}%", s.value()));
                });
                row.append(&lbl);
                row.append(&scale);
                row.append(&lbl_val);
                (row, scale, lbl_val)
            };

            let (row_tap, scale_tap, _lbl_tap_v) = {
                let row = gtk4::Box::builder().spacing(6).build();
                let lbl = gtk4::Label::builder().label("Taper:").css_classes(["caption", "dim-label"]).width_request(65).xalign(0.0).build();
                let adj = gtk4::Adjustment::new(ext.taper as f64, 0.2, 2.5, 0.05, 0.2, 0.0);
                let scale = gtk4::Scale::builder().adjustment(&adj).hexpand(true).draw_value(false).valign(gtk4::Align::Center).build();
                let lbl_val = gtk4::Label::builder().label(&format!("{:.2}x", ext.taper)).css_classes(["caption", "numeric"]).width_request(45).xalign(1.0).build();
                let lbl_v_c = lbl_val.clone();
                scale.connect_value_changed(move |s| {
                    lbl_v_c.set_label(&format!("{:.2}x", s.value()));
                });
                row.append(&lbl);
                row.append(&scale);
                row.append(&lbl_val);
                (row, scale, lbl_val)
            };

            let (row_twist, scale_twist, _lbl_twist_v) = {
                let row = gtk4::Box::builder().spacing(6).build();
                let lbl = gtk4::Label::builder().label("Twist:").css_classes(["caption", "dim-label"]).width_request(65).xalign(0.0).build();
                let adj = gtk4::Adjustment::new(ext.twist_deg as f64, -180.0, 180.0, 2.0, 15.0, 0.0);
                let scale = gtk4::Scale::builder().adjustment(&adj).hexpand(true).draw_value(false).valign(gtk4::Align::Center).build();
                let lbl_val = gtk4::Label::builder().label(&format!("{:.0}°", ext.twist_deg)).css_classes(["caption", "numeric"]).width_request(45).xalign(1.0).build();
                let lbl_v_c = lbl_val.clone();
                scale.connect_value_changed(move |s| {
                    lbl_v_c.set_label(&format!("{:.0}°", s.value()));
                });
                row.append(&lbl);
                row.append(&scale);
                row.append(&lbl_val);
                (row, scale, lbl_val)
            };

            let (row_c2d, scale_c2d, _lbl_c2d_v) = {
                let row = gtk4::Box::builder().spacing(6).build();
                let lbl = gtk4::Label::builder().label("Corner 2D:").css_classes(["caption", "dim-label"]).width_request(65).xalign(0.0).build();
                let adj = gtk4::Adjustment::new(ext.corner_radius_2d as f64, 0.0, 60.0, 1.0, 5.0, 0.0);
                let scale = gtk4::Scale::builder().adjustment(&adj).hexpand(true).draw_value(false).valign(gtk4::Align::Center).build();
                let lbl_val = gtk4::Label::builder().label(&format!("{:.0}px", ext.corner_radius_2d)).css_classes(["caption", "numeric"]).width_request(45).xalign(1.0).build();
                let lbl_v_c = lbl_val.clone();
                scale.connect_value_changed(move |s| {
                    lbl_v_c.set_label(&format!("{:.0}px", s.value()));
                });
                row.append(&lbl);
                row.append(&scale);
                row.append(&lbl_val);
                (row, scale, lbl_val)
            };

            box_adv.append(&row_pers);
            box_adv.append(&row_metal);
            box_adv.append(&row_rough);
            box_adv.append(&row_rim);
            box_adv.append(&row_spec);
            box_adv.append(&row_tap);
            box_adv.append(&row_twist);
            box_adv.append(&row_c2d);

            exp_adv.set_child(Some(&box_adv));

            // Row 4: Clean Libadwaita Material DropDown + Custom Side Color Picker
            let row_material = gtk4::Box::builder().spacing(6).build();
            let lbl_mat = gtk4::Label::builder()
                .label(&crate::core::gettext("Material:"))
                .css_classes(["caption", "dim-label"])
                .width_request(45)
                .xalign(0.0)
                .build();

            let combo_mat = gtk4::DropDown::from_strings(&[
                &crate::core::gettext("Default Plastic"),
                &crate::core::gettext("Chrome Mirror"),
                &crate::core::gettext("Polished Gold"),
                &crate::core::gettext("Crystal Glass"),
                &crate::core::gettext("Matte Clay"),
                &crate::core::gettext("Neon Glow"),
                &crate::core::gettext("Titanium Dark"),
                &crate::core::gettext("Royal Velvet"),
            ]);
            combo_mat.set_hexpand(true);

            let mat_idx = match ext.material_preset {
                crate::core::modifier::Material3DPreset::Chrome => 1,
                crate::core::modifier::Material3DPreset::Gold => 2,
                crate::core::modifier::Material3DPreset::Glass => 3,
                crate::core::modifier::Material3DPreset::MatteClay => 4,
                crate::core::modifier::Material3DPreset::Neon => 5,
                crate::core::modifier::Material3DPreset::Titanium => 6,
                crate::core::modifier::Material3DPreset::Velvet => 7,
                crate::core::modifier::Material3DPreset::Default => 0,
            };
            combo_mat.set_selected(mat_idx);

            let color_dialog = gtk4::ColorDialog::builder().with_alpha(true).build();
            let color_btn = gtk4::ColorDialogButton::new(Some(color_dialog));
            color_btn.set_tooltip_text(Some(&crate::core::gettext("Custom Side Color")));
            if let Some(c) = ext.custom_side_color {
                color_btn.set_rgba(&gtk4::gdk::RGBA::new(c.r, c.g, c.b, c.a));
            } else {
                color_btn.set_rgba(&gtk4::gdk::RGBA::new(0.5, 0.5, 0.5, 1.0));
            }

            row_material.append(&lbl_mat);
            row_material.append(&combo_mat);
            row_material.append(&color_btn);
            container_3d.append(&row_material);
            container_3d.append(&exp_adv);

            body.append(&container_3d);

            // GestureDrag on 3D Trackball
            let drag_tb = gtk4::GestureDrag::new();
            let start_rot_cell = Rc::new(std::cell::Cell::new((0.0f32, 0.0f32)));
            let rx_drag_c = rx_cell.clone();
            let ry_drag_c = ry_cell.clone();

            let sr_begin = start_rot_cell.clone();
            let rx_b = rx_drag_c.clone();
            let ry_b = ry_drag_c.clone();
            drag_tb.connect_drag_begin(move |_, _x, _y| {
                sr_begin.set((*rx_b.borrow(), *ry_b.borrow()));
            });

            // GestureDrag on Light Gizmo
            let drag_lg = gtk4::GestureDrag::new();
            let laz_drag_c = light_az_cell.clone();
            let lel_drag_c = light_el_cell.clone();
            let da_lg_drag = da_light.clone();

            let update_light_from_coords = {
                let laz_u = laz_drag_c.clone();
                let lel_u = lel_drag_c.clone();
                let da_u = da_lg_drag.clone();
                move |x: f64, y: f64, w: f64, h: f64| {
                    let cx = w * 0.5;
                    let cy = h * 0.5;
                    let max_r = (w.min(h) * 0.46) - 2.0;
                    let dx = x - cx;
                    let dy = -(y - cy);
                    let dist = (dx * dx + dy * dy).sqrt();
                    let angle_rad = dy.atan2(dx);
                    let mut deg = angle_rad.to_degrees() as f32;
                    if deg < 0.0 {
                        deg += 360.0;
                    }
                    let el = (1.0 - (dist / (max_r * 0.78)).min(1.0)) * 90.0;
                    *laz_u.borrow_mut() = deg;
                    *lel_u.borrow_mut() = el as f32;
                    da_u.queue_draw();
                }
            };

            let ulfc_begin = update_light_from_coords.clone();
            let da_lg_b = da_light.clone();
            drag_lg.connect_drag_begin(move |_, x, y| {
                ulfc_begin(x, y, da_lg_b.width() as f64, da_lg_b.height() as f64);
            });

            let ulfc_update = update_light_from_coords;
            let da_lg_u = da_light.clone();
            drag_lg.connect_drag_update(move |gesture, offset_x, offset_y| {
                if let Some((start_x, start_y)) = gesture.start_point() {
                    ulfc_update(start_x + offset_x, start_y + offset_y, da_lg_u.width() as f64, da_lg_u.height() as f64);
                }
            });
            da_light.add_controller(drag_lg);

            // Double-click on Light Gizmo resets light
            let click_lg = gtk4::GestureClick::new();
            let laz_click = light_az_cell.clone();
            let lel_click = light_el_cell.clone();
            let da_lg_click = da_light.clone();
            click_lg.connect_pressed(move |_, n_press, _, _| {
                if n_press >= 2 {
                    *laz_click.borrow_mut() = 135.0;
                    *lel_click.borrow_mut() = 45.0;
                    da_lg_click.queue_draw();
                }
            });
            da_light.add_controller(click_lg);

            // Live Update Callback
            let canvas_cb = canvas.clone();
            let active_cell_sync = active_color_cell.clone();
            let active_pres_sync = active_preset_cell.clone();

            let s_pers1 = scale_pers.clone();
            let s_dep1 = scale_dep.clone();
            let s_c2d1 = scale_c2d.clone();
            let c_bstyle1 = combo_bstyle.clone();
            let s_bev1 = scale_bev.clone();
            let s_tap1 = scale_tap.clone();
            let s_twist1 = scale_twist.clone();
            let sw_sh1 = sw_sh.clone();
            let s_spec1 = scale_spec.clone();
            let s_metal1 = scale_metal.clone();
            let s_rough1 = scale_rough.clone();
            let s_rim1 = scale_rim.clone();
            let combo_pres_sync = combo_pres.clone();
            let da_tb1 = da_trackball.clone();

            let rx_c_sync = rx_cell.clone();
            let ry_c_sync = ry_cell.clone();
            let rz_c_sync = rz_cell.clone();
            let laz_c_sync = light_az_cell;
            let lel_c_sync = light_el_cell;

            let combo_p_closure = combo_pres_sync.clone();
            let sync_all = move || {
                let rx = *rx_c_sync.borrow();
                let ry = *ry_c_sync.borrow();
                let rz = *rz_c_sync.borrow();
                let laz = *laz_c_sync.borrow();
                let lel = *lel_c_sync.borrow();

                let pers = s_pers1.value() as f32;
                let dep = s_dep1.value() as f32;
                let c2d = s_c2d1.value() as f32;
                let bev = s_bev1.value() as f32;
                let tap = s_tap1.value() as f32;
                let twist = s_twist1.value() as f32;
                let sh = sw_sh1.is_active();
                let spec = (s_spec1.value() / 100.0) as f32;
                let metal = (s_metal1.value() / 100.0) as f32;
                let rough = (s_rough1.value() / 100.0) as f32;
                let rim = (s_rim1.value() / 100.0) as f32;

                let sel_m = match combo_p_closure.selected() {
                    1 | 2 => Extrude3DMode::Isometric,
                    8 => Extrude3DMode::Cabinet,
                    _ => Extrude3DMode::Custom3D,
                };

                let sel_bstyle = match c_bstyle1.selected() {
                    1 => Bevel3DStyle::Chamfer,
                    2 => Bevel3DStyle::Convex,
                    _ => Bevel3DStyle::Round,
                };

                da_tb1.queue_draw();

                let side_col = *active_cell_sync.borrow();
                let pres = *active_pres_sync.borrow();
                let mut state = canvas_cb.state.borrow_mut();
                if let Some(elem) = state.document.find_element_mut(elem_id) {
                    if let Some(mods) = elem.modifiers_mut() {
                        if let Some(Modifier::Extrude3D(e)) = mods.get_mut(mod_idx) {
                            e.mode = sel_m;
                            e.rot_x = rx;
                            e.rot_y = ry;
                            e.rot_z = rz;
                            e.perspective = pers;
                            e.depth = dep;
                            e.corner_radius_2d = c2d;
                            e.bevel_style = sel_bstyle;
                            e.bevel_radius = bev;
                            e.taper = tap;
                            e.twist_deg = twist;
                            e.shading = sh;
                            e.light_angle_deg = laz;
                            e.light_elevation_deg = lel;
                            e.gloss_specular = spec;
                            e.material_preset = pres;
                            e.metallic = metal;
                            e.roughness = rough;
                            e.rim_light = rim;
                            e.custom_side_color = side_col;
                        }
                    }
                }
                state.mark_dirty();
                drop(state);
                canvas_cb.queue_draw();
            };

            let su_rc = Rc::new(sync_all);

            let su_drag_tb = su_rc.clone();
            let sr_update = start_rot_cell;
            let rx_u = rx_drag_c;
            let ry_u = ry_drag_c;
            drag_tb.connect_drag_update(move |_, offset_x, offset_y| {
                let (orig_rx, orig_ry) = sr_update.get();
                let new_rx = (orig_rx - offset_y as f32 * 1.5).clamp(-180.0, 180.0);
                let new_ry = (orig_ry + offset_x as f32 * 1.5).clamp(-180.0, 180.0);
                *rx_u.borrow_mut() = new_rx;
                *ry_u.borrow_mut() = new_ry;
                su_drag_tb();
            });
            da_trackball.add_controller(drag_tb);

            let su_drag_lg = su_rc.clone();
            let drag_lg_sync = gtk4::GestureDrag::new();
            drag_lg_sync.connect_drag_update(move |_, _, _| {
                su_drag_lg();
            });
            da_light.add_controller(drag_lg_sync);

            // Double click on 3D Trackball resets to Front view
            let click_tb = gtk4::GestureClick::new();
            let rx_click = rx_cell.clone();
            let ry_click = ry_cell.clone();
            let rz_click = rz_cell;
            let su_click_tb = su_rc.clone();
            click_tb.connect_pressed(move |_, n_press, _, _| {
                if n_press >= 2 {
                    *rx_click.borrow_mut() = 0.0;
                    *ry_click.borrow_mut() = 0.0;
                    *rz_click.borrow_mut() = 0.0;
                    su_click_tb();
                }
            });
            da_trackball.add_controller(click_tb);

            // Preset Dropdown Selection
            let rx_p = rx_cell;
            let ry_p = ry_cell;
            let s_pers_p = scale_pers.clone();
            let s_dep_p = scale_dep.clone();
            let s_tap_p = scale_tap.clone();
            let combo_pres_c = combo_pres.clone();
            let su_pres = su_rc.clone();
            combo_pres.connect_selected_notify(move |_| {
                match combo_pres_c.selected() {
                    1 => { *rx_p.borrow_mut() = 35.26; *ry_p.borrow_mut() = -45.0; s_pers_p.set_value(0.0); s_dep_p.set_value(40.0); s_tap_p.set_value(1.0); }
                    2 => { *rx_p.borrow_mut() = 35.26; *ry_p.borrow_mut() = 45.0; s_pers_p.set_value(0.0); s_dep_p.set_value(40.0); s_tap_p.set_value(1.0); }
                    3 => { *rx_p.borrow_mut() = 0.0; *ry_p.borrow_mut() = 0.0; s_pers_p.set_value(600.0); s_dep_p.set_value(40.0); s_tap_p.set_value(1.0); }
                    4 => { *rx_p.borrow_mut() = 90.0; *ry_p.borrow_mut() = 0.0; s_pers_p.set_value(0.0); s_dep_p.set_value(40.0); s_tap_p.set_value(1.0); }
                    5 => { *rx_p.borrow_mut() = 25.0; *ry_p.borrow_mut() = -35.0; s_pers_p.set_value(600.0); s_dep_p.set_value(45.0); s_tap_p.set_value(1.0); }
                    6 => { *rx_p.borrow_mut() = -20.0; *ry_p.borrow_mut() = 40.0; s_pers_p.set_value(450.0); s_dep_p.set_value(50.0); s_tap_p.set_value(0.8); }
                    7 => { *rx_p.borrow_mut() = 15.0; *ry_p.borrow_mut() = 75.0; s_pers_p.set_value(700.0); s_dep_p.set_value(30.0); s_tap_p.set_value(1.0); }
                    8 => { *rx_p.borrow_mut() = 0.0; *ry_p.borrow_mut() = 0.0; s_pers_p.set_value(0.0); s_dep_p.set_value(40.0); s_tap_p.set_value(1.0); }
                    _ => {}
                }
                su_pres();
            });

            let su1 = su_rc.clone();
            combo_pres_sync.connect_selected_notify(move |_| su1());

            let su2 = su_rc.clone();
            sw_sh.connect_active_notify(move |_| su2());

            let su6 = su_rc.clone();
            scale_dep.connect_value_changed(move |_| su6());

            let su7 = su_rc.clone();
            scale_pers.connect_value_changed(move |_| su7());

            let su8 = su_rc.clone();
            scale_c2d.connect_value_changed(move |_| su8());

            let su9 = su_rc.clone();
            combo_bstyle.connect_selected_notify(move |_| su9());

            let su10 = su_rc.clone();
            scale_bev.connect_value_changed(move |_| su10());

            let su11 = su_rc.clone();
            scale_tap.connect_value_changed(move |_| su11());

            let su12 = su_rc.clone();
            scale_twist.connect_value_changed(move |_| su12());

            let su13 = su_rc.clone();
            scale_metal.connect_value_changed(move |_| su13());

            let su14 = su_rc.clone();
            scale_rough.connect_value_changed(move |_| su14());

            let su15 = su_rc.clone();
            scale_rim.connect_value_changed(move |_| su15());

            let su16 = su_rc.clone();
            scale_spec.connect_value_changed(move |_| su16());

            let s_metal_p = scale_metal.clone();
            let s_rough_p = scale_rough.clone();
            let s_rim_p = scale_rim.clone();
            let s_spec_p = scale_spec.clone();
            let act_pres_c = active_preset_cell.clone();
            let act_col_c = active_color_cell.clone();
            let col_btn_c = color_btn.clone();
            let combo_mat_c = combo_mat.clone();
            let su_mat = su_rc.clone();

            combo_mat.connect_selected_notify(move |_| {
                let (pres, c_opt, p_metal, p_rough, p_rim, p_gloss) = match combo_mat_c.selected() {
                    1 => (crate::core::modifier::Material3DPreset::Chrome, Some(crate::core::Color::new(0.20, 0.22, 0.26, 1.0)), 0.95, 0.05, 0.70, 0.95),
                    2 => (crate::core::modifier::Material3DPreset::Gold, Some(crate::core::Color::new(0.95, 0.76, 0.18, 1.0)), 0.92, 0.12, 0.60, 0.88),
                    3 => (crate::core::modifier::Material3DPreset::Glass, Some(crate::core::Color::new(0.15, 0.85, 0.95, 1.0)), 0.0, 0.04, 0.95, 0.92),
                    4 => (crate::core::modifier::Material3DPreset::MatteClay, Some(crate::core::Color::new(0.88, 0.84, 0.80, 1.0)), 0.0, 0.95, 0.08, 0.02),
                    5 => (crate::core::modifier::Material3DPreset::Neon, Some(crate::core::Color::new(0.95, 0.15, 0.65, 1.0)), 0.15, 0.20, 1.0, 0.75),
                    6 => (crate::core::modifier::Material3DPreset::Titanium, Some(crate::core::Color::new(0.12, 0.15, 0.19, 1.0)), 0.88, 0.22, 0.55, 0.72),
                    7 => (crate::core::modifier::Material3DPreset::Velvet, Some(crate::core::Color::new(0.48, 0.12, 0.65, 1.0)), 0.0, 0.85, 0.90, 0.05),
                    _ => (crate::core::modifier::Material3DPreset::Default, None, 0.0, 0.35, 0.30, 0.40),
                };
                *act_pres_c.borrow_mut() = pres;
                *act_col_c.borrow_mut() = c_opt;
                if let Some(c) = c_opt {
                    col_btn_c.set_rgba(&gtk4::gdk::RGBA::new(c.r, c.g, c.b, c.a));
                }
                s_metal_p.set_value(p_metal as f64 * 100.0);
                s_rough_p.set_value(p_rough as f64 * 100.0);
                s_rim_p.set_value(p_rim as f64 * 100.0);
                s_spec_p.set_value(p_gloss as f64 * 100.0);
                su_mat();
            });

            let act_col_btn = active_color_cell;
            let su_col_btn = su_rc;
            color_btn.connect_rgba_notify(move |btn| {
                let rgba = btn.rgba();
                *act_col_btn.borrow_mut() = Some(crate::core::Color::new(rgba.red(), rgba.green(), rgba.blue(), rgba.alpha()));
                su_col_btn();
            });
        }
        Modifier::Twist(tw) => {
            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(6)
                .build();

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
            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(6)
                .build();

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
            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(6)
                .build();

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
            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(6)
                .build();

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
            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(6)
                .build();

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
            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(6)
                .build();

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
        state.mark_dirty();
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
