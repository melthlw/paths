use serde::{Deserialize, Serialize};
use skia_safe as skia;
use std::path::{Path, PathBuf};

use crate::core::color::Color;
use crate::core::geometry::Point;
use crate::core::svg_import::parse_svg;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomPatternDef {
    pub name: String,
    pub file_path: String,
    pub is_svg: bool,
}

pub fn get_user_patterns_dir() -> PathBuf {
    if let Some(cfg) = dirs::config_dir() {
        cfg.join("paths").join("libraries").join("patterns")
    } else {
        PathBuf::from(".config/paths/libraries/patterns")
    }
}

pub fn get_libraries_patterns_dir() -> PathBuf {
    get_user_patterns_dir()
}

pub fn ensure_user_patterns_dir() -> PathBuf {
    let dir = get_user_patterns_dir();
    let _ = std::fs::create_dir_all(&dir);

    // Auto-migrate files from legacy ~/.config/paths/patterns/ if it exists
    if let Some(cfg) = dirs::config_dir() {
        let legacy_dir = cfg.join("paths").join("patterns");
        if legacy_dir.is_dir() && legacy_dir != dir {
            if let Ok(entries) = std::fs::read_dir(&legacy_dir) {
                for entry in entries.flatten() {
                    let old_path = entry.path();
                    let new_path = dir.join(entry.file_name());
                    if !new_path.exists() {
                        let _ = std::fs::copy(&old_path, &new_path);
                    }
                }
            }
            let _ = std::fs::remove_dir_all(&legacy_dir);
        }
    }

    // If directory is empty, populate with curated default vector pattern presets
    let is_empty = match std::fs::read_dir(&dir) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => true,
    };

    if is_empty {
        seed_default_patterns(&dir);
    }

    dir
}

fn seed_default_patterns(dir: &Path) {
    // Remove obsolete circuit-board pattern if present
    let _ = std::fs::remove_file(dir.join("circuit-board.svg"));

    let default_svgs = [
        (
            "japanese-waves.svg",
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50" width="100" height="50">
  <rect x="0" y="0" width="100" height="50" fill="#121620"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M -30 0 A 30 30 0 0 1 30 0 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M -22 0 A 22 22 0 0 1 22 0"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M -14 0 A 14 14 0 0 1 14 0"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M -6 0 A 6 6 0 0 1 6 0"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M 20 0 A 30 30 0 0 1 80 0 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 28 0 A 22 22 0 0 1 72 0"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 36 0 A 14 14 0 0 1 64 0"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 44 0 A 6 6 0 0 1 56 0"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M 70 0 A 30 30 0 0 1 130 0 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 78 0 A 22 22 0 0 1 122 0"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 86 0 A 14 14 0 0 1 114 0"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 94 0 A 6 6 0 0 1 106 0"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M -5 25 A 30 30 0 0 1 55 25 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 3 25 A 22 22 0 0 1 47 25"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 11 25 A 14 14 0 0 1 39 25"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 19 25 A 6 6 0 0 1 31 25"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M 45 25 A 30 30 0 0 1 105 25 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 53 25 A 22 22 0 0 1 97 25"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 61 25 A 14 14 0 0 1 89 25"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 69 25 A 6 6 0 0 1 81 25"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M 95 25 A 30 30 0 0 1 155 25 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 103 25 A 22 22 0 0 1 147 25"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 111 25 A 14 14 0 0 1 139 25"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 119 25 A 6 6 0 0 1 131 25"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M -55 25 A 30 30 0 0 1 5 25 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M -47 25 A 22 22 0 0 1 -3 25"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M -39 25 A 14 14 0 0 1 -11 25"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M -31 25 A 6 6 0 0 1 -19 25"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M -30 50 A 30 30 0 0 1 30 50 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M -22 50 A 22 22 0 0 1 22 50"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M -14 50 A 14 14 0 0 1 14 50"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M -6 50 A 6 6 0 0 1 6 50"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M 20 50 A 30 30 0 0 1 80 50 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 28 50 A 22 22 0 0 1 72 50"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 36 50 A 14 14 0 0 1 64 50"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 44 50 A 6 6 0 0 1 56 50"/>
  <path fill="#121620" stroke="#3584e4" stroke-width="2.2" d="M 70 50 A 30 30 0 0 1 130 50 Z"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 78 50 A 22 22 0 0 1 122 50"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 86 50 A 14 14 0 0 1 114 50"/>
  <path fill="none" stroke="#3584e4" stroke-width="2" d="M 94 50 A 6 6 0 0 1 106 50"/>
</svg>"##,
        ),
        (
            "geometric-lattice.svg",
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 60 60">
  <path d="M 30 0 L 60 15 L 60 45 L 30 60 L 0 45 L 0 15 Z" fill="none" stroke="#2ec27e" stroke-width="2"/>
  <path d="M 30 0 L 30 60 M 60 15 L 0 45 M 0 15 L 60 45" fill="none" stroke="#2ec27e" stroke-width="1.5"/>
</svg>"##,
        ),
        (
            "moroccan-stars.svg",
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 80 80">
  <path d="M 40 0 L 48 24 L 72 16 L 56 32 L 80 40 L 56 48 L 72 64 L 48 56 L 40 80 L 32 56 L 8 64 L 24 48 L 0 40 L 24 32 L 8 16 L 32 24 Z" fill="none" stroke="#f6d32d" stroke-width="2.5"/>
</svg>"##,
        ),
        (
            "carbon-weave.svg",
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 40">
  <rect x="0" y="0" width="20" height="20" fill="#3d3846"/>
  <rect x="20" y="20" width="20" height="20" fill="#3d3846"/>
  <rect x="20" y="0" width="20" height="20" fill="#241f31"/>
  <rect x="0" y="20" width="20" height="20" fill="#241f31"/>
  <path d="M 0 0 L 20 20 M 20 0 L 40 20 M 0 20 L 20 40 M 20 20 L 40 40" stroke="#77767b" stroke-width="1.5"/>
</svg>"##,
        ),
    ];

    for (filename, svg_data) in default_svgs {
        let file_path = dir.join(filename);
        let _ = std::fs::write(file_path, svg_data);
    }
}

pub fn scan_user_patterns() -> Vec<CustomPatternDef> {
    ensure_user_patterns_dir();

    let mut patterns = Vec::new();
    let dirs = [get_user_patterns_dir(), get_libraries_patterns_dir()];

    for dir in &dirs {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_ascii_lowercase())
                {
                    let is_svg = ext == "svg";
                    let is_raster = ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "webp";
                    if is_svg || is_raster {
                        let stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Custom Pattern");
                        let name = stem
                            .replace(['-', '_'], " ")
                            .split_whitespace()
                            .map(|w| {
                                let mut c = w.chars();
                                match c.next() {
                                    None => String::new(),
                                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ");

                        let file_path = path.to_string_lossy().to_string();
                        if !patterns
                            .iter()
                            .any(|p: &CustomPatternDef| p.file_path == file_path)
                        {
                            patterns.push(CustomPatternDef {
                                name,
                                file_path,
                                is_svg,
                            });
                        }
                    }
                }
            }
        }
    }

    patterns.sort_by(|a, b| a.name.cmp(&b.name));
    patterns
}

pub fn create_custom_pattern_shader(
    file_path: &str,
    c1: Color,
    c2: Color,
    scale: f32,
    angle: f32,
    offset: Point,
) -> Option<skia::Shader> {
    let path = Path::new(file_path);
    if !path.exists() {
        return None;
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    let sz = scale.clamp(4.0, 2048.0);

    if ext == "svg" {
        let content = std::fs::read_to_string(path).ok()?;
        let import_result = parse_svg(&content).ok()?;
        let orig_w = import_result.width.max(1.0);
        let orig_h = import_result.height.max(1.0);

        let aspect = orig_h / orig_w;
        let tile_w = sz;
        let tile_h = (sz * aspect).max(4.0);
        let tile_rect = skia::Rect::from_xywh(0.0, 0.0, tile_w, tile_h);

        let mut recorder = skia::PictureRecorder::new();
        let canvas = recorder.begin_recording(tile_rect, false);

        // Draw primary background fill
        let mut bg_paint = skia::Paint::default();
        bg_paint.set_color4f(c1.to_skia(), None);
        bg_paint.set_style(skia::PaintStyle::Fill);
        canvas.draw_rect(tile_rect, &bg_paint);

        // Scale canvas to fit native SVG elements into the tile
        let sx = tile_w / orig_w;
        let sy = tile_h / orig_h;
        canvas.save();
        canvas.scale((sx, sy));

        for elem in &import_result.elements {
            match elem {
                crate::core::Element::Path(p) => {
                    let mut p_clone = p.clone();
                    if let Some(fc) = p_clone.fill_color {
                        if fc.r < 0.25 && fc.g < 0.25 && fc.b < 0.25 {
                            p_clone.fill_color = Some(c1);
                        }
                    }
                    if let Some(sc) = p_clone.stroke_color {
                        if sc.r > 0.1 || sc.g > 0.1 || sc.b > 0.1 {
                            p_clone.stroke_color = Some(c2);
                        }
                    } else if p_clone.fill_color.is_none() {
                        p_clone.stroke_color = Some(c2);
                        p_clone.stroke_width = 2.0;
                    }
                    p_clone.render(canvas);
                }
                crate::core::Element::Rect(r) => {
                    let mut r_clone = r.clone();
                    if let Some(fc) = r_clone.fill_color {
                        if fc.r < 0.25 && fc.g < 0.25 && fc.b < 0.25 {
                            r_clone.fill_color = Some(c1);
                        }
                    }
                    r_clone.render(canvas);
                }
                crate::core::Element::Text(t) => {
                    t.render(canvas);
                }
                crate::core::Element::Group(g) => {
                    g.render(canvas);
                }
                crate::core::Element::Brush(b) => {
                    b.render(canvas);
                }
                _ => {}
            }
        }
        canvas.restore();

        let picture = recorder.finish_recording_as_picture(None)?;

        // Build transformation matrix with Offset, Rotation, and Repeat
        let mut matrix = skia::Matrix::default();
        matrix.pre_translate((offset.x, offset.y));
        if angle.abs() > 0.01 {
            matrix.post_rotate(
                angle,
                Some(skia::Point::new(
                    offset.x + tile_w * 0.5,
                    offset.y + tile_h * 0.5,
                )),
            );
        }

        Some(picture.to_shader(
            (skia::TileMode::Repeat, skia::TileMode::Repeat),
            skia::FilterMode::Linear,
            Some(&matrix),
            Some(&tile_rect),
        ))
    } else {
        // Raster Image (PNG, JPG, WEBP)
        let bytes = std::fs::read(path).ok()?;
        let data = skia::Data::new_copy(&bytes);
        let image = skia::Image::from_encoded(data)?;

        let img_w = image.width() as f32;
        let img_h = image.height() as f32;
        if img_w <= 0.0 || img_h <= 0.0 {
            return None;
        }

        let aspect = img_h / img_w;
        let tile_w = sz;
        let tile_h = (sz * aspect).max(4.0);

        let sx = tile_w / img_w;
        let sy = tile_h / img_h;

        let mut matrix = skia::Matrix::default();
        matrix.pre_scale((sx, sy), None);
        matrix.post_translate((offset.x, offset.y));
        if angle.abs() > 0.01 {
            matrix.post_rotate(
                angle,
                Some(skia::Point::new(
                    offset.x + tile_w * 0.5,
                    offset.y + tile_h * 0.5,
                )),
            );
        }

        image.to_shader(
            (skia::TileMode::Repeat, skia::TileMode::Repeat),
            skia::SamplingOptions::default(),
            Some(&matrix),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_pattern_dir_creation() {
        let dir = ensure_user_patterns_dir();
        assert!(dir.exists());
        assert!(dir.is_dir());
    }

    #[test]
    fn test_scan_user_patterns() {
        let patterns = scan_user_patterns();
        assert!(!patterns.is_empty(), "Should load at least seeded patterns");
        assert!(patterns.iter().any(|p| p.is_svg));
    }
}
