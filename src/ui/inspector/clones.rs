use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::document::{TiledCloneParams, TiledCloneSymmetry};
use crate::core::element::Element;
use crate::ui::canvas::CanvasWidget;

pub fn build_clones_section(canvas: &CanvasWidget) -> (gtk4::Widget, Rc<dyn Fn()>) {
    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .propagate_natural_height(true)
        .build();

    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(10)
        .margin_top(6)
        .margin_bottom(12)
        .margin_start(10)
        .margin_end(10)
        .build();

    scrolled.set_child(Some(&container));

    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(10)
        .build();

    container.append(&content_box);

    let canvas_c = canvas.clone();
    let content_box_c = content_box.clone();
    let tiled_params_cell = Rc::new(RefCell::new(TiledCloneParams::default()));

    let update_fn: Rc<dyn Fn()> = {
        let canvas_c = canvas_c.clone();
        let content_box_c = content_box_c.clone();
        let tiled_params_rc = tiled_params_cell.clone();

        Rc::new(move || {
            crate::ui::inspector::appearance::clear_box(&content_box_c);

            let sel_count = canvas_c.selection_count();
            let has_clones = canvas_c.has_clones_selected();
            let has_masters = canvas_c.has_masters_selected();

            // ── Helper to build Tiled Clones Studio Card ──
            let tiled_card = build_tiled_clones_card(
                &canvas_c,
                &tiled_params_rc,
                sel_count > 0,
            );

            // 1. MASTER IS SELECTED
            if has_masters && sel_count == 1 {
                let sel_id = {
                    let mut found_master = None;
                    if let Ok(doc_state) = canvas_c.state.try_borrow() {
                        for id in &doc_state.document.selected_ids {
                            if let Some(el) = doc_state.document.find_element(*id) {
                                found_master = Some(el.clone());
                                break;
                            }
                        }
                    }
                    found_master
                };

                if let Some(master) = sel_id {
                    let master_id = master.id();
                    let clones = canvas_c.get_clones_for_master(master_id);

                    let header_card = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Vertical)
                        .spacing(8)
                        .css_classes(["card"])
                        .margin_bottom(4)
                        .build();

                    let top_row = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Horizontal)
                        .spacing(8)
                        .margin_top(8)
                        .margin_bottom(4)
                        .margin_start(10)
                        .margin_end(10)
                        .build();

                    let icon = gtk4::Image::from_icon_name("clone-master-symbolic");
                    icon.set_pixel_size(18);
                    icon.add_css_class("accent");
                    top_row.append(&icon);

                    let title_box = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Vertical)
                        .spacing(2)
                        .hexpand(true)
                        .build();

                    let title_lbl = gtk4::Label::builder()
                        .label(&master.name())
                        .css_classes(["heading"])
                        .halign(gtk4::Align::Start)
                        .build();

                    let sub_lbl = gtk4::Label::builder()
                        .label(&format!(
                            "{} • {} {}",
                            crate::core::gettext("Master Object"),
                            clones.len(),
                            if clones.len() == 1 {
                                crate::core::gettext("linked instance")
                            } else {
                                crate::core::gettext("linked instances")
                            }
                        ))
                        .css_classes(["caption", "dim-label"])
                        .halign(gtk4::Align::Start)
                        .build();

                    title_box.append(&title_lbl);
                    title_box.append(&sub_lbl);
                    top_row.append(&title_box);

                    let badge = gtk4::Label::builder()
                        .label(crate::core::gettext("MASTER"))
                        .css_classes(["badge", "accent"])
                        .valign(gtk4::Align::Center)
                        .build();
                    top_row.append(&badge);

                    header_card.append(&top_row);

                    // Action buttons for master
                    let actions_row = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Horizontal)
                        .spacing(6)
                        .homogeneous(true)
                        .margin_start(10)
                        .margin_end(10)
                        .margin_bottom(8)
                        .build();

                    let btn_create_clone = gtk4::Button::builder()
                        .label(crate::core::gettext("New Clone"))
                        .icon_name("list-add-symbolic")
                        .css_classes(["flat", "pill-button"])
                        .tooltip_text(crate::core::gettext("Create a single linked clone (Alt+D)"))
                        .build();
                    {
                        let can = canvas_c.clone();
                        btn_create_clone.connect_clicked(move |_| {
                            can.clone_master_by_id(master_id);
                        });
                    }
                    actions_row.append(&btn_create_clone);

                    let btn_select_all = gtk4::Button::builder()
                        .label(crate::core::gettext("Select Clones"))
                        .icon_name("check-symbolic")
                        .css_classes(["flat", "pill-button"])
                        .tooltip_text(crate::core::gettext("Select all instances linked to this master"))
                        .build();
                    {
                        let can = canvas_c.clone();
                        btn_select_all.connect_clicked(move |_| {
                            can.select_linked_clones();
                        });
                    }
                    actions_row.append(&btn_select_all);

                    let btn_unlink_all = gtk4::Button::builder()
                        .label(crate::core::gettext("Unlink All"))
                        .icon_name("edit-cut-symbolic")
                        .css_classes(["flat", "pill-button", "destructive-action"])
                        .tooltip_text(crate::core::gettext("Convert all instances to independent shapes"))
                        .build();
                    {
                        let can = canvas_c.clone();
                        btn_unlink_all.connect_clicked(move |_| {
                            can.unlink_all_clones_for_master(master_id);
                        });
                    }
                    actions_row.append(&btn_unlink_all);

                    header_card.append(&actions_row);
                    content_box_c.append(&header_card);

                    // Add Tiled Clones Studio card
                    content_box_c.append(&tiled_card);

                    // List of Clones
                    if !clones.is_empty() {
                        let instances_group = adw::PreferencesGroup::builder()
                            .title(crate::core::gettext("Linked Instances"))
                            .description(crate::core::gettext(
                                "Clones share master path & styling with independent spatial transform",
                            ))
                            .build();

                        for (idx, clone) in clones.iter().enumerate() {
                            let cid = clone.id;
                            let title_text = clone
                                .name
                                .clone()
                                .unwrap_or_else(|| format!("Clone #{}", idx + 1));
                            let row = adw::ActionRow::builder()
                                .title(glib::markup_escape_text(&title_text))
                                .subtitle(format!(
                                    "X: {:.1} px  Y: {:.1} px • Scale: {:.0}% × {:.0}% • Rot: {:.1}°",
                                    clone.offset.x,
                                    clone.offset.y,
                                    clone.scale.x * 100.0,
                                    clone.scale.y * 100.0,
                                    clone.rotation.to_degrees()
                                ))
                                .activatable(true)
                                .build();

                            let icon = gtk4::Image::from_icon_name("check-symbolic");
                            icon.set_pixel_size(16);
                            row.add_prefix(&icon);

                            let btn_select = gtk4::Button::builder()
                                .icon_name("eyedropper-pick-symbolic")
                                .css_classes(["flat", "circular"])
                                .tooltip_text(crate::core::gettext("Focus & Select on Canvas"))
                                .valign(gtk4::Align::Center)
                                .build();
                            {
                                let can = canvas_c.clone();
                                btn_select.connect_clicked(move |_| {
                                    can.select_element_by_id(cid);
                                });
                            }
                            row.add_suffix(&btn_select);

                            let btn_unlink = gtk4::Button::builder()
                                .icon_name("edit-cut-symbolic")
                                .css_classes(["flat", "circular"])
                                .tooltip_text(crate::core::gettext("Unlink instance into independent object"))
                                .valign(gtk4::Align::Center)
                                .build();
                            {
                                let can = canvas_c.clone();
                                btn_unlink.connect_clicked(move |_| {
                                    can.unlink_clone_by_id(cid);
                                });
                            }
                            row.add_suffix(&btn_unlink);

                            {
                                let can = canvas_c.clone();
                                row.connect_activated(move |_| {
                                    can.select_element_by_id(cid);
                                });
                            }

                            instances_group.add(&row);
                        }

                        content_box_c.append(&instances_group);
                    }
                    return;
                }
            }

            // 2. CLONE IS SELECTED
            if has_clones && sel_count == 1 {
                let clone_info = {
                    let mut found_clone = None;
                    if let Ok(doc_state) = canvas_c.state.try_borrow() {
                        for id in &doc_state.document.selected_ids {
                            if let Some(Element::Clone(c)) = doc_state.document.find_element(*id) {
                                found_clone = Some(c.clone());
                                break;
                            }
                        }
                    }
                    found_clone
                };

                if let Some(clone) = clone_info {
                    let clone_id = clone.id;
                    let _master_id = clone.source_id;
                    let master = canvas_c.get_master_for_clone(clone_id);

                    let header_card = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Vertical)
                        .spacing(8)
                        .css_classes(["card"])
                        .margin_bottom(4)
                        .build();

                    let top_row = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Horizontal)
                        .spacing(8)
                        .margin_top(8)
                        .margin_bottom(4)
                        .margin_start(10)
                        .margin_end(10)
                        .build();

                    let icon = gtk4::Image::from_icon_name("check-symbolic");
                    icon.set_pixel_size(18);
                    icon.add_css_class("accent");
                    top_row.append(&icon);

                    let title_box = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Vertical)
                        .spacing(2)
                        .hexpand(true)
                        .build();

                    let title_lbl = gtk4::Label::builder()
                        .label(clone.name.as_deref().unwrap_or("Linked Clone"))
                        .css_classes(["heading"])
                        .halign(gtk4::Align::Start)
                        .build();

                    let master_name = master
                        .as_ref()
                        .map(|m| m.name())
                        .unwrap_or_else(|| "Unknown".to_string());
                    let sub_lbl = gtk4::Label::builder()
                        .label(&format!(
                            "{} • {}",
                            crate::core::gettext("Instance of"),
                            master_name
                        ))
                        .css_classes(["caption", "dim-label"])
                        .halign(gtk4::Align::Start)
                        .build();

                    title_box.append(&title_lbl);
                    title_box.append(&sub_lbl);
                    top_row.append(&title_box);

                    let badge = gtk4::Label::builder()
                        .label(crate::core::gettext("INSTANCE"))
                        .css_classes(["badge"])
                        .valign(gtk4::Align::Center)
                        .build();
                    top_row.append(&badge);

                    header_card.append(&top_row);

                    // Action buttons for clone
                    let actions_row = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Horizontal)
                        .spacing(6)
                        .homogeneous(true)
                        .margin_start(10)
                        .margin_end(10)
                        .margin_bottom(8)
                        .build();

                    let btn_select_master = gtk4::Button::builder()
                        .label(crate::core::gettext("Go to Master"))
                        .icon_name("clone-jump-master-symbolic")
                        .css_classes(["flat", "pill-button"])
                        .tooltip_text(crate::core::gettext("Select and edit the master element"))
                        .build();
                    {
                        let can = canvas_c.clone();
                        btn_select_master.connect_clicked(move |_| {
                            can.select_original_element();
                        });
                    }
                    actions_row.append(&btn_select_master);

                    let btn_unlink = gtk4::Button::builder()
                        .label(crate::core::gettext("Unlink Clone"))
                        .icon_name("edit-cut-symbolic")
                        .css_classes(["flat", "pill-button"])
                        .tooltip_text(crate::core::gettext("Convert this clone into an independent vector shape"))
                        .build();
                    {
                        let can = canvas_c.clone();
                        btn_unlink.connect_clicked(move |_| {
                            can.unlink_clone_by_id(clone_id);
                        });
                    }
                    actions_row.append(&btn_unlink);

                    header_card.append(&actions_row);
                    content_box_c.append(&header_card);

                    // Add Tiled Clones Studio card
                    content_box_c.append(&tiled_card);

                    // Clone Transformation Properties Card
                    let transform_group = adw::PreferencesGroup::builder()
                        .title(crate::core::gettext("Spatial Transformations"))
                        .description(crate::core::gettext(
                            "Independent position, scale, and rotation applied to this instance",
                        ))
                        .build();

                    let pos_row = adw::ActionRow::builder()
                        .title(crate::core::gettext("Local Offset"))
                        .subtitle(format!("X: {:.1} px,  Y: {:.1} px", clone.offset.x, clone.offset.y))
                        .build();
                    transform_group.add(&pos_row);

                    let scale_row = adw::ActionRow::builder()
                        .title(crate::core::gettext("Instance Scale"))
                        .subtitle(format!("{:.1}%  ×  {:.1}%", clone.scale.x * 100.0, clone.scale.y * 100.0))
                        .build();
                    transform_group.add(&scale_row);

                    let rot_row = adw::ActionRow::builder()
                        .title(crate::core::gettext("Instance Rotation"))
                        .subtitle(format!("{:.1}°", clone.rotation.to_degrees()))
                        .build();
                    transform_group.add(&rot_row);

                    content_box_c.append(&transform_group);
                    return;
                }
            }

            // 3. SELECTION / OVERVIEW MODE (Any regular object or document overview)
            if sel_count > 0 {
                content_box_c.append(&tiled_card);
            } else {
                // Empty selection prompt + Tiled Card (with helper)
                content_box_c.append(&tiled_card);

                let all_clones = canvas_c.get_all_clone_relationships();
                if !all_clones.is_empty() {
                    let overview_group = adw::PreferencesGroup::builder()
                        .title(crate::core::gettext("Document Clones Overview"))
                        .description(crate::core::gettext(
                            "Overview of all master objects and linked instances in the project",
                        ))
                        .build();

                    for (master_id, clones) in all_clones {
                        let master_name = if let Some(m) = canvas_c
                            .state
                            .try_borrow()
                            .ok()
                            .and_then(|s| s.document.find_element(master_id).cloned())
                        {
                            m.name()
                        } else {
                            format!("Master #{}", master_id.0)
                        };

                        let expander = adw::ExpanderRow::builder()
                            .title(glib::markup_escape_text(&master_name))
                            .subtitle(format!(
                                "{} {}",
                                clones.len(),
                                if clones.len() == 1 {
                                    crate::core::gettext("linked instance")
                                } else {
                                    crate::core::gettext("linked instances")
                                }
                            ))
                            .build();
                        expander.add_prefix(&gtk4::Image::from_icon_name("clone-master-symbolic"));

                        let btn_sel_master = gtk4::Button::builder()
                            .icon_name("eyedropper-pick-symbolic")
                            .css_classes(["flat", "circular"])
                            .tooltip_text(crate::core::gettext("Select Master on Canvas"))
                            .valign(gtk4::Align::Center)
                            .build();
                        {
                            let can = canvas_c.clone();
                            btn_sel_master.connect_clicked(move |_| {
                                can.select_element_by_id(master_id);
                            });
                        }
                        expander.add_suffix(&btn_sel_master);

                        for (idx, clone) in clones.iter().enumerate() {
                            let cid = clone.id;
                            let title_text = clone
                                .name
                                .clone()
                                .unwrap_or_else(|| format!("Clone #{}", idx + 1));
                            let row = adw::ActionRow::builder()
                                .title(glib::markup_escape_text(&title_text))
                                .subtitle(format!(
                                    "X: {:.1} px  Y: {:.1} px • Scale: {:.0}%",
                                    clone.offset.x, clone.offset.y, clone.scale.x * 100.0
                                ))
                                .activatable(true)
                                .build();
                            row.add_prefix(&gtk4::Image::from_icon_name("clone-symbolic"));

                            let btn_sel = gtk4::Button::builder()
                                .icon_name("eyedropper-pick-symbolic")
                                .css_classes(["flat", "circular"])
                                .valign(gtk4::Align::Center)
                                .build();
                            {
                                let can = canvas_c.clone();
                                btn_sel.connect_clicked(move |_| {
                                    can.select_element_by_id(cid);
                                });
                            }
                            row.add_suffix(&btn_sel);

                            {
                                let can = canvas_c.clone();
                                row.connect_activated(move |_| {
                                    can.select_element_by_id(cid);
                                });
                            }

                            expander.add_row(&row);
                        }

                        overview_group.add(&expander);
                    }

                    content_box_c.append(&overview_group);
                }
            }
        })
    };

    update_fn();
    (scrolled.upcast(), update_fn)
}

// ─────────────────────────────────────────────────────────────
// Tiled Clones Studio UI Card Builder
// ─────────────────────────────────────────────────────────────

fn build_tiled_clones_card(
    canvas: &CanvasWidget,
    params_rc: &Rc<RefCell<TiledCloneParams>>,
    has_selection: bool,
) -> gtk4::Box {
    let card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .css_classes(["card"])
        .margin_bottom(4)
        .build();

    // 1. Header
    let header_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(2)
        .margin_start(10)
        .margin_end(10)
        .build();

    let icon = gtk4::Image::from_icon_name("view-grid-symbolic");
    icon.set_pixel_size(18);
    icon.add_css_class("accent");
    header_box.append(&icon);

    let title_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(1)
        .hexpand(true)
        .build();

    let title_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Create Tiled Clones"))
        .css_classes(["heading"])
        .halign(gtk4::Align::Start)
        .build();

    let sub_lbl = gtk4::Label::builder()
        .label(if has_selection {
            crate::core::gettext("Generate symmetrical tile matrices & radial arrays")
        } else {
            crate::core::gettext("Select any object on canvas to generate tiled clones")
        })
        .css_classes(["caption", "dim-label"])
        .halign(gtk4::Align::Start)
        .wrap(true)
        .build();

    title_box.append(&title_lbl);
    title_box.append(&sub_lbl);
    header_box.append(&title_box);

    let reset_btn = gtk4::Button::builder()
        .icon_name("view-refresh-symbolic")
        .css_classes(["flat", "circular"])
        .tooltip_text(crate::core::gettext("Reset Tiling Settings to Default"))
        .valign(gtk4::Align::Center)
        .build();

    header_box.append(&reset_btn);
    card.append(&header_box);

    // 2. Symmetry Selection Row
    let sym_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_start(10)
        .margin_end(10)
        .valign(gtk4::Align::Center)
        .build();

    let sym_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Symmetry"))
        .css_classes(["caption", "dim-label"])
        .build();
    sym_row.append(&sym_lbl);

    let sym_options = [
        "P1 — Simple Translation",
        "P2 — 180° Half-Turn Rotation",
        "PM — Horizontal Reflection (Mirror X)",
        "PMM — Double Reflection (Mirror X & Y)",
        "PG — Glide Reflection",
        "CM — Alternating Reflection",
        "PMG — Reflection & Glide",
        "PGG — Double Glide Reflection",
        "P4 — 90° Quadrant Rotation",
        "P6 — 60° Hexagonal Symmetry",
        "Radial — Circular Ring Pattern",
    ];

    let sym_dropdown = gtk4::DropDown::from_strings(&sym_options);
    sym_dropdown.set_hexpand(true);

    let cur_sym_idx = match params_rc.borrow().symmetry {
        TiledCloneSymmetry::P1 => 0,
        TiledCloneSymmetry::P2 => 1,
        TiledCloneSymmetry::PM => 2,
        TiledCloneSymmetry::PMM => 3,
        TiledCloneSymmetry::PG => 4,
        TiledCloneSymmetry::CM => 5,
        TiledCloneSymmetry::PMG => 6,
        TiledCloneSymmetry::PGG => 7,
        TiledCloneSymmetry::P4 => 8,
        TiledCloneSymmetry::P6 => 9,
        TiledCloneSymmetry::Radial => 10,
    };
    sym_dropdown.set_selected(cur_sym_idx);

    {
        let p_rc = params_rc.clone();
        sym_dropdown.connect_selected_notify(move |dd| {
            let sym = match dd.selected() {
                0 => TiledCloneSymmetry::P1,
                1 => TiledCloneSymmetry::P2,
                2 => TiledCloneSymmetry::PM,
                3 => TiledCloneSymmetry::PMM,
                4 => TiledCloneSymmetry::PG,
                5 => TiledCloneSymmetry::CM,
                6 => TiledCloneSymmetry::PMG,
                7 => TiledCloneSymmetry::PGG,
                8 => TiledCloneSymmetry::P4,
                9 => TiledCloneSymmetry::P6,
                10 => TiledCloneSymmetry::Radial,
                _ => TiledCloneSymmetry::P1,
            };
            p_rc.borrow_mut().symmetry = sym;
        });
    }
    sym_row.append(&sym_dropdown);
    card.append(&sym_row);

    // 3. Rows & Columns Grid Matrix
    let grid_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_start(10)
        .margin_end(10)
        .valign(gtk4::Align::Center)
        .build();

    let rows_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Rows"))
        .css_classes(["caption"])
        .build();
    grid_box.append(&rows_lbl);

    let rows_spin = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
    rows_spin.set_value(params_rc.borrow().rows as f64);
    rows_spin.set_hexpand(true);
    rows_spin.add_css_class("numeric");
    grid_box.append(&rows_spin);

    let cols_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Cols"))
        .css_classes(["caption"])
        .build();
    grid_box.append(&cols_lbl);

    let cols_spin = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
    cols_spin.set_value(params_rc.borrow().cols as f64);
    cols_spin.set_hexpand(true);
    cols_spin.add_css_class("numeric");
    grid_box.append(&cols_spin);

    let count_badge = gtk4::Label::builder()
        .label(format!(
            "{} {}",
            params_rc.borrow().rows * params_rc.borrow().cols,
            crate::core::gettext("tiles")
        ))
        .css_classes(["badge"])
        .valign(gtk4::Align::Center)
        .build();
    grid_box.append(&count_badge);
    card.append(&grid_box);

    // Sync spinners with params
    {
        let p_rc = params_rc.clone();
        let badge_c = count_badge.clone();
        let cols_sp = cols_spin.clone();
        rows_spin.connect_value_changed(move |sp| {
            let r = sp.value() as usize;
            p_rc.borrow_mut().rows = r;
            let c = cols_sp.value() as usize;
            badge_c.set_label(&format!("{} {}", r * c, crate::core::gettext("tiles")));
        });
    }
    {
        let p_rc = params_rc.clone();
        let badge_c = count_badge.clone();
        let rows_sp = rows_spin.clone();
        cols_spin.connect_value_changed(move |sp| {
            let c = sp.value() as usize;
            p_rc.borrow_mut().cols = c;
            let r = rows_sp.value() as usize;
            badge_c.set_label(&format!("{} {}", r * c, crate::core::gettext("tiles")));
        });
    }

    // 4. Detailed Transformation Groups (Expanders)
    let prefs_group = adw::PreferencesGroup::builder()
        .margin_start(4)
        .margin_end(4)
        .build();

    // 4A. Shift & Spacing Expander
    let shift_expander = adw::ExpanderRow::builder()
        .title(crate::core::gettext("Shift & Spacing"))
        .subtitle(crate::core::gettext("Offset percentage per row / column"))
        .build();
    shift_expander.add_prefix(&gtk4::Image::from_icon_name("transform-symbolic"));

    let shift_x_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Shift X (% width)"))
        .build();
    let shift_x_spin = gtk4::SpinButton::with_range(-500.0, 500.0, 5.0);
    shift_x_spin.set_value(params_rc.borrow().shift_x_pct as f64);
    shift_x_spin.add_css_class("numeric");
    shift_x_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        shift_x_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().shift_x_pct = sp.value() as f32;
        });
    }
    shift_x_row.add_suffix(&shift_x_spin);
    shift_expander.add_row(&shift_x_row);

    let shift_y_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Shift Y (% height)"))
        .build();
    let shift_y_spin = gtk4::SpinButton::with_range(-500.0, 500.0, 5.0);
    shift_y_spin.set_value(params_rc.borrow().shift_y_pct as f64);
    shift_y_spin.add_css_class("numeric");
    shift_y_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        shift_y_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().shift_y_pct = sp.value() as f32;
        });
    }
    shift_y_row.add_suffix(&shift_y_spin);
    shift_expander.add_row(&shift_y_row);

    let rand_shift_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Random Jitter (%)"))
        .build();
    let rand_shift_spin = gtk4::SpinButton::with_range(0.0, 100.0, 5.0);
    rand_shift_spin.set_value(params_rc.borrow().shift_x_rand_pct as f64);
    rand_shift_spin.add_css_class("numeric");
    rand_shift_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        rand_shift_spin.connect_value_changed(move |sp| {
            let val = sp.value() as f32;
            p_rc.borrow_mut().shift_x_rand_pct = val;
            p_rc.borrow_mut().shift_y_rand_pct = val;
        });
    }
    rand_shift_row.add_suffix(&rand_shift_spin);
    shift_expander.add_row(&rand_shift_row);

    prefs_group.add(&shift_expander);

    // 4B. Scale & Progression Expander
    let scale_expander = adw::ExpanderRow::builder()
        .title(crate::core::gettext("Scale & Progression"))
        .subtitle(crate::core::gettext("Scale growth or reduction per step"))
        .build();
    scale_expander.add_prefix(&gtk4::Image::from_icon_name("transform-scale-stroke-symbolic"));

    let scale_x_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Scale X Delta (% / col)"))
        .build();
    let scale_x_spin = gtk4::SpinButton::with_range(-50.0, 100.0, 5.0);
    scale_x_spin.set_value(params_rc.borrow().scale_x_pct as f64);
    scale_x_spin.add_css_class("numeric");
    scale_x_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        scale_x_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().scale_x_pct = sp.value() as f32;
        });
    }
    scale_x_row.add_suffix(&scale_x_spin);
    scale_expander.add_row(&scale_x_row);

    let scale_y_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Scale Y Delta (% / row)"))
        .build();
    let scale_y_spin = gtk4::SpinButton::with_range(-50.0, 100.0, 5.0);
    scale_y_spin.set_value(params_rc.borrow().scale_y_pct as f64);
    scale_y_spin.add_css_class("numeric");
    scale_y_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        scale_y_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().scale_y_pct = sp.value() as f32;
        });
    }
    scale_y_row.add_suffix(&scale_y_spin);
    scale_expander.add_row(&scale_y_row);

    let rand_scale_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Random Scale (%)"))
        .build();
    let rand_scale_spin = gtk4::SpinButton::with_range(0.0, 100.0, 5.0);
    rand_scale_spin.set_value(params_rc.borrow().scale_rand_pct as f64);
    rand_scale_spin.add_css_class("numeric");
    rand_scale_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        rand_scale_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().scale_rand_pct = sp.value() as f32;
        });
    }
    rand_scale_row.add_suffix(&rand_scale_spin);
    scale_expander.add_row(&rand_scale_row);

    prefs_group.add(&scale_expander);

    // 4C. Rotation & Angle Expander
    let rot_expander = adw::ExpanderRow::builder()
        .title(crate::core::gettext("Rotation & Angle"))
        .subtitle(crate::core::gettext("Angle increments per row / column"))
        .build();
    rot_expander.add_prefix(&gtk4::Image::from_icon_name("rotate-right-symbolic"));

    let rot_col_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Angle / Col (°)"))
        .build();
    let rot_col_spin = gtk4::SpinButton::with_range(-360.0, 360.0, 5.0);
    rot_col_spin.set_value(params_rc.borrow().rotate_per_col_deg as f64);
    rot_col_spin.add_css_class("numeric");
    rot_col_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        rot_col_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().rotate_per_col_deg = sp.value() as f32;
        });
    }
    rot_col_row.add_suffix(&rot_col_spin);
    rot_expander.add_row(&rot_col_row);

    let rot_row_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Angle / Row (°)"))
        .build();
    let rot_row_spin = gtk4::SpinButton::with_range(-360.0, 360.0, 5.0);
    rot_row_spin.set_value(params_rc.borrow().rotate_per_row_deg as f64);
    rot_row_spin.add_css_class("numeric");
    rot_row_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        rot_row_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().rotate_per_row_deg = sp.value() as f32;
        });
    }
    rot_row_row.add_suffix(&rot_row_spin);
    rot_expander.add_row(&rot_row_row);

    let rand_rot_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Random Angle (°)"))
        .build();
    let rand_rot_spin = gtk4::SpinButton::with_range(0.0, 360.0, 5.0);
    rand_rot_spin.set_value(params_rc.borrow().rotate_rand_deg as f64);
    rand_rot_spin.add_css_class("numeric");
    rand_rot_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        rand_rot_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().rotate_rand_deg = sp.value() as f32;
        });
    }
    rand_rot_row.add_suffix(&rand_rot_spin);
    rot_expander.add_row(&rand_rot_row);

    prefs_group.add(&rot_expander);

    // 4D. Opacity & Radial Expander
    let op_expander = adw::ExpanderRow::builder()
        .title(crate::core::gettext("Opacity & Radial"))
        .subtitle(crate::core::gettext("Fade gradients and ring distribution"))
        .build();
    op_expander.add_prefix(&gtk4::Image::from_icon_name("blend-mode-symbolic"));

    let op_fade_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Fade / Step (%)"))
        .build();
    let op_fade_spin = gtk4::SpinButton::with_range(0.0, 100.0, 5.0);
    op_fade_spin.set_value(params_rc.borrow().opacity_delta_per_row_pct as f64);
    op_fade_spin.add_css_class("numeric");
    op_fade_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        op_fade_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().opacity_delta_per_row_pct = sp.value() as f32;
        });
    }
    op_fade_row.add_suffix(&op_fade_spin);
    op_expander.add_row(&op_fade_row);

    let radial_rad_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Radial Radius (px)"))
        .subtitle(crate::core::gettext("0 px = auto-calculate based on shape size"))
        .build();
    let radial_rad_spin = gtk4::SpinButton::with_range(0.0, 5000.0, 10.0);
    radial_rad_spin.set_value(params_rc.borrow().radial_radius as f64);
    radial_rad_spin.add_css_class("numeric");
    radial_rad_spin.set_valign(gtk4::Align::Center);
    {
        let p_rc = params_rc.clone();
        radial_rad_spin.connect_value_changed(move |sp| {
            p_rc.borrow_mut().radial_radius = sp.value() as f32;
        });
    }
    radial_rad_row.add_suffix(&radial_rad_spin);
    op_expander.add_row(&radial_rad_row);

    prefs_group.add(&op_expander);
    card.append(&prefs_group);

    // 5. Action Buttons (Create, Clear, Unlink)
    let action_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(10)
        .build();

    let btn_create_tiles = gtk4::Button::builder()
        .label(crate::core::gettext("Create Tiled Clones"))
        .icon_name("view-grid-symbolic")
        .css_classes(["suggested-action", "pill-button"])
        .sensitive(has_selection)
        .tooltip_text(crate::core::gettext("Generate tiled clone matrix on canvas"))
        .build();
    {
        let can = canvas.clone();
        let p_rc = params_rc.clone();
        btn_create_tiles.connect_clicked(move |_| {
            let p = p_rc.borrow().clone();
            can.create_tiled_clones_for_selection(&p);
        });
    }
    action_box.append(&btn_create_tiles);

    let sec_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .build();

    let btn_clear_tiles = gtk4::Button::builder()
        .label(crate::core::gettext("Clear Clones"))
        .icon_name("user-trash-symbolic")
        .css_classes(["flat", "pill-button", "destructive-action"])
        .sensitive(has_selection)
        .tooltip_text(crate::core::gettext("Remove all clone instances for this master"))
        .build();
    {
        let can = canvas.clone();
        btn_clear_tiles.connect_clicked(move |_| {
            if let Some(mid) = can.first_selected_id() {
                can.delete_clones_for_master(mid);
            }
        });
    }
    sec_row.append(&btn_clear_tiles);

    let btn_unlink_tiles = gtk4::Button::builder()
        .label(crate::core::gettext("Unlink All"))
        .icon_name("edit-cut-symbolic")
        .css_classes(["flat", "pill-button"])
        .sensitive(has_selection)
        .tooltip_text(crate::core::gettext("Convert clones to independent vector paths"))
        .build();
    {
        let can = canvas.clone();
        btn_unlink_tiles.connect_clicked(move |_| {
            if let Some(mid) = can.first_selected_id() {
                can.unlink_all_clones_for_master(mid);
            }
        });
    }
    sec_row.append(&btn_unlink_tiles);
    action_box.append(&sec_row);

    card.append(&action_box);

    // Reset action
    {
        let p_rc = params_rc.clone();
        let sym_dd = sym_dropdown.clone();
        let r_sp = rows_spin.clone();
        let c_sp = cols_spin.clone();
        let sx_sp = shift_x_spin.clone();
        let sy_sp = shift_y_spin.clone();
        let rs_sp = rand_shift_spin.clone();
        let scx_sp = scale_x_spin.clone();
        let scy_sp = scale_y_spin.clone();
        let rsc_sp = rand_scale_spin.clone();
        let rc_sp = rot_col_spin.clone();
        let rr_sp = rot_row_spin.clone();
        let rrot_sp = rand_rot_spin.clone();
        let op_sp = op_fade_spin.clone();
        let rad_sp = radial_rad_spin.clone();

        reset_btn.connect_clicked(move |_| {
            *p_rc.borrow_mut() = TiledCloneParams::default();
            sym_dd.set_selected(0);
            r_sp.set_value(3.0);
            c_sp.set_value(3.0);
            sx_sp.set_value(100.0);
            sy_sp.set_value(100.0);
            rs_sp.set_value(0.0);
            scx_sp.set_value(0.0);
            scy_sp.set_value(0.0);
            rsc_sp.set_value(0.0);
            rc_sp.set_value(0.0);
            rr_sp.set_value(0.0);
            rrot_sp.set_value(0.0);
            op_sp.set_value(0.0);
            rad_sp.set_value(0.0);
        });
    }

    card
}
