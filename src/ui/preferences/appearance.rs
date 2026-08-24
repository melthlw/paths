use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use super::widgets::{make_color_swatches, make_page};
use crate::ui::theme::{
    current_interface_scale, current_toolbar_icon_size, current_visual_theme,
    set_interface_scale, set_toolbar_icon_size, set_visual_theme, InterfaceScale,
    ToolbarIconSize, VisualThemePreset,
};

pub fn build_appearance_page() -> gtk4::ScrolledWindow {
    // ─────────────────────────────────────────────────────────────
    // 1. Color Scheme: System / Light / Dark
    // ─────────────────────────────────────────────────────────────
    let scheme_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Color Scheme"))
        .description(crate::core::gettext(
            "Choose light, dark, or automatic system synchronization",
        ))
        .build();

    let style_mgr = adw::StyleManager::default();
    let current_scheme = style_mgr.color_scheme();

    let scheme_cards_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(16)
        .halign(gtk4::Align::Center)
        .margin_top(10)
        .margin_bottom(12)
        .build();

    let make_scheme_card =
        |title: &str, subtitle: &str, preview_cls: &str, is_active: bool| -> gtk4::Button {
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
                .width_request(110)
                .height_request(44)
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

    let btn_system = make_scheme_card(
        &crate::core::gettext("System Default"),
        &crate::core::gettext("Follows GNOME"),
        "theme-preview-system",
        matches!(current_scheme, adw::ColorScheme::Default),
    );

    let btn_light = make_scheme_card(
        &crate::core::gettext("Light"),
        &crate::core::gettext("Bright interface"),
        "theme-preview-light",
        matches!(current_scheme, adw::ColorScheme::ForceLight),
    );

    let btn_dark = make_scheme_card(
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
            crate::core::AppSettings::set_color_scheme("system");
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
            crate::core::AppSettings::set_color_scheme("light");
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
            crate::core::AppSettings::set_color_scheme("dark");
            b_sys.remove_css_class("active");
            b_lgt.remove_css_class("active");
            b_drk.add_css_class("active");
        }
    });

    scheme_cards_box.append(&btn_system);
    scheme_cards_box.append(&btn_light);
    scheme_cards_box.append(&btn_dark);

    scheme_group.add(&scheme_cards_box);

    // ─────────────────────────────────────────────────────────────
    // 2. Visual Theme Presets (GNOME Builder / Text Editor Inspired)
    // ─────────────────────────────────────────────────────────────
    let theme_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Visual Themes and Palettes"))
        .description(crate::core::gettext(
            "Color schemes inspired by GNOME Builder, Text Editor, and modern developer aesthetics",
        ))
        .build();

    let cur_theme = current_visual_theme();

    // Flow box or grid of theme cards
    let flow_box = gtk4::FlowBox::builder()
        .valign(gtk4::Align::Start)
        .max_children_per_line(4)
        .min_children_per_line(2)
        .selection_mode(gtk4::SelectionMode::None)
        .homogeneous(true)
        .column_spacing(10)
        .row_spacing(10)
        .margin_top(8)
        .margin_bottom(10)
        .build();

    let mut theme_buttons: Vec<(VisualThemePreset, gtk4::Button)> = Vec::new();

    for preset in VisualThemePreset::ALL {
        let btn = gtk4::Button::builder()
            .css_classes(["theme-preset-card", "flat"])
            .build();
        if preset == cur_theme {
            btn.add_css_class("active");
        }

        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(8)
            .margin_end(8)
            .build();

        // 3-color preview pill: [bg, view, accent]
        let (bg, view, accent) = preset.preview_colors();
        let preview_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .height_request(24)
            .halign(gtk4::Align::Fill)
            .build();

        let chip_bg = gtk4::DrawingArea::builder().hexpand(true).build();
        chip_bg.set_draw_func(move |_, cr, w, h| {
            let (w, h, r) = (w as f64, h as f64, 4.0);
            let degrees = std::f64::consts::PI / 180.0;
            cr.new_sub_path();
            cr.arc(w - r, r, r, -90.0 * degrees, 0.0 * degrees);
            cr.arc(w - r, h - r, r, 0.0 * degrees, 90.0 * degrees);
            cr.arc(r, h - r, r, 90.0 * degrees, 180.0 * degrees);
            cr.arc(r, r, r, 180.0 * degrees, 270.0 * degrees);
            cr.close_path();
            if let Some(c) = crate::core::Color::from_hex(bg) {
                cr.set_source_rgba(c.r as f64, c.g as f64, c.b as f64, 1.0);
            }
            let _ = cr.fill();
        });

        let chip_view = gtk4::DrawingArea::builder().hexpand(true).build();
        chip_view.set_draw_func(move |_, cr, w, h| {
            let (w, h, r) = (w as f64, h as f64, 4.0);
            let degrees = std::f64::consts::PI / 180.0;
            cr.new_sub_path();
            cr.arc(w - r, r, r, -90.0 * degrees, 0.0 * degrees);
            cr.arc(w - r, h - r, r, 0.0 * degrees, 90.0 * degrees);
            cr.arc(r, h - r, r, 90.0 * degrees, 180.0 * degrees);
            cr.arc(r, r, r, 180.0 * degrees, 270.0 * degrees);
            cr.close_path();
            if let Some(c) = crate::core::Color::from_hex(view) {
                cr.set_source_rgba(c.r as f64, c.g as f64, c.b as f64, 1.0);
            }
            let _ = cr.fill();
        });

        let chip_acc = gtk4::DrawingArea::builder().hexpand(true).build();
        chip_acc.set_draw_func(move |_, cr, w, h| {
            let (w, h, r) = (w as f64, h as f64, 4.0);
            let degrees = std::f64::consts::PI / 180.0;
            cr.new_sub_path();
            cr.arc(w - r, r, r, -90.0 * degrees, 0.0 * degrees);
            cr.arc(w - r, h - r, r, 0.0 * degrees, 90.0 * degrees);
            cr.arc(r, h - r, r, 90.0 * degrees, 180.0 * degrees);
            cr.arc(r, r, r, 180.0 * degrees, 270.0 * degrees);
            cr.close_path();
            if let Some(c) = crate::core::Color::from_hex(accent) {
                cr.set_source_rgba(c.r as f64, c.g as f64, c.b as f64, 1.0);
            }
            let _ = cr.fill();
        });

        let card_header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::Fill)
            .build();

        preview_box.set_hexpand(true);
        preview_box.append(&chip_bg);
        preview_box.append(&chip_view);
        preview_box.append(&chip_acc);

        let check_icon = gtk4::Image::builder()
            .icon_name("object-select-symbolic")
            .css_classes(["theme-check-icon"])
            .valign(gtk4::Align::Center)
            .build();

        card_header.append(&preview_box);
        card_header.append(&check_icon);

        let lbl_t = gtk4::Label::builder()
            .label(&preset.title())
            .css_classes(["heading", "caption"])
            .halign(gtk4::Align::Start)
            .build();

        let lbl_s = gtk4::Label::builder()
            .label(&preset.subtitle())
            .css_classes(["dim-label"])
            .halign(gtk4::Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();

        vbox.append(&card_header);
        vbox.append(&lbl_t);
        vbox.append(&lbl_s);
        btn.set_child(Some(&vbox));

        theme_buttons.push((preset, btn.clone()));
        flow_box.append(&btn);
    }

    let all_btns = theme_buttons.clone();
    for (preset, btn) in theme_buttons {
        let all_b = all_btns.clone();
        let p_val = preset;
        btn.connect_clicked(move |_| {
            for (_, b) in &all_b {
                b.remove_css_class("active");
            }
            set_visual_theme(p_val);
            crate::core::AppSettings::set_visual_theme(p_val.id());
        });
    }

    theme_group.add(&flow_box);

    // ─────────────────────────────────────────────────────────────
    // 3. Ergonomics & Density (Scale + Toolbar Icon Sizes)
    // ─────────────────────────────────────────────────────────────
    let layout_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Interface and Ergonomics"))
        .description(crate::core::gettext(
            "Adjust icon sizing, interface density, and overall scale",
        ))
        .build();

    // Interface Scale Row
    let cur_scale = current_interface_scale();
    let scale_names = [
        crate::core::gettext("Compact (90%)"),
        crate::core::gettext("Default (100%)"),
        crate::core::gettext("Comfortable (110%)"),
        crate::core::gettext("Large (125%)"),
    ];
    let scale_str_list: Vec<&str> = scale_names.iter().map(|s| s.as_str()).collect();
    let scale_dd = gtk4::DropDown::from_strings(&scale_str_list);
    scale_dd.set_selected(cur_scale.to_index());
    scale_dd.set_valign(gtk4::Align::Center);
    scale_dd.connect_selected_notify(move |dd| {
        let scale = InterfaceScale::from_index(dd.selected());
        set_interface_scale(scale);
        crate::core::AppSettings::set_interface_scale(scale.id());
    });

    let scale_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Interface Scale"))
        .subtitle(crate::core::gettext("Scale controls, fonts, and panel proportions"))
        .build();
    scale_row.add_suffix(&scale_dd);
    layout_group.add(&scale_row);

    // Toolbar Icon Size Row
    let cur_icon_size = current_toolbar_icon_size();
    let icon_size_names = [
        crate::core::gettext("Small (16px)"),
        crate::core::gettext("Medium (20px)"),
        crate::core::gettext("Large (24px)"),
    ];
    let icon_size_str_list: Vec<&str> = icon_size_names.iter().map(|s| s.as_str()).collect();
    let icon_size_dd = gtk4::DropDown::from_strings(&icon_size_str_list);
    icon_size_dd.set_selected(cur_icon_size.to_index());
    icon_size_dd.set_valign(gtk4::Align::Center);
    icon_size_dd.connect_selected_notify(move |dd| {
        let size = ToolbarIconSize::from_index(dd.selected());
        set_toolbar_icon_size(size);
        crate::core::AppSettings::set_toolbar_icon_size(size.id());
    });

    let icon_size_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Toolbar Icon Size"))
        .subtitle(crate::core::gettext(
            "Change the pixel size of tools and HUD buttons",
        ))
        .build();
    icon_size_row.add_suffix(&icon_size_dd);
    layout_group.add(&icon_size_row);

    // ─────────────────────────────────────────────────────────────
    // 4. Accent Color Palette
    // ─────────────────────────────────────────────────────────────
    let accent_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Accent Color"))
        .description(crate::core::gettext(
            "Choose the highlight color for selectors and active elements",
        ))
        .build();

    let cur_accent = crate::ui::theme::current_custom_accent().unwrap_or_else(|| "system".to_string());

    let accent_chips = make_color_swatches(
        vec![
            ("System Default", "chip-system-accent", "system".to_string()),
            ("GNOME Blue", "chip-blue", "#3584e4".to_string()),
            ("Teal", "chip-teal", "#21a477".to_string()),
            ("Emerald Green", "chip-green", "#2ec27e".to_string()),
            ("Orange", "chip-orange", "#e66100".to_string()),
            ("Ruby Red", "chip-red", "#e01b24".to_string()),
            ("Magenta Pink", "chip-pink", "#e01b84".to_string()),
            ("Purple", "chip-purple", "#9141ac".to_string()),
            ("Slate", "chip-slate", "#62a0ea".to_string()),
            ("Amber Gold", "chip-gold", "#f6d32d".to_string()),
        ],
        cur_accent,
        |color_code| {
            if color_code == "system" {
                crate::ui::theme::set_custom_accent(None);
                crate::core::AppSettings::set_custom_accent_color(None);
            } else {
                crate::ui::theme::set_custom_accent(Some(color_code.clone()));
                crate::core::AppSettings::set_custom_accent_color(Some(&color_code));
            }
        },
    );

    let accent_row = adw::PreferencesRow::builder()
        .activatable(false)
        .selectable(false)
        .child(&accent_chips)
        .build();
    accent_group.add(&accent_row);

    // ─────────────────────────────────────────────────────────────
    // 5. Interface Icons Color Group
    // ─────────────────────────────────────────────────────────────
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
            ("Teal", "chip-teal", "#21a477".to_string()),
            ("Emerald Green", "chip-green", "#2ec27e".to_string()),
            ("Orange", "chip-orange", "#e66100".to_string()),
            ("Ruby Red", "chip-red", "#e01b24".to_string()),
            ("Amber Gold", "chip-gold", "#f6d32d".to_string()),
            ("Purple", "chip-purple", "#9141ac".to_string()),
            ("Slate", "chip-slate", "#62a0ea".to_string()),
        ],
        cur_icon_color,
        |color_code| {
            crate::ui::icons::set_interface_icon_color(&color_code);
        },
    );

    let icons_row = adw::PreferencesRow::builder()
        .activatable(false)
        .selectable(false)
        .child(&icon_color_chips)
        .build();
    icons_group.add(&icons_row);

    make_page(vec![
        scheme_group,
        theme_group,
        layout_group,
        accent_group,
        icons_group,
    ])
}
