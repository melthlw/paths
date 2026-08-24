//! GNOME Paths — Example Plugin: Color Palette Toolbar
//!
//! This plugin provides a dynamic floating color palette toolbar for GNOME Paths,
//! with Fill/Stroke mode toggles, color presets, custom palette management,
//! and 4-way screen repositioning.

#[allow(dead_code)]
pub struct ColorPalettePlugin {
    id: &'static str,
    name: &'static str,
}

impl ColorPalettePlugin {
    pub fn new() -> Self {
        Self {
            id: "color_palette",
            name: "Color Palette Toolbar",
        }
    }
}

impl Default for ColorPalettePlugin {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────
// Dynamic FFI Export for GNOME Paths (.so)
// ─────────────────────────────────────────────────────────────

/// Entry point inspected by the GNOME Paths external plugin loader.
#[no_mangle]
pub unsafe extern "C" fn gnome_paths_plugin_info() -> *const u8 {
    b"Color Palette Toolbar Plugin\0".as_ptr()
}
