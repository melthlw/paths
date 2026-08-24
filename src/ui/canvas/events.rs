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
                let mut state = state_clone.borrow_mut();
                if let Some(surface) = state.render_frame(width, height) {
                    let _ = cr.set_source_surface(&surface, 0.0, 0.0);
                    let _ = cr.paint();
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

                let (redraw, cursor) = state_drag.borrow_mut().pointer_down(
                    Point::new(x as f32, y as f32),
                    current_btn,
                    shift,
                    ctrl,
                    alt,
                );

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

                    let (redraw, cursor) = state_update.borrow_mut().pointer_move(
                        Point::new(cur_x as f32, cur_y as f32),
                        shift,
                        ctrl,
                        alt,
                    );

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

                    let (redraw, cursor) = state_end.borrow_mut().pointer_up(
                        Point::new(cur_x as f32, cur_y as f32),
                        current_btn,
                        shift,
                        ctrl,
                        alt,
                    );

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

                let (redraw, cursor) = state_motion.borrow_mut().pointer_move(
                    Point::new(x as f32, y as f32),
                    shift,
                    ctrl,
                    alt,
                );

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

        key_controller.connect_key_pressed(glib::clone!(
            #[weak]
            area_key,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_controller, keyval, _keycode, state_flags| {
                let mut state = state_key.borrow_mut();

                if keyval == gdk::Key::space && !state.plugin_manager.is_editing() {
                    state.is_space_down = true;
                    Self::apply_cursor(&area_key, &cc_key, "grab");
                    return glib::Propagation::Stop;
                }

                let key_event = crate::core::KeyEvent {
                    key: keyval,
                    shift_pressed: state_flags.contains(gdk::ModifierType::SHIFT_MASK),
                    ctrl_pressed: state_flags.contains(gdk::ModifierType::CONTROL_MASK),
                    alt_pressed: state_flags.contains(gdk::ModifierType::ALT_MASK),
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
    }
}
