use gtk4::gdk;
use gtk4::prelude::*;
use libadwaita as adw;
use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use super::canvas::CanvasWidget;
use super::context_menu::ObjectContextMenu;
use crate::core::layer::LayerItemInfo;
use crate::core::{ElementId, Point};

pub struct LayersSidebar {
    toolbar_view: adw::ToolbarView,
    list_box: gtk4::ListBox,
    empty_label: gtk4::Box,
    canvas: CanvasWidget,
    context_menu: Rc<ObjectContextMenu>,
    is_syncing: Rc<Cell<bool>>,
    current_layers: Rc<std::cell::RefCell<Vec<LayerItemInfo>>>,
    expanded_groups: Rc<std::cell::RefCell<HashSet<ElementId>>>,
    btn_group: gtk4::Button,
    btn_ungroup: gtk4::Button,
    btn_up: gtk4::Button,
    btn_down: gtk4::Button,
    btn_del: gtk4::Button,
}

impl LayersSidebar {
    pub fn new(canvas: CanvasWidget) -> Rc<Self> {
        let toolbar_view = adw::ToolbarView::builder()
            .width_request(260)
            .css_classes(["sidebar"])
            .build();

        // 1. Sidebar HeaderBar
        let header_bar = adw::HeaderBar::builder()
            .show_start_title_buttons(true)
            .show_end_title_buttons(false)
            .build();

        let title_lbl = gtk4::Label::builder()
            .label(&crate::core::gettext("Layers"))
            .css_classes(["heading"])
            .build();
        header_bar.set_title_widget(Some(&title_lbl));

        // Action buttons inside HeaderBar (initially hidden until something is selected)
        let actions_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(2)
            .build();

        let img_group = crate::ui::icons::make_symbolic_image("group", 16);
        let btn_group = gtk4::Button::builder()
            .tooltip_text(&crate::core::gettext("Group Selected (Ctrl+G)"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .visible(false)
            .child(&img_group)
            .build();

        let img_ungroup = crate::ui::icons::make_symbolic_image("ungroup", 16);
        let btn_ungroup = gtk4::Button::builder()
            .tooltip_text(&crate::core::gettext("Ungroup (Shift+Ctrl+G)"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .visible(false)
            .child(&img_ungroup)
            .build();

        let btn_up = gtk4::Button::builder()
            .icon_name("go-up-symbolic")
            .tooltip_text(&crate::core::gettext("Move Layer Up"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .visible(false)
            .build();

        let btn_down = gtk4::Button::builder()
            .icon_name("go-down-symbolic")
            .tooltip_text(&crate::core::gettext("Move Layer Down"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .visible(false)
            .build();

        let btn_del = gtk4::Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text(&crate::core::gettext("Delete Layer"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .visible(false)
            .build();

        let canvas_grp = canvas.clone();
        btn_group.connect_clicked(move |_| {
            canvas_grp.group_selected();
        });

        let canvas_ungrp = canvas.clone();
        btn_ungroup.connect_clicked(move |_| {
            canvas_ungrp.ungroup_selected();
        });

        let cur_layers = Rc::new(std::cell::RefCell::new(Vec::<LayerItemInfo>::new()));
        let expanded_groups = Rc::new(std::cell::RefCell::new(HashSet::<ElementId>::new()));

        let canvas_up = canvas.clone();
        let cur_layers_up_c = cur_layers.clone();
        btn_up.connect_clicked(move |_| {
            let sel_id = {
                let layers = cur_layers_up_c.borrow();
                layers.iter().find(|l| l.is_selected).map(|l| l.id)
            };
            if let Some(id) = sel_id {
                canvas_up.move_layer_up(id);
            }
        });

        let canvas_down = canvas.clone();
        let cur_layers_down_c = cur_layers.clone();
        btn_down.connect_clicked(move |_| {
            let sel_id = {
                let layers = cur_layers_down_c.borrow();
                layers.iter().find(|l| l.is_selected).map(|l| l.id)
            };
            if let Some(id) = sel_id {
                canvas_down.move_layer_down(id);
            }
        });

        let canvas_del = canvas.clone();
        btn_del.connect_clicked(move |_| {
            canvas_del.delete_selected_layers();
        });

        actions_box.append(&btn_group);
        actions_box.append(&btn_ungroup);
        actions_box.append(&btn_up);
        actions_box.append(&btn_down);
        actions_box.append(&btn_del);
        header_bar.pack_end(&actions_box);

        toolbar_view.add_top_bar(&header_bar);

        // Content Section inside Sidebar
        let content_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .vexpand(true)
            .hexpand(true)
            .build();

        // List Box inside ScrolledWindow
        let list_box = gtk4::ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(["layers-list"])
            .build();

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .hexpand(true)
            .child(&list_box)
            .build();

        // Empty state placeholder
        let empty_label = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .vexpand(true)
            .spacing(8)
            .margin_top(48)
            .margin_bottom(48)
            .build();

        let empty_icon = gtk4::Image::from_icon_name("view-paged-symbolic");
        empty_icon.set_pixel_size(48);
        empty_icon.set_opacity(0.35);

        let empty_txt = gtk4::Label::builder()
            .label(&crate::core::gettext("No layers"))
            .css_classes(["dim-label", "title-4"])
            .build();

        let empty_sub = gtk4::Label::builder()
            .label(&crate::core::gettext(
                "Draw shapes or text on canvas\nto create layers.",
            ))
            .justify(gtk4::Justification::Center)
            .css_classes(["dim-label", "caption"])
            .build();

        empty_label.append(&empty_icon);
        empty_label.append(&empty_txt);
        empty_label.append(&empty_sub);

        content_box.append(&empty_label);
        content_box.append(&scrolled);

        toolbar_view.set_content(Some(&content_box));

        let is_syncing = Rc::new(Cell::new(false));
        let context_menu = ObjectContextMenu::new(canvas.clone(), &toolbar_view);

        Rc::new(Self {
            toolbar_view,
            list_box,
            empty_label,
            canvas,
            context_menu,
            is_syncing,
            current_layers: cur_layers,
            expanded_groups,
            btn_group,
            btn_ungroup,
            btn_up,
            btn_down,
            btn_del,
        })
    }

    pub fn widget(&self) -> &adw::ToolbarView {
        &self.toolbar_view
    }

    pub fn update_state(self: &Rc<Self>, layers: &[LayerItemInfo]) {
        // Fast-path: Skip expensive GTK widget tree rebuild if layers are identical
        if *self.current_layers.borrow() == layers {
            return;
        }
        self.is_syncing.set(true);
        *self.current_layers.borrow_mut() = layers.to_vec();

        // 1. Calculate selection metrics to dynamically toggle header action buttons
        fn count_selected_and_check_group(
            layer: &LayerItemInfo,
            count: &mut usize,
            has_sel_group: &mut bool,
        ) {
            if layer.is_selected {
                *count += 1;
                if layer.is_group {
                    *has_sel_group = true;
                }
            }
            for child in &layer.children {
                count_selected_and_check_group(child, count, has_sel_group);
            }
        }

        let mut sel_count = 0;
        let mut has_sel_group = false;
        for l in layers {
            count_selected_and_check_group(l, &mut sel_count, &mut has_sel_group);
        }

        let can_group = sel_count >= 2;
        let can_ungroup = has_sel_group || self.canvas.can_ungroup();
        let has_selection = sel_count >= 1;

        self.btn_group.set_visible(can_group);
        self.btn_ungroup.set_visible(can_ungroup);
        self.btn_up.set_visible(has_selection);
        self.btn_down.set_visible(has_selection);
        self.btn_del.set_visible(has_selection);

        // 2. Auto-expand parent groups if a child is selected
        fn expand_selected_parents(
            layer: &LayerItemInfo,
            expanded: &mut HashSet<ElementId>,
        ) -> bool {
            let mut child_selected = false;
            for child in &layer.children {
                if expand_selected_parents(child, expanded) {
                    child_selected = true;
                }
            }
            if (layer.is_selected || child_selected) && layer.is_group {
                expanded.insert(layer.id);
            }
            layer.is_selected || child_selected
        }

        {
            let mut exp = self.expanded_groups.borrow_mut();
            for l in layers {
                expand_selected_parents(l, &mut exp);
            }
        }

        self.rebuild_tree();
        self.is_syncing.set(false);
    }

    fn rebuild_tree(self: &Rc<Self>) {
        let layers = self.current_layers.borrow().clone();
        let is_empty = layers.is_empty();
        self.empty_label.set_visible(is_empty);
        self.list_box.set_visible(!is_empty);

        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        for layer in &layers {
            self.append_tree_node(layer, 0);
        }
    }

    fn append_tree_node(self: &Rc<Self>, layer: &LayerItemInfo, depth: usize) {
        let row = self.create_layer_row(layer, depth);
        self.list_box.append(&row);

        if layer.is_group && self.expanded_groups.borrow().contains(&layer.id) {
            for child in &layer.children {
                self.append_tree_node(child, depth + 1);
            }
        }
    }

    fn create_layer_row(self: &Rc<Self>, layer: &LayerItemInfo, depth: usize) -> gtk4::ListBoxRow {
        let row = gtk4::ListBoxRow::builder()
            .activatable(false)
            .selectable(false)
            .css_classes(["layer-row"])
            .build();

        if layer.is_selected {
            row.add_css_class("selected");
        }

        if !layer.visible {
            row.add_css_class("layer-hidden-state");
        }

        let item_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .css_classes(["layer-row-box"])
            .margin_start(2 + (depth as i32) * 18)
            .build();

        // 1. Expander arrow (for groups) or aligning spacer (for non-groups)
        if layer.is_group {
            let is_exp = self.expanded_groups.borrow().contains(&layer.id);
            let arrow_icon = if is_exp {
                "pan-down-symbolic"
            } else {
                "pan-end-symbolic"
            };

            let btn_exp = gtk4::Button::builder()
                .icon_name(arrow_icon)
                .css_classes(["flat", "layer-expander-btn"])
                .focus_on_click(false)
                .valign(gtk4::Align::Center)
                .build();

            let self_exp = self.clone();
            let gid = layer.id;
            btn_exp.connect_clicked(move |_| {
                {
                    let mut exp = self_exp.expanded_groups.borrow_mut();
                    if exp.contains(&gid) {
                        exp.remove(&gid);
                    } else {
                        exp.insert(gid);
                    }
                }
                self_exp.rebuild_tree();
            });
            item_box.append(&btn_exp);
        } else {
            // Spacer to align all element icons perfectly
            let spacer = gtk4::Box::builder().width_request(20).build();
            item_box.append(&spacer);
        }

        // 2. Middle Content Box (Icon + Name + Badge)
        let content_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .hexpand(true)
            .valign(gtk4::Align::Center)
            .build();

        // 2a. Element Icon (from resource SVG or system icon)
        let icon_img = crate::ui::icons::make_symbolic_image(layer.icon_name, 16);
        icon_img.add_css_class("layer-icon");
        content_box.append(&icon_img);

        // 2b. Name Label (Clean text without event swallowing)
        let name_label = gtk4::Label::builder()
            .label(&layer.name)
            .hexpand(true)
            .xalign(0.0)
            .css_classes(["layer-name-entry"])
            .valign(gtk4::Align::Center)
            .build();
        content_box.append(&name_label);

        // 2c. Count badge for groups
        if layer.is_group {
            let badge = gtk4::Label::builder()
                .label(&format!("{}", layer.children.len()))
                .css_classes(["layer-badge"])
                .valign(gtk4::Align::Center)
                .build();
            content_box.append(&badge);
        }

        // Selection Gesture on content box (Primary click selects layer)
        let gesture_left = gtk4::GestureClick::builder()
            .button(gdk::BUTTON_PRIMARY)
            .build();
        let canvas_click = self.canvas.clone();
        let target_id = layer.id;
        gesture_left.connect_pressed(move |g, _, _, _| {
            let state = g.current_event_state();
            let is_ctrl = state.contains(gdk::ModifierType::CONTROL_MASK)
                || state.contains(gdk::ModifierType::SHIFT_MASK);
            canvas_click.select_layer(target_id, is_ctrl);
        });
        content_box.add_controller(gesture_left);

        // 4. Right-Click Gesture for Context Menu on content box
        let gesture_right = gtk4::GestureClick::builder()
            .button(gdk::BUTTON_SECONDARY)
            .build();
        let canvas_rclick = self.canvas.clone();
        let menu_rclick = self.context_menu.clone();
        let row_rc = row.clone();
        let tb_view = self.toolbar_view.clone();
        gesture_right.connect_pressed(move |_, _, x, y| {
            if !canvas_rclick.is_element_selected(target_id) {
                canvas_rclick.select_layer(target_id, false);
            }
            let (rx, ry) = row_rc
                .compute_point(&tb_view, &gtk4::graphene::Point::new(x as f32, y as f32))
                .map(|p| (p.x() as f64, p.y() as f64))
                .unwrap_or((x, y));
            menu_rclick.popup_at(Point::new(rx as f32, ry as f32));
        });
        content_box.add_controller(gesture_right);

        item_box.append(&content_box);

        // 3. Actions Box: Lock Toggle Button + Visibility Toggle Button
        let actions_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(2)
            .valign(gtk4::Align::Center)
            .build();

        // 3a. Lock Toggle Button
        let lock_icon = if layer.locked {
            "changes-prevent-symbolic"
        } else {
            "changes-allow-symbolic"
        };
        let lock_tooltip = if layer.locked {
            crate::core::gettext("Unlock")
        } else {
            crate::core::gettext("Lock")
        };
        let btn_lock = gtk4::Button::builder()
            .icon_name(lock_icon)
            .tooltip_text(&lock_tooltip)
            .css_classes(["flat", "layer-action-btn"])
            .focus_on_click(false)
            .valign(gtk4::Align::Center)
            .build();

        if layer.locked {
            btn_lock.add_css_class("active-state");
        }

        let canvas_lock = self.canvas.clone();
        let target_id_lock = layer.id;
        btn_lock.connect_clicked(move |_| {
            canvas_lock.toggle_element_locked(target_id_lock);
        });
        actions_box.append(&btn_lock);

        // 3b. Visibility Toggle Button
        let eye_icon = if layer.visible {
            "view-reveal-symbolic"
        } else {
            "view-conceal-symbolic"
        };
        let vis_tooltip = if layer.visible {
            crate::core::gettext("Hide")
        } else {
            crate::core::gettext("Show")
        };
        let btn_vis = gtk4::Button::builder()
            .icon_name(eye_icon)
            .tooltip_text(&vis_tooltip)
            .css_classes(["flat", "layer-action-btn"])
            .focus_on_click(false)
            .valign(gtk4::Align::Center)
            .build();

        if !layer.visible {
            btn_vis.add_css_class("active-state");
        }

        let canvas_vis = self.canvas.clone();
        let target_id_vis = layer.id;
        btn_vis.connect_clicked(move |_| {
            canvas_vis.toggle_element_visibility(target_id_vis);
        });
        actions_box.append(&btn_vis);

        item_box.append(&actions_box);

        row.set_child(Some(&item_box));
        row
    }
}
