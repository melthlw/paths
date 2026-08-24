pub mod events;
pub mod ops_document;
pub mod ops_style;
pub mod ops_transform;
pub mod state;

pub use state::CanvasState;

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct CanvasWidget {
    pub(crate) drawing_area: gtk4::DrawingArea,
    pub(crate) state: Rc<RefCell<CanvasState>>,
    pub(crate) unit: Rc<std::cell::Cell<crate::core::Unit>>,
    pub(crate) enabled_plugins: Rc<RefCell<std::collections::HashSet<String>>>,
}

impl CanvasWidget {
    pub fn new() -> Self {
        let drawing_area = gtk4::DrawingArea::builder()
            .hexpand(true)
            .vexpand(true)
            .focusable(true)
            .can_focus(true)
            .css_classes(["canvas-area", "view"])
            .build();

        let state = Rc::new(RefCell::new(CanvasState::new()));
        let unit = Rc::new(std::cell::Cell::new(crate::core::Unit::Px));
        let enabled_plugins = Rc::new(RefCell::new(std::collections::HashSet::new()));

        let widget = Self {
            drawing_area,
            state,
            unit,
            enabled_plugins,
        };

        widget.setup_drawing();
        widget.setup_event_controllers();

        widget
    }

    pub fn widget(&self) -> &gtk4::DrawingArea {
        &self.drawing_area
    }

    pub fn state(&self) -> Rc<RefCell<CanvasState>> {
        self.state.clone()
    }

    pub fn shortcuts(&self) -> crate::core::ShortcutManager {
        self.state
            .try_borrow()
            .map(|s| s.shortcuts.clone())
            .unwrap_or_default()
    }

    pub fn set_shortcut_preset(&self, preset: crate::core::ShortcutPreset) {
        let mut state = self.state.borrow_mut();
        state.shortcuts.load_preset(preset);
    }

    pub fn set_custom_shortcut(
        &self,
        action: crate::core::ShortcutAction,
        combo: crate::core::KeyCombo,
    ) {
        let mut state = self.state.borrow_mut();
        state.shortcuts.set_shortcut(action, combo);
    }

    pub fn set_active_tool(&self, tool_id: &'static str) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.set_active_tool(tool_id);
        }
        self.drawing_area.queue_draw();
    }

    pub fn undo(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            if state.plugin_manager.is_editing() {
                let key_event = crate::core::KeyEvent {
                    key: gtk4::gdk::Key::z,
                    shift_pressed: false,
                    ctrl_pressed: true,
                    alt_pressed: false,
                };
                let (handled, redraw) = state.on_key_pressed(&key_event);
                if handled || redraw {
                    state.notify_status();
                    drop(state);
                    self.drawing_area.queue_draw();
                    return;
                }
            }
            if state.document.undo() {
                state.notify_status();
                drop(state);
                self.drawing_area.queue_draw();
            }
        }
    }

    pub fn redo(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            if state.document.redo() {
                state.notify_status();
                drop(state);
                self.drawing_area.queue_draw();
            }
        }
    }

    pub fn notify_status(&self) {
        self.state.borrow().notify_status();
    }
}
