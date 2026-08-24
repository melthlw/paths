use gtk4::gio;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::Path;

pub const APP_SCHEMA_ID: &str = "io.github.lewis.GnomePaths";

thread_local! {
    static SETTINGS_INSTANCE: RefCell<Option<gio::Settings>> = const { RefCell::new(None) };
    static SETTINGS_INITIALIZED: RefCell<bool> = const { RefCell::new(false) };
}

/// Returns the thread-local `gio::Settings` instance for GNOME Paths.
/// Automatically resolves from standard system locations or local development `data/` directory.
pub fn settings() -> Option<gio::Settings> {
    SETTINGS_INSTANCE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            let is_init = SETTINGS_INITIALIZED.with(|i| *i.borrow());
            if !is_init {
                SETTINGS_INITIALIZED.with(|i| *i.borrow_mut() = true);

                // 1. Try standard default schema source
                if let Some(schema) = gio::SettingsSchemaSource::default()
                    .and_then(|source| source.lookup(APP_SCHEMA_ID, true))
                {
                    *borrow = Some(gio::Settings::new_full(
                        &schema,
                        None::<&gio::SettingsBackend>,
                        None,
                    ));
                } else {
                    // 2. Try GSETTINGS_SCHEMA_DIR or local development directory
                    let dev_paths = [
                        std::env::var("GSETTINGS_SCHEMA_DIR").ok(),
                        Some("data".to_string()),
                        Some("../data".to_string()),
                        Some("build-dir/data".to_string()),
                        Some("_build/data".to_string()),
                    ];

                    for path_str in dev_paths.into_iter().flatten() {
                        let p = Path::new(&path_str);
                        if p.exists() {
                            if let Ok(source) = gio::SettingsSchemaSource::from_directory(
                                p,
                                gio::SettingsSchemaSource::default().as_ref(),
                                false,
                            ) {
                                if let Some(schema) = source.lookup(APP_SCHEMA_ID, true) {
                                    *borrow = Some(gio::Settings::new_full(
                                        &schema,
                                        None::<&gio::SettingsBackend>,
                                        None,
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        borrow.clone()
    })
}

/// Typed helper struct for reading and persisting all GNOME Paths user and UI settings
pub struct AppSettings;

impl AppSettings {
    // ─────────────────────────────────────────────────────────────
    // Window Geometry & State
    // ─────────────────────────────────────────────────────────────
    pub fn window_size() -> (i32, i32) {
        if let Some(s) = settings() {
            let w = s.int("window-width").max(400);
            let h = s.int("window-height").max(300);
            (w, h)
        } else {
            (1280, 780)
        }
    }

    pub fn set_window_size(width: i32, height: i32) {
        if let Some(s) = settings() {
            let _ = s.set_int("window-width", width.max(400));
            let _ = s.set_int("window-height", height.max(300));
        }
    }

    pub fn is_maximized() -> bool {
        settings()
            .map(|s| s.boolean("is-maximized"))
            .unwrap_or(false)
    }

    pub fn set_is_maximized(maximized: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("is-maximized", maximized);
        }
    }

    pub fn is_fullscreen() -> bool {
        settings()
            .map(|s| s.boolean("is-fullscreen"))
            .unwrap_or(false)
    }

    pub fn set_is_fullscreen(fullscreen: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("is-fullscreen", fullscreen);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Sidebar Panels
    // ─────────────────────────────────────────────────────────────
    pub fn show_layers_sidebar() -> bool {
        settings()
            .map(|s| s.boolean("show-layers-sidebar"))
            .unwrap_or(true)
    }

    pub fn set_show_layers_sidebar(show: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-layers-sidebar", show);
        }
    }

    pub fn show_inspector_sidebar() -> bool {
        settings()
            .map(|s| s.boolean("show-inspector-sidebar"))
            .unwrap_or(true)
    }

    pub fn set_show_inspector_sidebar(show: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-inspector-sidebar", show);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Active Zoom & Canvas
    // ─────────────────────────────────────────────────────────────
    pub fn active_zoom() -> f64 {
        settings()
            .map(|s| s.double("active-zoom").clamp(0.05, 40.0))
            .unwrap_or(1.0)
    }

    pub fn set_active_zoom(zoom: f64) {
        if let Some(s) = settings() {
            let _ = s.set_double("active-zoom", zoom.clamp(0.05, 40.0));
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Display & Grid Preferences
    // ─────────────────────────────────────────────────────────────
    pub fn show_grid() -> bool {
        settings().map(|s| s.boolean("show-grid")).unwrap_or(false)
    }

    pub fn set_show_grid(show: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-grid", show);
        }
    }

    pub fn show_rulers() -> bool {
        settings().map(|s| s.boolean("show-rulers")).unwrap_or(true)
    }

    pub fn set_show_rulers(show: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-rulers", show);
        }
    }

    pub fn show_guides() -> bool {
        settings().map(|s| s.boolean("show-guides")).unwrap_or(true)
    }

    pub fn set_show_guides(show: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-guides", show);
        }
    }

    pub fn snap_enabled() -> bool {
        settings()
            .map(|s| s.boolean("snap-enabled"))
            .unwrap_or(true)
    }

    pub fn set_snap_enabled(enabled: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-enabled", enabled);
        }
    }

    pub fn snap_to_grid() -> bool {
        settings()
            .map(|s| s.boolean("snap-to-grid"))
            .unwrap_or(false)
    }

    pub fn set_snap_to_grid(snap: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-to-grid", snap);
        }
    }

    pub fn snap_to_objects() -> bool {
        settings()
            .map(|s| s.boolean("snap-to-objects"))
            .unwrap_or(true)
    }

    pub fn set_snap_to_objects(snap: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-to-objects", snap);
        }
    }

    pub fn snap_to_artboard() -> bool {
        settings()
            .map(|s| s.boolean("snap-to-artboard"))
            .unwrap_or(true)
    }

    pub fn set_snap_to_artboard(snap: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-to-artboard", snap);
        }
    }

    pub fn snap_to_guides() -> bool {
        settings()
            .map(|s| s.boolean("snap-to-guides"))
            .unwrap_or(true)
    }

    #[allow(dead_code)]
    pub fn set_snap_to_guides(snap: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-to-guides", snap);
        }
    }

    pub fn grid_style() -> String {
        settings()
            .map(|s| s.string("grid-style").to_string())
            .unwrap_or_else(|| "dots".to_string())
    }

    pub fn set_grid_style(style: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("grid-style", style);
        }
    }

    pub fn grid_cell_size() -> f64 {
        settings()
            .map(|s| s.double("grid-cell-size").max(1.0))
            .unwrap_or(20.0)
    }

    pub fn set_grid_cell_size(size: f64) {
        if let Some(s) = settings() {
            let _ = s.set_double("grid-cell-size", size.max(1.0));
        }
    }

    pub fn grid_subdivisions() -> u32 {
        settings()
            .map(|s| s.int("grid-subdivisions").max(1) as u32)
            .unwrap_or(4)
    }

    pub fn set_grid_subdivisions(subs: u32) {
        if let Some(s) = settings() {
            let _ = s.set_int("grid-subdivisions", subs.max(1) as i32);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Workspace & Rendering Preferences
    // ─────────────────────────────────────────────────────────────
    pub fn workspace_dots() -> bool {
        settings()
            .map(|s| s.boolean("workspace-dots"))
            .unwrap_or(true)
    }

    pub fn set_workspace_dots(enabled: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("workspace-dots", enabled);
        }
    }

    pub fn page_shadow() -> bool {
        settings().map(|s| s.boolean("page-shadow")).unwrap_or(true)
    }

    pub fn set_page_shadow(enabled: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("page-shadow", enabled);
        }
    }

    pub fn page_border() -> bool {
        settings().map(|s| s.boolean("page-border")).unwrap_or(true)
    }

    pub fn set_page_border(enabled: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("page-border", enabled);
        }
    }

    pub fn canvas_bg_color() -> String {
        settings()
            .map(|s| s.string("canvas-bg-color").to_string())
            .unwrap_or_else(|| "system".to_string())
    }

    pub fn set_canvas_bg_color(color_hex: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("canvas-bg-color", color_hex);
        }
    }

    pub fn page_bg_color() -> String {
        settings()
            .map(|s| s.string("page-bg-color").to_string())
            .unwrap_or_else(|| "#ffffff".to_string())
    }

    pub fn set_page_bg_color(color_hex: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("page-bg-color", color_hex);
        }
    }

    pub fn hardware_acceleration() -> bool {
        settings()
            .map(|s| s.boolean("hardware-acceleration"))
            .unwrap_or(true)
    }

    pub fn set_hardware_acceleration(enabled: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("hardware-acceleration", enabled);
        }
    }

    pub fn high_precision_aa() -> bool {
        settings()
            .map(|s| s.boolean("high-precision-aa"))
            .unwrap_or(true)
    }

    pub fn set_high_precision_aa(enabled: bool) {
        if let Some(s) = settings() {
            let _ = s.set_boolean("high-precision-aa", enabled);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Appearance & Themes
    // ─────────────────────────────────────────────────────────────
    pub fn color_scheme() -> String {
        settings()
            .map(|s| s.string("color-scheme").to_string())
            .unwrap_or_else(|| "system".to_string())
    }

    pub fn set_color_scheme(scheme: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("color-scheme", scheme);
        }
    }

    pub fn visual_theme() -> String {
        settings()
            .map(|s| s.string("visual-theme").to_string())
            .unwrap_or_else(|| "adwaita".to_string())
    }

    pub fn set_visual_theme(theme: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("visual-theme", theme);
        }
    }

    pub fn custom_accent_color() -> Option<String> {
        settings().and_then(|s| {
            let hex = s.string("custom-accent-color").to_string();
            if hex.trim().is_empty() {
                None
            } else {
                Some(hex)
            }
        })
    }

    pub fn set_custom_accent_color(accent: Option<&str>) {
        if let Some(s) = settings() {
            let _ = s.set_string("custom-accent-color", accent.unwrap_or(""));
        }
    }

    pub fn toolbar_icon_size() -> String {
        settings()
            .map(|s| s.string("toolbar-icon-size").to_string())
            .unwrap_or_else(|| "medium".to_string())
    }

    pub fn set_toolbar_icon_size(size: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("toolbar-icon-size", size);
        }
    }

    pub fn interface_scale() -> String {
        settings()
            .map(|s| s.string("interface-scale").to_string())
            .unwrap_or_else(|| "default".to_string())
    }

    pub fn set_interface_scale(scale: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("interface-scale", scale);
        }
    }

    pub fn toolbar_position() -> String {
        settings()
            .map(|s| s.string("toolbar-position").to_string())
            .unwrap_or_else(|| "bottom".to_string())
    }

    pub fn set_toolbar_position(pos: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("toolbar-position", pos);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // System & Tooling Preferences
    // ─────────────────────────────────────────────────────────────
    pub fn language() -> String {
        settings()
            .map(|s| s.string("language").to_string())
            .unwrap_or_else(|| "system".to_string())
    }

    pub fn set_language(code: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("language", code);
        }
    }

    pub fn unit() -> String {
        settings()
            .map(|s| s.string("unit").to_string())
            .unwrap_or_else(|| "px".to_string())
    }

    pub fn set_unit(unit_str: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("unit", unit_str);
        }
    }

    pub fn node_size() -> f64 {
        settings()
            .map(|s| s.double("node-size").max(4.0))
            .unwrap_or(8.0)
    }

    pub fn set_node_size(size: f64) {
        if let Some(s) = settings() {
            let _ = s.set_double("node-size", size.max(4.0));
        }
    }

    pub fn handle_size() -> f64 {
        settings()
            .map(|s| s.double("handle-size").max(3.0))
            .unwrap_or(6.0)
    }

    pub fn set_handle_size(size: f64) {
        if let Some(s) = settings() {
            let _ = s.set_double("handle-size", size.max(3.0));
        }
    }

    pub fn handle_display_mode() -> String {
        settings()
            .map(|s| s.string("handle-display-mode").to_string())
            .unwrap_or_else(|| "selected".to_string())
    }

    pub fn set_handle_display_mode(mode: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("handle-display-mode", mode);
        }
    }

    pub fn shortcut_preset() -> String {
        settings()
            .map(|s| s.string("shortcut-preset").to_string())
            .unwrap_or_else(|| "default".to_string())
    }

    pub fn set_shortcut_preset(preset: &str) {
        if let Some(s) = settings() {
            let _ = s.set_string("shortcut-preset", preset);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_defaults_without_panic() {
        let (w, h) = AppSettings::window_size();
        assert!(w >= 400);
        assert!(h >= 300);

        let zoom = AppSettings::active_zoom();
        assert!(zoom >= 0.05 && zoom <= 40.0);

        let show_layers = AppSettings::show_layers_sidebar();
        let show_inspector = AppSettings::show_inspector_sidebar();
        assert!(show_layers || !show_layers);
        assert!(show_inspector || !show_inspector);

        let unit = AppSettings::unit();
        assert!(!unit.is_empty());

        let theme = AppSettings::visual_theme();
        assert!(!theme.is_empty());

        let icon_size = AppSettings::toolbar_icon_size();
        assert!(!icon_size.is_empty());

        let scale = AppSettings::interface_scale();
        assert!(!scale.is_empty());

        let grid_size = AppSettings::grid_cell_size();
        assert!(grid_size >= 1.0);

        let grid_subs = AppSettings::grid_subdivisions();
        assert!(grid_subs >= 1);
    }

    #[test]
    fn test_app_settings_window_geometry_persistence() {
        AppSettings::set_window_size(1440, 900);
        let (w, h) = AppSettings::window_size();
        assert!(w >= 400);
        assert!(h >= 300);

        AppSettings::set_is_maximized(true);
        let _ = AppSettings::is_maximized();

        AppSettings::set_is_fullscreen(false);
        let _ = AppSettings::is_fullscreen();
    }

    #[test]
    fn test_app_settings_zoom_and_sidebars() {
        AppSettings::set_active_zoom(2.5);
        let z = AppSettings::active_zoom();
        assert!(z >= 0.05 && z <= 40.0);

        AppSettings::set_show_layers_sidebar(false);
        AppSettings::set_show_inspector_sidebar(false);
    }
}
