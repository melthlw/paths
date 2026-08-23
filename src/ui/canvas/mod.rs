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

        let widget = Self {
            drawing_area,
            state,
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
        self.state.borrow_mut().set_active_tool(tool_id);
        self.drawing_area.queue_draw();
    }

    pub fn undo(&self) {
        let mut state = self.state.borrow_mut();
        if state.document.undo() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn redo(&self) {
        let mut state = self.state.borrow_mut();
        if state.document.redo() {
            state.notify_status();
            self.drawing_area.queue_draw();
        }
    }

    pub fn notify_status(&self) {
        self.state.borrow().notify_status();
    }
}
