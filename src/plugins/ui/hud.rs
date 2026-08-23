use gtk4::prelude::*;
use crate::plugins::traits::UiPlugin;

pub struct HudUiPlugin;

impl UiPlugin for HudUiPlugin {
    fn id(&self) -> &'static str {
        "hud"
    }

    fn name(&self) -> &'static str {
        "Status HUD"
    }

    fn create_overlay_widget(&self) -> Option<gtk4::Widget> {
        let label = gtk4::Label::builder()
            .label(&crate::core::gettext("Canvas Ready • Zoom: 100%"))
            .css_classes(["floating-hud", "caption", "numeric"])
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::End)
            .margin_start(16)
            .margin_bottom(16)
            .build();

        Some(label.upcast())
    }
}
