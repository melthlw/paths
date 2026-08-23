use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use super::widgets::{make_page, show_language_chooser_dialog};
use crate::ui::canvas::CanvasWidget;

pub fn build_general_page(window: &adw::Window, canvas: &CanvasWidget) -> gtk4::ScrolledWindow {
    let lang_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Language"))
        .description(crate::core::gettext(
            "Change application interface language and number formatting",
        ))
        .build();

    let cur_info = crate::core::get_language().info();

    let lang_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Application Language"))
        .subtitle(crate::core::gettext(
            "Change user interface language and regional translation",
        ))
        .activatable(true)
        .build();

    let lang_icon = gtk4::Image::from_icon_name("preferences-desktop-locale-symbolic");
    lang_icon.set_pixel_size(18);
    lang_row.add_prefix(&lang_icon);

    let lang_btn_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .valign(gtk4::Align::Center)
        .build();

    let lang_lbl = gtk4::Label::builder()
        .label(cur_info.native_name)
        .css_classes(["heading"])
        .build();

    let lang_code_badge = gtk4::Label::builder()
        .label(cur_info.code)
        .css_classes(["dim-label", "numeric"])
        .build();

    let chevron_icon = gtk4::Image::from_icon_name("go-next-symbolic");
    chevron_icon.set_pixel_size(14);
    chevron_icon.add_css_class("dim-label");

    lang_btn_box.append(&lang_lbl);
    lang_btn_box.append(&lang_code_badge);
    lang_btn_box.append(&chevron_icon);

    let lang_btn = gtk4::Button::builder()
        .css_classes(["flat"])
        .child(&lang_btn_box)
        .valign(gtk4::Align::Center)
        .tooltip_text(crate::core::gettext("Select language..."))
        .build();

    {
        let win_c = window.clone();
        let lbl_c = lang_lbl.clone();
        let bdg_c = lang_code_badge.clone();
        let trigger_chooser = move || {
            show_language_chooser_dialog(&win_c, lbl_c.clone(), bdg_c.clone());
        };
        let tc_btn = trigger_chooser.clone();
        lang_btn.connect_clicked(move |_| tc_btn());
        let tc_row = trigger_chooser.clone();
        lang_row.connect_activated(move |_| tc_row());
    }

    lang_row.add_suffix(&lang_btn);
    lang_group.add(&lang_row);

    let perf_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Performance"))
        .build();

    let hw_accel_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Hardware Acceleration"))
        .subtitle(crate::core::gettext("Render canvas using GPU"))
        .active(canvas.is_hardware_accelerated())
        .build();
    {
        let canvas_c = canvas.clone();
        hw_accel_row.connect_active_notify(move |sw| {
            canvas_c.set_hardware_accelerated(sw.is_active());
        });
    }
    perf_group.add(&hw_accel_row);

    let msaa_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Antialiasing"))
        .subtitle(crate::core::gettext("Smooth vector paths and curves"))
        .active(canvas.is_high_precision_aa())
        .build();
    {
        let canvas_c = canvas.clone();
        msaa_row.connect_active_notify(move |sw| {
            canvas_c.set_high_precision_aa(sw.is_active());
        });
    }
    perf_group.add(&msaa_row);

    make_page(vec![lang_group, perf_group])
}
