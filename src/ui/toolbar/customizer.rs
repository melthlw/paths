use adw::prelude::*;
use gtk4::gdk;
use gtk4::prelude::*;
use libadwaita as adw;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use super::items::{generate_default_items, rebuild_toolbar_items, SubToolItem, ToolItem};
use crate::plugins::manifest::BarPosition;
use crate::ui::canvas::CanvasWidget;

pub fn show_customize_toolbar_dialog_standalone(
    parent: &impl IsA<gtk4::Widget>,
    canvas: CanvasWidget,
) {
    let dummy_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let state_rc = canvas.state();
    let default_groups = state_rc.borrow().plugin_manager.toolbar_groups();
    let initial_items = generate_default_items(&default_groups);
    let items_model = Rc::new(RefCell::new(initial_items));
    let is_updating = Rc::new(Cell::new(false));
    let buttons = Rc::new(RefCell::new(std::collections::HashMap::new()));
    let tool_icon_setters = Rc::new(RefCell::new(std::collections::HashMap::new()));
    let popovers = Rc::new(RefCell::new(Vec::new()));
    let position = Rc::new(Cell::new(BarPosition::Bottom));

    show_customize_toolbar_dialog(
        parent,
        &dummy_box,
        &items_model,
        &canvas,
        &is_updating,
        &buttons,
        &tool_icon_setters,
        &popovers,
        &position,
    );
}

/// Show a dedicated Libadwaita modal window to customize visible tools, reorder them, and create/manage groups
pub fn show_customize_toolbar_dialog(
    parent: &impl IsA<gtk4::Widget>,
    items_box: &gtk4::Box,
    items_model: &Rc<RefCell<Vec<ToolItem>>>,
    canvas: &CanvasWidget,
    is_updating: &Rc<Cell<bool>>,
    buttons: &Rc<RefCell<HashMap<&'static str, gtk4::ToggleButton>>>,
    tool_icon_setters: &Rc<RefCell<HashMap<&'static str, Box<dyn Fn()>>>>,
    popovers: &Rc<RefCell<Vec<gtk4::Popover>>>,
    position: &Rc<Cell<BarPosition>>,
) {
    let window = adw::Window::builder()
        .title(&crate::core::gettext("Customize Toolbar"))
        .modal(true)
        .default_width(480)
        .default_height(600)
        .build();

    if let Some(root) = parent.root() {
        if let Some(win) = root.downcast_ref::<gtk4::Window>() {
            window.set_transient_for(Some(win));
        }
    }

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::builder().show_title(true).build();

    let state_rc = canvas.state();
    let default_groups = state_rc.borrow().plugin_manager.toolbar_groups();
    let initial_items = generate_default_items(&default_groups);

    // Reset button in HeaderBar
    let reset_btn = gtk4::Button::builder()
        .tooltip_text(&crate::core::gettext("Restore Factory Default"))
        .icon_name("view-refresh-symbolic")
        .css_classes(["flat"])
        .build();
    header.pack_end(&reset_btn);

    // Add New Group / Folder button in HeaderBar
    let add_group_btn = gtk4::Button::builder()
        .tooltip_text(&crate::core::gettext("Create Tool Folder"))
        .icon_name("folder-new-symbolic")
        .css_classes(["flat"])
        .build();
    header.pack_start(&add_group_btn);

    toolbar_view.add_top_bar(&header);

    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(16)
        .margin_top(16)
        .margin_bottom(24)
        .margin_start(16)
        .margin_end(16)
        .build();

    let list_container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .build();
    content_box.append(&list_container);

    let clamp = adw::Clamp::builder()
        .maximum_size(460)
        .child(&content_box)
        .build();

    let scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .child(&clamp)
        .build();

    toolbar_view.set_content(Some(&scroll));
    window.set_content(Some(&toolbar_view));

    let list_container_rc = Rc::new(list_container);
    let flatten_mode = Rc::new(Cell::new(false));
    let rebuild_fn: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

    let items_box_clone = items_box.clone();
    let canvas_clone = canvas.clone();
    let is_updating_clone = is_updating.clone();
    let buttons_clone = buttons.clone();
    let setters_clone = tool_icon_setters.clone();
    let popovers_clone = popovers.clone();
    let position_clone = position.clone();
    let flatten_mode_clone = flatten_mode.clone();

    let rebuild_fn_impl = {
        let list_container = list_container_rc.clone();
        let items_model = items_model.clone();
        let items_box = items_box_clone.clone();
        let canvas = canvas_clone.clone();
        let is_updating = is_updating_clone.clone();
        let buttons = buttons_clone.clone();
        let setters = setters_clone.clone();
        let popovers = popovers_clone.clone();
        let position = position_clone.clone();
        let flatten_mode = flatten_mode_clone.clone();
        let rebuild_fn_holder = rebuild_fn.clone();

        Rc::new(move || {
            // 1. Sync live floating toolbar
            let is_flat = flatten_mode.get();
            let effective_items: Vec<ToolItem> = if is_flat {
                let mut flat = Vec::new();
                for it in items_model.borrow().iter() {
                    if it.sub_tools.is_empty() {
                        flat.push(it.clone());
                    } else {
                        for sub in &it.sub_tools {
                            flat.push(ToolItem {
                                id: sub.id,
                                name: sub.name.clone(),
                                shortcut: sub.shortcut.clone(),
                                icon_resource: sub.icon_resource,
                                icon_name: sub.icon_name,
                                tooltip: sub.tooltip,
                                visible: sub.visible && it.visible,
                                sub_tools: Vec::new(),
                            });
                        }
                    }
                }
                flat
            } else {
                items_model.borrow().clone()
            };

            rebuild_toolbar_items(
                &items_box,
                &effective_items,
                &canvas,
                &is_updating,
                &buttons,
                &setters,
                &popovers,
                position.get(),
            );

            // 2. Clear dialog UI
            while let Some(child) = list_container.first_child() {
                list_container.remove(&child);
            }

            // Top Options Card: Flat Mode Switch
            let mode_group = adw::PreferencesGroup::builder().build();
            let flat_row = adw::ActionRow::builder()
                .title(&crate::core::gettext("Flat Mode (No Folders)"))
                .subtitle(&crate::core::gettext(
                    "Display all tools loose on the toolbar without submenus",
                ))
                .build();
            let flat_sw = gtk4::Switch::builder()
                .active(flatten_mode.get())
                .valign(gtk4::Align::Center)
                .build();
            let flatten_mode_sw = flatten_mode.clone();
            let rebuild_flat_sw = rebuild_fn_holder.clone();
            flat_sw.connect_active_notify(move |s| {
                flatten_mode_sw.set(s.is_active());
                if let Some(ref r) = *rebuild_flat_sw.borrow() {
                    r();
                }
            });
            flat_row.add_suffix(&flat_sw);
            mode_group.add(&flat_row);
            list_container.append(&mode_group);

            let pref_group = adw::PreferencesGroup::builder()
                .title(&crate::core::gettext("Tools"))
                .description(&crate::core::gettext(
                    "Drag to reorder, manage folders and tools",
                ))
                .build();

            let items = items_model.borrow().clone();
            let total_len = items.len();

            for (idx, item) in items.iter().enumerate() {
                let is_first = idx == 0;
                let is_last = idx + 1 >= total_len;

                // Linked Up / Down Reorder buttons
                let reorder_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .css_classes(["linked"])
                    .valign(gtk4::Align::Center)
                    .build();

                let btn_up = gtk4::Button::builder()
                    .icon_name("go-up-symbolic")
                    .tooltip_text(&crate::core::gettext("Move up / before"))
                    .css_classes(["flat"])
                    .sensitive(!is_first)
                    .build();

                let btn_down = gtk4::Button::builder()
                    .icon_name("go-down-symbolic")
                    .tooltip_text(&crate::core::gettext("Move down / after"))
                    .css_classes(["flat"])
                    .sensitive(!is_last)
                    .build();

                reorder_box.append(&btn_up);
                reorder_box.append(&btn_down);

                let items_model_up = items_model.clone();
                let rebuild_up = rebuild_fn_holder.clone();
                btn_up.connect_clicked(move |_| {
                    if idx > 0 {
                        items_model_up.borrow_mut().swap(idx, idx - 1);
                        if let Some(ref r) = *rebuild_up.borrow() {
                            r();
                        }
                    }
                });

                let items_model_down = items_model.clone();
                let rebuild_down = rebuild_fn_holder.clone();
                btn_down.connect_clicked(move |_| {
                    let mut model = items_model_down.borrow_mut();
                    if idx + 1 < model.len() {
                        model.swap(idx, idx + 1);
                        drop(model);
                        if let Some(ref r) = *rebuild_down.borrow() {
                            r();
                        }
                    }
                });

                // Main visibility switch
                let switch = gtk4::Switch::builder()
                    .active(item.visible)
                    .valign(gtk4::Align::Center)
                    .build();
                let items_model_sw = items_model.clone();
                let rebuild_sw = rebuild_fn_holder.clone();
                switch.connect_active_notify(move |s| {
                    if let Some(target) = items_model_sw.borrow_mut().get_mut(idx) {
                        target.visible = s.is_active();
                    }
                    if let Some(ref r) = *rebuild_sw.borrow() {
                        r();
                    }
                });

                // Drag Handle
                let drag_handle = gtk4::Image::from_icon_name("list-drag-handle-symbolic");
                drag_handle.set_opacity(0.5);
                drag_handle.set_cursor_from_name(Some("grab"));
                let drag_source = gtk4::DragSource::new();
                drag_source.set_actions(gdk::DragAction::MOVE);
                let idx_str = format!("{}", idx);
                let content = gdk::ContentProvider::for_value(&idx_str.to_value());
                drag_source.set_content(Some(&content));
                drag_handle.add_controller(drag_source);

                let drop_target =
                    gtk4::DropTarget::new(String::static_type(), gdk::DragAction::MOVE);
                let items_model_drop = items_model.clone();
                let rebuild_drop = rebuild_fn_holder.clone();
                let target_idx = idx;
                drop_target.connect_drop(move |_, val, _, _| {
                    if let Ok(str_val) = val.get::<String>() {
                        if let Ok(from_idx) = str_val.parse::<usize>() {
                            if from_idx != target_idx {
                                let mut model = items_model_drop.borrow_mut();
                                if from_idx < model.len() && target_idx < model.len() {
                                    let item = model.remove(from_idx);
                                    model.insert(target_idx, item);
                                    drop(model);
                                    if let Some(ref r) = *rebuild_drop.borrow() {
                                        r();
                                    }
                                    return true;
                                }
                            }
                        }
                    }
                    false
                });

                if item.sub_tools.is_empty() {
                    // STANDALONE TOOL ROW
                    let row = adw::ActionRow::builder()
                        .title(glib::markup_escape_text(&item.name))
                        .build();
                    row.add_controller(drop_target);
                    row.add_prefix(&drag_handle);

                    let row_icon = if let Some(r) = item.icon_resource {
                        crate::ui::icons::make_symbolic_image(r, 20)
                    } else {
                        crate::ui::icons::make_symbolic_image(item.icon_name, 20)
                    };
                    row.add_prefix(&row_icon);

                    if let Some(ref sc) = item.shortcut {
                        let badge = gtk4::Label::builder()
                            .label(sc)
                            .css_classes(["dim-label", "caption", "badge"])
                            .valign(gtk4::Align::Center)
                            .build();
                        row.add_suffix(&badge);
                    }

                    // Button: "Agrupar com..."
                    let btn_group_with = gtk4::Button::builder()
                        .icon_name("folder-new-symbolic")
                        .tooltip_text(&crate::core::gettext("Add this tool to a group"))
                        .css_classes(["flat"])
                        .valign(gtk4::Align::Center)
                        .build();

                    let pop = gtk4::Popover::builder().has_arrow(true).build();
                    pop.set_parent(&btn_group_with);
                    {
                        let pop_c = pop.clone();
                        btn_group_with.connect_destroy(move |_| {
                            if pop_c.parent().is_some() {
                                pop_c.unparent();
                            }
                        });
                    }

                    let items_model_gw = items_model.clone();
                    let rebuild_gw = rebuild_fn_holder.clone();
                    let pop_open = pop.clone();
                    btn_group_with.connect_clicked(move |_| {
                        let pop = pop_open.clone();

                        let pop_box = gtk4::Box::builder()
                            .orientation(gtk4::Orientation::Vertical)
                            .spacing(6)
                            .margin_top(8)
                            .margin_bottom(8)
                            .margin_start(8)
                            .margin_end(8)
                            .build();

                        let lbl = gtk4::Label::builder()
                            .label(&crate::core::gettext("Move to group:"))
                            .css_classes(["heading"])
                            .halign(gtk4::Align::Start)
                            .build();
                        pop_box.append(&lbl);

                        let current_groups: Vec<(usize, String)> = items_model_gw
                            .borrow()
                            .iter()
                            .enumerate()
                            .filter(|(_, it)| !it.sub_tools.is_empty())
                            .map(|(i, it)| (i, it.name.clone()))
                            .collect();

                        if current_groups.is_empty() {
                            let empty_lbl = gtk4::Label::builder()
                                .label(&crate::core::gettext(
                                    "No existing groups.\nUse the '+' button at the top to create a group.",
                                ))
                                .css_classes(["dim-label", "caption"])
                                .build();
                            pop_box.append(&empty_lbl);
                        } else {
                            for (g_idx, g_name) in current_groups {
                                let g_btn = gtk4::Button::builder()
                                    .label(&g_name)
                                    .css_classes(["flat"])
                                    .halign(gtk4::Align::Fill)
                                    .build();

                                let items_model_mv = items_model_gw.clone();
                                let rebuild_mv = rebuild_gw.clone();
                                let pop_close = pop.clone();
                                g_btn.connect_clicked(move |_| {
                                    pop_close.popdown();
                                    let mut model = items_model_mv.borrow_mut();
                                    if idx < model.len() {
                                        let standalone = model.remove(idx);
                                        let target_group_idx =
                                            if g_idx > idx { g_idx - 1 } else { g_idx };
                                        if let Some(target_group) = model.get_mut(target_group_idx)
                                        {
                                            target_group.sub_tools.push(SubToolItem {
                                                id: standalone.id,
                                                name: standalone.name,
                                                shortcut: standalone.shortcut,
                                                icon_resource: standalone.icon_resource,
                                                icon_name: standalone.icon_name,
                                                tooltip: standalone.tooltip,
                                                visible: standalone.visible,
                                            });
                                        }
                                    }
                                    drop(model);
                                    if let Some(ref r) = *rebuild_mv.borrow() {
                                        r();
                                    }
                                });
                                pop_box.append(&g_btn);
                            }
                        }

                        pop.set_child(Some(&pop_box));
                        pop.popup();
                    });

                    row.add_suffix(&btn_group_with);
                    row.add_suffix(&reorder_box);
                    row.add_suffix(&switch);
                    pref_group.add(&row);
                } else {
                    // TOOL GROUP (EXPANDER ROW)
                    let exp_row = adw::ExpanderRow::builder()
                        .title(glib::markup_escape_text(&item.name))
                        .subtitle(&format!("Grupo com {} ferramentas", item.sub_tools.len()))
                        .show_enable_switch(false)
                        .build();
                    exp_row.add_controller(drop_target);
                    exp_row.add_prefix(&drag_handle);

                    let row_icon = if let Some(r) = item.icon_resource {
                        crate::ui::icons::make_symbolic_image(r, 20)
                    } else {
                        crate::ui::icons::make_symbolic_image(item.icon_name, 20)
                    };
                    exp_row.add_prefix(&row_icon);

                    // Button: "Desagrupar Tudo"
                    let btn_ungroup_all = gtk4::Button::builder()
                        .icon_name("object-ungroup-symbolic")
                        .tooltip_text(&crate::core::gettext("Ungroup and separate all tools"))
                        .css_classes(["flat"])
                        .valign(gtk4::Align::Center)
                        .build();

                    let items_model_uga = items_model.clone();
                    let rebuild_uga = rebuild_fn_holder.clone();
                    btn_ungroup_all.connect_clicked(move |_| {
                        let mut model = items_model_uga.borrow_mut();
                        if idx < model.len() {
                            let group = model.remove(idx);
                            for (offset, sub) in group.sub_tools.into_iter().enumerate() {
                                model.insert(
                                    idx + offset,
                                    ToolItem {
                                        id: sub.id,
                                        name: sub.name,
                                        shortcut: sub.shortcut,
                                        icon_resource: sub.icon_resource,
                                        icon_name: sub.icon_name,
                                        tooltip: sub.tooltip,
                                        visible: sub.visible,
                                        sub_tools: Vec::new(),
                                    },
                                );
                            }
                        }
                        drop(model);
                        if let Some(ref r) = *rebuild_uga.borrow() {
                            r();
                        }
                    });

                    exp_row.add_suffix(&btn_ungroup_all);
                    exp_row.add_suffix(&reorder_box);
                    exp_row.add_suffix(&switch);

                    // Render Subtools inside Group
                    for (sub_idx, sub) in item.sub_tools.iter().enumerate() {
                        let sub_row = adw::ActionRow::builder()
                            .title(glib::markup_escape_text(&sub.name))
                            .build();

                        let sub_icon = if let Some(r) = sub.icon_resource {
                            crate::ui::icons::make_symbolic_image(r, 18)
                        } else {
                            crate::ui::icons::make_symbolic_image(sub.icon_name, 18)
                        };
                        sub_row.add_prefix(&sub_icon);

                        if let Some(ref sc) = sub.shortcut {
                            let badge = gtk4::Label::builder()
                                .label(sc)
                                .css_classes(["dim-label", "caption", "badge"])
                                .valign(gtk4::Align::Center)
                                .build();
                            sub_row.add_suffix(&badge);
                        }

                        // Button: "Tirar do Grupo"
                        let btn_remove_sub = gtk4::Button::builder()
                            .icon_name("list-remove-symbolic")
                            .tooltip_text(&crate::core::gettext(
                                "Remove from group (Move outside)",
                            ))
                            .css_classes(["flat"])
                            .valign(gtk4::Align::Center)
                            .build();

                        let items_model_rm = items_model.clone();
                        let rebuild_rm = rebuild_fn_holder.clone();
                        btn_remove_sub.connect_clicked(move |_| {
                            let mut model = items_model_rm.borrow_mut();
                            if let Some(group_item) = model.get_mut(idx) {
                                if sub_idx < group_item.sub_tools.len() {
                                    let sub_extracted = group_item.sub_tools.remove(sub_idx);
                                    let standalone = ToolItem {
                                        id: sub_extracted.id,
                                        name: sub_extracted.name,
                                        shortcut: sub_extracted.shortcut,
                                        icon_resource: sub_extracted.icon_resource,
                                        icon_name: sub_extracted.icon_name,
                                        tooltip: sub_extracted.tooltip,
                                        visible: sub_extracted.visible,
                                        sub_tools: Vec::new(),
                                    };

                                    if group_item.sub_tools.len() == 1 {
                                        let last = group_item.sub_tools.remove(0);
                                        *group_item = ToolItem {
                                            id: last.id,
                                            name: last.name,
                                            shortcut: last.shortcut,
                                            icon_resource: last.icon_resource,
                                            icon_name: last.icon_name,
                                            tooltip: last.tooltip,
                                            visible: last.visible,
                                            sub_tools: Vec::new(),
                                        };
                                        model.insert(idx + 1, standalone);
                                    } else {
                                        model.insert(idx + 1, standalone);
                                    }
                                }
                            }
                            drop(model);
                            if let Some(ref r) = *rebuild_rm.borrow() {
                                r();
                            }
                        });

                        sub_row.add_suffix(&btn_remove_sub);

                        let sub_switch = gtk4::Switch::builder()
                            .active(sub.visible)
                            .valign(gtk4::Align::Center)
                            .build();
                        let items_model_sub_sw = items_model.clone();
                        let rebuild_sub_sw = rebuild_fn_holder.clone();
                        sub_switch.connect_active_notify(move |s| {
                            let mut model = items_model_sub_sw.borrow_mut();
                            if let Some(group_item) = model.get_mut(idx) {
                                if let Some(sub_target) = group_item.sub_tools.get_mut(sub_idx) {
                                    sub_target.visible = s.is_active();
                                }
                            }
                            drop(model);
                            if let Some(ref r) = *rebuild_sub_sw.borrow() {
                                r();
                            }
                        });
                        sub_row.add_suffix(&sub_switch);

                        exp_row.add_row(&sub_row);
                    }

                    pref_group.add(&exp_row);
                }
            }

            list_container.append(&pref_group);
        })
    };

    *rebuild_fn.borrow_mut() = Some(rebuild_fn_impl.clone());
    rebuild_fn_impl();

    // Reset button handler
    let initial_items_clone = initial_items.clone();
    let items_model_reset = items_model.clone();
    let rebuild_reset = rebuild_fn.clone();
    reset_btn.connect_clicked(move |_| {
        *items_model_reset.borrow_mut() = initial_items_clone.clone();
        if let Some(ref r) = *rebuild_reset.borrow() {
            r();
        }
    });

    // Add New Group Dialog
    let items_model_new_g = items_model.clone();
    let rebuild_new_g = rebuild_fn.clone();
    let win_ref = window.clone();
    add_group_btn.connect_clicked(move |_| {
        let dialog = adw::Window::builder()
            .title(&crate::core::gettext("Create New Tool Group"))
            .modal(true)
            .transient_for(&win_ref)
            .default_width(380)
            .default_height(420)
            .build();

        let tv = adw::ToolbarView::new();
        let hb = adw::HeaderBar::builder().show_title(true).build();
        tv.add_top_bar(&hb);

        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .margin_top(16)
            .margin_bottom(16)
            .margin_start(16)
            .margin_end(16)
            .build();

        let name_entry = adw::EntryRow::builder()
            .title(&crate::core::gettext("Group Name"))
            .text(&crate::core::gettext("New Group"))
            .build();
        content.append(&name_entry);

        let sel_group = adw::PreferencesGroup::builder()
            .title(&crate::core::gettext("Select Tools to Group"))
            .build();

        let standalone_items: Vec<(usize, ToolItem)> = items_model_new_g
            .borrow()
            .iter()
            .enumerate()
            .filter(|(_, it)| it.sub_tools.is_empty())
            .map(|(i, it)| (i, it.clone()))
            .collect();

        let checked_indices = Rc::new(RefCell::new(Vec::<usize>::new()));

        for (orig_idx, tool) in &standalone_items {
            let chk_row = adw::ActionRow::builder()
                .title(glib::markup_escape_text(&tool.name))
                .build();

            let row_icon = if let Some(r) = tool.icon_resource {
                crate::ui::icons::make_symbolic_image(r, 20)
            } else {
                crate::ui::icons::make_symbolic_image(tool.icon_name, 20)
            };
            chk_row.add_prefix(&row_icon);

            let check = gtk4::CheckButton::builder()
                .valign(gtk4::Align::Center)
                .build();

            let checked_list = checked_indices.clone();
            let idx_val = *orig_idx;
            check.connect_toggled(move |c| {
                let mut list = checked_list.borrow_mut();
                if c.is_active() {
                    list.push(idx_val);
                } else {
                    list.retain(|&x| x != idx_val);
                }
            });
            chk_row.add_suffix(&check);

            sel_group.add(&chk_row);
        }

        content.append(&sel_group);

        let create_btn = gtk4::Button::builder()
            .label(&crate::core::gettext("Create Group"))
            .css_classes(["suggested-action", "pill-btn"])
            .halign(gtk4::Align::Fill)
            .margin_top(8)
            .build();

        let items_model_create = items_model_new_g.clone();
        let rebuild_create = rebuild_new_g.clone();
        let diag_close = dialog.clone();
        let checked_on_create = checked_indices.clone();
        let entry_on_create = name_entry.clone();

        create_btn.connect_clicked(move |_| {
            let mut selected = checked_on_create.borrow().clone();
            if selected.len() >= 2 {
                selected.sort_unstable();
                selected.reverse(); // Remove from back to keep indices valid

                let group_name = entry_on_create.text().to_string();
                let mut sub_items = Vec::new();
                let mut primary_meta: Option<(Option<&'static str>, &'static str, &'static str)> =
                    None;

                let mut model = items_model_create.borrow_mut();
                for s_idx in selected {
                    if s_idx < model.len() {
                        let extracted = model.remove(s_idx);
                        if primary_meta.is_none() {
                            primary_meta = Some((
                                extracted.icon_resource,
                                extracted.icon_name,
                                extracted.tooltip,
                            ));
                        }
                        sub_items.push(SubToolItem {
                            id: extracted.id,
                            name: extracted.name,
                            shortcut: extracted.shortcut,
                            icon_resource: extracted.icon_resource,
                            icon_name: extracted.icon_name,
                            tooltip: extracted.tooltip,
                            visible: extracted.visible,
                        });
                    }
                }

                sub_items.reverse();

                let (ic_res, ic_name, tt) =
                    primary_meta.unwrap_or((None, "applications-system-symbolic", "Tool Group"));

                model.push(ToolItem {
                    id: sub_items[0].id,
                    name: if group_name.trim().is_empty() {
                        crate::core::gettext("New Group")
                    } else {
                        group_name
                    },
                    shortcut: sub_items[0].shortcut.clone(),
                    icon_resource: ic_res,
                    icon_name: ic_name,
                    tooltip: tt,
                    visible: true,
                    sub_tools: sub_items,
                });

                drop(model);
                if let Some(ref r) = *rebuild_create.borrow() {
                    r();
                }
                diag_close.close();
            }
        });

        content.append(&create_btn);

        let sc = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&content)
            .build();
        tv.set_content(Some(&sc));
        dialog.set_content(Some(&tv));
        dialog.present();
    });

    window.present();
}
