mod application;
mod core;
mod plugins;
mod ui;

use application::DesignApplication;
use gtk4::gio;
use gtk4::glib;

fn main() -> glib::ExitCode {
    gio::resources_register_include!("gnome_paths.gresource")
        .expect("Failed to register gresources bundle");

    core::init_i18n();

    let app = DesignApplication::new();
    app.run()
}
