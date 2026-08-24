use gtk4::prelude::*;

pub const FONT_WEIGHT_NAMES: &[&str] = &[
    "Thin (100)",
    "Light (300)",
    "Regular (400)",
    "Medium (500)",
    "SemiBold (600)",
    "Bold (700)",
    "Black (900)",
];

pub const FONT_WEIGHT_VALUES: &[u32] = &[100, 300, 400, 500, 600, 700, 900];

pub fn create_resource_btn_with_img(resource_path: &str, tooltip: &str) -> (gtk4::Button, gtk4::Image) {
    let img = crate::ui::icons::make_symbolic_image(resource_path, 20);
    let btn = gtk4::Button::builder()
        .child(&img)
        .tooltip_text(tooltip)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    (btn, img)
}

pub fn create_resource_btn(resource_path: &str, tooltip: &str) -> gtk4::Button {
    let (btn, _) = create_resource_btn_with_img(resource_path, tooltip);
    btn
}

pub fn create_resource_toggle_btn(resource_path: &str, tooltip: &str, active: bool) -> gtk4::ToggleButton {
    let img = crate::ui::icons::make_symbolic_image(resource_path, 18);
    gtk4::ToggleButton::builder()
        .child(&img)
        .tooltip_text(tooltip)
        .css_classes(["flat"])
        .active(active)
        .focus_on_click(false)
        .build()
}

pub fn create_coord_entry(prefix: &str) -> (gtk4::Box, gtk4::Entry) {
    let box_widget = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();

    let prefix_lbl = gtk4::Label::builder()
        .label(prefix)
        .css_classes(["dim-label", "caption"])
        .build();

    let entry = gtk4::Entry::builder()
        .placeholder_text("—")
        .width_chars(6)
        .max_width_chars(8)
        .css_classes(["numeric"])
        .valign(gtk4::Align::Center)
        .build();

    box_widget.append(&prefix_lbl);
    box_widget.append(&entry);

    (box_widget, entry)
}
