use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::core::modifier::Modifier;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifierAsset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub modifier: Modifier,
}

pub fn get_local_assets_dir() -> PathBuf {
    PathBuf::from("assets/modifiers")
}

pub fn get_user_modifiers_dir() -> PathBuf {
    if let Some(cfg) = dirs::config_dir() {
        cfg.join("paths").join("modifiers")
    } else {
        PathBuf::from(".config/paths/modifiers")
    }
}

pub fn ensure_modifier_asset_dirs() -> Vec<PathBuf> {
    let local_dir = get_local_assets_dir();
    let _ = fs::create_dir_all(&local_dir);

    let user_dir = get_user_modifiers_dir();
    let _ = fs::create_dir_all(&user_dir);

    vec![local_dir, user_dir]
}

/// Save a modifier configuration as a JSON asset in local directory `assets/modifiers/`
pub fn save_modifier_asset(name: &str, description: &str, modifier: &Modifier) -> Result<PathBuf, String> {
    let dirs = ensure_modifier_asset_dirs();
    let target_dir = &dirs[0]; // assets/modifiers

    let safe_name = name.to_lowercase().replace(' ', "_");
    let file_path = target_dir.join(format!("{}.json", safe_name));

    let asset = ModifierAsset {
        id: safe_name,
        name: name.to_string(),
        description: description.to_string(),
        modifier: modifier.clone(),
    };

    let json_str = serde_json::to_string_pretty(&asset)
        .map_err(|e| format!("Failed to serialize modifier asset: {}", e))?;

    fs::write(&file_path, json_str)
        .map_err(|e| format!("Failed to write modifier asset file: {}", e))?;

    Ok(file_path)
}

/// Load all modifier assets from `./assets/modifiers/` and `~/.config/paths/modifiers/`
pub fn load_all_modifier_assets() -> Vec<ModifierAsset> {
    let dirs = ensure_modifier_asset_dirs();
    let mut assets = Vec::new();

    // Seed default preset assets into assets/modifiers/ if empty
    seed_default_assets_if_empty(&dirs[0]);

    for dir in dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(asset) = serde_json::from_str::<ModifierAsset>(&content) {
                            assets.push(asset);
                        }
                    }
                }
            }
        }
    }

    assets
}

fn seed_default_assets_if_empty(dir: &PathBuf) {
    if let Ok(entries) = fs::read_dir(dir) {
        if entries.count() > 0 {
            return;
        }
    }

    let defaults = vec![
        (
            "Linear Array",
            "Linear, Radial & Grid duplication",
            Modifier::Array(crate::core::modifier::ArrayModifier::default()),
        ),
        (
            "Offset Path Outline",
            "Expanded 12px vector contour outline",
            Modifier::OffsetPath(crate::core::modifier::OffsetPathModifier {
                enabled: true,
                offset: 12.0,
                miter_limit: 4.0,
            }),
        ),
        (
            "ZigZag Serrated Border",
            "6 ridges zigzag vector distortion",
            Modifier::ZigZag(crate::core::modifier::ZigZagModifier {
                enabled: true,
                ridges: 6,
                amplitude: 10.0,
            }),
        ),
        (
            "Sine Wave Contour",
            "15px amplitude wave contour",
            Modifier::WaveDeform(crate::core::modifier::WaveDeformModifier {
                enabled: true,
                amplitude: 15.0,
                wavelength: 60.0,
            }),
        ),
    ];

    for (name, desc, m) in defaults {
        let _ = save_modifier_asset(name, desc, &m);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::modifier::{ArrayMode, ArrayModifier, Modifier};

    #[test]
    fn test_modifier_assets_save_and_load() {
        let arr_mod = Modifier::Array(ArrayModifier {
            enabled: true,
            mode: ArrayMode::Linear {
                count: 5,
                offset_x: 25.0,
                offset_y: 0.0,
                scale_step: 1.0,
                rotate_step_deg: 0.0,
            },
        });

        let res = save_modifier_asset("Test Linear Array", "Test description", &arr_mod);
        assert!(res.is_ok(), "Should save modifier asset without error");

        let loaded_assets = load_all_modifier_assets();
        assert!(!loaded_assets.is_empty(), "Should load seeded and saved modifier assets");
        let found = loaded_assets.iter().any(|a| a.name == "Test Linear Array");
        assert!(found, "Should find the newly saved modifier asset");
    }
}
