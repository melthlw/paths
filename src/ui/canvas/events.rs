use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;

use crate::core::{Point, PointerButton};
use super::CanvasWidget;

impl CanvasWidget {
    pub(crate) fn setup_drawing(&self) {
        let state_clone = self.state.clone();
        self.drawing_area
            .set_draw_func(move |_area, cr, width, height| {
                if let Ok(mut state) = state_clone.try_borrow_mut() {
                    if let Some(surface) = state.render_frame(width, height) {
                        let _ = cr.set_source_surface(&surface, 0.0, 0.0);
                        let _ = cr.paint();
                    }
                }
            });
    }

    pub(crate) fn setup_event_controllers(&self) {
        let area = &self.drawing_area;
        let cursor_cache = self.cursor_cache.clone();

        // 0. Gesture Click (Double click detection to edit text / select all)
        let gesture_click = gtk4::GestureClick::new();
        gesture_click.set_button(1);
        let state_click = self.state.clone();
        let area_click = self.drawing_area.clone();
        let cc_click = cursor_cache.clone();
        gesture_click.connect_pressed(glib::clone!(
            #[weak]
            area_click,
            move |_gesture, n_press, x, y| {
                if n_press == 2 {
                    let (redraw, cursor) = state_click
                        .borrow_mut()
                        .double_click(Point::new(x as f32, y as f32));
                    if let Some(c) = cursor {
                        Self::apply_cursor(&area_click, &cc_click, c);
                    }
                    if redraw {
                        area_click.queue_draw();
                    }
                }
            }
        ));
        area.add_controller(gesture_click);

        // 0b. Gesture Right Click (Context Menu for Objects & Canvas)
        let gesture_right_click = gtk4::GestureClick::new();
        gesture_right_click.set_button(3);
        let state_menu = self.state.clone();
        let area_menu = self.drawing_area.clone();
        let cc_menu = cursor_cache.clone();
        let context_menu = crate::ui::context_menu::ObjectContextMenu::new(self.clone(), area);
        let menu_clone = context_menu.clone();

        gesture_right_click.connect_pressed(glib::clone!(
            #[weak]
            area_menu,
            move |_gesture, _n_press, x, y| {
                let mut state = state_menu.borrow_mut();
                let screen_pt = Point::new(x as f32, y as f32);
                let world_pt = state.viewport.screen_to_world(screen_pt, state.widget_size);

                // If currently editing with a tool (like Pen drawing path), dispatch right-click to finish
                if state.plugin_manager.is_editing() {
                    let (redraw, cursor) = state.pointer_down(
                        world_pt,
                        PointerButton::Secondary,
                        false,
                        false,
                        false,
                    );
                    if let Some(c) = cursor {
                        Self::apply_cursor(&area_menu, &cc_menu, c);
                    }
                    if redraw {
                        area_menu.queue_draw();
                    }
                    state.notify_status();
                    return;
                }

                if let Some(hit_id) = state.document.hit_test(world_pt) {
                    if !state.document.selected_ids.contains(&hit_id) {
                        state.document.select(hit_id, false);
                        state.notify_status();
                        area_menu.queue_draw();
                    }
                } else {
                    // Right-clicked outside on empty canvas: deselect all objects
                    if !state.document.selected_ids.is_empty() {
                        state.document.selected_ids.clear();
                        state.notify_status();
                        area_menu.queue_draw();
                    }
                }
                drop(state);
                menu_clone.popup_at(screen_pt);
            }
        ));
        area.add_controller(gesture_right_click);

        // 1. Gesture Drag (Click, Drag, Release on any button: left, middle, right)
        let gesture_drag = gtk4::GestureDrag::new();
        gesture_drag.set_button(0);
        let state_drag = self.state.clone();
        let area_drag = self.drawing_area.clone();
        let cc_drag = cursor_cache.clone();

        gesture_drag.connect_drag_begin(glib::clone!(
            #[weak]
            area_drag,
            move |gesture, x, y| {
                area_drag.grab_focus();
                let current_btn = match gesture.current_button() {
                    1 => PointerButton::Primary,
                    2 => PointerButton::Middle,
                    3 => PointerButton::Secondary,
                    _ => PointerButton::Primary,
                };
                let event = gesture.current_event();
                let shift = event.as_ref().map_or(false, |e| {
                    e.modifier_state().contains(gdk::ModifierType::SHIFT_MASK)
                });
                let ctrl = event.as_ref().map_or(false, |e| {
                    e.modifier_state().contains(gdk::ModifierType::CONTROL_MASK)
                });
                let alt = event.as_ref().map_or(false, |e| {
                    e.modifier_state().contains(gdk::ModifierType::ALT_MASK)
                });

                let (redraw, cursor) = if let Ok(mut state) = state_drag.try_borrow_mut() {
                    state.pointer_down(
                        Point::new(x as f32, y as f32),
                        current_btn,
                        shift,
                        ctrl,
                        alt,
                    )
                } else {
                    (false, None)
                };

                if let Ok(st) = state_drag.try_borrow() {
                    st.notify_status();
                }

                if let Some(cursor_name) = cursor {
                    Self::apply_cursor(&area_drag, &cc_drag, cursor_name);
                }
                if redraw {
                    area_drag.queue_draw();
                }
            }
        ));

        let state_update = self.state.clone();
        let area_update = self.drawing_area.clone();
        let cc_update = cursor_cache.clone();
        gesture_drag.connect_drag_update(glib::clone!(
            #[weak]
            area_update,
            move |gesture, offset_x, offset_y| {
                if let Some((start_x, start_y)) = gesture.start_point() {
                    let cur_x = start_x + offset_x;
                    let cur_y = start_y + offset_y;
                    let event = gesture.current_event();
                    let shift = event.as_ref().map_or(false, |e| {
                        e.modifier_state().contains(gdk::ModifierType::SHIFT_MASK)
                    });
                    let ctrl = event.as_ref().map_or(false, |e| {
                        e.modifier_state().contains(gdk::ModifierType::CONTROL_MASK)
                    });
                    let alt = event.as_ref().map_or(false, |e| {
                        e.modifier_state().contains(gdk::ModifierType::ALT_MASK)
                    });

                    let (redraw, cursor) = if let Ok(mut state) = state_update.try_borrow_mut() {
                        state.pointer_move(
                            Point::new(cur_x as f32, cur_y as f32),
                            shift,
                            ctrl,
                            alt,
                        )
                    } else {
                        (false, None)
                    };

                    if let Some(cursor_name) = cursor {
                        Self::apply_cursor(&area_update, &cc_update, cursor_name);
                    }
                    if redraw {
                        area_update.queue_draw();
                    }
                }
            }
        ));

        let state_end = self.state.clone();
        let area_end = self.drawing_area.clone();
        let cc_end = cursor_cache.clone();
        gesture_drag.connect_drag_end(glib::clone!(
            #[weak]
            area_end,
            move |gesture, offset_x, offset_y| {
                if let Some((start_x, start_y)) = gesture.start_point() {
                    let cur_x = start_x + offset_x;
                    let cur_y = start_y + offset_y;
                    let current_btn = match gesture.current_button() {
                        1 => PointerButton::Primary,
                        2 => PointerButton::Middle,
                        3 => PointerButton::Secondary,
                        _ => PointerButton::Primary,
                    };
                    let event = gesture.current_event();
                    let shift = event.as_ref().map_or(false, |e| {
                        e.modifier_state().contains(gdk::ModifierType::SHIFT_MASK)
                    });
                    let ctrl = event.as_ref().map_or(false, |e| {
                        e.modifier_state().contains(gdk::ModifierType::CONTROL_MASK)
                    });
                    let alt = event.as_ref().map_or(false, |e| {
                        e.modifier_state().contains(gdk::ModifierType::ALT_MASK)
                    });

                    let (redraw, cursor) = if let Ok(mut state) = state_end.try_borrow_mut() {
                        state.pointer_up(
                            Point::new(cur_x as f32, cur_y as f32),
                            current_btn,
                            shift,
                            ctrl,
                            alt,
                        )
                    } else {
                        (false, None)
                    };

                    if let Ok(st) = state_end.try_borrow() {
                        st.notify_status();
                    }

                    if let Some(cursor_name) = cursor {
                        Self::apply_cursor(&area_end, &cc_end, cursor_name);
                    }
                    if redraw {
                        area_end.queue_draw();
                    }
                }
            }
        ));

        area.add_controller(gesture_drag);

        // 2. Motion Controller (Hover & Cursor Tracking)
        let motion_controller = gtk4::EventControllerMotion::new();
        let state_motion = self.state.clone();
        let area_motion = self.drawing_area.clone();
        let cc_motion = cursor_cache.clone();

        motion_controller.connect_motion(glib::clone!(
            #[weak]
            area_motion,
            move |controller, x, y| {
                let event = controller.current_event();
                let shift = event.as_ref().map_or(false, |e| {
                    e.modifier_state().contains(gdk::ModifierType::SHIFT_MASK)
                });
                let ctrl = event.as_ref().map_or(false, |e| {
                    e.modifier_state().contains(gdk::ModifierType::CONTROL_MASK)
                });
                let alt = event.as_ref().map_or(false, |e| {
                    e.modifier_state().contains(gdk::ModifierType::ALT_MASK)
                });

                let (redraw, cursor) = if let Ok(mut state) = state_motion.try_borrow_mut() {
                    state.pointer_move(
                        Point::new(x as f32, y as f32),
                        shift,
                        ctrl,
                        alt,
                    )
                } else {
                    (false, None)
                };

                if let Some(cursor_name) = cursor {
                    Self::apply_cursor(&area_motion, &cc_motion, cursor_name);
                }
                if redraw {
                    area_motion.queue_draw();
                }
            }
        ));

        area.add_controller(motion_controller);

        // 3. Scroll Controller (Smooth Zoom at Cursor)
        let scroll_controller =
            gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
        let state_scroll = self.state.clone();
        let area_scroll = self.drawing_area.clone();

        scroll_controller.connect_scroll(glib::clone!(
            #[weak]
            area_scroll,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_controller, _dx, dy| {
                let mut state = state_scroll.borrow_mut();
                let zoom_factor = if dy < 0.0 { 1.15 } else { 1.0 / 1.15 };
                let cursor_pos = state.cursor_pos;
                let widget_size = state.widget_size;

                state.viewport.zoom_at(cursor_pos, zoom_factor, widget_size);
                state.notify_status();
                area_scroll.queue_draw();
                glib::Propagation::Stop
            }
        ));

        area.add_controller(scroll_controller);

        // 4. Keyboard Controller (Dynamic shortcuts driven by ShortcutManager)
        let key_controller = gtk4::EventControllerKey::new();
        let state_key = self.state.clone();
        let area_key = self.drawing_area.clone();
        let cc_key = cursor_cache.clone();

        let canvas_key = self.clone();
        key_controller.connect_key_pressed(glib::clone!(
            #[weak]
            area_key,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_controller, keyval, _keycode, state_flags| {
                let is_ctrl = state_flags.contains(gdk::ModifierType::CONTROL_MASK);
                let is_alt = state_flags.contains(gdk::ModifierType::ALT_MASK);

                if is_ctrl && is_alt && (keyval == gdk::Key::c || keyval == gdk::Key::C) {
                    canvas_key.copy_selected_style();
                    return glib::Propagation::Stop;
                }
                if is_ctrl && is_alt && (keyval == gdk::Key::v || keyval == gdk::Key::V) {
                    canvas_key.paste_style_to_selected();
                    return glib::Propagation::Stop;
                }
                if is_ctrl && !is_alt && (keyval == gdk::Key::v || keyval == gdk::Key::V) {
                    canvas_key.paste_from_clipboard();
                    return glib::Propagation::Stop;
                }

                let mut state = state_key.borrow_mut();

                if keyval == gdk::Key::space && !state.plugin_manager.is_editing() {
                    state.is_space_down = true;
                    Self::apply_cursor(&area_key, &cc_key, "grab");
                    return glib::Propagation::Stop;
                }

                let key_event = crate::core::KeyEvent {
                    key: keyval,
                    shift_pressed: state_flags.contains(gdk::ModifierType::SHIFT_MASK),
                    ctrl_pressed: is_ctrl,
                    alt_pressed: is_alt,
                };

                let (handled, redraw) = state.on_key_pressed(&key_event);

                if handled || redraw {
                    state.notify_status();
                    area_key.queue_draw();
                    return glib::Propagation::Stop;
                }

                glib::Propagation::Proceed
            }
        ));

        let state_keyup = self.state.clone();
        let area_keyup = self.drawing_area.clone();
        let cc_keyup = cursor_cache.clone();
        key_controller.connect_key_released(glib::clone!(
            #[weak]
            area_keyup,
            move |_controller, keyval, _keycode, _state_flags| {
                if keyval == gdk::Key::space {
                    state_keyup.borrow_mut().is_space_down = false;
                    Self::apply_cursor(&area_keyup, &cc_keyup, "default");
                }
            }
        ));

        area.add_controller(key_controller);

        // 5. Drag and Drop Target (Files, Images, Assets, Swatches, Patterns, Icons, Shapes)
        let drop_target = gtk4::DropTarget::new(glib::types::Type::INVALID, gdk::DragAction::COPY);
        drop_target.set_types(&[
            gdk::FileList::static_type(),
            gio::File::static_type(),
            gdk::Texture::static_type(),
            glib::types::Type::STRING,
        ]);

        let state_drop = self.state.clone();
        let area_drop = self.drawing_area.clone();
        drop_target.connect_drop(move |_target, value, x, y| {
            let screen_pt = Point::new(x as f32, y as f32);
            let mut success = false;

            if let Ok(file_list) = value.get::<gdk::FileList>() {
                if let Ok(mut state) = state_drop.try_borrow_mut() {
                    let world_pt = state.viewport.screen_to_world(screen_pt, state.widget_size);
                    for file in file_list.files() {
                        if let Some(path) = file.path() {
                            if handle_file_import(&mut state, &path, world_pt) {
                                success = true;
                            }
                        }
                    }
                    if success {
                        state.notify_status();
                    }
                }
            } else if let Ok(file) = value.get::<gio::File>() {
                if let Some(path) = file.path() {
                    if let Ok(mut state) = state_drop.try_borrow_mut() {
                        let world_pt = state.viewport.screen_to_world(screen_pt, state.widget_size);
                        if handle_file_import(&mut state, &path, world_pt) {
                            success = true;
                            state.notify_status();
                        }
                    }
                }
            } else if let Ok(payload) = value.get::<String>() {
                if let Ok(mut state) = state_drop.try_borrow_mut() {
                    let world_pt = state.viewport.screen_to_world(screen_pt, state.widget_size);
                    if payload.starts_with("file://") || payload.starts_with('/') {
                        let path_str = payload.trim_start_matches("file://").trim();
                        let path = std::path::PathBuf::from(path_str);
                        if handle_file_import(&mut state, &path, world_pt) {
                            success = true;
                        }
                    } else if handle_asset_drop(&mut state, &payload, world_pt) {
                        success = true;
                    }
                    if success {
                        state.notify_status();
                    }
                }
            }

            if success {
                area_drop.queue_draw();
            }
            success
        });
        area.add_controller(drop_target);
    }
}

pub fn handle_asset_drop(state: &mut super::state::CanvasState, payload: &str, world_pt: Point) -> bool {
    use crate::core::element::{Element, FillLayer, FillStyle, PatternType, StrokeLayer, StrokeStyle};
    use crate::core::Color;

    if let Some(hex) = payload.strip_prefix("gnome-paths:fill:") {
        if let Some(col) = Color::from_hex(hex) {
            state.document.snapshot();
            if let Some(hit_id) = state.document.hit_test(world_pt) {
                if let Some(el) = state.document.find_element_mut(hit_id) {
                    el.set_fill_color(Some(col));
                }
                state.document.selected_ids.clear();
                state.document.selected_ids.insert(hit_id);
            } else {
                state.active_fill_color = col;
                if !state.document.selected_ids.is_empty() {
                    state.document.set_selected_fill_color(Some(col));
                }
            }
            return true;
        }
    } else if payload == "gnome-paths:fill-none" {
        state.document.snapshot();
        if let Some(hit_id) = state.document.hit_test(world_pt) {
            if let Some(el) = state.document.find_element_mut(hit_id) {
                el.set_fill_color(None);
            }
            state.document.selected_ids.clear();
            state.document.selected_ids.insert(hit_id);
        } else {
            state.active_fill_color = Color::new(0.0, 0.0, 0.0, 0.0);
            if !state.document.selected_ids.is_empty() {
                state.document.set_selected_fill_color(None);
            }
        }
        return true;
    } else if let Some(hex) = payload.strip_prefix("gnome-paths:stroke:") {
        if let Some(col) = Color::from_hex(hex) {
            state.document.snapshot();
            if let Some(hit_id) = state.document.hit_test(world_pt) {
                if let Some(el) = state.document.find_element_mut(hit_id) {
                    el.set_stroke_color(Some(col));
                }
                state.document.selected_ids.clear();
                state.document.selected_ids.insert(hit_id);
            } else {
                state.active_stroke_color = Some(col);
                if !state.document.selected_ids.is_empty() {
                    state.document.set_selected_stroke_color(Some(col));
                }
            }
            return true;
        }
    } else if payload == "gnome-paths:stroke-none" {
        state.document.snapshot();
        if let Some(hit_id) = state.document.hit_test(world_pt) {
            if let Some(el) = state.document.find_element_mut(hit_id) {
                el.set_stroke_color(None);
            }
            state.document.selected_ids.clear();
            state.document.selected_ids.insert(hit_id);
        } else {
            state.active_stroke_color = None;
            if !state.document.selected_ids.is_empty() {
                state.document.set_selected_stroke_color(None);
            }
        }
        return true;
    } else if let Some(hex) = payload.strip_prefix("gnome-paths:swatch:") {
        if let Some(col) = Color::from_hex(hex) {
            state.document.snapshot();
            if let Some(hit_id) = state.document.hit_test(world_pt) {
                if let Some(el) = state.document.find_element_mut(hit_id) {
                    el.set_fill_color(Some(col));
                }
                state.document.selected_ids.clear();
                state.document.selected_ids.insert(hit_id);
            } else {
                state.active_fill_color = col;
                if !state.document.selected_ids.is_empty() {
                    state.document.set_selected_fill_color(Some(col));
                }
            }
            return true;
        }
    } else if let Some(pattern_str) = payload.strip_prefix("gnome-paths:pattern:") {
        let parts: Vec<&str> = pattern_str.split(':').collect();
        let pt = match parts.first().copied().unwrap_or("") {
            "Grid" => PatternType::Grid,
            "Dots" => PatternType::Dots,
            "Stripes" => PatternType::Stripes,
            "Checkerboard" => PatternType::Checkerboard,
            "Hexagon" => PatternType::Hexagon,
            "Crosshatch" => PatternType::Crosshatch,
            "Brick" => PatternType::Brick,
            "Scales" => PatternType::Scales,
            "Houndstooth" => PatternType::Houndstooth,
            "Basketweave" => PatternType::Basketweave,
            "Custom" => PatternType::Custom,
            _ => PatternType::Grid,
        };
        let scale: f32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(16.0);
        let custom_pattern_path = if pt == PatternType::Custom {
            let pattern_name = parts.get(2).copied().unwrap_or("");
            let all_custom = crate::core::scan_user_patterns();
            all_custom.into_iter().find(|cp| cp.name == pattern_name).map(|cp| cp.file_path)
        } else {
            None
        };
        let p_layer = FillLayer {
            style: FillStyle::Pattern,
            color: state.active_fill_color,
            secondary_color: Color::new(0.12, 0.14, 0.18, 1.0),
            stops: Vec::new(),
            angle: 0.0,
            opacity: 1.0,
            enabled: true,
            pattern_type: pt,
            pattern_scale: scale,
            pattern_offset: crate::core::Point::ZERO,
            custom_pattern_path,
            mesh: None,
        };
        state.document.snapshot();
        if let Some(hit_id) = state.document.hit_test(world_pt) {
            if let Some(el) = state.document.find_element_mut(hit_id) {
                el.set_fills(vec![p_layer]);
            }
            state.document.selected_ids.clear();
            state.document.selected_ids.insert(hit_id);
        } else if !state.document.selected_ids.is_empty() {
            state.document.set_selected_fills(vec![p_layer]);
        } else {
            let mut rect_elem = crate::core::element::RectElement::new(
                crate::core::geometry::Rect::new(world_pt.x - 60.0, world_pt.y - 60.0, 120.0, 120.0),
                None,
                None,
            );
            rect_elem.fills = vec![p_layer];
            let el = Element::Rect(rect_elem);
            let id = el.id();
            state.document.add_element(el);
            state.document.selected_ids.clear();
            state.document.selected_ids.insert(id);
        }
        return true;
    } else if let Some(icon_str) = payload.strip_prefix("gnome-paths:icon:") {
        if let Some(idx) = icon_str.find(':') {
            let path_d = &icon_str[idx + 1..];
            let path_elements = crate::core::svg_import::parse_svg_path_to_elements(
                path_d,
                Some(state.active_fill_color),
                None,
                1.0,
            );
            if !path_elements.is_empty() {
                state.document.snapshot();
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for el in &path_elements {
                    let b = el.bounds();
                    min_x = min_x.min(b.x);
                    min_y = min_y.min(b.y);
                    max_x = max_x.max(b.x + b.width);
                    max_y = max_y.max(b.y + b.height);
                }
                let total_b = crate::core::geometry::Rect::new(min_x, min_y, (max_x - min_x).max(0.1), (max_y - min_y).max(0.1));
                let target_size = 64.0f32;
                let max_dim = total_b.width.max(total_b.height);
                let scale = if max_dim > 0.1 && max_dim < 36.0 {
                    target_size / max_dim
                } else {
                    1.0
                };
                let center_b = Point::new(total_b.x + total_b.width / 2.0, total_b.y + total_b.height / 2.0);
                let dx = world_pt.x - center_b.x;
                let dy = world_pt.y - center_b.y;

                state.document.selected_ids.clear();
                for mut el in path_elements {
                    if scale != 1.0 {
                        el.scale(center_b, scale, scale);
                    }
                    el.translate(dx, dy);
                    let element_id = el.id;
                    state.document.add_element(Element::Path(el));
                    state.document.selected_ids.insert(element_id);
                }
                return true;
            }
        }
    } else if let Some(shape_str) = payload.strip_prefix("gnome-paths:shape:") {
        if let Some(idx) = shape_str.find(':') {
            let path_d = &shape_str[idx + 1..];
            let path_elements = crate::core::svg_import::parse_svg_path_to_elements(
                path_d,
                Some(state.active_fill_color),
                None,
                1.0,
            );
            if !path_elements.is_empty() {
                state.document.snapshot();
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for el in &path_elements {
                    let b = el.bounds();
                    min_x = min_x.min(b.x);
                    min_y = min_y.min(b.y);
                    max_x = max_x.max(b.x + b.width);
                    max_y = max_y.max(b.y + b.height);
                }
                let total_b = crate::core::geometry::Rect::new(min_x, min_y, (max_x - min_x).max(0.1), (max_y - min_y).max(0.1));
                let target_size = 80.0f32;
                let max_dim = total_b.width.max(total_b.height);
                let scale = if max_dim > 0.1 && max_dim < 36.0 {
                    target_size / max_dim
                } else {
                    1.0
                };
                let center_b = Point::new(total_b.x + total_b.width / 2.0, total_b.y + total_b.height / 2.0);
                let dx = world_pt.x - center_b.x;
                let dy = world_pt.y - center_b.y;

                state.document.selected_ids.clear();
                for mut el in path_elements {
                    if scale != 1.0 {
                        el.scale(center_b, scale, scale);
                    }
                    el.translate(dx, dy);
                    let element_id = el.id;
                    state.document.add_element(Element::Path(el));
                    state.document.selected_ids.insert(element_id);
                }
                return true;
            }
        }
    } else if let Some(stroke_str) = payload.strip_prefix("gnome-paths:stroke:") {
        let parts: Vec<&str> = stroke_str.split(':').collect();
        let s_style = match parts.first().copied().unwrap_or("") {
            "Solid" => StrokeStyle::Solid,
            "Dashed" => StrokeStyle::Dashed,
            "Dotted" => StrokeStyle::Dotted,
            _ => StrokeStyle::Solid,
        };
        let s_layer = StrokeLayer {
            color: state.active_stroke_color.unwrap_or(Color::BLACK),
            width: state.active_stroke_width.max(1.5),
            style: s_style,
            opacity: 1.0,
            enabled: true,
        };
        state.document.snapshot();
        if let Some(hit_id) = state.document.hit_test(world_pt) {
            if let Some(el) = state.document.find_element_mut(hit_id) {
                let mut strokes = el.strokes().to_vec();
                if strokes.is_empty() {
                    strokes.push(s_layer);
                } else {
                    strokes[0] = s_layer;
                }
                el.set_strokes(strokes);
            }
            state.document.selected_ids.clear();
            state.document.selected_ids.insert(hit_id);
        } else {
            state.active_stroke_width = s_layer.width;
            if !state.document.selected_ids.is_empty() {
                state.document.set_selected_strokes(vec![s_layer]);
            }
        }
        return true;
    } else if let Some(text_str) = payload.strip_prefix("gnome-paths:text:") {
        let parts: Vec<&str> = text_str.split(':').collect();
        let fam = parts.first().copied().unwrap_or("Sans");
        let size: f32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(24.0);
        let weight: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(400);
        let sample = parts.get(3).copied().unwrap_or("Text");
        state.document.snapshot();
        if let Some(hit_id) = state.document.hit_test(world_pt) {
            if let Some(el) = state.document.find_element_mut(hit_id) {
                if let Element::Text(t) = el {
                    t.font_family = fam.to_string();
                    t.font_size = size;
                    t.font_weight = weight;
                }
            }
            state.document.selected_ids.clear();
            state.document.selected_ids.insert(hit_id);
        } else {
            let mut text_elem = crate::core::element::TextElement::new(
                world_pt,
                sample.to_string(),
                size,
                state.active_fill_color,
            );
            text_elem.font_family = fam.to_string();
            text_elem.font_weight = weight;
            let el = Element::Text(text_elem);
            let id = el.id();
            state.document.add_element(el);
            state.document.selected_ids.clear();
            state.document.selected_ids.insert(id);
        }
        return true;
    }
    false
}

pub fn handle_file_import(state: &mut super::state::CanvasState, path: &std::path::Path, world_pt: Point) -> bool {
    use crate::core::element::Element;
    use crate::core::geometry::Rect;
    use skia_safe as skia;

    if !path.exists() {
        return false;
    }
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let file_name = path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string());

    if ext == "svg" {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(svg_res) = crate::core::parse_svg(&content) {
                if !svg_res.elements.is_empty() {
                    state.document.snapshot();
                    state.document.selected_ids.clear();

                    let mut b_opt: Option<Rect> = None;
                    for el in &svg_res.elements {
                        let eb = el.bounds();
                        b_opt = Some(match b_opt {
                            Some(acc) => acc.union(eb),
                            None => eb,
                        });
                    }
                    let total_b = b_opt.unwrap_or(Rect::new(0.0, 0.0, 100.0, 100.0));
                    let dx = world_pt.x - (total_b.x + total_b.width * 0.5);
                    let dy = world_pt.y - (total_b.y + total_b.height * 0.5);

                    for mut el in svg_res.elements {
                        el.translate(dx, dy);
                        let id = el.id();
                        state.document.add_element(el);
                        state.document.selected_ids.insert(id);
                    }
                    return true;
                }
            }
        }
    } else if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp") {
        if let Ok(bytes) = std::fs::read(path) {
            if let Some(sk_img) = skia::Image::from_encoded(skia::Data::new_copy(&bytes)) {
                let w = sk_img.width() as f32;
                let h = sk_img.height() as f32;
                let rect = Rect::new(world_pt.x - w * 0.5, world_pt.y - h * 0.5, w, h);
                let img_elem = crate::core::element::ImageElement::new(rect, bytes, file_name);
                let el = Element::Image(img_elem);
                let id = el.id();
                state.document.snapshot();
                state.document.add_element(el);
                state.document.selected_ids.clear();
                state.document.selected_ids.insert(id);
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::element::RectElement;
    use crate::core::geometry::Rect;
    use crate::core::{Color, Element};

    #[test]
    fn test_handle_asset_drop_fill_and_stroke() {
        let mut state = crate::ui::canvas::state::CanvasState::new();

        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let green = Color::new(0.0, 1.0, 0.0, 1.0);
        let rect = RectElement::new(Rect::new(0.0, 0.0, 100.0, 100.0), Some(Color::BLACK), None);
        let rect_id = rect.id;
        state.document.add_element(Element::Rect(rect));

        // 1. Drag fill onto unselected element
        let success = handle_asset_drop(&mut state, &format!("gnome-paths:fill:{}", red.to_hex()), Point::new(50.0, 50.0));
        assert!(success);
        let el = state.document.find_element(rect_id).unwrap();
        assert_eq!(el.fill_color(), Some(red));
        assert!(state.document.selected_ids.contains(&rect_id));

        // 2. Drag stroke onto element
        let success_stroke = handle_asset_drop(&mut state, &format!("gnome-paths:stroke:{}", green.to_hex()), Point::new(50.0, 50.0));
        assert!(success_stroke);
        let el_after_stroke = state.document.find_element(rect_id).unwrap();
        assert_eq!(el_after_stroke.stroke_color(), Some(green));

        // 3. Drag fill-none onto element
        let success_none = handle_asset_drop(&mut state, "gnome-paths:fill-none", Point::new(50.0, 50.0));
        assert!(success_none);
        let el_no_fill = state.document.find_element(rect_id).unwrap();
        assert_eq!(el_no_fill.fill_color(), None);

        // 4. Drag stroke-none onto element
        let success_stroke_none = handle_asset_drop(&mut state, "gnome-paths:stroke-none", Point::new(50.0, 50.0));
        assert!(success_stroke_none);
        let el_no_stroke = state.document.find_element(rect_id).unwrap();
        assert_eq!(el_no_stroke.stroke_color(), None);
    }

    #[test]
    fn test_element_style_copy_and_paste() {
        let r1 = RectElement::new(Rect::new(0.0, 0.0, 50.0, 50.0), Some(Color::RED), Some(Color::BLUE));
        let mut el1 = Element::Rect(r1);
        el1.set_stroke_width(4.5);
        el1.set_opacity(0.8);

        let snapshot = el1.extract_style_snapshot();
        assert_eq!(snapshot.fill_color, Some(Color::RED));
        assert_eq!(snapshot.stroke_color, Some(Color::BLUE));
        assert_eq!(snapshot.stroke_width, 4.5);
        assert_eq!(snapshot.opacity, 0.8);

        let r2 = RectElement::new(Rect::new(100.0, 100.0, 50.0, 50.0), Some(Color::BLACK), None);
        let mut el2 = Element::Rect(r2);
        el2.apply_style_snapshot(&snapshot);

        assert_eq!(el2.fill_color(), Some(Color::RED));
        assert_eq!(el2.stroke_color(), Some(Color::BLUE));
        assert_eq!(el2.stroke_width(), 4.5);
        assert_eq!(el2.opacity(), 0.8);
    }
}
