use super::geometry::Point;
use gtk4::gdk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub struct PointerEvent {
    pub screen_pos: Point,
    pub world_pos: Point,
    pub button: Option<PointerButton>,
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: gdk::Key,
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,
}
