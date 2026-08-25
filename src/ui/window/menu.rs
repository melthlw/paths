use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::file_ops::WindowFileOps;
use crate::ui::canvas::CanvasWidget;

pub fn create_menu_btn(icon_name: &str, label: &str, accel: Option<&str>) -> gtk4::Button {
    let btn = gtk4::Button::builder()
        .css_classes(["flat", "menu-item-btn"])
        .build();
    let box_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(10)
        .margin_start(8)
        .margin_end(8)
        .margin_top(5)
        .margin_bottom(5)
        .valign(gtk4::Align::Center)
        .build();
    let icon = gtk4::Image::from_icon_name(icon_name);
    icon.set_pixel_size(16);
    let lbl = gtk4::Label::builder()
        .label(label)
        .hexpand(true)
        .halign(gtk4::Align::Start)
        .build();
    box_row.append(&icon);
    box_row.append(&lbl);
    if let Some(accel_text) = accel {
        let accel_lbl = gtk4::Label::builder()
            .label(accel_text)
            .css_classes(["dim-label", "caption"])
            .halign(gtk4::Align::End)
            .build();
        box_row.append(&accel_lbl);
    }
    btn.set_child(Some(&box_row));
    btn
}

pub fn build_main_menu(
    file_ops: &WindowFileOps,
    canvas: &CanvasWidget,
    main_win_holder: &Rc<RefCell<Option<adw::ApplicationWindow>>>,
) -> gtk4::MenuButton {
    let menu_btn = gtk4::MenuButton::builder()
        .icon_name("app-menu-symbolic")
        .tooltip_text(&crate::core::gettext("Main Menu"))
        .css_classes(["flat"])
        .build();

    let menu_popover = gtk4::Popover::builder()
        .has_arrow(true)
        .css_classes(["app-menu-popover"])
        .build();

    let menu_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(3)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .width_request(240)
        .build();

    // 0. New Document
    let new_menu_btn = create_menu_btn(
        "document-new-symbolic",
        &crate::core::gettext("New Document"),
        Some("Ctrl+N"),
    );
    let pop_new = menu_popover.clone();
    let act_new = file_ops.perform_new_doc.clone();
    new_menu_btn.connect_clicked(move |_| {
        pop_new.popdown();
        act_new();
    });

    // 1. Open Document
    let open_menu_btn = create_menu_btn(
        "document-open-symbolic",
        &crate::core::gettext("Open Document..."),
        Some("Ctrl+O"),
    );
    let pop_open = menu_popover.clone();
    let act_open = file_ops.perform_open.clone();
    open_menu_btn.connect_clicked(move |_| {
        pop_open.popdown();
        act_open();
    });

    // 2. Save Document
    let save_menu_btn = create_menu_btn(
        "document-save-symbolic",
        &crate::core::gettext("Save"),
        Some("Ctrl+S"),
    );
    let pop_save = menu_popover.clone();
    let act_save = file_ops.perform_save.clone();
    save_menu_btn.connect_clicked(move |_| {
        pop_save.popdown();
        act_save();
    });

    // 3. Save As Document
    let save_as_menu_btn = create_menu_btn(
        "document-save-as-symbolic",
        &crate::core::gettext("Save As..."),
        Some("Ctrl+Shift+S"),
    );
    let pop_save_as = menu_popover.clone();
    let act_save_as = file_ops.perform_save_as.clone();
    save_as_menu_btn.connect_clicked(move |_| {
        pop_save_as.popdown();
        act_save_as();
    });

    // 4. Import Item
    let import_btn = create_menu_btn(
        "insert-image-symbolic",
        &crate::core::gettext("Import..."),
        Some("Ctrl+I"),
    );
    let pop_imp = menu_popover.clone();
    let act_imp = file_ops.perform_import.clone();
    import_btn.connect_clicked(move |_| {
        pop_imp.popdown();
        act_imp();
    });

    let sep0 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    // 5. Preferences Item
    let pref_btn = create_menu_btn(
        "prefs-general-symbolic",
        &crate::core::gettext("Preferences"),
        Some("Ctrl+,"),
    );

    // 6. Keyboard Shortcuts Item
    let shortcuts_btn = create_menu_btn(
        "prefs-shortcuts-symbolic",
        &crate::core::gettext("Keyboard Shortcuts"),
        Some("Ctrl+?"),
    );

    // 7. Separator
    let sep1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    // 8. About Item
    let about_btn = create_menu_btn(
        "prefs-about-symbolic",
        &crate::core::gettext("About Paths"),
        None,
    );

    menu_box.append(&new_menu_btn);
    menu_box.append(&open_menu_btn);
    menu_box.append(&save_menu_btn);
    menu_box.append(&save_as_menu_btn);
    menu_box.append(&import_btn);
    menu_box.append(&sep0);
    menu_box.append(&pref_btn);
    menu_box.append(&shortcuts_btn);
    menu_box.append(&sep1);
    menu_box.append(&about_btn);

    menu_popover.set_child(Some(&menu_box));
    menu_btn.set_popover(Some(&menu_popover));

    // Connect preferences, shortcuts, and about callbacks
    {
        let win_holder = main_win_holder.clone();
        let canvas_pref = canvas.clone();
        let pop_pref = menu_popover.clone();
        pref_btn.connect_clicked(move |_| {
            pop_pref.popdown();
            if let Some(win) = win_holder.borrow().as_ref() {
                crate::ui::preferences::show_preferences_window(win, canvas_pref.clone());
            }
        });
    }

    {
        let win_holder = main_win_holder.clone();
        let canvas_sc = canvas.clone();
        let pop_sc = menu_popover.clone();
        shortcuts_btn.connect_clicked(move |_| {
            pop_sc.popdown();
            if let Some(win) = win_holder.borrow().as_ref() {
                crate::ui::preferences::show_preferences_window(win, canvas_sc.clone());
            }
        });
    }

    {
        let win_holder = main_win_holder.clone();
        let pop_ab = menu_popover.clone();
        about_btn.connect_clicked(move |_| {
            pop_ab.popdown();
            if let Some(win) = win_holder.borrow().as_ref() {
                let about = adw::AboutDialog::builder()
                    .application_name("Paths")
                    .application_icon("io.gitlab.lewisHeart.Paths")
                    .version("0.3.0-alpha")
                    .developer_name("Lewis")
                    .developers(["Lewis"])
                    .artists(["Lewis"])
                    .issue_url("https://gitlab.com/lewisHeart/paths/-/issues")
                    .website("https://gitlab.com/lewisHeart/paths")
                    .license_type(gtk4::License::Gpl30)
                    .comments(&crate::core::gettext("Modern vector design editor accelerated by Skia GPU, built with GTK4, Libadwaita and Rust."))
                    .build();
                about.present(Some(win));
            }
        });
    }

    menu_btn
}
