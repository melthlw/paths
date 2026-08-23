//! Dynamic loading of external shared library plugins (.so).

use super::traits::StudioPlugin;
use libloading::Library;
use std::path::{Path, PathBuf};

pub type PluginCreateFn = unsafe extern "Rust" fn() -> *mut dyn StudioPlugin;

/// A loaded external plugin keeping its library handle alive
pub struct ExternalPlugin {
    _lib: Library,
    pub plugin: Box<dyn StudioPlugin>,
}

impl ExternalPlugin {
    /// Load an external plugin from a .so shared library file
    pub unsafe fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let lib = Library::new(path)?;
        let create_symbol: libloading::Symbol<PluginCreateFn> =
            lib.get(b"gnome_paths_plugin_create\0")?;
        let raw_ptr = create_symbol();
        if raw_ptr.is_null() {
            return Err("Plugin constructor returned NULL pointer".into());
        }
        let plugin = Box::from_raw(raw_ptr);
        Ok(Self { _lib: lib, plugin })
    }
}

/// Returns the user plugins directory (~/.local/share/gnome-paths/plugins)
pub fn get_plugins_dir() -> PathBuf {
    if let Some(mut data_dir) = dirs::data_dir() {
        data_dir.push("gnome-paths");
        data_dir.push("plugins");
        data_dir
    } else {
        PathBuf::from("./plugins")
    }
}

/// Scan plugins directory and load all valid .so plugins
pub fn scan_and_load_external_plugins() -> Vec<ExternalPlugin> {
    let plugins_dir = get_plugins_dir();
    let mut loaded = Vec::new();

    if !plugins_dir.exists() {
        let _ = std::fs::create_dir_all(&plugins_dir);
        return loaded;
    }

    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "so") {
                unsafe {
                    match ExternalPlugin::load(&path) {
                        Ok(p) => {
                            loaded.push(p);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to load external plugin {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    loaded
}
