use gtk4::prelude::*;

pub fn create_item(
    icon_name: &str,
    label_text: &str,
    shortcut: Option<&str>,
) -> (gtk4::Button, gtk4::Label) {
    let btn = gtk4::Button::builder()
        .css_classes(["flat", "menu-button"])
        .focus_on_click(false)
        .build();

    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(10)
        .hexpand(true)
        .build();

    let icon = crate::ui::icons::make_symbolic_image(icon_name, 16);
    icon.add_css_class("dim-label");

    let label = gtk4::Label::builder()
        .label(label_text)
        .xalign(0.0)
        .hexpand(true)
        .build();

    row.append(&icon);
    row.append(&label);

    if let Some(sc) = shortcut {
        let sc_lbl = gtk4::Label::builder()
            .label(sc)
            .xalign(1.0)
            .css_classes(["dim-label", "caption"])
            .build();
        row.append(&sc_lbl);
    }

    btn.set_child(Some(&row));
    (btn, label)
}

pub fn create_toggle_item(
    icon_name: &str,
    label_text: &str,
    shortcut: Option<&str>,
) -> (gtk4::Button, gtk4::Image) {
    let btn = gtk4::Button::builder()
        .css_classes(["flat", "menu-button"])
        .focus_on_click(false)
        .build();

    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(10)
        .hexpand(true)
        .build();

    let icon = crate::ui::icons::make_symbolic_image(icon_name, 16);
    icon.add_css_class("dim-label");

    let label = gtk4::Label::builder()
        .label(label_text)
        .xalign(0.0)
        .hexpand(true)
        .build();

    let check_icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
    check_icon.set_pixel_size(14);
    check_icon.add_css_class("accent");
    check_icon.set_visible(false);

    row.append(&icon);
    row.append(&label);

    if let Some(sc) = shortcut {
        let sc_lbl = gtk4::Label::builder()
            .label(sc)
            .xalign(1.0)
            .css_classes(["dim-label", "caption"])
            .build();
        row.append(&sc_lbl);
    }

    row.append(&check_icon);

    btn.set_child(Some(&row));
    (btn, check_icon)
}
