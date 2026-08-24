use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use super::widgets::make_page;
use crate::ui::canvas::CanvasWidget;

pub fn build_toolbars_page(window: &adw::Window, canvas: &CanvasWidget) -> gtk4::ScrolledWindow {
    let toolbars_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Toolbar Customization"))
        .description(crate::core::gettext(
            "Manage on-screen tools, folders, and layout arrangement",
        ))
        .build();

    let customize_tools_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Customize Tools and Groups"))
        .subtitle(crate::core::gettext(
            "Add, remove, reorder tools, or organize them into folders",
        ))
        .activatable(true)
        .build();
    let customize_btn = gtk4::Button::builder()
        .label(crate::core::gettext("Customize..."))
        .icon_name("prefs-toolbars-symbolic")
        .valign(gtk4::Align::Center)
        .css_classes(["suggested-action"])
        .build();
    {
        let canvas_t = canvas.clone();
        let win_t = window.clone();
        let open_cust = move || {
            crate::ui::toolbar::show_customize_toolbar_dialog_standalone(&win_t, canvas_t.clone());
        };
        let oc_btn = open_cust.clone();
        customize_btn.connect_clicked(move |_| oc_btn());
        customize_tools_row.connect_activated(move |_| open_cust());
    }
    customize_tools_row.add_suffix(&customize_btn);
    toolbars_group.add(&customize_tools_row);

    let tip_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Quick Repositioning Tip"))
        .subtitle(crate::core::gettext(
            "You can drag the toolbar handle on the canvas or right-click it to quickly dock it to the top, bottom, left, or right edges.",
        ))
        .build();
    let tip_icon = gtk4::Image::from_icon_name("info-symbolic");
    tip_icon.set_pixel_size(16);
    tip_row.add_prefix(&tip_icon);
    toolbars_group.add(&tip_row);

    make_page(vec![toolbars_group])
}
