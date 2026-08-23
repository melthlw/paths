use gtk4::gdk;
use gtk4::gio;
use gtk4::prelude::*;
use std::path::{Path, PathBuf};

use crate::core::{
    Color, ElementId, GridConfig, GridStyle, Point, RulerConfig, SnapConfig, ARTBOARD_HEIGHT,
    ARTBOARD_WIDTH,
};
use super::CanvasWidget;

#[allow(dead_code)]
impl CanvasWidget {
    pub fn is_dirty(&self) -> bool {
        self.state.try_borrow().map(|s| s.is_dirty).unwrap_or(false)
    }

    pub fn set_on_file_state_changed<F: Fn(Option<&Path>, bool) + 'static>(&self, f: F) {
        let mut state = self.state.borrow_mut();
        state.on_file_state_changed = Some(Box::new(f));
        state.notify_file_state();
    }

    pub fn set_on_file_dialog_request<F: Fn(crate::core::ShortcutAction) + 'static>(&self, f: F) {
        let mut state = self.state.borrow_mut();
        state.on_file_dialog_request = Some(Box::new(f));
    }

    pub fn save(&self) -> Result<bool, String> {
        let mut state = self.state.borrow_mut();
        if let Some(ref path) = state.current_file_path.clone() {
            crate::core::save_document_to_file(&state.document, path)?;
            state.is_dirty = false;
            state.notify_file_state();
            state.notify_status();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        let mut state = self.state.borrow_mut();
        crate::core::save_document_to_file(&state.document, path)?;
        state.current_file_path = Some(path.to_path_buf());
        state.is_dirty = false;
        state.notify_file_state();
        state.notify_status();
        Ok(())
    }

    pub fn open_from_path(&self, path: &Path) -> Result<(), String> {
        let doc = crate::core::load_document_from_file(path)?;
        let mut state = self.state.borrow_mut();
        let target_rect = doc
            .active_page()
            .map(|p| p.rect.normalize())
            .unwrap_or_else(|| crate::core::Rect::new(0.0, 0.0, 794.0, 1123.0));
        let ws = state.widget_size;
        state.document = doc;
        state.current_file_path = Some(path.to_path_buf());
        state.is_dirty = false;
        state.viewport.zoom_to_rect(target_rect, ws);
        state.notify_file_state();
        state.notify_status();
        drop(state);
        self.drawing_area.queue_draw();
        Ok(())
    }

    pub fn new_document(&self) {
        let mut state = self.state.borrow_mut();
        state.document = crate::core::Document::new();
        state.current_file_path = None;
        state.is_dirty = false;
        state.has_centered_initial_page = true;
        state.viewport = crate::core::Viewport::default();
        let target_rect = state
            .document
            .active_page()
            .map(|p| p.rect.normalize())
            .unwrap_or_else(|| crate::core::Rect::new(0.0, 0.0, 794.0, 1123.0));
        let ws = state.widget_size;
        state.viewport.zoom_to_rect(target_rect, ws);
        state.notify_file_state();
        state.notify_status();
        drop(state);
        self.drawing_area.queue_draw();
    }

    pub fn import_file(&self, path: &str) -> Result<(), String> {
        let bytes = std::fs::read(path).map_err(|e| crate::i18n!("Error reading file: {}", e))?;
        let filename = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("import")
            .to_string();

        let mut state = self.state.borrow_mut();
        let center = if let Some(p) = state.document.active_page() {
            let r = p.rect.normalize();
            Point::new(r.x + r.width / 2.0, r.y + r.height / 2.0)
        } else {
            Point::new(ARTBOARD_WIDTH / 2.0, ARTBOARD_HEIGHT / 2.0)
        };

        // 1. Try vector SVG parsing first
        let is_svg = path.to_ascii_lowercase().ends_with(".svg")
            || (bytes.starts_with(b"<?xml") || bytes.starts_with(b"<svg"));

        if is_svg {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                if let Ok(svg_res) = crate::core::parse_svg(text) {
                    if !svg_res.elements.is_empty() {
                        let mut min_x = f32::INFINITY;
                        let mut min_y = f32::INFINITY;
                        let mut max_x = f32::NEG_INFINITY;
                        let mut max_y = f32::NEG_INFINITY;

                        for elem in &svg_res.elements {
                            let b = elem.bounds();
                            min_x = min_x.min(b.x);
                            min_y = min_y.min(b.y);
                            max_x = max_x.max(b.x + b.width);
                            max_y = max_y.max(b.y + b.height);
                        }

                        let svg_w = if max_x > min_x {
                            max_x - min_x
                        } else {
                            svg_res.width.max(10.0)
                        };
                        let svg_h = if max_y > min_y {
                            max_y - min_y
                        } else {
                            svg_res.height.max(10.0)
                        };
                        let orig_center_x = if min_x.is_finite() && max_x.is_finite() {
                            min_x + svg_w / 2.0
                        } else {
                            svg_w / 2.0
                        };
                        let orig_center_y = if min_y.is_finite() && max_y.is_finite() {
                            min_y + svg_h / 2.0
                        } else {
                            svg_h / 2.0
                        };

                        let max_dim = 800.0f32;
                        let scale = if svg_w > max_dim || svg_h > max_dim {
                            (max_dim / svg_w).min(max_dim / svg_h)
                        } else {
                            1.0
                        };

                        let offset_x = center.x - orig_center_x * scale;
                        let offset_y = center.y - orig_center_y * scale;

                        for mut elem in svg_res.elements {
                            if (scale - 1.0).abs() > 0.001 {
                                elem.scale(
                                    Point::new(orig_center_x, orig_center_y),
                                    scale,
                                    scale,
                                );
                            }
                            elem.translate(offset_x, offset_y);
                            state.document.add_element(elem);
                        }

                        state.notify_status();
                        drop(state);
                        self.drawing_area.queue_draw();
                        return Ok(());
                    }
                }
            }
        }

        // 2. Try loading as Skia Encoded Image (PNG, JPG, WebP, GIF, BMP, etc.)
        if let Some(image) = skia_safe::Image::from_encoded(skia_safe::Data::new_copy(&bytes)) {
            let iw = image.width() as f32;
            let ih = image.height() as f32;
            let max_dim = 800.0f32;
            let scale = if iw > max_dim || ih > max_dim {
                (max_dim / iw).min(max_dim / ih)
            } else {
                1.0
            };
            let w = (iw * scale).max(10.0);
            let h = (ih * scale).max(10.0);
            let rect = crate::core::Rect::new(center.x - w / 2.0, center.y - h / 2.0, w, h);
            let elem = crate::core::ImageElement::new(rect, bytes, Some(filename));
            state
                .document
                .add_element(crate::core::Element::Image(elem));
            state.notify_status();
            drop(state);
            self.drawing_area.queue_draw();
            return Ok(());
        }

        // 3. Fallback: try loading via GDK Texture
        let file = gio::File::for_path(path);
        if let Ok(texture) = gdk::Texture::from_file(&file) {
            let png_bytes = texture.save_to_png_bytes().to_vec();
            let iw = texture.width() as f32;
            let ih = texture.height() as f32;
            let max_dim = 800.0f32;
            let scale = if iw > max_dim || ih > max_dim {
                (max_dim / iw).min(max_dim / ih)
            } else {
                1.0
            };
            let w = (iw * scale).max(10.0);
            let h = (ih * scale).max(10.0);
            let rect = crate::core::Rect::new(center.x - w / 2.0, center.y - h / 2.0, w, h);
            let elem = crate::core::ImageElement::new(rect, png_bytes, Some(filename));
            state
                .document
                .add_element(crate::core::Element::Image(elem));
            state.notify_status();
            drop(state);
            self.drawing_area.queue_draw();
            return Ok(());
        }

        Err("Formato de arquivo não suportado ou corrompido".to_string())
    }

    pub fn current_file_path(&self) -> Option<PathBuf> {
        self.state
            .try_borrow()
            .ok()
            .and_then(|s| s.current_file_path.clone())
    }

    pub fn clear_canvas(&self) {
        let mut state = self.state.borrow_mut();
        state.document.clear();
        state.mark_dirty();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    // Page Management
    pub fn set_active_page_name(&self, name: String) {
        let mut state = self.state.borrow_mut();
        if let Some(pid) = state.document.active_page().map(|p| p.id) {
            state.document.rename_page(pid, name);
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn set_active_page_size(&self, width: f32, height: f32) {
        let mut state = self.state.borrow_mut();
        if let Some(page) = state.document.active_page() {
            let pid = page.id;
            let mut r = page.rect;
            r.width = width.max(50.0);
            r.height = height.max(50.0);
            state.document.set_page_rect(pid, r);
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn toggle_active_page_orientation(&self) {
        let mut state = self.state.borrow_mut();
        if let Some(page) = state.document.active_page() {
            let pid = page.id;
            let mut r = page.rect;
            std::mem::swap(&mut r.width, &mut r.height);
            state.document.set_page_rect(pid, r);
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn add_new_page(&self) {
        let mut state = self.state.borrow_mut();
        state.document.add_next_page();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn add_new_page_auto(&self) {
        let mut state = self.state.borrow_mut();
        let (w, h) = if let Some(last_p) = state.document.pages.last() {
            (last_p.rect.width, last_p.rect.height)
        } else {
            (ARTBOARD_WIDTH, ARTBOARD_HEIGHT)
        };
        let max_x = state
            .document
            .pages
            .iter()
            .map(|p| p.rect.x + p.rect.width)
            .fold(0.0f32, f32::max);
        let new_x = if max_x > 0.0 { max_x + 100.0 } else { 0.0 };
        let page_num = state.document.pages.len() + 1;
        state.document.add_page(
            Some(format!("Page {}", page_num)),
            crate::core::Rect::new(new_x, 0.0, w, h),
        );
        state.notify_status();
        drop(state);
        self.drawing_area.queue_draw();
    }

    pub fn delete_active_page(&self) {
        let mut state = self.state.borrow_mut();
        if let Some(pid) = state.document.active_page().map(|p| p.id) {
            if state.document.remove_page(pid) {
                state.notify_status();
                self.drawing_area.queue_draw();
            }
        }
    }

    // Guides & Grids
    pub fn grid_config(&self) -> GridConfig {
        self.state
            .try_borrow()
            .map(|s| s.grid_config)
            .unwrap_or_default()
    }

    pub fn set_grid_visible(&self, visible: bool) {
        let mut state = self.state.borrow_mut();
        state.grid_config.visible = visible;
        self.drawing_area.queue_draw();
    }

    pub fn is_grid_visible(&self) -> bool {
        self.state.borrow().grid_config.visible
    }

    pub fn set_grid_style(&self, style: GridStyle) {
        self.state.borrow_mut().grid_config.style = style;
        self.drawing_area.queue_draw();
    }

    pub fn set_grid_cell_size(&self, size: f32) {
        self.state.borrow_mut().grid_config.cell_size = size;
        self.drawing_area.queue_draw();
    }

    pub fn set_grid_subdivisions(&self, subs: u32) {
        let mut state = self.state.borrow_mut();
        state.grid_config.subdivisions = subs;
        self.drawing_area.queue_draw();
    }

    pub fn set_grid_opacity(&self, opacity: f32) {
        self.state.borrow_mut().grid_config.opacity = opacity;
        self.drawing_area.queue_draw();
    }

    pub fn set_grid_color(&self, color: Option<Color>) {
        self.state.borrow_mut().grid_config.color = color;
        self.drawing_area.queue_draw();
    }

    pub fn snap_config(&self) -> SnapConfig {
        self.state
            .try_borrow()
            .map(|s| s.snap_config)
            .unwrap_or_default()
    }

    pub fn set_snap_enabled(&self, enabled: bool) {
        let mut state = self.state.borrow_mut();
        state.snap_config.enabled = enabled;
        self.drawing_area.queue_draw();
    }

    pub fn set_snap_to_grid(&self, snap: bool) {
        let mut state = self.state.borrow_mut();
        state.snap_config.snap_to_grid = snap;
        self.drawing_area.queue_draw();
    }

    pub fn is_snap_to_grid(&self) -> bool {
        self.state.borrow().snap_config.snap_to_grid
    }

    pub fn set_snap_to_objects(&self, snap: bool) {
        let mut state = self.state.borrow_mut();
        state.snap_config.snap_to_objects = snap;
        self.drawing_area.queue_draw();
    }

    pub fn set_snap_to_artboard(&self, snap: bool) {
        let mut state = self.state.borrow_mut();
        state.snap_config.snap_to_artboard = snap;
        self.drawing_area.queue_draw();
    }

    pub fn set_snap_to_guides(&self, snap: bool) {
        let mut state = self.state.borrow_mut();
        state.snap_config.snap_to_guides = snap;
        self.drawing_area.queue_draw();
    }

    pub fn is_snap_to_guides(&self) -> bool {
        self.state.borrow().snap_config.snap_to_guides
    }

    pub fn ruler_config(&self) -> RulerConfig {
        self.state
            .try_borrow()
            .map(|s| s.ruler_config)
            .unwrap_or_default()
    }

    pub fn set_rulers_visible(&self, visible: bool) {
        let mut state = self.state.borrow_mut();
        state.ruler_config.visible = visible;
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn set_guides_visible(&self, visible: bool) {
        let mut state = self.state.borrow_mut();
        state.ruler_config.guides_visible = visible;
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn is_guides_visible(&self) -> bool {
        self.state.borrow().ruler_config.guides_visible
    }

    pub fn has_user_guides(&self) -> bool {
        !self.state.borrow().document.guides.is_empty()
    }

    pub fn clear_user_guides(&self) {
        let mut state = self.state.borrow_mut();
        state.document.guides.clear();
        state.notify_status();
        drop(state);
        self.drawing_area.queue_draw();
    }

    // Viewport & Zoom
    pub fn reset_zoom(&self) {
        let mut state = self.state.borrow_mut();
        state.viewport.reset();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn zoom_by(&self, factor: f32) {
        let mut state = self.state.borrow_mut();
        let center = Point::new(state.widget_size.0 / 2.0, state.widget_size.1 / 2.0);
        let widget_size = state.widget_size;
        state.viewport.zoom_at(center, factor, widget_size);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn zoom_to_fit_all(&self) {
        let mut state = self.state.borrow_mut();
        let target = if let Some(p) = state.document.active_page() {
            p.rect.normalize()
        } else {
            state.document.bounds().unwrap_or(crate::core::Rect::new(
                0.0,
                0.0,
                ARTBOARD_WIDTH,
                ARTBOARD_HEIGHT,
            ))
        };
        let ws = state.widget_size;
        state.viewport.zoom_to_rect(target, ws);
        drop(state);
        self.drawing_area.queue_draw();
    }

    // Layers & Selection
    pub fn toggle_element_visibility(&self, id: ElementId) {
        let mut state = self.state.borrow_mut();
        let cur_vis = state
            .document
            .elements
            .iter()
            .find(|e| e.id() == id)
            .map(|e| e.visible())
            .unwrap_or(true);
        state.document.set_element_visibility(id, !cur_vis);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn toggle_element_locked(&self, id: ElementId) {
        let mut state = self.state.borrow_mut();
        let cur_locked = state
            .document
            .elements
            .iter()
            .find(|e| e.id() == id)
            .map(|e| e.locked())
            .unwrap_or(false);
        state.document.set_element_locked(id, !cur_locked);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn move_layer_up(&self, id: ElementId) {
        let mut state = self.state.borrow_mut();
        state.document.move_layer_up(id);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn move_layer_down(&self, id: ElementId) {
        let mut state = self.state.borrow_mut();
        state.document.move_layer_down(id);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn select_layer(&self, id: ElementId, additive: bool) {
        let mut state = self.state.borrow_mut();
        state.document.select(id, additive);
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn is_element_selected(&self, id: ElementId) -> bool {
        self.state
            .try_borrow()
            .map(|s| s.document.selected_ids.contains(&id))
            .unwrap_or(false)
    }

    pub fn select_all(&self) {
        let mut state = self.state.borrow_mut();
        state.document.selected_ids = state.document.elements.iter().map(|e| e.id()).collect();
        state.notify_status();
        drop(state);
        self.drawing_area.queue_draw();
    }

    pub fn deselect_all(&self) {
        let mut state = self.state.borrow_mut();
        state.document.selected_ids.clear();
        state.notify_status();
        drop(state);
        self.drawing_area.queue_draw();
    }

    pub fn has_selection(&self) -> bool {
        self.state
            .try_borrow()
            .map(|s| !s.document.selected_ids.is_empty())
            .unwrap_or(false)
    }

    pub fn selection_count(&self) -> usize {
        self.state
            .try_borrow()
            .map(|s| s.document.selected_ids.len())
            .unwrap_or(0)
    }

    pub fn select_same_fill(&self) {
        let mut state = self.state.borrow_mut();
        state.document.select_same_fill();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn select_same_stroke(&self) {
        let mut state = self.state.borrow_mut();
        state.document.select_same_stroke();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn select_same_type(&self) {
        let mut state = self.state.borrow_mut();
        state.document.select_same_type();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn hide_selected(&self) {
        let mut state = self.state.borrow_mut();
        state.document.hide_selected();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn lock_selected(&self) {
        let mut state = self.state.borrow_mut();
        state.document.lock_selected();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn delete_selected_layers(&self) {
        let mut state = self.state.borrow_mut();
        state.document.remove_selected();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn bring_to_front(&self) {
        let mut state = self.state.borrow_mut();
        state.document.bring_selected_to_front();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn bring_forward(&self) {
        let mut state = self.state.borrow_mut();
        state.document.bring_selected_forward();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn send_backward(&self) {
        let mut state = self.state.borrow_mut();
        state.document.send_selected_backward();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn send_to_back(&self) {
        let mut state = self.state.borrow_mut();
        state.document.send_selected_to_back();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    // Grouping & Hierarchy
    pub fn can_group(&self) -> bool {
        self.state
            .try_borrow()
            .map(|s| s.document.selected_ids.len() >= 2)
            .unwrap_or(false)
    }

    pub fn can_ungroup(&self) -> bool {
        self.state
            .try_borrow()
            .map(|s| {
                s.document.elements.iter().any(|e| {
                    s.document.selected_ids.contains(&e.id())
                        && matches!(e, crate::core::Element::Group(_))
                })
            })
            .unwrap_or(false)
    }

    pub fn group_selected(&self) {
        let mut state = self.state.borrow_mut();
        if state.document.group_selected().is_some() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn ungroup_selected(&self) {
        let mut state = self.state.borrow_mut();
        if !state.document.ungroup_selected().is_empty() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn set_clip_group(&self) {
        let mut state = self.state.borrow_mut();
        if state.document.set_clip_group_selected().is_some() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn selected_group_name(&self) -> Option<String> {
        self.state.try_borrow().ok().and_then(|s| {
            s.document.elements.iter().find_map(|e| {
                if s.document.selected_ids.contains(&e.id()) {
                    if let crate::core::Element::Group(g) = e {
                        return Some(g.name.clone().unwrap_or_else(|| format!("#g{}", g.id.0)));
                    }
                }
                None
            })
        })
    }

    // Clipboard & Clones
    pub fn has_clipboard(&self) -> bool {
        self.state
            .try_borrow()
            .map(|s| !s.document.clipboard.is_empty())
            .unwrap_or(false)
    }

    pub fn copy_selected(&self) {
        let mut state = self.state.borrow_mut();
        state.document.copy_selected();
    }

    pub fn cut_selected(&self) {
        let mut state = self.state.borrow_mut();
        state.document.cut_selected();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn paste(&self, offset: Option<Point>) {
        let mut state = self.state.borrow_mut();
        let pasted = state.document.paste(offset);
        if !pasted.is_empty() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn paste_at_screen_pos(&self, screen_pos: Point) {
        let mut state = self.state.borrow_mut();
        if state.document.clipboard.is_empty() {
            return;
        }
        let world_pt = state
            .viewport
            .screen_to_world(screen_pos, state.widget_size);
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for el in &state.document.clipboard {
            let r = el.bounds();
            min_x = min_x.min(r.x);
            min_y = min_y.min(r.y);
            max_x = max_x.max(r.x + r.width);
            max_y = max_y.max(r.y + r.height);
        }
        let center_x = if min_x <= max_x {
            (min_x + max_x) * 0.5
        } else {
            0.0
        };
        let center_y = if min_y <= max_y {
            (min_y + max_y) * 0.5
        } else {
            0.0
        };
        let offset = Point::new(world_pt.x - center_x, world_pt.y - center_y);
        let pasted = state.document.paste(Some(offset));
        if !pasted.is_empty() {
            state.notify_status();
            drop(state);
            self.drawing_area.queue_draw();
        }
    }

    pub fn paste_in_place(&self) {
        self.paste(Some(Point::new(0.0, 0.0)));
    }

    pub fn duplicate_selected(&self) {
        let mut state = self.state.borrow_mut();
        let dups = state.document.duplicate_selected();
        if !dups.is_empty() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn clone_selected(&self) {
        let mut state = self.state.borrow_mut();
        if !state.document.clone_selected().is_empty() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn unlink_selected_clones(&self) {
        let mut state = self.state.borrow_mut();
        if !state.document.unlink_selected_clones().is_empty() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn select_original_element(&self) {
        let mut state = self.state.borrow_mut();
        state.document.select_original_element();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn select_linked_clones(&self) {
        let mut state = self.state.borrow_mut();
        state.document.select_linked_clones();
        state.notify_status();
        self.drawing_area.queue_draw();
    }

    pub fn has_clones_selected(&self) -> bool {
        self.state
            .try_borrow()
            .map(|s| s.document.has_clones_selected())
            .unwrap_or(false)
    }

    pub fn has_masters_selected(&self) -> bool {
        self.state
            .try_borrow()
            .map(|s| s.document.has_masters_selected())
            .unwrap_or(false)
    }

    pub fn can_convert_selected_to_path(&self) -> bool {
        self.state
            .try_borrow()
            .map(|s| s.document.has_non_path_selected())
            .unwrap_or(false)
    }

    // Render Options
    pub fn is_hardware_accelerated(&self) -> bool {
        self.state.borrow().render_options.hardware_accelerated
    }

    pub fn set_hardware_accelerated(&self, enabled: bool) {
        self.state.borrow_mut().render_options.hardware_accelerated = enabled;
        self.drawing_area.queue_draw();
    }

    pub fn is_high_precision_aa(&self) -> bool {
        self.state.borrow().render_options.high_precision_aa
    }

    pub fn set_high_precision_aa(&self, enabled: bool) {
        self.state.borrow_mut().render_options.high_precision_aa = enabled;
        self.drawing_area.queue_draw();
    }

    pub fn graphics_device_info(&self) -> String {
        if let Some(display) = gtk4::gdk::Display::default() {
            let hw = self.state.borrow().render_options.hardware_accelerated;
            let mode = if hw {
                crate::core::gettext("Hardware Accelerated (GPU / Skia Pipeline)")
            } else {
                crate::core::gettext("Software Fallback (CPU Raster)")
            };
            format!("{} — {}", display.type_().name(), mode)
        } else {
            crate::core::gettext("Skia Graphics Pipeline")
        }
    }

    pub fn canvas_bg_color(&self) -> Option<Color> {
        self.state.borrow().render_options.canvas_bg_color
    }

    pub fn set_canvas_bg_color(&self, color: Option<Color>) {
        self.state.borrow_mut().render_options.canvas_bg_color = color;
        self.drawing_area.queue_draw();
    }

    pub fn page_bg_color(&self) -> Option<Color> {
        self.state.borrow().render_options.page_bg_color
    }

    pub fn set_page_bg_color(&self, color: Option<Color>) {
        self.state.borrow_mut().render_options.page_bg_color = color;
        self.drawing_area.queue_draw();
    }

    pub fn page_shadow(&self) -> bool {
        self.state.borrow().render_options.page_shadow
    }

    pub fn set_page_shadow(&self, enabled: bool) {
        self.state.borrow_mut().render_options.page_shadow = enabled;
        self.drawing_area.queue_draw();
    }

    pub fn page_border(&self) -> bool {
        self.state.borrow().render_options.page_border
    }

    pub fn set_page_border(&self, enabled: bool) {
        self.state.borrow_mut().render_options.page_border = enabled;
        self.drawing_area.queue_draw();
    }

    pub fn show_workspace_dots(&self) -> bool {
        self.state.borrow().render_options.show_workspace_dots
    }

    pub fn set_show_workspace_dots(&self, enabled: bool) {
        self.state.borrow_mut().render_options.show_workspace_dots = enabled;
        self.drawing_area.queue_draw();
    }

    pub fn path_editor_config(&self) -> crate::core::PathEditorConfig {
        self.state.borrow().path_editor_config
    }

    pub fn set_path_editor_config(&self, config: crate::core::PathEditorConfig) {
        self.state.borrow_mut().path_editor_config = config;
        self.drawing_area.queue_draw();
    }
}
