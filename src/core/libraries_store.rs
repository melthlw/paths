use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwatchItem {
    pub hex: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteDef {
    pub name: String,
    pub colors: Vec<SwatchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDef {
    pub key: String,
    pub name: String,
    pub scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconDef {
    pub name: String,
    #[serde(default)]
    pub icon_name: String,
    #[serde(default)]
    pub path_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeDef {
    pub name: String,
    pub desc: String,
    pub path_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokePresetDef {
    pub key: String,
    pub name: String,
    pub desc: String,
    pub width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyPresetDef {
    pub name: String,
    pub family: String,
    pub size: f32,
    pub weight: u32,
    pub badge: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibrariesData {
    pub custom_colors: Vec<SwatchItem>,
    pub palettes: Vec<PaletteDef>,
    pub patterns: Vec<PatternDef>,
    pub icons: Vec<IconDef>,
    pub shapes: Vec<ShapeDef>,
    pub strokes: Vec<StrokePresetDef>,
    pub typography: Vec<TypographyPresetDef>,
}

pub fn get_libraries_dir() -> PathBuf {
    if let Some(cfg) = dirs::config_dir() {
        cfg.join("gnome-paths").join("libraries")
    } else {
        PathBuf::from(".config/gnome-paths/libraries")
    }
}

pub fn open_libraries_folder() {
    let dir = get_libraries_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(uri) = glib::filename_to_uri(&dir, None) {
        let _ = gtk4::gio::AppInfo::launch_default_for_uri(&uri, None::<&gtk4::gio::AppLaunchContext>);
    }
}

impl LibrariesData {
    pub fn load_or_init() -> Self {
        let dir = get_libraries_dir();
        let _ = std::fs::create_dir_all(&dir);

        // Ensure subdirectories for each tab exist in ~/.config/gnome-paths/libraries/
        let swatches_subdir = dir.join("swatches");
        let patterns_subdir = dir.join("patterns");
        let icons_subdir = dir.join("icons");
        let shapes_subdir = dir.join("shapes");
        let strokes_subdir = dir.join("strokes");
        let typography_subdir = dir.join("typography");

        let _ = std::fs::create_dir_all(&swatches_subdir);
        let _ = std::fs::create_dir_all(&patterns_subdir);
        let _ = std::fs::create_dir_all(&icons_subdir);
        let _ = std::fs::create_dir_all(&shapes_subdir);
        let _ = std::fs::create_dir_all(&strokes_subdir);
        let _ = std::fs::create_dir_all(&typography_subdir);

        let custom_colors = Self::load_or_init_file(
            &dir.join("custom_palette.json"),
            Self::default_custom_colors(),
        );

        let palettes = Self::load_or_init_file(
            &dir.join("swatches.json"),
            Self::default_palettes(),
        );

        let mut patterns = Self::default_patterns();
        let saved_patterns: Vec<PatternDef> = Self::load_or_init_file(
            &dir.join("patterns.json"),
            Vec::new(),
        );
        for sp in saved_patterns {
            if !patterns.iter().any(|p| p.name == sp.name) {
                patterns.push(sp);
            }
        }

        // Scan for additional .svg pattern files in ~/.config/gnome-paths/libraries/patterns/
        if patterns_subdir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&patterns_subdir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("svg") {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Custom Pattern");
                        let name = stem.replace('-', " ");
                        if !patterns.iter().any(|p| p.name == name) {
                            patterns.push(PatternDef { key: "Grid".to_string(), name, scale: 20.0 });
                        }
                    }
                }
            }
        }

        // Always load all GNOME Adwaita system icons
        let mut system_icons = Self::load_system_gnome_icons();
        if system_icons.is_empty() {
            system_icons = Self::default_icons();
        }

        // Merge any additional custom icons from icons.json
        let saved_icons: Vec<IconDef> = Self::load_or_init_file(
            &dir.join("icons.json"),
            Vec::new(),
        );
        for si in saved_icons {
            if !system_icons.iter().any(|i| i.name == si.name || (i.icon_name == si.icon_name && !si.icon_name.is_empty())) {
                system_icons.push(si);
            }
        }

        let mut icons = system_icons;

        // Scan for additional user .svg files in ~/.config/gnome-paths/libraries/icons/
        if icons_subdir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&icons_subdir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("svg") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Custom Icon");
                            let name = stem.replace('-', " ");
                            if let Some(d) = extract_path_d(&content) {
                                if !d.is_empty() {
                                    icons.push(IconDef { name, icon_name: String::new(), path_data: d });
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut shapes = Self::load_or_init_file(
            &dir.join("shapes.json"),
            Self::default_shapes(),
        );

        // Scan for additional .svg files in ~/.config/gnome-paths/libraries/shapes/
        if shapes_subdir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&shapes_subdir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("svg") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Custom Shape").to_string();
                            if let Some(d) = extract_path_d(&content) {
                                shapes.push(ShapeDef { name: name.clone(), desc: "Custom SVG Shape".to_string(), path_data: d });
                            }
                        }
                    }
                }
            }
        }

        let strokes = Self::load_or_init_file(
            &dir.join("strokes.json"),
            Self::default_strokes(),
        );

        let typography = Self::load_or_init_file(
            &dir.join("typography.json"),
            Self::default_typography(),
        );

        Self {
            custom_colors,
            palettes,
            patterns,
            icons,
            shapes,
            strokes,
            typography,
        }
    }

    pub fn load_system_gnome_icons() -> Vec<IconDef> {
        let mut result = Vec::new();
        let icon_dirs = [
            std::path::Path::new("/usr/share/icons/Adwaita/symbolic"),
            std::path::Path::new("/usr/share/icons/hicolor/scalable/actions"),
            std::path::Path::new("/usr/share/icons/hicolor/scalable/apps"),
            std::path::Path::new("/usr/share/icons/hicolor/scalable/status"),
        ];

        for &base_dir in &icon_dirs {
            if !base_dir.is_dir() {
                continue;
            }
            // If the base dir has subdirectories (like Adwaita/symbolic/{actions,apps,devices,...})
            let mut search_dirs = vec![base_dir.to_path_buf()];
            if let Ok(sub_entries) = std::fs::read_dir(base_dir) {
                for sub in sub_entries.flatten() {
                    let p = sub.path();
                    if p.is_dir() {
                        search_dirs.push(p);
                    }
                }
            }

            for dir in search_dirs {
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("svg") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                                let icon_name = stem.to_string();
                                let display_name = stem
                                    .strip_suffix("-symbolic")
                                    .unwrap_or(stem)
                                    .replace('-', " ");
                                let formatted_name: String = display_name
                                    .split_whitespace()
                                    .map(|word| {
                                        let mut c = word.chars();
                                        match c.next() {
                                            None => String::new(),
                                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" ");

                                if let Some(d) = extract_path_d(&content) {
                                    if !d.is_empty() {
                                        result.push(IconDef {
                                            name: formatted_name,
                                            icon_name,
                                            path_data: d,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result.dedup_by(|a, b| a.icon_name == b.icon_name);
        result
    }

    pub fn save_custom_colors(&self) {
        let dir = get_libraries_dir();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("custom_palette.json");
        if let Ok(json_str) = serde_json::to_string_pretty(&self.custom_colors) {
            let _ = std::fs::write(path, json_str);
        }
    }

    pub fn default_custom_colors() -> Vec<SwatchItem> {
        vec![
            SwatchItem { hex: "#3584e4".to_string(), name: "Primary Blue".to_string() },
            SwatchItem { hex: "#2ec27e".to_string(), name: "Accent Green".to_string() },
            SwatchItem { hex: "#f6d32d".to_string(), name: "Highlight Yellow".to_string() },
            SwatchItem { hex: "#ff7800".to_string(), name: "Warm Orange".to_string() },
            SwatchItem { hex: "#e01b24".to_string(), name: "Vibrant Red".to_string() },
            SwatchItem { hex: "#9141ac".to_string(), name: "Royal Purple".to_string() },
            SwatchItem { hex: "#ffffff".to_string(), name: "Clean White".to_string() },
            SwatchItem { hex: "#24283b".to_string(), name: "Night Dark".to_string() },
        ]
    }

    fn load_or_init_file<T: Serialize + for<'de> Deserialize<'de>>(path: &Path, default: T) -> T {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(val) = serde_json::from_str::<T>(&content) {
                    return val;
                }
            }
        }

        // Initialize file on disk
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json_str) = serde_json::to_string_pretty(&default) {
            let _ = std::fs::write(path, json_str);
        }

        default
    }

    pub fn default_palettes() -> Vec<PaletteDef> {
        vec![
            PaletteDef {
                name: "GNOME Adwaita".to_string(),
                colors: vec![
                    SwatchItem { hex: "#3584e4".to_string(), name: "Blue 1".to_string() },
                    SwatchItem { hex: "#1c71d8".to_string(), name: "Blue 2".to_string() },
                    SwatchItem { hex: "#2ec27e".to_string(), name: "Green 1".to_string() },
                    SwatchItem { hex: "#26a269".to_string(), name: "Green 2".to_string() },
                    SwatchItem { hex: "#f6d32d".to_string(), name: "Yellow 1".to_string() },
                    SwatchItem { hex: "#e5a50a".to_string(), name: "Yellow 2".to_string() },
                    SwatchItem { hex: "#ff7800".to_string(), name: "Orange 1".to_string() },
                    SwatchItem { hex: "#e01b24".to_string(), name: "Red 1".to_string() },
                    SwatchItem { hex: "#9141ac".to_string(), name: "Purple 1".to_string() },
                    SwatchItem { hex: "#613583".to_string(), name: "Purple 2".to_string() },
                    SwatchItem { hex: "#c061cb".to_string(), name: "Purple 3".to_string() },
                    SwatchItem { hex: "#986a44".to_string(), name: "Brown 1".to_string() },
                    SwatchItem { hex: "#63452c".to_string(), name: "Brown 2".to_string() },
                    SwatchItem { hex: "#b5835a".to_string(), name: "Brown 3".to_string() },
                    SwatchItem { hex: "#ffffff".to_string(), name: "White".to_string() },
                    SwatchItem { hex: "#f6f5f4".to_string(), name: "Light 1".to_string() },
                    SwatchItem { hex: "#deddda".to_string(), name: "Light 2".to_string() },
                    SwatchItem { hex: "#77767b".to_string(), name: "Dark 1".to_string() },
                    SwatchItem { hex: "#3d3846".to_string(), name: "Dark 2".to_string() },
                    SwatchItem { hex: "#241f31".to_string(), name: "Dark 3".to_string() },
                    SwatchItem { hex: "#000000".to_string(), name: "Black".to_string() },
                ],
            },
            PaletteDef {
                name: "Tailwind Modern".to_string(),
                colors: vec![
                    SwatchItem { hex: "#6366f1".to_string(), name: "Indigo".to_string() },
                    SwatchItem { hex: "#3b82f6".to_string(), name: "Blue".to_string() },
                    SwatchItem { hex: "#0ea5e9".to_string(), name: "Sky".to_string() },
                    SwatchItem { hex: "#06b6d4".to_string(), name: "Cyan".to_string() },
                    SwatchItem { hex: "#14b8a6".to_string(), name: "Teal".to_string() },
                    SwatchItem { hex: "#10b981".to_string(), name: "Emerald".to_string() },
                    SwatchItem { hex: "#84cc16".to_string(), name: "Lime".to_string() },
                    SwatchItem { hex: "#eab308".to_string(), name: "Yellow".to_string() },
                    SwatchItem { hex: "#f97316".to_string(), name: "Orange".to_string() },
                    SwatchItem { hex: "#ef4444".to_string(), name: "Red".to_string() },
                    SwatchItem { hex: "#ec4899".to_string(), name: "Pink".to_string() },
                    SwatchItem { hex: "#d946ef".to_string(), name: "Fuchsia".to_string() },
                    SwatchItem { hex: "#8b5cf6".to_string(), name: "Violet".to_string() },
                    SwatchItem { hex: "#64748b".to_string(), name: "Slate".to_string() },
                    SwatchItem { hex: "#71717a".to_string(), name: "Zinc".to_string() },
                    SwatchItem { hex: "#18181b".to_string(), name: "Zinc Dark".to_string() },
                ],
            },
            PaletteDef {
                name: "Cyberpunk & Neon".to_string(),
                colors: vec![
                    SwatchItem { hex: "#00f0ff".to_string(), name: "Neon Cyan".to_string() },
                    SwatchItem { hex: "#ff007f".to_string(), name: "Hot Pink".to_string() },
                    SwatchItem { hex: "#ffe600".to_string(), name: "Laser Yellow".to_string() },
                    SwatchItem { hex: "#39ff14".to_string(), name: "Acid Green".to_string() },
                    SwatchItem { hex: "#bf00ff".to_string(), name: "Electric Purple".to_string() },
                    SwatchItem { hex: "#ff3131".to_string(), name: "Laser Red".to_string() },
                    SwatchItem { hex: "#00ffff".to_string(), name: "Aqua Glow".to_string() },
                    SwatchItem { hex: "#ffaa00".to_string(), name: "Amber Flare".to_string() },
                    SwatchItem { hex: "#ff0055".to_string(), name: "Cyber Magenta".to_string() },
                    SwatchItem { hex: "#0d0221".to_string(), name: "Abyss Dark".to_string() },
                ],
            },
            PaletteDef {
                name: "Pastel & Soft".to_string(),
                colors: vec![
                    SwatchItem { hex: "#bbf7d0".to_string(), name: "Soft Mint".to_string() },
                    SwatchItem { hex: "#bfdbfe".to_string(), name: "Soft Sky".to_string() },
                    SwatchItem { hex: "#fbcfe8".to_string(), name: "Soft Rose".to_string() },
                    SwatchItem { hex: "#fef08a".to_string(), name: "Soft Butter".to_string() },
                    SwatchItem { hex: "#ddd6fe".to_string(), name: "Soft Lavender".to_string() },
                    SwatchItem { hex: "#fed7aa".to_string(), name: "Soft Peach".to_string() },
                    SwatchItem { hex: "#cbd5e1".to_string(), name: "Soft Slate".to_string() },
                    SwatchItem { hex: "#f1f5f9".to_string(), name: "Soft Fog".to_string() },
                ],
            },
            PaletteDef {
                name: "Monochrome & Grayscale".to_string(),
                colors: vec![
                    SwatchItem { hex: "#ffffff".to_string(), name: "Pure White".to_string() },
                    SwatchItem { hex: "#f3f4f6".to_string(), name: "Gray 100".to_string() },
                    SwatchItem { hex: "#e5e7eb".to_string(), name: "Gray 200".to_string() },
                    SwatchItem { hex: "#d1d5db".to_string(), name: "Gray 300".to_string() },
                    SwatchItem { hex: "#9ca3af".to_string(), name: "Gray 400".to_string() },
                    SwatchItem { hex: "#6b7280".to_string(), name: "Gray 500".to_string() },
                    SwatchItem { hex: "#4b5563".to_string(), name: "Gray 600".to_string() },
                    SwatchItem { hex: "#374151".to_string(), name: "Gray 700".to_string() },
                    SwatchItem { hex: "#1f2937".to_string(), name: "Gray 800".to_string() },
                    SwatchItem { hex: "#111827".to_string(), name: "Gray 900".to_string() },
                    SwatchItem { hex: "#000000".to_string(), name: "Pure Black".to_string() },
                ],
            },
        ]
    }

    pub fn default_patterns() -> Vec<PatternDef> {
        vec![
            PatternDef { key: "Grid".to_string(), name: "Technical Grid".to_string(), scale: 16.0 },
            PatternDef { key: "Dots".to_string(), name: "Halftone Dots".to_string(), scale: 14.0 },
            PatternDef { key: "Stripes".to_string(), name: "Diagonal Stripes".to_string(), scale: 18.0 },
            PatternDef { key: "Checkerboard".to_string(), name: "Checkerboard".to_string(), scale: 20.0 },
            PatternDef { key: "Hexagon".to_string(), name: "Honeycomb".to_string(), scale: 22.0 },
            PatternDef { key: "Crosshatch".to_string(), name: "Crosshatch".to_string(), scale: 16.0 },
            PatternDef { key: "Brick".to_string(), name: "Brick Wall".to_string(), scale: 20.0 },
            PatternDef { key: "Scales".to_string(), name: "Seigaiha Scales".to_string(), scale: 20.0 },
            PatternDef { key: "Houndstooth".to_string(), name: "Houndstooth".to_string(), scale: 24.0 },
            PatternDef { key: "Basketweave".to_string(), name: "Basketweave".to_string(), scale: 20.0 },
        ]
    }

    pub fn default_icons() -> Vec<IconDef> {
        vec![
            IconDef { name: "Home".to_string(), icon_name: "user-home-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "User".to_string(), icon_name: "avatar-default-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Search".to_string(), icon_name: "edit-find-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Settings".to_string(), icon_name: "emblem-system-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Favorite".to_string(), icon_name: "emblem-favorite-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Check".to_string(), icon_name: "emblem-default-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Close".to_string(), icon_name: "window-close-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Folder".to_string(), icon_name: "folder-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Mail".to_string(), icon_name: "mail-unread-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Trash".to_string(), icon_name: "user-trash-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Edit".to_string(), icon_name: "document-edit-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "New Document".to_string(), icon_name: "document-new-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Open".to_string(), icon_name: "document-open-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Save".to_string(), icon_name: "document-save-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Refresh".to_string(), icon_name: "view-refresh-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Grid".to_string(), icon_name: "view-grid-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Play".to_string(), icon_name: "media-playback-start-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Pause".to_string(), icon_name: "media-playback-pause-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Camera".to_string(), icon_name: "camera-photo-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Palette".to_string(), icon_name: "applications-graphics-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Bookmark".to_string(), icon_name: "bookmark-new-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Printer".to_string(), icon_name: "printer-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Sun".to_string(), icon_name: "weather-clear-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Moon".to_string(), icon_name: "weather-clear-night-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Network".to_string(), icon_name: "network-wireless-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Lock".to_string(), icon_name: "changes-prevent-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Unlock".to_string(), icon_name: "changes-allow-symbolic".to_string(), path_data: String::new() },
            IconDef { name: "Info".to_string(), icon_name: "dialog-information-symbolic".to_string(), path_data: String::new() },
        ]
    }

    pub fn default_shapes() -> Vec<ShapeDef> {
        vec![
            ShapeDef { name: "Shield".to_string(), desc: "Vector Shield".to_string(), path_data: "M 12 2 L 4 5 L 4 11 C 4 16.55 7.41 21.74 12 23 C 16.59 21.74 20 16.55 20 11 L 20 5 Z".to_string() },
            ShapeDef { name: "Badge 8P".to_string(), desc: "8-Point Badge".to_string(), path_data: "M 12 2 L 15 5 L 19 5 L 19 9 L 22 12 L 19 15 L 19 19 L 15 19 L 12 22 L 9 19 L 5 19 L 5 15 L 2 12 L 5 9 L 5 5 L 9 5 Z".to_string() },
            ShapeDef { name: "Ribbon".to_string(), desc: "Banner Ribbon".to_string(), path_data: "M 4 4 L 20 4 L 18 12 L 20 20 L 4 20 L 6 12 Z".to_string() },
            ShapeDef { name: "Speech Bubble".to_string(), desc: "Dialog Bubble".to_string(), path_data: "M 20 2 L 4 2 C 2.9 2 2 2.9 2 4 L 2 16 C 2 17.1 2.9 18 4 18 L 8 18 L 8 22 L 14 18 L 20 18 C 21.1 18 22 17.1 22 16 L 22 4 C 22 2.9 21.1 2 20 2 Z".to_string() },
            ShapeDef { name: "Lightning".to_string(), desc: "Energy Bolt".to_string(), path_data: "M 7 2 L 17 2 L 11 11 L 18 11 L 6 22 L 9 13 L 4 13 Z".to_string() },
            ShapeDef { name: "Hexagon".to_string(), desc: "Hexagonal Polygon".to_string(), path_data: "M 12 2 L 21 7 L 21 17 L 12 22 L 3 17 L 3 7 Z".to_string() },
            ShapeDef { name: "Octagon".to_string(), desc: "Octagonal Polygon".to_string(), path_data: "M 8 2 L 16 2 L 22 8 L 22 16 L 16 22 L 8 22 L 2 16 L 2 8 Z".to_string() },
            ShapeDef { name: "Tag / Label".to_string(), desc: "Price Tag".to_string(), path_data: "M 21.41 11.58 L 12.41 2.58 A 2 2 0 0 0 11 2 L 4 2 C 2.9 2 2 2.9 2 4 L 2 11 A 2 2 0 0 0 2.59 12.42 L 11.59 21.42 A 2 2 0 0 0 14.41 21.42 L 21.41 14.42 A 2 2 0 0 0 21.41 11.58 Z M 6.5 8 A 1.5 1.5 0 1 1 8 6.5 A 1.5 1.5 0 0 1 6.5 8 Z".to_string() },
            ShapeDef { name: "Arrow Right".to_string(), desc: "Directional Arrow".to_string(), path_data: "M 12 4 L 10.59 5.41 L 16.17 11 L 4 11 L 4 13 L 16.17 13 L 10.59 18.59 L 12 20 L 20 12 Z".to_string() },
            ShapeDef { name: "Diamond".to_string(), desc: "Perfect Diamond".to_string(), path_data: "M 12 2 L 22 12 L 12 22 L 2 12 Z".to_string() },
        ]
    }

    pub fn default_strokes() -> Vec<StrokePresetDef> {
        vec![
            StrokePresetDef { key: "Solid".to_string(), name: "Solid Stroke".to_string(), desc: "Continuous vector stroke".to_string(), width: 2.0 },
            StrokePresetDef { key: "Dashed".to_string(), name: "Dashed Line".to_string(), desc: "Standard dashed stroke".to_string(), width: 2.5 },
            StrokePresetDef { key: "Dotted".to_string(), name: "Dotted Line".to_string(), desc: "Fine dotted stroke".to_string(), width: 2.0 },
            StrokePresetDef { key: "Solid".to_string(), name: "Thick Outline".to_string(), desc: "Prominent 4px stroke".to_string(), width: 4.0 },
            StrokePresetDef { key: "Solid".to_string(), name: "Hairline".to_string(), desc: "Fine 1px line".to_string(), width: 1.0 },
        ]
    }

    pub fn default_typography() -> Vec<TypographyPresetDef> {
        vec![
            TypographyPresetDef { name: "Display Hero".to_string(), family: "Sans".to_string(), size: 48.0, weight: 700, badge: "Aa".to_string() },
            TypographyPresetDef { name: "Heading 1".to_string(), family: "Sans".to_string(), size: 32.0, weight: 700, badge: "H1".to_string() },
            TypographyPresetDef { name: "Heading 2".to_string(), family: "Sans".to_string(), size: 24.0, weight: 600, badge: "H2".to_string() },
            TypographyPresetDef { name: "Body Regular".to_string(), family: "Sans".to_string(), size: 16.0, weight: 400, badge: "Body".to_string() },
            TypographyPresetDef { name: "Code / Monospace".to_string(), family: "Monospace".to_string(), size: 14.0, weight: 400, badge: "</>".to_string() },
            TypographyPresetDef { name: "Caption".to_string(), family: "Sans".to_string(), size: 11.0, weight: 400, badge: "Cap".to_string() },
        ]
    }
}

pub fn extract_path_d(svg_content: &str) -> Option<String> {
    let mut all_paths = Vec::new();
    let mut rest = svg_content;
    while let Some(pos) = rest.find(" d=\"") {
        let after = &rest[pos + 4..];
        if let Some(end_pos) = after.find('"') {
            all_paths.push(after[..end_pos].trim().to_string());
            rest = &after[end_pos + 1..];
        } else {
            break;
        }
    }
    if all_paths.is_empty() {
        let mut rest_sq = svg_content;
        while let Some(pos) = rest_sq.find(" d='") {
            let after = &rest_sq[pos + 4..];
            if let Some(end_pos) = after.find('\'') {
                all_paths.push(after[..end_pos].trim().to_string());
                rest_sq = &after[end_pos + 1..];
            } else {
                break;
            }
        }
    }
    if !all_paths.is_empty() {
        Some(all_paths.join(" "))
    } else {
        None
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_extract_path_d_from_svg() {
        let svg = r##"<svg viewBox="0 0 24 24"><path d="M 10 20 L 10 12 Z" fill="#000"/></svg>"##;
        assert_eq!(extract_path_d(svg), Some("M 10 20 L 10 12 Z".to_string()));
    }

    #[test]
    fn test_libraries_default_generation_and_serialization() {
        let data = LibrariesData {
            custom_colors: LibrariesData::default_custom_colors(),
            palettes: LibrariesData::default_palettes(),
            patterns: LibrariesData::default_patterns(),
            icons: LibrariesData::default_icons(),
            shapes: LibrariesData::default_shapes(),
            strokes: LibrariesData::default_strokes(),
            typography: LibrariesData::default_typography(),
        };

        assert!(!data.palettes.is_empty());
        assert_eq!(data.patterns.len(), 10);
        assert!(!data.icons.is_empty());
        assert!(!data.shapes.is_empty());
        assert!(!data.strokes.is_empty());
        assert!(!data.typography.is_empty());

        let json = serde_json::to_string(&data).expect("Serialization failed");
        let decoded: LibrariesData = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(decoded.palettes.len(), data.palettes.len());
        assert_eq!(decoded.patterns.len(), data.patterns.len());
        assert_eq!(decoded.icons.len(), data.icons.len());
        assert_eq!(decoded.shapes.len(), data.shapes.len());
    }
}
