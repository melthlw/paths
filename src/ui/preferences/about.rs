use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use super::widgets::make_page;

pub fn build_about_page() -> gtk4::ScrolledWindow {
    let app_info_group = adw::PreferencesGroup::builder()
        .title("GNOME Paths")
        .description(crate::core::gettext(
            "Vector graphics and illustration editor for the GNOME desktop.",
        ))
        .build();

    let banner_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .margin_top(16)
        .margin_bottom(16)
        .build();

    let logo_img = gtk4::Image::from_icon_name("io.gitlab.lewisHeart.GnomePaths");
    logo_img.set_pixel_size(64);
    let name_lbl = gtk4::Label::builder()
        .label("GNOME Paths")
        .css_classes(["title-1"])
        .halign(gtk4::Align::Center)
        .build();
    let subtitle_lbl = gtk4::Label::builder()
        .label(crate::core::gettext(
            "Lightweight, vector design tool built with GTK4, Libadwaita, Rust, and the Skia 2D engine.",
        ))
        .css_classes(["dim-label", "caption"])
        .halign(gtk4::Align::Center)
        .justify(gtk4::Justification::Center)
        .wrap(true)
        .max_width_chars(50)
        .build();

    banner_box.append(&logo_img);
    banner_box.append(&name_lbl);
    banner_box.append(&subtitle_lbl);

    let banner_row = adw::ActionRow::builder().build();
    banner_row.set_child(Some(&banner_box));
    app_info_group.add(&banner_row);

    let version_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Version"))
        .subtitle("0.3.0-alpha (Modern Skia Engine)")
        .build();
    app_info_group.add(&version_row);

    let license_row = adw::ActionRow::builder()
        .title(crate::core::gettext("License"))
        .subtitle("GNU General Public License v3.0 or later (GPL-3.0-or-later)")
        .build();
    app_info_group.add(&license_row);

    let tech_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Technologies"))
        .subtitle("Rust • GTK4 • Libadwaita • Skia 2D • GPU Acceleration • Cairo • GIO")
        .build();
    app_info_group.add(&tech_row);

    let repo_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Source Code"))
        .subtitle("https://gitlab.com/lewisHeart/gnome-paths")
        .activatable(true)
        .build();
    let link_icon = gtk4::Image::builder()
        .icon_name("link-external-symbolic")
        .valign(gtk4::Align::Center)
        .build();
    repo_row.add_suffix(&link_icon);

    {
        repo_row.connect_activated(|_| {
            let _ = gtk4::gio::AppInfo::launch_default_for_uri(
                "https://gitlab.com/lewisHeart/gnome-paths",
                None::<&gtk4::gio::AppLaunchContext>,
            );
        });
    }
    app_info_group.add(&repo_row);

    let support_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Support Development"))
        .subtitle("https://ko-fi.com/lauel")
        .activatable(true)
        .build();
    let heart_icon = gtk4::Image::builder()
        .icon_name("favorite-symbolic")
        .valign(gtk4::Align::Center)
        .build();
    support_row.add_suffix(&heart_icon);

    {
        support_row.connect_activated(|_| {
            let _ = gtk4::gio::AppInfo::launch_default_for_uri(
                "https://ko-fi.com/lauel",
                None::<&gtk4::gio::AppLaunchContext>,
            );
        });
    }
    app_info_group.add(&support_row);

    make_page(vec![app_info_group])
}
