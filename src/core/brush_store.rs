use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::core::element::brush::{BrushStyle, MarkerShape, StrokeCap, StrokeJoin};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomBrushPreset {
    pub id: String,
    pub name: String,
    pub style: BrushStyle,
    pub width: f32,
    pub smoothing: f32,
    pub calligraphy_angle: f32,
    pub auto_close: bool,
    pub cap_style: StrokeCap,
    pub join_style: StrokeJoin,
    pub taper_start: bool,
    pub taper_end: bool,
    pub start_marker: MarkerShape,
    pub body_marker: MarkerShape,
    pub end_marker: MarkerShape,
    pub body_spacing: f32,
    pub marker_scale: f32,
    pub svg_path_d: Option<String>,
}

impl Default for CustomBrushPreset {
    fn default() -> Self {
        Self {
            id: "default_brush".to_string(),
            name: "Default Round".to_string(),
            style: BrushStyle::Round,
            width: 6.0,
            smoothing: 0.5,
            calligraphy_angle: 45.0,
            auto_close: false,
            cap_style: StrokeCap::Round,
            join_style: StrokeJoin::Round,
            taper_start: false,
            taper_end: false,
            start_marker: MarkerShape::None,
            body_marker: MarkerShape::None,
            end_marker: MarkerShape::None,
            body_spacing: 2.0,
            marker_scale: 1.0,
            svg_path_d: None,
        }
    }
}

pub fn get_user_brushes_dir() -> PathBuf {
    if let Some(cfg) = dirs::config_dir() {
        cfg.join("paths").join("brushes")
    } else {
        PathBuf::from(".config/paths/brushes")
    }
}

pub fn ensure_user_brushes_dir() -> PathBuf {
    let dir = get_user_brushes_dir();
    let _ = std::fs::create_dir_all(&dir);

    // Auto-migrate files from legacy ~/.config/gnome-paths/brushes/ if it exists
    if let Some(cfg) = dirs::config_dir() {
        let legacy_dir = cfg.join("gnome-paths").join("brushes");
        if legacy_dir.is_dir() && legacy_dir != dir {
            if let Ok(entries) = std::fs::read_dir(&legacy_dir) {
                for entry in entries.flatten() {
                    let old_path = entry.path();
                    if let Some(name) = old_path.file_name() {
                        let new_path = dir.join(name);
                        let _ = std::fs::copy(&old_path, &new_path);
                    }
                }
            }
        }
    }

    dir
}

#[allow(dead_code)]
pub fn load_custom_brush_presets() -> Vec<CustomBrushPreset> {
    let dir = ensure_user_brushes_dir();
    let mut presets = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(preset) = serde_json::from_str::<CustomBrushPreset>(&content) {
                        presets.push(preset);
                    }
                }
            }
        }
    }

    presets
}

pub fn save_custom_brush_preset(preset: &CustomBrushPreset) -> Result<(), String> {
    let dir = ensure_user_brushes_dir();
    let file_path = dir.join(format!("{}.json", preset.id));

    let json_str = serde_json::to_string_pretty(preset)
        .map_err(|e| format!("Failed to serialize brush preset: {}", e))?;

    std::fs::write(&file_path, json_str)
        .map_err(|e| format!("Failed to write brush preset file {}: {}", file_path.display(), e))?;

    Ok(())
}

#[allow(dead_code)]
pub fn delete_custom_brush_preset(id: &str) -> bool {
    let dir = get_user_brushes_dir();
    let file_path = dir.join(format!("{}.json", id));
    std::fs::remove_file(file_path).is_ok()
}

pub fn create_brush_from_clipboard_or_path(
    name: &str,
    path_d: &str,
    style: BrushStyle,
    width: f32,
    smoothing: f32,
) -> Result<CustomBrushPreset, String> {
    let sanitize_name = if name.trim().is_empty() {
        "Custom Brush"
    } else {
        name.trim()
    };

    let id = format!(
        "brush_{}_{}",
        sanitize_name
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>(),
        gtk4::glib::monotonic_time()
    );

    let preset = CustomBrushPreset {
        id,
        name: sanitize_name.to_string(),
        style,
        width,
        smoothing,
        calligraphy_angle: 45.0,
        auto_close: false,
        cap_style: StrokeCap::Round,
        join_style: StrokeJoin::Round,
        taper_start: false,
        taper_end: false,
        start_marker: MarkerShape::None,
        body_marker: MarkerShape::None,
        end_marker: MarkerShape::None,
        body_spacing: 2.0,
        marker_scale: 1.0,
        svg_path_d: if path_d.trim().is_empty() {
            None
        } else {
            Some(path_d.to_string())
        },
    };

    save_custom_brush_preset(&preset)?;
    Ok(preset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_brush_preset_lifecycle() {
        let preset = CustomBrushPreset {
            id: "test_brush_1".to_string(),
            name: "Test Pencil Brush".to_string(),
            style: BrushStyle::Pencil,
            width: 12.0,
            smoothing: 0.8,
            calligraphy_angle: 45.0,
            auto_close: false,
            cap_style: StrokeCap::Square,
            join_style: StrokeJoin::Miter,
            taper_start: true,
            taper_end: true,
            start_marker: MarkerShape::Arrow,
            body_marker: MarkerShape::Circle,
            end_marker: MarkerShape::Star,
            body_spacing: 3.0,
            marker_scale: 1.5,
            svg_path_d: Some("M 0 0 L 100 100".to_string()),
        };

        assert!(save_custom_brush_preset(&preset).is_ok());
        let presets = load_custom_brush_presets();
        assert!(presets.iter().any(|p| p.id == "test_brush_1"));

        assert!(delete_custom_brush_preset("test_brush_1"));
    }

    #[test]
    fn test_create_brush_from_clipboard_or_path() {
        let res = create_brush_from_clipboard_or_path(
            "Star Brush",
            "M 10 10 L 20 20 Z",
            BrushStyle::Round,
            8.0,
            0.6,
        );
        assert!(res.is_ok());
        let preset = res.unwrap();
        assert_eq!(preset.name, "Star Brush");
        assert_eq!(preset.svg_path_d, Some("M 10 10 L 20 20 Z".to_string()));

        let _ = delete_custom_brush_preset(&preset.id);
    }
}
