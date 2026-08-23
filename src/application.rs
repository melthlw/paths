use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::ui::DesignWindow;

pub const APP_ID: &str = "io.github.lewis.GnomePaths";
pub const RESOURCE_PATH: &str = "/io/github/lewis/GnomePaths";

pub struct DesignApplication {
    app: adw::Application,
}

impl DesignApplication {
    pub fn new() -> Self {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .flags(gio::ApplicationFlags::empty())
            .build();

        let design_app = Self { app };
        design_app.setup_signals();
        design_app
    }

    pub fn run(&self) -> glib::ExitCode {
        self.app.run()
    }

    fn setup_signals(&self) {
        self.app.connect_startup(|app| {
            // Follow system color scheme (light or dark according to user settings)
            adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default);

            Self::load_css();
            Self::setup_actions(app);
        });

        self.app.connect_activate(|app| {
            if let Some(window) = app.active_window() {
                window.present();
            } else {
                let window = DesignWindow::new(app);
                window.present();
            }
        });
    }

    fn load_css() {
        let provider = gtk4::CssProvider::new();
        let resource_css = format!("{}/style.css", RESOURCE_PATH);
        provider.load_from_resource(&resource_css);

        if let Some(display) = gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            let icon_theme = gtk4::IconTheme::for_display(&display);
            icon_theme.add_resource_path(RESOURCE_PATH);
            icon_theme.add_resource_path(&format!("{}/icons", RESOURCE_PATH));
        }
    }

    fn setup_actions(app: &adw::Application) {
        // Quit action
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(|app: &adw::Application, _, _| {
                app.quit();
            })
            .build();

        // About action
        let about_action = gio::ActionEntry::builder("about")
            .activate(|app: &adw::Application, _, _| {
                Self::show_about_dialog(app);
            })
            .build();

        // Shortcuts action
        let shortcuts_action = gio::ActionEntry::builder("shortcuts")
            .activate(|app: &adw::Application, _, _| {
                Self::show_shortcuts_dialog(app);
            })
            .build();

        app.add_action_entries([quit_action, about_action, shortcuts_action]);

        app.set_accels_for_action("app.quit", &["<Control>q"]);
        app.set_accels_for_action("app.shortcuts", &["<Control>question"]);
    }

    fn show_about_dialog(app: &adw::Application) {
        let active_window = app.active_window();

        let dialog = adw::AboutDialog::builder()
            .application_name("GNOME Paths")
            .application_icon("io.github.lewis.GnomePaths")
            .developer_name("Lewis")
            .version("0.1.0")
            .copyright("© 2026 Lewis")
            .license_type(gtk4::License::Gpl30Only)
            .comments(&crate::core::gettext(
                "Modern vector design editor accelerated by Skia GPU, built with GTK4, Libadwaita and Rust.",
            ))
            .website("https://github.com/lewis/gnome-paths")
            .issue_url("https://github.com/lewis/gnome-paths/issues")
            .build();

        if let Some(win) = active_window.as_ref() {
            dialog.present(Some(win));
        } else {
            dialog.present(gtk4::Widget::NONE);
        }
    }

    #[allow(deprecated)]
    fn show_shortcuts_dialog(app: &adw::Application) {
        let resource_ui = format!("{}/ui/shortcuts.ui", RESOURCE_PATH);
        let builder = gtk4::Builder::from_resource(&resource_ui);
        if let Some(shortcuts_dialog) = builder.object::<gtk4::ShortcutsWindow>("shortcuts_dialog")
        {
            if let Some(win) = app.active_window() {
                shortcuts_dialog.set_transient_for(Some(&win));
                shortcuts_dialog.present();
            } else {
                shortcuts_dialog.present();
            }
        }
    }
}
