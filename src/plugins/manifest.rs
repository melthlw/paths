//! Plugin Manifest — Declarative descriptors for plugin capabilities.

/// Which side of the canvas a bar is positioned on
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BarPosition {
    Left,
    Right,
    Top,
    #[default]
    Bottom,
}
