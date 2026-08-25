pub mod canvas_menu;
pub mod helpers;
pub mod selection_menu;

use gtk4::gdk;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::core::Point;
use crate::ui::canvas::CanvasWidget;
use canvas_menu::{build_canvas_menu, CanvasMenuWidgets};
use selection_menu::{build_selection_menu, SelectionMenuWidgets};

pub struct ObjectContextMenu {
    popover: gtk4::Popover,
    stack: gtk4::Stack,
    canvas: CanvasWidget,
    last_click_pos: Cell<Point>,
    canvas_menu: CanvasMenuWidgets,
    selection_menu: SelectionMenuWidgets,
}

impl ObjectContextMenu {
    pub fn new(canvas: CanvasWidget, parent: &impl IsA<gtk4::Widget>) -> Rc<Self> {
        let popover = gtk4::Popover::builder()
            .has_arrow(false)
            .autohide(true)
            .position(gtk4::PositionType::Bottom)
            .css_classes(["menu", "context-menu-popover"])
            .build();
        popover.set_parent(parent);
        {
            let pop_c = popover.clone();
            parent.as_ref().connect_destroy(move |_| {
                if pop_c.parent().is_some() {
                    pop_c.unparent();
                }
            });
        }

        let stack = gtk4::Stack::builder()
            .transition_type(gtk4::StackTransitionType::SlideLeftRight)
            .transition_duration(150)
            .build();

        let last_click_pos = Cell::new(Point::new(0.0, 0.0));

        let canvas_menu = build_canvas_menu(&canvas, &popover, &stack, &last_click_pos);
        let selection_menu = build_selection_menu(&canvas, &popover, &stack);

        let root_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(1)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(4)
            .margin_end(4)
            .width_request(240)
            .build();

        // Append canvas level actions
        root_box.append(&canvas_menu.paste_here_btn);
        root_box.append(&canvas_menu.paste_canvas_btn);
        root_box.append(&canvas_menu.paste_in_place_btn);
        root_box.append(&canvas_menu.sep_canvas_clip);

        root_box.append(&canvas_menu.select_all_btn);
        root_box.append(&canvas_menu.sep_canvas_sel);

        root_box.append(&canvas_menu.insert_submenu_btn);
        root_box.append(&canvas_menu.guides_grid_submenu_btn);
        root_box.append(&canvas_menu.sep_canvas_tools);

        root_box.append(&canvas_menu.zoom_fit_btn);
        root_box.append(&canvas_menu.zoom_100_btn);
        root_box.append(&canvas_menu.zoom_in_btn);
        root_box.append(&canvas_menu.zoom_out_btn);
        root_box.append(&canvas_menu.sep_canvas_zoom);

        root_box.append(&canvas_menu.add_page_btn);

        // Append selection level actions
        root_box.append(&selection_menu.copy_btn);
        root_box.append(&selection_menu.cut_btn);
        root_box.append(&selection_menu.paste_btn);
        root_box.append(&selection_menu.sep_clip);

        root_box.append(&selection_menu.duplicate_btn);
        root_box.append(&selection_menu.clone_btn);
        root_box.append(&selection_menu.unlink_clone_btn);
        root_box.append(&selection_menu.select_original_btn);
        root_box.append(&selection_menu.select_clones_btn);
        root_box.append(&selection_menu.delete_btn);
        root_box.append(&selection_menu.sep_edit);

        root_box.append(&selection_menu.convert_path_btn);
        root_box.append(&selection_menu.attach_path_btn);
        root_box.append(&selection_menu.detach_path_btn);
        root_box.append(&selection_menu.bring_front_btn);
        root_box.append(&selection_menu.bring_forward_btn);
        root_box.append(&selection_menu.send_backward_btn);
        root_box.append(&selection_menu.send_back_btn);
        root_box.append(&selection_menu.sep_arrange);

        root_box.append(&selection_menu.group_btn);
        root_box.append(&selection_menu.set_clip_group_btn);
        root_box.append(&selection_menu.ungroup_btn);
        root_box.append(&selection_menu.enter_group_btn);
        root_box.append(&selection_menu.sep_group);

        root_box.append(&selection_menu.select_same_btn);
        root_box.append(&selection_menu.sep_select_same);

        root_box.append(&selection_menu.hide_btn);
        root_box.append(&selection_menu.lock_btn);

        stack.add_named(&root_box, Some("main"));
        stack.add_named(&selection_menu.select_same_box, Some("select_same"));
        stack.add_named(&canvas_menu.insert_box, Some("insert"));
        stack.add_named(&canvas_menu.guides_grid_box, Some("guides_grid"));

        popover.set_child(Some(&stack));

        Rc::new(Self {
            popover,
            stack,
            canvas,
            last_click_pos,
            canvas_menu,
            selection_menu,
        })
    }

    pub fn popup_at(&self, pos: Point) {
        self.last_click_pos.set(pos);

        let has_sel = self.canvas.has_selection();
        let sel_count = self.canvas.selection_count();
        let has_clip = self.canvas.has_clipboard();
        let can_group = self.canvas.can_group();
        let can_ungroup = self.canvas.can_ungroup();
        let can_convert = self.canvas.can_convert_selected_to_path();

        self.stack.set_visible_child_name("main");

        if !has_sel {
            self.canvas_menu.paste_here_btn.set_visible(true);
            self.canvas_menu.paste_here_btn.set_sensitive(has_clip);
            self.canvas_menu.paste_canvas_btn.set_visible(true);
            self.canvas_menu.paste_canvas_btn.set_sensitive(has_clip);
            self.canvas_menu.paste_in_place_btn.set_visible(true);
            self.canvas_menu.paste_in_place_btn.set_sensitive(has_clip);
            self.canvas_menu.sep_canvas_clip.set_visible(true);

            self.canvas_menu.select_all_btn.set_visible(true);
            self.canvas_menu.sep_canvas_sel.set_visible(true);

            self.canvas_menu.insert_submenu_btn.set_visible(true);
            self.canvas_menu.guides_grid_submenu_btn.set_visible(true);
            self.canvas_menu.sep_canvas_tools.set_visible(true);

            self.canvas_menu.zoom_fit_btn.set_visible(true);
            self.canvas_menu.zoom_100_btn.set_visible(true);
            self.canvas_menu.zoom_in_btn.set_visible(true);
            self.canvas_menu.zoom_out_btn.set_visible(true);
            self.canvas_menu.sep_canvas_zoom.set_visible(true);

            self.canvas_menu.add_page_btn.set_visible(true);

            self.canvas_menu
                .grid_check_img
                .set_visible(self.canvas.is_grid_visible());
            self.canvas_menu
                .snap_grid_check_img
                .set_visible(self.canvas.is_snap_to_grid());
            self.canvas_menu
                .guides_check_img
                .set_visible(self.canvas.is_guides_visible());
            self.canvas_menu
                .snap_guides_check_img
                .set_visible(self.canvas.is_snap_to_guides());
            self.canvas_menu
                .clear_guides_btn
                .set_sensitive(self.canvas.has_user_guides());

            // Hide object-specific options
            self.selection_menu.copy_btn.set_visible(false);
            self.selection_menu.cut_btn.set_visible(false);
            self.selection_menu.paste_btn.set_visible(false);
            self.selection_menu.sep_clip.set_visible(false);

            self.selection_menu.duplicate_btn.set_visible(false);
            self.selection_menu.clone_btn.set_visible(false);
            self.selection_menu.unlink_clone_btn.set_visible(false);
            self.selection_menu.select_original_btn.set_visible(false);
            self.selection_menu.select_clones_btn.set_visible(false);
            self.selection_menu.delete_btn.set_visible(false);
            self.selection_menu.sep_edit.set_visible(false);

            self.selection_menu.convert_path_btn.set_visible(false);
            self.selection_menu.attach_path_btn.set_visible(false);
            self.selection_menu.detach_path_btn.set_visible(false);
            self.selection_menu.bring_front_btn.set_visible(false);
            self.selection_menu.bring_forward_btn.set_visible(false);
            self.selection_menu.send_backward_btn.set_visible(false);
            self.selection_menu.send_back_btn.set_visible(false);
            self.selection_menu.sep_arrange.set_visible(false);

            self.selection_menu.group_btn.set_visible(false);
            self.selection_menu.ungroup_btn.set_visible(false);
            self.selection_menu.enter_group_btn.set_visible(false);
            self.selection_menu.set_clip_group_btn.set_visible(false);
            self.selection_menu.sep_group.set_visible(false);

            self.selection_menu.select_same_btn.set_visible(false);
            self.selection_menu.sep_select_same.set_visible(false);

            self.selection_menu.hide_btn.set_visible(false);
            self.selection_menu.lock_btn.set_visible(false);
        } else {
            self.canvas_menu.paste_here_btn.set_visible(false);
            self.canvas_menu.paste_canvas_btn.set_visible(false);
            self.canvas_menu.paste_in_place_btn.set_visible(false);
            self.canvas_menu.sep_canvas_clip.set_visible(false);

            self.canvas_menu.select_all_btn.set_visible(false);
            self.canvas_menu.sep_canvas_sel.set_visible(false);

            self.canvas_menu.insert_submenu_btn.set_visible(false);
            self.canvas_menu.guides_grid_submenu_btn.set_visible(false);
            self.canvas_menu.sep_canvas_tools.set_visible(false);

            self.canvas_menu.zoom_fit_btn.set_visible(false);
            self.canvas_menu.zoom_100_btn.set_visible(false);
            self.canvas_menu.zoom_in_btn.set_visible(false);
            self.canvas_menu.zoom_out_btn.set_visible(false);
            self.canvas_menu.sep_canvas_zoom.set_visible(false);

            self.canvas_menu.add_page_btn.set_visible(false);

            self.selection_menu.copy_btn.set_visible(true);
            self.selection_menu.cut_btn.set_visible(true);
            self.selection_menu.paste_btn.set_visible(has_clip);
            self.selection_menu.sep_clip.set_visible(true);

            let has_clones = self.canvas.has_clones_selected();
            let has_masters = self.canvas.has_masters_selected();
            self.selection_menu.duplicate_btn.set_visible(true);
            self.selection_menu.clone_btn.set_visible(true);
            self.selection_menu
                .unlink_clone_btn
                .set_visible(has_clones);
            self.selection_menu
                .select_original_btn
                .set_visible(has_clones);
            self.selection_menu
                .select_clones_btn
                .set_visible(has_masters);
            self.selection_menu.delete_btn.set_visible(true);
            self.selection_menu.sep_edit.set_visible(true);

            self.selection_menu
                .convert_path_btn
                .set_visible(can_convert);
            self.selection_menu.attach_path_btn.set_visible(has_sel);
            self.selection_menu.detach_path_btn.set_visible(has_sel);
            self.selection_menu.bring_front_btn.set_visible(true);
            self.selection_menu.bring_forward_btn.set_visible(true);
            self.selection_menu.send_backward_btn.set_visible(true);
            self.selection_menu.send_back_btn.set_visible(true);
            self.selection_menu.sep_arrange.set_visible(true);

            let show_group_section = can_group || can_ungroup;
            self.selection_menu.group_btn.set_visible(can_group);
            self.selection_menu
                .set_clip_group_btn
                .set_visible(can_group);
            self.selection_menu.ungroup_btn.set_visible(can_ungroup);

            if can_ungroup && sel_count == 1 {
                if let Some(grp_name) = self.canvas.selected_group_name() {
                    self.selection_menu
                        .enter_group_label
                        .set_label(&crate::i18n!("Enter group {}", grp_name));
                } else {
                    self.selection_menu
                        .enter_group_label
                        .set_label(&crate::core::gettext("Enter Group"));
                }
                self.selection_menu.enter_group_btn.set_visible(true);
            } else {
                self.selection_menu.enter_group_btn.set_visible(false);
            }
            self.selection_menu.sep_group.set_visible(show_group_section);

            self.selection_menu.select_same_btn.set_visible(true);
            self.selection_menu.sep_select_same.set_visible(true);

            self.selection_menu.hide_btn.set_visible(true);
            self.selection_menu.lock_btn.set_visible(true);
        }

        let menu_width = 240;
        let rect = gdk::Rectangle::new((pos.x as i32) + (menu_width / 2), pos.y as i32, 1, 1);
        self.popover.set_pointing_to(Some(&rect));
        self.popover.popup();
    }
}
