use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::rc::Rc;

use crate::core::element::Element;
use crate::ui::canvas::CanvasWidget;

pub fn build_clones_section(canvas: &CanvasWidget) -> (gtk4::Widget, Rc<dyn Fn()>) {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(10)
        .margin_top(6)
        .margin_bottom(12)
        .margin_start(10)
        .margin_end(10)
        .build();

    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(10)
        .build();

    container.append(&content_box);

    let canvas_c = canvas.clone();
    let content_box_c = content_box.clone();

    let update_fn = Rc::new(move || {
        crate::ui::inspector::appearance::clear_box(&content_box_c);

        let sel_count = canvas_c.selection_count();
        let has_clones = canvas_c.has_clones_selected();
        let has_masters = canvas_c.has_masters_selected();

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

                let icon = gtk4::Image::from_icon_name("starred-symbolic");
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
                    .tooltip_text(crate::core::gettext("Create a new linked clone instance (Alt+D)"))
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
                    .icon_name("object-select-symbolic")
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
                    .icon_name("edit-undo-symbolic")
                    .css_classes(["flat", "pill-button", "destructive-action"])
                    .tooltip_text(crate::core::gettext("Convert all instances to independent objects"))
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

                // List of Clones
                let instances_group = adw::PreferencesGroup::builder()
                    .title(crate::core::gettext("Linked Instances"))
                    .description(crate::core::gettext(
                        "Clones share the master shape and styling with independent spatial positioning",
                    ))
                    .build();

                for (idx, clone) in clones.iter().enumerate() {
                    let cid = clone.id;
                    let row = adw::ActionRow::builder()
                        .title(clone.name.as_deref().unwrap_or(&format!("Clone #{}", idx + 1)))
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

                    let icon = gtk4::Image::from_icon_name("object-select-symbolic");
                    icon.set_pixel_size(16);
                    row.add_prefix(&icon);

                    let btn_select = gtk4::Button::builder()
                        .icon_name("find-location-symbolic")
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
                let master_id = clone.source_id;
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

                let icon = gtk4::Image::from_icon_name("object-select-symbolic");
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

                let master_name = master.as_ref().map(|m| m.name()).unwrap_or_else(|| "Unknown".to_string());
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
                    .icon_name("go-jump-symbolic")
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

                // Sibling instances
                let siblings = canvas_c.get_clones_for_master(master_id);
                if siblings.len() > 1 {
                    let sib_group = adw::PreferencesGroup::builder()
                        .title(crate::core::gettext("Sibling Instances"))
                        .build();

                    for (idx, sib) in siblings.iter().enumerate() {
                        if sib.id == clone_id {
                            continue;
                        }
                        let s_id = sib.id;
                        let row = adw::ActionRow::builder()
                            .title(sib.name.as_deref().unwrap_or(&format!("Clone #{}", idx + 1)))
                            .subtitle(format!("X: {:.1} px  Y: {:.1} px", sib.offset.x, sib.offset.y))
                            .activatable(true)
                            .build();

                        let btn_sel = gtk4::Button::builder()
                            .icon_name("find-location-symbolic")
                            .css_classes(["flat", "circular"])
                            .valign(gtk4::Align::Center)
                            .build();
                        {
                            let can = canvas_c.clone();
                            btn_sel.connect_clicked(move |_| {
                                can.select_element_by_id(s_id);
                            });
                        }
                        row.add_suffix(&btn_sel);

                        {
                            let can = canvas_c.clone();
                            row.connect_activated(move |_| {
                                can.select_element_by_id(s_id);
                            });
                        }

                        sib_group.add(&row);
                    }
                    content_box_c.append(&sib_group);
                }
                return;
            }
        }

        // 3. DOCUMENT OVERVIEW MODE (No or multiple elements selected)
        let all_clones = canvas_c.get_all_clone_relationships();

        if all_clones.is_empty() {
            // Empty State
            let empty_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(12)
                .valign(gtk4::Align::Center)
                .halign(gtk4::Align::Center)
                .vexpand(true)
                .margin_top(40)
                .margin_bottom(40)
                .margin_start(16)
                .margin_end(16)
                .build();

            let empty_icon = gtk4::Image::from_icon_name("object-select-symbolic");
            empty_icon.set_pixel_size(48);
            empty_icon.set_opacity(0.35);

            let empty_lbl = gtk4::Label::builder()
                .label(crate::core::gettext("No Linked Clones"))
                .css_classes(["heading"])
                .build();

            let empty_sub = gtk4::Label::builder()
                .label(crate::core::gettext(
                    "Select any object and press Alt+D to create a linked clone.\nClones share paths and styling while allowing independent position, scale, and rotation.",
                ))
                .justify(gtk4::Justification::Center)
                .wrap(true)
                .css_classes(["dim-label", "caption"])
                .build();

            empty_box.append(&empty_icon);
            empty_box.append(&empty_lbl);
            empty_box.append(&empty_sub);
            content_box_c.append(&empty_box);
        } else {
            // Overview of all master/clone relationships
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
                    .title(&master_name)
                    .subtitle(format!(
                        "{} {}",
                        clones.len(),
                        if clones.len() == 1 {
                            crate::core::gettext("linked instance")
                        } else {
                            crate::core::gettext("linked instances")
                        }
                    ))
                    .expanded(true)
                    .build();

                let m_id = master_id;
                let btn_select_m = gtk4::Button::builder()
                    .icon_name("starred-symbolic")
                    .css_classes(["flat", "circular"])
                    .tooltip_text(crate::core::gettext("Select Master Object"))
                    .valign(gtk4::Align::Center)
                    .build();
                {
                    let can = canvas_c.clone();
                    btn_select_m.connect_clicked(move |_| {
                        can.select_element_by_id(m_id);
                    });
                }
                expander.add_suffix(&btn_select_m);

                for (idx, clone) in clones.iter().enumerate() {
                    let cid = clone.id;
                    let row = adw::ActionRow::builder()
                        .title(clone.name.as_deref().unwrap_or(&format!("Clone #{}", idx + 1)))
                        .subtitle(format!(
                            "X: {:.1} px  Y: {:.1} px • Scale: {:.0}%",
                            clone.offset.x,
                            clone.offset.y,
                            clone.scale.x * 100.0
                        ))
                        .activatable(true)
                        .build();

                    let btn_select_c = gtk4::Button::builder()
                        .icon_name("find-location-symbolic")
                        .css_classes(["flat", "circular"])
                        .tooltip_text(crate::core::gettext("Focus & Select on Canvas"))
                        .valign(gtk4::Align::Center)
                        .build();
                    {
                        let can = canvas_c.clone();
                        btn_select_c.connect_clicked(move |_| {
                            can.select_element_by_id(cid);
                        });
                    }
                    row.add_suffix(&btn_select_c);

                    let btn_unlink = gtk4::Button::builder()
                        .icon_name("edit-cut-symbolic")
                        .css_classes(["flat", "circular"])
                        .tooltip_text(crate::core::gettext("Unlink this clone"))
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

                    expander.add_row(&row);
                }

                overview_group.add(&expander);
            }

            content_box_c.append(&overview_group);
        }
    });

    // Run initial population
    update_fn();

    (container.upcast(), update_fn)
}
