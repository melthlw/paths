use gtk4::gio;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

pub const APP_SCHEMA_ID: &str = "io.github.lewis.GnomePaths";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    // Window Geometry & State
    pub window_width: i32,
    pub window_height: i32,
    pub is_maximized: bool,
    pub is_fullscreen: bool,

    // Sidebar Panels
    pub show_layers_sidebar: bool,
    pub show_inspector_sidebar: bool,

    // Inspector Tab Layout
    pub inspector_tab_locations: Vec<String>,
    pub inspector_active_tabs: Vec<usize>,
    pub inspector_tab_order: Vec<usize>,

    // Active Zoom & Viewport
    pub active_zoom: f64,

    // Display & Grid Preferences
    pub show_grid: bool,
    pub show_rulers: bool,
    pub show_guides: bool,
    pub snap_enabled: bool,
    pub snap_to_grid: bool,
    pub snap_to_objects: bool,
    pub snap_to_artboard: bool,
    pub snap_to_guides: bool,
    pub grid_style: String,
    pub grid_cell_size: f64,
    pub grid_subdivisions: i32,

    // Workspace & Rendering Preferences
    pub workspace_dots: bool,
    pub page_shadow: bool,
    pub page_border: bool,
    pub canvas_bg_color: String,
    pub page_bg_color: String,
    pub hardware_acceleration: bool,
    pub high_precision_aa: bool,

    // Appearance & Theme Preferences
    pub color_scheme: String,
    pub visual_theme: String,
    pub custom_accent_color: String,
    pub interface_icon_color: String,
    pub toolbar_icon_size: String,
    pub interface_scale: String,
    pub toolbar_position: String,

    // Plugins & Tooling Preferences
    pub enabled_plugins: Vec<String>,
    pub toolbar_customization: Option<String>,
    pub language: String,
    pub unit: String,
    pub node_size: f64,
    pub handle_size: f64,
    pub handle_display_mode: String,
    pub shortcut_preset: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 780,
            is_maximized: false,
            is_fullscreen: false,
            show_layers_sidebar: true,
            show_inspector_sidebar: true,
            inspector_tab_locations: vec![
                "docked:0".to_string(), // Appearance
                "docked:0".to_string(), // Alignment
                "closed".to_string(),   // Transform
                "closed".to_string(),   // Clones
                "closed".to_string(),   // Export
                "closed".to_string(),   // Libraries
            ],
            inspector_active_tabs: vec![0, 1, 2, 3, 4, 5],
            inspector_tab_order: vec![0, 1, 2, 3, 4, 5],
            active_zoom: 1.0,
            show_grid: false,
            show_rulers: true,
            show_guides: true,
            snap_enabled: true,
            snap_to_grid: false,
            snap_to_objects: true,
            snap_to_artboard: true,
            snap_to_guides: true,
            grid_style: "dots".to_string(),
            grid_cell_size: 20.0,
            grid_subdivisions: 4,
            workspace_dots: true,
            page_shadow: true,
            page_border: true,
            canvas_bg_color: "system".to_string(),
            page_bg_color: "#ffffff".to_string(),
            hardware_acceleration: true,
            high_precision_aa: true,
            color_scheme: "system".to_string(),
            visual_theme: "adwaita".to_string(),
            custom_accent_color: "".to_string(),
            interface_icon_color: "default".to_string(),
            toolbar_icon_size: "medium".to_string(),
            interface_scale: "default".to_string(),
            toolbar_position: "bottom".to_string(),
            enabled_plugins: vec![],
            toolbar_customization: None,
            language: "system".to_string(),
            unit: "px".to_string(),
            node_size: 8.0,
            handle_size: 6.0,
            handle_display_mode: "selected".to_string(),
            shortcut_preset: "default".to_string(),
        }
    }
}

thread_local! {
    static SETTINGS_INSTANCE: RefCell<Option<gio::Settings>> = const { RefCell::new(None) };
    static SETTINGS_INITIALIZED: RefCell<bool> = const { RefCell::new(false) };
    static GLOBAL_CONFIG: RefCell<AppConfig> = RefCell::new(load_initial_config());
}

pub fn config_file_path() -> PathBuf {
    glib::user_config_dir().join("gnome-paths").join("settings.json")
}

fn load_initial_config() -> AppConfig {
    // 1. Try reading from ~/.config/gnome-paths/settings.json
    let path = config_file_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(loaded) = serde_json::from_str::<AppConfig>(&content) {
                return loaded;
            }
        }
    }

    let mut config = AppConfig::default();

    // 2. If no settings.json yet, populate from GSettings if available
    if let Some(s) = settings() {
        if let Some(schema) = s.settings_schema() {
            if schema.has_key("window-width") {
                let w = s.int("window-width");
                if w >= 400 {
                    config.window_width = w;
                }
            }
            if schema.has_key("window-height") {
                let h = s.int("window-height");
                if h >= 300 {
                    config.window_height = h;
                }
            }
            if schema.has_key("is-maximized") {
                config.is_maximized = s.boolean("is-maximized");
            }
            if schema.has_key("is-fullscreen") {
                config.is_fullscreen = s.boolean("is-fullscreen");
            }
            if schema.has_key("show-layers-sidebar") {
                config.show_layers_sidebar = s.boolean("show-layers-sidebar");
            }
            if schema.has_key("show-inspector-sidebar") {
                config.show_inspector_sidebar = s.boolean("show-inspector-sidebar");
            }
            if schema.has_key("active-zoom") {
                config.active_zoom = s.double("active-zoom").clamp(0.05, 40.0);
            }
            if schema.has_key("show-grid") {
                config.show_grid = s.boolean("show-grid");
            }
            if schema.has_key("show-rulers") {
                config.show_rulers = s.boolean("show-rulers");
            }
            if schema.has_key("show-guides") {
                config.show_guides = s.boolean("show-guides");
            }
            if schema.has_key("snap-enabled") {
                config.snap_enabled = s.boolean("snap-enabled");
            }
            if schema.has_key("snap-to-grid") {
                config.snap_to_grid = s.boolean("snap-to-grid");
            }
            if schema.has_key("snap-to-objects") {
                config.snap_to_objects = s.boolean("snap-to-objects");
            }
            if schema.has_key("snap-to-artboard") {
                config.snap_to_artboard = s.boolean("snap-to-artboard");
            }
            if schema.has_key("snap-to-guides") {
                config.snap_to_guides = s.boolean("snap-to-guides");
            }
            if schema.has_key("grid-style") {
                config.grid_style = s.string("grid-style").to_string();
            }
            if schema.has_key("grid-cell-size") {
                config.grid_cell_size = s.double("grid-cell-size").max(1.0);
            }
            if schema.has_key("grid-subdivisions") {
                config.grid_subdivisions = s.int("grid-subdivisions").max(1);
            }
            if schema.has_key("workspace-dots") {
                config.workspace_dots = s.boolean("workspace-dots");
            }
            if schema.has_key("page-shadow") {
                config.page_shadow = s.boolean("page-shadow");
            }
            if schema.has_key("page-border") {
                config.page_border = s.boolean("page-border");
            }
            if schema.has_key("canvas-bg-color") {
                config.canvas_bg_color = s.string("canvas-bg-color").to_string();
            }
            if schema.has_key("page-bg-color") {
                config.page_bg_color = s.string("page-bg-color").to_string();
            }
            if schema.has_key("hardware-acceleration") {
                config.hardware_acceleration = s.boolean("hardware-acceleration");
            }
            if schema.has_key("high-precision-aa") {
                config.high_precision_aa = s.boolean("high-precision-aa");
            }
            if schema.has_key("color-scheme") {
                config.color_scheme = s.string("color-scheme").to_string();
            }
            if schema.has_key("visual-theme") {
                config.visual_theme = s.string("visual-theme").to_string();
            }
            if schema.has_key("custom-accent-color") {
                config.custom_accent_color = s.string("custom-accent-color").to_string();
            }
            if schema.has_key("interface-icon-color") {
                config.interface_icon_color = s.string("interface-icon-color").to_string();
            }
            if schema.has_key("toolbar-icon-size") {
                config.toolbar_icon_size = s.string("toolbar-icon-size").to_string();
            }
            if schema.has_key("interface-scale") {
                config.interface_scale = s.string("interface-scale").to_string();
            }
            if schema.has_key("toolbar-position") {
                config.toolbar_position = s.string("toolbar-position").to_string();
            }
            if schema.has_key("language") {
                config.language = s.string("language").to_string();
            }
            if schema.has_key("unit") {
                config.unit = s.string("unit").to_string();
            }
            if schema.has_key("node-size") {
                config.node_size = s.double("node-size").max(4.0);
            }
            if schema.has_key("handle-size") {
                config.handle_size = s.double("handle-size").max(3.0);
            }
            if schema.has_key("handle-display-mode") {
                config.handle_display_mode = s.string("handle-display-mode").to_string();
            }
            if schema.has_key("shortcut-preset") {
                config.shortcut_preset = s.string("shortcut-preset").to_string();
            }
        }
    }

    save_config_internal(&config);
    config
}

fn save_config_internal(config: &AppConfig) {
    let path = config_file_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(&path, json);
    }
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
        GLOBAL_CONFIG.with(|c| {
            let cfg = c.borrow();
            (cfg.window_width.max(400), cfg.window_height.max(300))
        })
    }

    pub fn set_window_size(width: i32, height: i32) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.window_width = width.max(400);
            cfg.window_height = height.max(300);
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_int("window-width", width.max(400));
            let _ = s.set_int("window-height", height.max(300));
        }
    }

    pub fn is_maximized() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().is_maximized)
    }

    pub fn set_is_maximized(maximized: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.is_maximized = maximized;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("is-maximized", maximized);
        }
    }

    pub fn is_fullscreen() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().is_fullscreen)
    }

    pub fn set_is_fullscreen(fullscreen: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.is_fullscreen = fullscreen;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("is-fullscreen", fullscreen);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Sidebar Panels
    // ─────────────────────────────────────────────────────────────
    pub fn show_layers_sidebar() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().show_layers_sidebar)
    }

    pub fn set_show_layers_sidebar(show: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.show_layers_sidebar = show;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-layers-sidebar", show);
        }
    }

    pub fn show_inspector_sidebar() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().show_inspector_sidebar)
    }

    pub fn set_show_inspector_sidebar(show: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.show_inspector_sidebar = show;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-inspector-sidebar", show);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Inspector Tab Layout
    // ─────────────────────────────────────────────────────────────
    pub fn inspector_tab_locations() -> Vec<String> {
        GLOBAL_CONFIG.with(|c| c.borrow().inspector_tab_locations.clone())
    }

    pub fn set_inspector_tab_locations(locs: Vec<String>) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.inspector_tab_locations = locs;
            save_config_internal(&cfg);
        });
    }

    pub fn inspector_active_tabs() -> Vec<usize> {
        GLOBAL_CONFIG.with(|c| c.borrow().inspector_active_tabs.clone())
    }

    pub fn set_inspector_active_tabs(tabs: Vec<usize>) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.inspector_active_tabs = tabs;
            save_config_internal(&cfg);
        });
    }

    pub fn inspector_tab_order() -> Vec<usize> {
        GLOBAL_CONFIG.with(|c| c.borrow().inspector_tab_order.clone())
    }

    pub fn set_inspector_tab_order(order: Vec<usize>) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.inspector_tab_order = order;
            save_config_internal(&cfg);
        });
    }

    // ─────────────────────────────────────────────────────────────
    // Active Zoom & Canvas
    // ─────────────────────────────────────────────────────────────
    pub fn active_zoom() -> f64 {
        GLOBAL_CONFIG.with(|c| c.borrow().active_zoom.clamp(0.05, 40.0))
    }

    pub fn set_active_zoom(zoom: f64) {
        let z = zoom.clamp(0.05, 40.0);
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.active_zoom = z;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_double("active-zoom", z);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Display & Grid Preferences
    // ─────────────────────────────────────────────────────────────
    pub fn show_grid() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().show_grid)
    }

    pub fn set_show_grid(show: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.show_grid = show;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-grid", show);
        }
    }

    pub fn show_rulers() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().show_rulers)
    }

    pub fn set_show_rulers(show: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.show_rulers = show;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-rulers", show);
        }
    }

    pub fn show_guides() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().show_guides)
    }

    pub fn set_show_guides(show: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.show_guides = show;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("show-guides", show);
        }
    }

    pub fn snap_enabled() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().snap_enabled)
    }

    pub fn set_snap_enabled(enabled: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.snap_enabled = enabled;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-enabled", enabled);
        }
    }

    pub fn snap_to_grid() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().snap_to_grid)
    }

    pub fn set_snap_to_grid(snap: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.snap_to_grid = snap;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-to-grid", snap);
        }
    }

    pub fn snap_to_objects() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().snap_to_objects)
    }

    pub fn set_snap_to_objects(snap: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.snap_to_objects = snap;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-to-objects", snap);
        }
    }

    pub fn snap_to_artboard() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().snap_to_artboard)
    }

    pub fn set_snap_to_artboard(snap: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.snap_to_artboard = snap;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-to-artboard", snap);
        }
    }

    pub fn snap_to_guides() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().snap_to_guides)
    }

    pub fn set_snap_to_guides(snap: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.snap_to_guides = snap;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("snap-to-guides", snap);
        }
    }

    pub fn grid_style() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().grid_style.clone())
    }

    pub fn set_grid_style(style: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.grid_style = style.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("grid-style", style);
        }
    }

    pub fn grid_cell_size() -> f64 {
        GLOBAL_CONFIG.with(|c| c.borrow().grid_cell_size.max(1.0))
    }

    pub fn set_grid_cell_size(size: f64) {
        let sz = size.max(1.0);
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.grid_cell_size = sz;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_double("grid-cell-size", sz);
        }
    }

    pub fn grid_subdivisions() -> u32 {
        GLOBAL_CONFIG.with(|c| c.borrow().grid_subdivisions.max(1) as u32)
    }

    pub fn set_grid_subdivisions(subs: u32) {
        let sb = subs.max(1);
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.grid_subdivisions = sb as i32;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_int("grid-subdivisions", sb as i32);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Workspace & Rendering Preferences
    // ─────────────────────────────────────────────────────────────
    pub fn workspace_dots() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().workspace_dots)
    }

    pub fn set_workspace_dots(enabled: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.workspace_dots = enabled;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("workspace-dots", enabled);
        }
    }

    pub fn page_shadow() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().page_shadow)
    }

    pub fn set_page_shadow(enabled: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.page_shadow = enabled;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("page-shadow", enabled);
        }
    }

    pub fn page_border() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().page_border)
    }

    pub fn set_page_border(enabled: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.page_border = enabled;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("page-border", enabled);
        }
    }

    pub fn canvas_bg_color() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().canvas_bg_color.clone())
    }

    pub fn set_canvas_bg_color(color_hex: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.canvas_bg_color = color_hex.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("canvas-bg-color", color_hex);
        }
    }

    pub fn page_bg_color() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().page_bg_color.clone())
    }

    pub fn set_page_bg_color(color_hex: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.page_bg_color = color_hex.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("page-bg-color", color_hex);
        }
    }

    pub fn hardware_acceleration() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().hardware_acceleration)
    }

    pub fn set_hardware_acceleration(enabled: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.hardware_acceleration = enabled;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("hardware-acceleration", enabled);
        }
    }

    pub fn high_precision_aa() -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().high_precision_aa)
    }

    pub fn set_high_precision_aa(enabled: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.high_precision_aa = enabled;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_boolean("high-precision-aa", enabled);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Appearance & Themes
    // ─────────────────────────────────────────────────────────────
    pub fn color_scheme() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().color_scheme.clone())
    }

    pub fn set_color_scheme(scheme: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.color_scheme = scheme.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("color-scheme", scheme);
        }
    }

    pub fn visual_theme() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().visual_theme.clone())
    }

    pub fn set_visual_theme(theme: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.visual_theme = theme.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("visual-theme", theme);
        }
    }

    pub fn custom_accent_color() -> Option<String> {
        GLOBAL_CONFIG.with(|c| {
            let hex = c.borrow().custom_accent_color.clone();
            if hex.trim().is_empty() {
                None
            } else {
                Some(hex)
            }
        })
    }

    pub fn set_custom_accent_color(accent: Option<&str>) {
        let hex = accent.unwrap_or("").to_string();
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.custom_accent_color = hex.clone();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("custom-accent-color", &hex);
        }
    }

    pub fn interface_icon_color() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().interface_icon_color.clone())
    }

    pub fn set_interface_icon_color(color: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.interface_icon_color = color.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("interface-icon-color", color);
        }
    }

    pub fn toolbar_icon_size() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().toolbar_icon_size.clone())
    }

    pub fn set_toolbar_icon_size(size: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.toolbar_icon_size = size.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("toolbar-icon-size", size);
        }
    }

    pub fn interface_scale() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().interface_scale.clone())
    }

    pub fn set_interface_scale(scale: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.interface_scale = scale.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("interface-scale", scale);
        }
    }

    pub fn toolbar_position() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().toolbar_position.clone())
    }

    pub fn set_toolbar_position(pos: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.toolbar_position = pos.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("toolbar-position", pos);
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Plugins & Extension Management
    // ─────────────────────────────────────────────────────────────
    pub fn enabled_plugins() -> Vec<String> {
        GLOBAL_CONFIG.with(|c| c.borrow().enabled_plugins.clone())
    }

    pub fn is_plugin_enabled(id: &str) -> bool {
        GLOBAL_CONFIG.with(|c| c.borrow().enabled_plugins.iter().any(|p| p == id))
    }

    pub fn set_plugin_enabled(id: &str, enabled: bool) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            if enabled {
                if !cfg.enabled_plugins.iter().any(|p| p == id) {
                    cfg.enabled_plugins.push(id.to_string());
                }
            } else {
                cfg.enabled_plugins.retain(|p| p != id);
            }
            save_config_internal(&cfg);
        });
    }

    // ─────────────────────────────────────────────────────────────
    // Toolbar Customization
    // ─────────────────────────────────────────────────────────────
    pub fn toolbar_customization() -> Option<String> {
        GLOBAL_CONFIG.with(|c| c.borrow().toolbar_customization.clone())
    }

    pub fn set_toolbar_customization(custom: Option<String>) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.toolbar_customization = custom;
            save_config_internal(&cfg);
        });
    }

    // ─────────────────────────────────────────────────────────────
    // System & Tooling Preferences
    // ─────────────────────────────────────────────────────────────
    pub fn language() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().language.clone())
    }

    pub fn set_language(code: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.language = code.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("language", code);
        }
    }

    pub fn unit() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().unit.clone())
    }

    pub fn set_unit(unit_str: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.unit = unit_str.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("unit", unit_str);
        }
    }

    pub fn node_size() -> f64 {
        GLOBAL_CONFIG.with(|c| c.borrow().node_size.max(4.0))
    }

    pub fn set_node_size(size: f64) {
        let sz = size.max(4.0);
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.node_size = sz;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_double("node-size", sz);
        }
    }

    pub fn handle_size() -> f64 {
        GLOBAL_CONFIG.with(|c| c.borrow().handle_size.max(3.0))
    }

    pub fn set_handle_size(size: f64) {
        let sz = size.max(3.0);
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.handle_size = sz;
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_double("handle-size", sz);
        }
    }

    pub fn handle_display_mode() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().handle_display_mode.clone())
    }

    pub fn set_handle_display_mode(mode: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.handle_display_mode = mode.to_string();
            save_config_internal(&cfg);
        });
        if let Some(s) = settings() {
            let _ = s.set_string("handle-display-mode", mode);
        }
    }

    pub fn shortcut_preset() -> String {
        GLOBAL_CONFIG.with(|c| c.borrow().shortcut_preset.clone())
    }

    pub fn set_shortcut_preset(preset: &str) {
        GLOBAL_CONFIG.with(|c| {
            let mut cfg = c.borrow_mut();
            cfg.shortcut_preset = preset.to_string();
            save_config_internal(&cfg);
        });
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
        assert_eq!(w, 1440);
        assert_eq!(h, 900);

        AppSettings::set_is_maximized(true);
        assert!(AppSettings::is_maximized());

        AppSettings::set_is_fullscreen(false);
        assert!(!AppSettings::is_fullscreen());
    }

    #[test]
    fn test_app_settings_zoom_and_sidebars() {
        AppSettings::set_active_zoom(2.5);
        let z = AppSettings::active_zoom();
        assert!((z - 2.5).abs() < 1e-6);

        AppSettings::set_show_layers_sidebar(false);
        assert!(!AppSettings::show_layers_sidebar());
        AppSettings::set_show_inspector_sidebar(false);
        assert!(!AppSettings::show_inspector_sidebar());
    }

    #[test]
    fn test_app_settings_plugins_and_inspector_tabs() {
        AppSettings::set_plugin_enabled("color_palette", true);
        assert!(AppSettings::is_plugin_enabled("color_palette"));
        AppSettings::set_plugin_enabled("color_palette", false);
        assert!(!AppSettings::is_plugin_enabled("color_palette"));

        let locs = vec!["docked:0".to_string(), "closed".to_string(), "docked:1".to_string()];
        AppSettings::set_inspector_tab_locations(locs.clone());
        assert_eq!(AppSettings::inspector_tab_locations(), locs);
    }
}
