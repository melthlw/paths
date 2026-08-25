mod application;
mod core;
mod plugins;
mod ui;

use application::DesignApplication;
use gtk4::gio;
use gtk4::glib;

fn main() -> glib::ExitCode {
    // Silence non-actionable GTK system theme & Vulkan driver warnings
    glib::log_set_writer_func(|level, fields| {
        for field in fields {
            if let Some(msg) = field.value_str() {
                if msg.contains("Theme directory") || msg.contains("vkAcquireNextImageKHR") {
                    return glib::LogWriterOutput::Handled;
                }
            }
        }
        glib::log_writer_default(level, fields)
    });

    gio::resources_register_include!("gnome_paths.gresource")
        .expect("Failed to register gresources bundle");

    core::init_i18n();

    let app = DesignApplication::new();
    app.run()
}
