use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::catalog;
use super::TabLocation;
use crate::ui::canvas::CanvasWidget;

pub fn render_dock_sections(
    tab_info: &[(String, Option<&'static str>, &'static str); 5],
    tab_locations: &Rc<RefCell<[TabLocation; 5]>>,
    active_section_tabs: &Rc<RefCell<[usize; 5]>>,
    tab_order: &Rc<RefCell<Vec<usize>>>,
    tab_widgets: &[gtk4::Widget; 5],
    sections_container: &gtk4::Box,
    canvas: &CanvasWidget,
    refresh_fn: Rc<dyn Fn()>,
) {
    let locs = *tab_locations.borrow();

    // 1. Unparent all tab widgets
    for w in tab_widgets {
        w.unparent();
    }

    // 2. Clear sections container with full popover unparenting
    crate::ui::inspector::appearance::clear_box(sections_container);

    // 3. Collect active docked section indices in sorted order
    let mut active_sections: Vec<usize> = Vec::new();
    for i in 0..5 {
        if let TabLocation::Docked(sec) = locs[i] {
            if !active_sections.contains(&sec) {
                active_sections.push(sec);
            }
        }
    }
    active_sections.sort_unstable();

    // Compact section IDs to 0..num_sections
    let mut remapped_locs = locs;
    for i in 0..5 {
        if let TabLocation::Docked(sec) = locs[i] {
            let new_sec = active_sections.iter().position(|&s| s == sec).unwrap_or(0);
            remapped_locs[i] = TabLocation::Docked(new_sec);
        }
    }
    if remapped_locs != locs {
        *tab_locations.borrow_mut() = remapped_locs;
    }

    let num_sections = active_sections.len();

    // If no sections are docked
    if num_sections == 0 {
        let has_floating = (0..5).any(|k| locs[k] == TabLocation::Floating);
        let empty_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .vexpand(true)
            .margin_start(16)
            .margin_end(16)
            .css_classes(["empty-tab-drop-zone"])
            .build();

        let empty_icon = gtk4::Image::from_icon_name(if has_floating {
            "sidebar-show-right-symbolic"
        } else {
            "view-grid-symbolic"
        });
        empty_icon.set_pixel_size(48);
        empty_icon.set_opacity(0.35);

        let title_text = if has_floating {
            crate::core::gettext("Tabs in Floating Windows")
        } else {
            crate::core::gettext("No Panels Open")
        };
        let empty_lbl = gtk4::Label::builder()
            .label(&title_text)
            .css_classes(["heading"])
            .build();

        let sub_text = if has_floating {
            crate::core::gettext("Drag a floating tab here\nto dock it back.")
        } else {
            crate::core::gettext(
                "Click '+' above to open panels like\nAppearance, Alignment, Transform or Export.",
            )
        };
        let empty_sub = gtk4::Label::builder()
            .label(&sub_text)
            .justify(gtk4::Justification::Center)
            .css_classes(["dim-label", "caption"])
            .build();

        let open_all_pill_btn = gtk4::Button::builder()
            .label(crate::core::gettext("Open All Panels"))
            .css_classes(["pill", "suggested-action"])
            .halign(gtk4::Align::Center)
            .margin_top(8)
            .build();
        let locs_all = tab_locations.clone();
        let re_all = refresh_fn.clone();
        open_all_pill_btn.connect_clicked(move |_| {
            *locs_all.borrow_mut() = [
                TabLocation::Docked(0),
                TabLocation::Docked(0),
                TabLocation::Docked(0),
                TabLocation::Docked(0),
                TabLocation::Docked(0),
            ];
            re_all();
        });

        empty_box.append(&empty_icon);
        empty_box.append(&empty_lbl);
        empty_box.append(&empty_sub);
        if !has_floating {
            empty_box.append(&open_all_pill_btn);
        }
        sections_container.append(&empty_box);
    } else {
        for sec_idx in 0..num_sections {
            let tabs_in_sec: Vec<usize> = tab_order
                .borrow()
                .iter()
                .cloned()
                .filter(|&i| remapped_locs[i] == TabLocation::Docked(sec_idx))
                .collect();

            if tabs_in_sec.is_empty() {
                continue;
            }

            let sec_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .css_classes(["studio-dock-section"])
                .build();

            let sec_tab_bar = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .css_classes(["studio-tab-bar"])
                .build();

            // Drop target on this specific section's tab bar (empty space)
            let drop_sec_bar =
                gtk4::DropTarget::new(glib::types::Type::STRING, gdk::DragAction::MOVE);
            let locs_drop_bar = tab_locations.clone();
            let tab_order_drop_bar = tab_order.clone();
            let active_sec_tabs_d = active_section_tabs.clone();
            let re_bar = refresh_fn.clone();
            let sec_bar_weak = sec_tab_bar.downgrade();

            drop_sec_bar.connect_enter(move |_, _, _| {
                if let Some(bar) = sec_bar_weak.upgrade() {
                    bar.add_css_class("hover-active");
                }
                gdk::DragAction::MOVE
            });
            let sec_bar_weak_l = sec_tab_bar.downgrade();
            drop_sec_bar.connect_leave(move |_| {
                if let Some(bar) = sec_bar_weak_l.upgrade() {
                    bar.remove_css_class("hover-active");
                }
            });

            drop_sec_bar.connect_drop(move |_, value, _, _| {
                if let Ok(s) = value.get::<String>() {
                    if let Some(idx_str) = s.strip_prefix("tab:") {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            if idx < 5 {
                                locs_drop_bar.borrow_mut()[idx] = TabLocation::Docked(sec_idx);

                                let mut order = tab_order_drop_bar.borrow().clone();
                                if let Some(pos) = order.iter().position(|&x| x == idx) {
                                    order.remove(pos);
                                }
                                order.push(idx);
                                *tab_order_drop_bar.borrow_mut() = order;

                                if sec_idx < active_sec_tabs_d.borrow().len() {
                                    active_sec_tabs_d.borrow_mut()[sec_idx] = idx;
                                }

                                re_bar();
                                return true;
                            }
                        }
                    }
                }
                false
            });

            sec_tab_bar.add_controller(drop_sec_bar);

            // Find active tab in this section
            let mut cur_active = if sec_idx < active_section_tabs.borrow().len() {
                active_section_tabs.borrow()[sec_idx]
            } else {
                tabs_in_sec[0]
            };
            if !tabs_in_sec.contains(&cur_active) {
                cur_active = tabs_in_sec[0];
                if sec_idx < active_section_tabs.borrow().len() {
                    active_section_tabs.borrow_mut()[sec_idx] = cur_active;
                }
            }

            // Build tab buttons
            for &tab_i in &tabs_in_sec {
                let (ref title, icon_res, icon_name) = tab_info[tab_i];
                let tab_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(0)
                    .css_classes(["studio-tab-item"])
                    .valign(gtk4::Align::Center)
                    .build();

                if tab_i == cur_active {
                    tab_box.add_css_class("active");
                }

                let btn = gtk4::Button::builder()
                    .css_classes(["flat", "studio-tab-btn"])
                    .focus_on_click(false)
                    .tooltip_text(&crate::i18n!(
                        "{} (Drag to reorder, split or float)",
                        title
                    ))
                    .build();

                let btn_content = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(6)
                    .margin_start(4)
                    .margin_end(2)
                    .valign(gtk4::Align::Center)
                    .build();
                let icon_img = if let Some(res) = icon_res {
                    crate::ui::icons::make_symbolic_image(res, 14)
                } else {
                    crate::ui::icons::make_symbolic_image(icon_name, 14)
                };
                let lbl = gtk4::Label::new(Some(title.as_str()));
                btn_content.append(&icon_img);
                btn_content.append(&lbl);
                btn.set_child(Some(&btn_content));

                // Close button directly on tab
                let close_tab_btn = gtk4::Button::builder()
                    .icon_name("window-close-symbolic")
                    .tooltip_text(crate::core::gettext("Close Panel"))
                    .css_classes(["flat", "circular", "tab-close-btn"])
                    .valign(gtk4::Align::Center)
                    .focus_on_click(false)
                    .build();
                let locs_close = tab_locations.clone();
                let re_close = refresh_fn.clone();
                close_tab_btn.connect_clicked(move |_| {
                    locs_close.borrow_mut()[tab_i] = TabLocation::Closed;
                    re_close();
                });

                tab_box.append(&btn);
                tab_box.append(&close_tab_btn);

                // Click tab button
                let act_sec_tabs_c = active_section_tabs.clone();
                let re_click = refresh_fn.clone();
                btn.connect_clicked(move |_| {
                    if sec_idx < act_sec_tabs_c.borrow().len() {
                        act_sec_tabs_c.borrow_mut()[sec_idx] = tab_i;
                    }
                    re_click();
                });

                // 1. Cross-window drag source with live widget paintable icon
                let drag_source = gtk4::DragSource::new();
                drag_source.set_actions(gdk::DragAction::MOVE);
                let tab_idx = tab_i;
                drag_source.connect_prepare(move |_, _, _| {
                    Some(gdk::ContentProvider::for_value(
                        &format!("tab:{}", tab_idx).to_value(),
                    ))
                });
                let paintable = gtk4::WidgetPaintable::new(Some(&tab_box));
                drag_source.set_icon(Some(&paintable), 20, 14);
                btn.add_controller(drag_source);

                // 2. Drop target on tab box for exact before/after reordering
                let drop_btn =
                    gtk4::DropTarget::new(glib::types::Type::STRING, gdk::DragAction::MOVE);
                let box_weak_m = tab_box.downgrade();
                drop_btn.connect_motion(move |_, x, _| {
                    if let Some(b) = box_weak_m.upgrade() {
                        let w = b.width() as f64;
                        if x < w * 0.5 {
                            b.add_css_class("drop-before");
                            b.remove_css_class("drop-after");
                        } else {
                            b.add_css_class("drop-after");
                            b.remove_css_class("drop-before");
                        }
                    }
                    gdk::DragAction::MOVE
                });
                let box_weak_l = tab_box.downgrade();
                drop_btn.connect_leave(move |_| {
                    if let Some(b) = box_weak_l.upgrade() {
                        b.remove_css_class("drop-before");
                        b.remove_css_class("drop-after");
                    }
                });

                let locs_drop_btn = tab_locations.clone();
                let tab_order_drop_btn = tab_order.clone();
                let act_sec_tabs_b = active_section_tabs.clone();
                let re_drop_b = refresh_fn.clone();
                let box_weak_d = tab_box.downgrade();
                drop_btn.connect_drop(move |_, value, x, _| {
                    if let Some(b) = box_weak_d.upgrade() {
                        b.remove_css_class("drop-before");
                        b.remove_css_class("drop-after");
                    }
                    if let Ok(s) = value.get::<String>() {
                        if let Some(idx_str) = s.strip_prefix("tab:") {
                            if let Ok(idx) = idx_str.parse::<usize>() {
                                if idx < 5 {
                                    locs_drop_btn.borrow_mut()[idx] = TabLocation::Docked(sec_idx);

                                    let mut order = tab_order_drop_btn.borrow().clone();
                                    if let Some(pos) = order.iter().position(|&x| x == idx) {
                                        order.remove(pos);
                                    }

                                    let target_pos = order
                                        .iter()
                                        .position(|&x| x == tab_i)
                                        .unwrap_or(order.len());
                                    let is_left = if let Some(b) = box_weak_d.upgrade() {
                                        x < (b.width() as f64) * 0.5
                                    } else {
                                        true
                                    };

                                    let insert_pos = if is_left {
                                        target_pos
                                    } else {
                                        target_pos + 1
                                    };
                                    order.insert(insert_pos.min(order.len()), idx);
                                    *tab_order_drop_btn.borrow_mut() = order;

                                    if sec_idx < act_sec_tabs_b.borrow().len() {
                                        act_sec_tabs_b.borrow_mut()[sec_idx] = idx;
                                    }

                                    re_drop_b();
                                    return true;
                                }
                            }
                        }
                    }
                    false
                });

                tab_box.add_controller(drop_btn);

                // Tab Context Menu
                let popover = gtk4::Popover::builder()
                    .has_arrow(false)
                    .autohide(true)
                    .position(gtk4::PositionType::Bottom)
                    .css_classes(["menu", "tab-context-popover"])
                    .build();
                popover.set_parent(&tab_box);
                {
                    let pop_destroy = popover.clone();
                    tab_box.connect_destroy(move |_| {
                        if pop_destroy.parent().is_some() {
                            pop_destroy.unparent();
                        }
                    });
                }

                let menu_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(1)
                    .margin_top(4)
                    .margin_bottom(4)
                    .margin_start(4)
                    .margin_end(4)
                    .width_request(230)
                    .build();

                let create_menu_item =
                    |icon_name: &str, label_text: &str| -> (gtk4::Button, gtk4::Box) {
                        let b = gtk4::Button::builder()
                            .css_classes(["flat", "menu-button"])
                            .halign(gtk4::Align::Fill)
                            .focus_on_click(false)
                            .build();
                        let row = gtk4::Box::builder()
                            .orientation(gtk4::Orientation::Horizontal)
                            .spacing(10)
                            .margin_start(4)
                            .margin_end(6)
                            .margin_top(2)
                            .margin_bottom(2)
                            .build();
                        let img = gtk4::Image::from_icon_name(icon_name);
                        img.set_pixel_size(16);
                        img.add_css_class("dim-label");
                        let lbl = gtk4::Label::builder()
                            .label(label_text)
                            .xalign(0.0)
                            .hexpand(true)
                            .build();
                        row.append(&img);
                        row.append(&lbl);
                        b.set_child(Some(&row));
                        (b, row)
                    };

                // Close this panel
                let (btn_close, _) =
                    create_menu_item("window-close-symbolic", &crate::core::gettext("Close Panel"));
                let locs_c = tab_locations.clone();
                let re_c = refresh_fn.clone();
                let pop_c = popover.clone();
                btn_close.connect_clicked(move |_| {
                    pop_c.popdown();
                    locs_c.borrow_mut()[tab_i] = TabLocation::Closed;
                    re_c();
                });
                menu_box.append(&btn_close);

                // Detach to Floating Window
                let (btn_detach, _) = create_menu_item(
                    "view-restore-symbolic",
                    &crate::core::gettext("Detach into Floating Window"),
                );
                let locs_c = tab_locations.clone();
                let re_c = refresh_fn.clone();
                let pop_c = popover.clone();
                btn_detach.connect_clicked(move |_| {
                    pop_c.popdown();
                    locs_c.borrow_mut()[tab_i] = TabLocation::Floating;
                    re_c();
                });
                menu_box.append(&btn_detach);

                // Move to New Section Below
                let (btn_split, _) = create_menu_item(
                    "go-down-symbolic",
                    &crate::core::gettext("Move to New Section Below"),
                );
                let locs_c = tab_locations.clone();
                let act_sec_tabs_c = active_section_tabs.clone();
                let re_c = refresh_fn.clone();
                let pop_c = popover.clone();
                btn_split.connect_clicked(move |_| {
                    pop_c.popdown();
                    let cur_locs = *locs_c.borrow();
                    let max_sec = (0..4)
                        .filter_map(|k| {
                            if let TabLocation::Docked(s) = cur_locs[k] {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .max()
                        .unwrap_or(0);
                    locs_c.borrow_mut()[tab_i] = TabLocation::Docked(max_sec + 1);
                    if max_sec + 1 < act_sec_tabs_c.borrow().len() {
                        act_sec_tabs_c.borrow_mut()[max_sec + 1] = tab_i;
                    }
                    re_c();
                });
                menu_box.append(&btn_split);

                // Move to Section Above
                if sec_idx > 0 {
                    let (btn_up, _) = create_menu_item(
                        "go-up-symbolic",
                        &crate::core::gettext("Move to Section Above"),
                    );
                    let locs_c = tab_locations.clone();
                    let act_sec_tabs_c = active_section_tabs.clone();
                    let re_c = refresh_fn.clone();
                    let pop_c = popover.clone();
                    btn_up.connect_clicked(move |_| {
                        pop_c.popdown();
                        locs_c.borrow_mut()[tab_i] = TabLocation::Docked(sec_idx - 1);
                        if sec_idx > 0 && sec_idx - 1 < act_sec_tabs_c.borrow().len() {
                            act_sec_tabs_c.borrow_mut()[sec_idx - 1] = tab_i;
                        }
                        re_c();
                    });
                    menu_box.append(&btn_up);
                }

                // Reorder items in current section
                let cur_pos_in_sec = tabs_in_sec.iter().position(|&x| x == tab_i).unwrap_or(0);
                if tabs_in_sec.len() > 1 {
                    menu_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

                    if cur_pos_in_sec > 0 {
                        let (btn_l, _) = create_menu_item(
                            "go-previous-symbolic",
                            &crate::core::gettext("Move Tab Left"),
                        );
                        let order_c = tab_order.clone();
                        let re_c = refresh_fn.clone();
                        let pop_c = popover.clone();
                        btn_l.connect_clicked(move |_| {
                            pop_c.popdown();
                            let mut order = order_c.borrow().clone();
                            if let Some(pos) = order.iter().position(|&x| x == tab_i) {
                                if pos > 0 {
                                    order.swap(pos, pos - 1);
                                    *order_c.borrow_mut() = order;
                                    re_c();
                                }
                            }
                        });
                        menu_box.append(&btn_l);
                    }

                    if cur_pos_in_sec + 1 < tabs_in_sec.len() {
                        let (btn_r, _) = create_menu_item(
                            "go-next-symbolic",
                            &crate::core::gettext("Move Tab Right"),
                        );
                        let order_c = tab_order.clone();
                        let re_c = refresh_fn.clone();
                        let pop_c = popover.clone();
                        btn_r.connect_clicked(move |_| {
                            pop_c.popdown();
                            let mut order = order_c.borrow().clone();
                            if let Some(pos) = order.iter().position(|&x| x == tab_i) {
                                if pos + 1 < order.len() {
                                    order.swap(pos, pos + 1);
                                    *order_c.borrow_mut() = order;
                                    re_c();
                                }
                            }
                        });
                        menu_box.append(&btn_r);
                    }
                }

                // Close Other Panels
                menu_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
                let (btn_close_others, _) = create_menu_item(
                    "view-paged-symbolic",
                    &crate::core::gettext("Close Other Panels"),
                );
                let locs_c = tab_locations.clone();
                let act_sec_tabs_c = active_section_tabs.clone();
                let re_c = refresh_fn.clone();
                let pop_c = popover.clone();
                btn_close_others.connect_clicked(move |_| {
                    pop_c.popdown();
                    for k in 0..4 {
                        if k != tab_i {
                            locs_c.borrow_mut()[k] = TabLocation::Closed;
                        } else {
                            locs_c.borrow_mut()[k] = TabLocation::Docked(0);
                        }
                    }
                    act_sec_tabs_c.borrow_mut()[0] = tab_i;
                    re_c();
                });
                menu_box.append(&btn_close_others);

                // Unify / Restore all tabs
                let total_docked = (0..4)
                    .filter(|&k| matches!(locs[k], TabLocation::Docked(_)))
                    .count();
                if num_sections > 1 || total_docked < 4 {
                    let (btn_merge_all, _) = create_menu_item(
                        "view-grid-symbolic",
                        &crate::core::gettext("Restore All Panels"),
                    );
                    let locs_c = tab_locations.clone();
                    let act_sec_tabs_c = active_section_tabs.clone();
                    let re_c = refresh_fn.clone();
                    let pop_c = popover.clone();
                    btn_merge_all.connect_clicked(move |_| {
                        pop_c.popdown();
                        *locs_c.borrow_mut() = [
                            TabLocation::Docked(0),
                            TabLocation::Docked(0),
                            TabLocation::Docked(0),
                            TabLocation::Docked(0),
                            TabLocation::Docked(0),
                        ];
                        act_sec_tabs_c.borrow_mut()[0] = tab_i;
                        re_c();
                    });
                    menu_box.append(&btn_merge_all);
                }

                popover.set_child(Some(&menu_box));

                let gesture_rclick = gtk4::GestureClick::builder()
                    .button(gdk::BUTTON_SECONDARY)
                    .build();
                let pop_rc = popover.clone();
                gesture_rclick.connect_pressed(move |_, _, _, _| {
                    pop_rc.popup();
                });
                tab_box.add_controller(gesture_rclick);

                let gesture_lpress = gtk4::GestureLongPress::new();
                let pop_lp = popover.clone();
                gesture_lpress.connect_pressed(move |_, _, _| {
                    pop_lp.popup();
                });
                tab_box.add_controller(gesture_lpress);

                sec_tab_bar.append(&tab_box);
            }

            // Add button at the end of section tab bar
            let spacer = gtk4::Box::builder().hexpand(true).build();
            sec_tab_bar.append(&spacer);

            let add_sec_btn = gtk4::MenuButton::builder()
                .icon_name("list-add-symbolic")
                .tooltip_text(crate::core::gettext("Add Panel"))
                .css_classes(["flat", "add-panel-tab-btn"])
                .valign(gtk4::Align::Center)
                .focus_on_click(false)
                .build();
            let add_sec_pop = catalog::build_catalog_popover(
                tab_info,
                tab_locations,
                active_section_tabs,
                canvas,
                refresh_fn.clone(),
            );
            add_sec_btn.set_popover(Some(&add_sec_pop));
            sec_tab_bar.append(&add_sec_btn);

            sec_box.append(&sec_tab_bar);
            sec_box.append(&tab_widgets[cur_active]);
            sections_container.append(&sec_box);
        }
    }
}
