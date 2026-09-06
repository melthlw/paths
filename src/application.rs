use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

pub const APP_ID: &str = "io.gitlab.lewisHeart.Paths";
pub const RESOURCE_PATH: &str = "/io/gitlab/lewisHeart/Paths";

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
            // Apply persisted language
            let lang_code = crate::core::AppSettings::language();
            if lang_code != "system" {
                crate::core::set_language(crate::core::Language::from_code(&lang_code));
            }

            // Apply persisted color scheme
            let scheme_str = crate::core::AppSettings::color_scheme();
            match scheme_str.as_str() {
                "light" => {
                    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceLight)
                }
                "dark" => {
                    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark)
                }
                _ => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default),
            }

            // Apply persisted visual theme & custom accent
            let theme_str = crate::core::AppSettings::visual_theme();
            let theme = crate::ui::theme::VisualThemePreset::from_id(&theme_str);
            crate::ui::theme::set_visual_theme(theme);

            let custom_accent = crate::core::AppSettings::custom_accent_color();
            crate::ui::theme::set_custom_accent(custom_accent);

            // Apply persisted toolbar icon size & interface scale
            let icon_size_str = crate::core::AppSettings::toolbar_icon_size();
            let icon_size = crate::ui::theme::ToolbarIconSize::from_id(&icon_size_str);
            crate::ui::theme::set_toolbar_icon_size(icon_size);

            let scale_str = crate::core::AppSettings::interface_scale();
            let scale = crate::ui::theme::InterfaceScale::from_id(&scale_str);
            crate::ui::theme::set_interface_scale(scale);

            // Apply persisted interface icon color
            let icon_color = crate::core::AppSettings::interface_icon_color();
            crate::ui::icons::set_interface_icon_color(&icon_color);

            Self::load_css();
            Self::setup_actions(app);
        });

        self.app.connect_activate(|app| {
            if let Some(window) = app.active_window() {
                window.set_visible(true);
                window.present();
            } else {
                crate::ui::welcome::show_welcome_app_window(app);
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
            icon_theme.add_resource_path(&format!("{}/icons/hicolor", RESOURCE_PATH));
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
            .application_name("Paths")
            .application_icon("io.gitlab.lewisHeart.GnomePaths")
            .developer_name("Lewis")
            .version("0.4.5")
            .copyright("© 2026 Lewis")
            .license_type(gtk4::License::Gpl30Only)
            .comments(&crate::core::gettext(
                "Modern vector design editor accelerated by Skia GPU, built with GTK4, Libadwaita and Rust.",
            ))
            .website("https://gitlab.com/lewisHeart/gnome-paths")
            .issue_url("https://gitlab.com/lewisHeart/gnome-paths/-/issues")
            .build();

        if let Some(win) = active_window.as_ref() {
            dialog.present(Some(win));
        } else {
            dialog.present(gtk4::Widget::NONE);
        }
    }

    fn show_shortcuts_dialog(app: &adw::Application) {
        let resource_ui = format!("{}/ui/shortcuts.ui", RESOURCE_PATH);
        let builder = gtk4::Builder::from_resource(&resource_ui);
        if let Some(shortcuts_dialog) = builder.object::<gtk4::Window>("shortcuts_dialog") {
            if let Some(win) = app.active_window() {
                shortcuts_dialog.set_transient_for(Some(&win));
            }
            shortcuts_dialog.present();
        }
    }
}
