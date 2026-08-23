use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use super::widgets::make_page;
use crate::ui::canvas::CanvasWidget;

pub fn build_plugins_page(window: &adw::Window, canvas: &CanvasWidget) -> gtk4::ScrolledWindow {
    let plugins_list_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Installed Plugins"))
        .description(crate::core::gettext(
            "Manage external extensions added to the studio",
        ))
        .build();

    let external_list = {
        let state = canvas.state();
        let state_b = state.borrow();
        state_b.plugin_manager.external_plugins()
    };

    if external_list.is_empty() {
        let empty_row = adw::ActionRow::builder()
            .title(crate::core::gettext("No external plugins installed"))
            .subtitle(crate::core::gettext("Add a new plugin to manage."))
            .build();
        plugins_list_group.add(&empty_row);
    } else {
        for (id, name, enabled) in external_list {
            let row = adw::SwitchRow::builder()
                .title(name)
                .subtitle(format!(
                    "{}: {} • {}",
                    crate::core::gettext("Identifier"),
                    id,
                    crate::core::gettext("External extension")
                ))
                .active(enabled)
                .build();
            plugins_list_group.add(&row);
        }
    }

    let plugin_mgmt_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Extension Management"))
        .build();

    let add_plugin_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Install plugin"))
        .subtitle(crate::core::gettext("Import plugins (.so)"))
        .activatable(true)
        .build();
    let add_icon = gtk4::Image::builder()
        .icon_name("list-add-symbolic")
        .valign(gtk4::Align::Center)
        .build();
    add_plugin_row.add_suffix(&add_icon);

    {
        let window_weak = window.downgrade();
        let canvas_imp = canvas.clone();
        add_plugin_row.connect_activated(move |_| {
            if let Some(win) = window_weak.upgrade() {
                let dialog = gtk4::FileDialog::builder()
                    .title(crate::core::gettext("Select Native Plugin (.so)"))
                    .modal(true)
                    .build();

                let filter = gtk4::FileFilter::new();
                filter.set_name(Some(&crate::core::gettext("Shared Library (*.so)")));
                filter.add_pattern("*.so");

                let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
                filters.append(&filter);
                dialog.set_filters(Some(&filters));

                let canvas_cb = canvas_imp.clone();
                dialog.open(Some(&win), gtk4::gio::Cancellable::NONE, move |res| {
                    if let Ok(file) = res {
                        if let Some(src_path) = file.path() {
                            let plugins_dir = crate::plugins::external::get_plugins_dir();
                            let _ = std::fs::create_dir_all(&plugins_dir);
                            if let Some(file_name) = src_path.file_name() {
                                let dest_path = plugins_dir.join(file_name);
                                let _ = std::fs::copy(&src_path, &dest_path);
                                unsafe {
                                    if let Ok(ext) =
                                        crate::plugins::external::ExternalPlugin::load(&dest_path)
                                    {
                                        canvas_cb
                                            .state()
                                            .borrow_mut()
                                            .plugin_manager
                                            .register_external(ext.plugin);
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });
    }

    plugin_mgmt_group.add(&add_plugin_row);

    let folder_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Open plugins folder"))
        .subtitle(crate::core::gettext(
            "Opens local extension folder in file manager",
        ))
        .activatable(true)
        .build();
    let folder_icon = gtk4::Image::builder()
        .icon_name("folder-open-symbolic")
        .valign(gtk4::Align::Center)
        .build();
    folder_row.add_suffix(&folder_icon);

    {
        folder_row.connect_activated(move |_| {
            let plugins_dir = crate::plugins::external::get_plugins_dir();
            let _ = std::fs::create_dir_all(&plugins_dir);
            if let Some(uri) = plugins_dir.to_str() {
                let _ = gtk4::gio::AppInfo::launch_default_for_uri(
                    &format!("file://{}", uri),
                    None::<&gtk4::gio::AppLaunchContext>,
                );
            }
        });
    }

    plugin_mgmt_group.add(&folder_row);

    make_page(vec![plugins_list_group, plugin_mgmt_group])
}
