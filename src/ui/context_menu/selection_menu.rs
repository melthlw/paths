use gtk4::prelude::*;

use crate::ui::canvas::CanvasWidget;
use super::helpers::create_item;

pub struct SelectionMenuWidgets {
    pub copy_btn: gtk4::Button,
    pub cut_btn: gtk4::Button,
    pub paste_btn: gtk4::Button,
    pub sep_clip: gtk4::Separator,

    pub duplicate_btn: gtk4::Button,
    pub clone_btn: gtk4::Button,
    pub unlink_clone_btn: gtk4::Button,
    pub select_original_btn: gtk4::Button,
    pub select_clones_btn: gtk4::Button,
    pub delete_btn: gtk4::Button,
    pub sep_edit: gtk4::Separator,

    pub convert_path_btn: gtk4::Button,
    pub bring_front_btn: gtk4::Button,
    pub bring_forward_btn: gtk4::Button,
    pub send_backward_btn: gtk4::Button,
    pub send_back_btn: gtk4::Button,
    pub sep_arrange: gtk4::Separator,

    pub group_btn: gtk4::Button,
    pub ungroup_btn: gtk4::Button,
    pub enter_group_btn: gtk4::Button,
    pub enter_group_label: gtk4::Label,
    pub set_clip_group_btn: gtk4::Button,
    pub sep_group: gtk4::Separator,

    pub select_same_btn: gtk4::Button,
    pub sep_select_same: gtk4::Separator,

    pub hide_btn: gtk4::Button,
    pub lock_btn: gtk4::Button,

    pub select_same_box: gtk4::Box,
}

pub fn build_selection_menu(
    canvas: &CanvasWidget,
    popover: &gtk4::Popover,
    stack: &gtk4::Stack,
) -> SelectionMenuWidgets {
    let (copy_btn, _) = create_item(
        "edit-copy-symbolic",
        &crate::core::gettext("Copy"),
        Some("Ctrl + C"),
    );
    let (cut_btn, _) = create_item(
        "edit-cut-symbolic",
        &crate::core::gettext("Cut"),
        Some("Ctrl + X"),
    );
    let (paste_btn, _) = create_item(
        "edit-paste-symbolic",
        &crate::core::gettext("Paste"),
        Some("Ctrl + V"),
    );

    let sep_clip = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let (duplicate_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/duplicate.svg",
        &crate::core::gettext("Duplicate"),
        Some("Ctrl + D"),
    );
    let (clone_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/clone.svg",
        &crate::core::gettext("Clone"),
        Some("Alt + D"),
    );
    let (unlink_clone_btn, _) = create_item(
        "edit-cut-symbolic",
        &crate::core::gettext("Unlink Clone"),
        Some("Shift + Alt + D"),
    );
    let (select_original_btn, _) = create_item(
        "go-jump-symbolic",
        &crate::core::gettext("Select Original"),
        Some("Shift + D"),
    );
    let (select_clones_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/clone.svg",
        &crate::core::gettext("Select Linked Clones"),
        None,
    );
    let (delete_btn, _) = create_item(
        "user-trash-symbolic",
        &crate::core::gettext("Delete"),
        Some("Del"),
    );

    let sep_edit = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let (convert_path_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/object-to-path.svg",
        &crate::core::gettext("Convert to Path"),
        None,
    );
    let (bring_front_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/layer-bring-to-front.svg",
        &crate::core::gettext("Bring to Front"),
        Some("Ctrl + Shift + ]"),
    );
    let (bring_forward_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/layer-bring-forward.svg",
        &crate::core::gettext("Bring Forward"),
        Some("Ctrl + ]"),
    );
    let (send_backward_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/layer-send-backward.svg",
        &crate::core::gettext("Send Backward"),
        Some("Ctrl + ["),
    );
    let (send_back_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/layer-send-to-back.svg",
        &crate::core::gettext("Send to Back"),
        Some("Ctrl + Shift + ["),
    );

    let sep_arrange = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let (group_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/group.svg",
        &crate::core::gettext("Group"),
        Some("Ctrl + G"),
    );
    let (set_clip_group_btn, _) = create_item(
        "crop-symbolic",
        &crate::core::gettext("Create Clipping Mask"),
        None,
    );
    let (ungroup_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/ungroup.svg",
        &crate::core::gettext("Ungroup"),
        Some("Shift + Ctrl + G"),
    );
    let (enter_group_btn, enter_group_label) = create_item(
        "go-jump-symbolic",
        &crate::core::gettext("Enter Group"),
        None,
    );

    let sep_group = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let (select_same_btn, _) = create_item(
        "edit-select-all-symbolic",
        &crate::core::gettext("Select Same"),
        Some("›"),
    );

    let sep_select_same = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let (hide_btn, _) = create_item(
        "view-conceal-symbolic",
        &crate::core::gettext("Hide Selection"),
        None,
    );
    let (lock_btn, _) = create_item(
        "changes-prevent-symbolic",
        &crate::core::gettext("Lock Selection"),
        None,
    );

    // Page 2: Select Same Submenu
    let select_same_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(1)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .width_request(240)
        .build();

    let back_same_btn = gtk4::Button::builder()
        .css_classes(["flat", "menu-button"])
        .focus_on_click(false)
        .build();
    let back_same_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    let back_same_icon = gtk4::Image::from_icon_name("go-previous-symbolic");
    back_same_icon.set_pixel_size(16);
    let back_same_label = gtk4::Label::builder()
        .label(&crate::core::gettext("Select Same"))
        .css_classes(["heading"])
        .xalign(0.0)
        .build();
    back_same_row.append(&back_same_icon);
    back_same_row.append(&back_same_label);
    back_same_btn.set_child(Some(&back_same_row));

    select_same_box.append(&back_same_btn);
    select_same_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    let (sel_fill_btn, _) = create_item(
        "color-select-symbolic",
        &crate::core::gettext("Same Fill Color"),
        None,
    );
    let (sel_stroke_btn, _) = create_item(
        "draw-brush-symbolic",
        &crate::core::gettext("Same Stroke Color"),
        None,
    );
    let (sel_type_btn, _) = create_item(
        "view-grid-symbolic",
        &crate::core::gettext("Same Object Type"),
        None,
    );

    select_same_box.append(&sel_fill_btn);
    select_same_box.append(&sel_stroke_btn);
    select_same_box.append(&sel_type_btn);

    {
        let stack = stack.clone();
        select_same_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("select_same");
        });
    }
    {
        let stack = stack.clone();
        back_same_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("main");
        });
    }

    // Connect handlers for selection actions
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        copy_btn.connect_clicked(move |_| {
            canvas.copy_selected();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        cut_btn.connect_clicked(move |_| {
            canvas.cut_selected();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        paste_btn.connect_clicked(move |_| {
            canvas.paste(None);
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        duplicate_btn.connect_clicked(move |_| {
            canvas.duplicate_selected();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        clone_btn.connect_clicked(move |_| {
            canvas.clone_selected();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        unlink_clone_btn.connect_clicked(move |_| {
            canvas.unlink_selected_clones();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        select_original_btn.connect_clicked(move |_| {
            canvas.select_original_element();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        select_clones_btn.connect_clicked(move |_| {
            canvas.select_linked_clones();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        delete_btn.connect_clicked(move |_| {
            canvas.delete_selected_layers();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        convert_path_btn.connect_clicked(move |_| {
            canvas.convert_selected_to_path();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        bring_front_btn.connect_clicked(move |_| {
            canvas.bring_to_front();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        bring_forward_btn.connect_clicked(move |_| {
            canvas.bring_forward();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        send_backward_btn.connect_clicked(move |_| {
            canvas.send_backward();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        send_back_btn.connect_clicked(move |_| {
            canvas.send_to_back();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        sel_fill_btn.connect_clicked(move |_| {
            canvas.select_same_fill();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        sel_stroke_btn.connect_clicked(move |_| {
            canvas.select_same_stroke();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        sel_type_btn.connect_clicked(move |_| {
            canvas.select_same_type();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        group_btn.connect_clicked(move |_| {
            canvas.group_selected();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        ungroup_btn.connect_clicked(move |_| {
            canvas.ungroup_selected();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        enter_group_btn.connect_clicked(move |_| {
            canvas.ungroup_selected();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        set_clip_group_btn.connect_clicked(move |_| {
            canvas.set_clip_group();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        hide_btn.connect_clicked(move |_| {
            canvas.hide_selected();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        lock_btn.connect_clicked(move |_| {
            canvas.lock_selected();
            popover.popdown();
        });
    }

    SelectionMenuWidgets {
        copy_btn,
        cut_btn,
        paste_btn,
        sep_clip,
        duplicate_btn,
        clone_btn,
        unlink_clone_btn,
        select_original_btn,
        select_clones_btn,
        delete_btn,
        sep_edit,
        convert_path_btn,
        bring_front_btn,
        bring_forward_btn,
        send_backward_btn,
        send_back_btn,
        sep_arrange,
        group_btn,
        ungroup_btn,
        enter_group_btn,
        enter_group_label,
        set_clip_group_btn,
        sep_group,
        select_same_btn,
        sep_select_same,
        hide_btn,
        lock_btn,
        select_same_box,
    }
}
