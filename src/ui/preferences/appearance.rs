use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use super::widgets::{make_color_swatches, make_page};

pub fn build_appearance_page() -> gtk4::ScrolledWindow {
    let theme_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Application Theme"))
        .description(crate::core::gettext(
            "Choose interface color scheme and visual theme",
        ))
        .build();

    let style_mgr = adw::StyleManager::default();
    let current_scheme = style_mgr.color_scheme();

    let theme_cards_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(16)
        .halign(gtk4::Align::Center)
        .margin_top(12)
        .margin_bottom(16)
        .build();

    let make_theme_card = |title: &str, subtitle: &str, preview_cls: &str, is_active: bool| -> gtk4::Button {
        let btn = gtk4::Button::builder()
            .css_classes(["theme-card-btn", "flat"])
            .build();
        if is_active {
            btn.add_css_class("active");
        }

        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .halign(gtk4::Align::Center)
            .build();

        let preview = gtk4::Box::builder()
            .css_classes(["theme-card-preview", preview_cls])
            .width_request(120)
            .height_request(48)
            .build();

        let lbl_title = gtk4::Label::builder()
            .label(title)
            .css_classes(["heading"])
            .halign(gtk4::Align::Center)
            .build();

        let lbl_sub = gtk4::Label::builder()
            .label(subtitle)
            .css_classes(["caption", "dim-label"])
            .halign(gtk4::Align::Center)
            .build();

        vbox.append(&preview);
        vbox.append(&lbl_title);
        vbox.append(&lbl_sub);
        btn.set_child(Some(&vbox));

        btn
    };

    let btn_system = make_theme_card(
        &crate::core::gettext("System Default"),
        &crate::core::gettext("Follows GNOME"),
        "theme-preview-system",
        matches!(current_scheme, adw::ColorScheme::Default),
    );

    let btn_light = make_theme_card(
        &crate::core::gettext("Light"),
        &crate::core::gettext("Bright interface"),
        "theme-preview-light",
        matches!(current_scheme, adw::ColorScheme::ForceLight),
    );

    let btn_dark = make_theme_card(
        &crate::core::gettext("Dark"),
        &crate::core::gettext("Low brightness"),
        "theme-preview-dark",
        matches!(current_scheme, adw::ColorScheme::ForceDark),
    );

    let b_sys = btn_system.clone();
    let b_lgt = btn_light.clone();
    let b_drk = btn_dark.clone();

    btn_system.connect_clicked({
        let b_sys = b_sys.clone();
        let b_lgt = b_lgt.clone();
        let b_drk = b_drk.clone();
        move |_| {
            adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default);
            b_sys.add_css_class("active");
            b_lgt.remove_css_class("active");
            b_drk.remove_css_class("active");
        }
    });

    btn_light.connect_clicked({
        let b_sys = b_sys.clone();
        let b_lgt = b_lgt.clone();
        let b_drk = b_drk.clone();
        move |_| {
            adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceLight);
            b_sys.remove_css_class("active");
            b_lgt.add_css_class("active");
            b_drk.remove_css_class("active");
        }
    });

    btn_dark.connect_clicked({
        let b_sys = b_sys.clone();
        let b_lgt = b_lgt.clone();
        let b_drk = b_drk.clone();
        move |_| {
            adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
            b_sys.remove_css_class("active");
            b_lgt.remove_css_class("active");
            b_drk.add_css_class("active");
        }
    });

    theme_cards_box.append(&btn_system);
    theme_cards_box.append(&btn_light);
    theme_cards_box.append(&btn_dark);

    let theme_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Color Scheme"))
        .subtitle(crate::core::gettext("Select light, dark or automatic system synchronization"))
        .build();
    theme_row.set_child(Some(&theme_cards_box));
    theme_group.add(&theme_row);

    // Accent Color Palette (Visual Color Chips)
    let accent_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Accent Color"))
        .description(crate::core::gettext(
            "Choose the highlight color for selectors and active elements",
        ))
        .build();

    let accent_chips = make_color_swatches(
        vec![
            ("GNOME Blue", "chip-blue", "#3584e4"),
            ("Purple", "chip-purple", "#9141ac"),
            ("Emerald Green", "chip-green", "#2ec27e"),
            ("Orange", "chip-orange", "#e66100"),
            ("Ruby Red", "chip-red", "#e01b24"),
            ("Slate", "chip-slate", "#62a0ea"),
            ("Amber Gold", "chip-gold", "#f6d32d"),
            ("Magenta Pink", "chip-pink", "#e01b84"),
        ],
        "#3584e4",
        |color_hex| {
            let css_prov = gtk4::CssProvider::new();
            let css_data = format!(
                "@define-color accent_color {}; @define-color accent_bg_color {};",
                color_hex, color_hex
            );
            css_prov.load_from_string(&css_data);
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &css_prov,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 10,
                );
            }
        },
    );

    let accent_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::Center)
        .margin_top(8)
        .margin_bottom(12)
        .build();
    accent_box.append(&accent_chips);

    let accent_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Accent Color"))
        .subtitle(crate::core::gettext(
            "Applies chromatic highlight to selectors, badges and focus elements",
        ))
        .build();
    accent_row.set_child(Some(&accent_box));
    accent_group.add(&accent_row);

    // Interface Icons Color Group
    let icons_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Interface Icons"))
        .description(crate::core::gettext(
            "Customize the color of toolbar, tool options and inspector icons",
        ))
        .build();

    let cur_icon_color = crate::ui::icons::current_interface_icon_color();

    let icon_color_chips = make_color_swatches(
        vec![
            ("Theme Default", "chip-system-gradient", "default".to_string()),
            ("Crisp White", "chip-white", "#ffffff".to_string()),
            ("Pure Black", "chip-black", "#000000".to_string()),
            ("GNOME Blue", "chip-blue", "#3584e4".to_string()),
            ("Purple", "chip-purple", "#9141ac".to_string()),
            ("Emerald Green", "chip-green", "#2ec27e".to_string()),
            ("Orange", "chip-orange", "#e66100".to_string()),
            ("Ruby Red", "chip-red", "#e01b24".to_string()),
            ("Amber Gold", "chip-gold", "#f6d32d".to_string()),
            ("Slate", "chip-slate", "#62a0ea".to_string()),
        ],
        cur_icon_color,
        |color_code| {
            crate::ui::icons::set_interface_icon_color(&color_code);
        },
    );

    let icons_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(10)
        .halign(gtk4::Align::Center)
        .margin_top(8)
        .margin_bottom(12)
        .build();
    icons_box.append(&icon_color_chips);

    let icon_color_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Icon Color"))
        .subtitle(crate::core::gettext(
            "Tint toolbars, HUDs and inspector icons with selected chromatic hue",
        ))
        .build();
    icon_color_row.set_child(Some(&icons_box));
    icons_group.add(&icon_color_row);

    make_page(vec![theme_group, accent_group, icons_group])
}
