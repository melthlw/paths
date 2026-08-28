use gtk4::gdk;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    // ── Tool Selection ──
    ToolSelect,
    ToolPathEditor,
    ToolRectangle,
    ToolCircle,
    ToolStar,
    ToolTriangle,
    ToolSpiral,
    ToolPen,
    ToolBrush,
    ToolText,
    ToolImage,
    ToolPaintBucket,
    ToolEyedropper,
    ToolGradient,
    ToolMeshGradient,
    ToolMeasure,
    ToolZoom,
    ToolPage,

    // ── Zoom & View ──
    Zoom100,
    ZoomPage,
    ZoomSelection,
    ZoomFitAll,
    ToggleRulers,
    ToggleGuides,
    ToggleGrid,
    ToggleSnap,

    // ── Edit & Document ──
    Save,
    SaveAs,
    Open,
    NewDocument,
    Export,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Duplicate,
    Delete,
    SelectAll,
    Deselect,
    Group,
    Ungroup,
    ConvertToPath,
    BringForward,
    BringToFront,
    SendBackward,
    SendToBack,
    ToggleLock,
    ToggleHide,
    CloneSelected,
    UnlinkClone,
    SelectOriginal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutCategory {
    Tools,
    View,
    Edit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCombo {
    pub key: gdk::Key,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyCombo {
    pub const fn new(key: gdk::Key, ctrl: bool, shift: bool, alt: bool) -> Self {
        Self {
            key,
            ctrl,
            shift,
            alt,
        }
    }

    pub fn matches(&self, key: gdk::Key, ctrl: bool, shift: bool, alt: bool) -> bool {
        if self.ctrl != ctrl || self.shift != shift || self.alt != alt {
            return false;
        }

        // Direct match
        if self.key == key {
            return true;
        }

        // Case-insensitive letter matching (e.g. gdk::Key::v vs gdk::Key::V)
        let key_a = self.key.to_unicode();
        let key_b = key.to_unicode();
        if let (Some(a), Some(b)) = (key_a, key_b) {
            if a.to_ascii_lowercase() == b.to_ascii_lowercase() {
                return true;
            }
        }

        // Number keys (e.g. gdk::Key::_1 vs numeric 1)
        if self.key.name() == key.name() {
            return true;
        }

        false
    }

    pub fn to_display_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }

        let key_str = match self.key {
            gdk::Key::Delete => "Delete".to_string(),
            gdk::Key::BackSpace => "Backspace".to_string(),
            gdk::Key::Escape => "Esc".to_string(),
            gdk::Key::space => "Espaço".to_string(),
            gdk::Key::Return => "Enter".to_string(),
            gdk::Key::Tab => "Tab".to_string(),
            gdk::Key::bracketleft => "[".to_string(),
            gdk::Key::bracketright => "]".to_string(),
            gdk::Key::apostrophe => "'".to_string(),
            gdk::Key::quotedbl => "\"".to_string(),
            gdk::Key::semicolon => ";".to_string(),
            gdk::Key::colon => ":".to_string(),
            gdk::Key::comma => ",".to_string(),
            gdk::Key::period => ".".to_string(),
            gdk::Key::slash => "/".to_string(),
            gdk::Key::backslash => "\\".to_string(),
            gdk::Key::minus => "-".to_string(),
            gdk::Key::equal => "=".to_string(),
            gdk::Key::_0 => "0".to_string(),
            gdk::Key::_1 => "1".to_string(),
            gdk::Key::_2 => "2".to_string(),
            gdk::Key::_3 => "3".to_string(),
            gdk::Key::_4 => "4".to_string(),
            gdk::Key::_5 => "5".to_string(),
            gdk::Key::_6 => "6".to_string(),
            gdk::Key::_7 => "7".to_string(),
            gdk::Key::_8 => "8".to_string(),
            gdk::Key::_9 => "9".to_string(),
            _ => {
                if let Some(name) = self.key.name() {
                    let s = name.as_str();
                    if s.len() == 1 {
                        s.to_uppercase()
                    } else if s.starts_with("KP_") {
                        format!("Num {}", &s[3..])
                    } else if s.starts_with('F') && s[1..].parse::<u32>().is_ok() {
                        s.to_string()
                    } else {
                        s.to_uppercase()
                    }
                } else {
                    "?".to_string()
                }
            }
        };

        parts.push(&key_str);
        parts.join(" + ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutPreset {
    Default,
    Figma,
    Illustrator,
    Inkscape,
}

#[derive(Debug, Clone)]
pub struct ShortcutManager {
    pub preset: ShortcutPreset,
    bindings: HashMap<ShortcutAction, KeyCombo>,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        let mut manager = Self {
            preset: ShortcutPreset::Default,
            bindings: HashMap::new(),
        };
        manager.load_preset(ShortcutPreset::Default);
        manager
    }
}

impl ShortcutManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_shortcut(&self, action: ShortcutAction) -> Option<KeyCombo> {
        self.bindings.get(&action).cloned()
    }

    pub fn set_shortcut(&mut self, action: ShortcutAction, combo: KeyCombo) {
        self.bindings.insert(action, combo);
    }

    pub fn action_for_event(
        &self,
        key: gdk::Key,
        ctrl: bool,
        shift: bool,
        alt: bool,
    ) -> Option<ShortcutAction> {
        for (action, combo) in &self.bindings {
            if combo.matches(key, ctrl, shift, alt) {
                return Some(*action);
            }
        }
        None
    }

    pub fn load_preset(&mut self, preset: ShortcutPreset) {
        self.preset = preset;
        self.bindings.clear();

        // 1. Common core bindings across all presets
        self.set_shortcut(
            ShortcutAction::Save,
            KeyCombo::new(gdk::Key::s, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::SaveAs,
            KeyCombo::new(gdk::Key::s, true, true, false),
        );
        self.set_shortcut(
            ShortcutAction::Open,
            KeyCombo::new(gdk::Key::o, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::NewDocument,
            KeyCombo::new(gdk::Key::n, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Export,
            KeyCombo::new(gdk::Key::e, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Undo,
            KeyCombo::new(gdk::Key::z, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Redo,
            KeyCombo::new(gdk::Key::z, true, true, false),
        );
        self.set_shortcut(
            ShortcutAction::Copy,
            KeyCombo::new(gdk::Key::c, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Cut,
            KeyCombo::new(gdk::Key::x, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Paste,
            KeyCombo::new(gdk::Key::v, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Duplicate,
            KeyCombo::new(gdk::Key::d, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Delete,
            KeyCombo::new(gdk::Key::Delete, false, false, false),
        );
        self.set_shortcut(
            ShortcutAction::SelectAll,
            KeyCombo::new(gdk::Key::a, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Deselect,
            KeyCombo::new(gdk::Key::Escape, false, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Group,
            KeyCombo::new(gdk::Key::g, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::Ungroup,
            KeyCombo::new(gdk::Key::g, true, true, false),
        );
        self.set_shortcut(
            ShortcutAction::ConvertToPath,
            KeyCombo::new(gdk::Key::c, true, true, false),
        );
        self.set_shortcut(
            ShortcutAction::BringForward,
            KeyCombo::new(gdk::Key::bracketright, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::BringToFront,
            KeyCombo::new(gdk::Key::bracketright, true, true, false),
        );
        self.set_shortcut(
            ShortcutAction::SendBackward,
            KeyCombo::new(gdk::Key::bracketleft, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::SendToBack,
            KeyCombo::new(gdk::Key::bracketleft, true, true, false),
        );
        self.set_shortcut(
            ShortcutAction::ToggleLock,
            KeyCombo::new(gdk::Key::l, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::ToggleHide,
            KeyCombo::new(gdk::Key::h, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::CloneSelected,
            KeyCombo::new(gdk::Key::d, false, false, true),
        );
        self.set_shortcut(
            ShortcutAction::UnlinkClone,
            KeyCombo::new(gdk::Key::d, false, true, true),
        );
        self.set_shortcut(
            ShortcutAction::SelectOriginal,
            KeyCombo::new(gdk::Key::d, false, true, false),
        );

        // 2. View actions
        self.set_shortcut(
            ShortcutAction::ToggleRulers,
            KeyCombo::new(gdk::Key::r, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::ToggleGuides,
            KeyCombo::new(gdk::Key::r, false, true, false),
        );
        self.set_shortcut(
            ShortcutAction::ToggleGrid,
            KeyCombo::new(gdk::Key::apostrophe, true, false, false),
        );
        self.set_shortcut(
            ShortcutAction::ToggleSnap,
            KeyCombo::new(gdk::Key::apostrophe, true, true, false),
        );

        match preset {
            ShortcutPreset::Default => {
                // Tools
                self.set_shortcut(
                    ShortcutAction::ToolSelect,
                    KeyCombo::new(gdk::Key::v, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPathEditor,
                    KeyCombo::new(gdk::Key::a, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolRectangle,
                    KeyCombo::new(gdk::Key::r, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolCircle,
                    KeyCombo::new(gdk::Key::c, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolStar,
                    KeyCombo::new(gdk::Key::s, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolTriangle,
                    KeyCombo::new(gdk::Key::t, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolSpiral,
                    KeyCombo::new(gdk::Key::x, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPen,
                    KeyCombo::new(gdk::Key::p, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolBrush,
                    KeyCombo::new(gdk::Key::b, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolText,
                    KeyCombo::new(gdk::Key::t, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolImage,
                    KeyCombo::new(gdk::Key::I, false, true, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPaintBucket,
                    KeyCombo::new(gdk::Key::k, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolEyedropper,
                    KeyCombo::new(gdk::Key::i, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolGradient,
                    KeyCombo::new(gdk::Key::g, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolMeshGradient,
                    KeyCombo::new(gdk::Key::m, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolMeasure,
                    KeyCombo::new(gdk::Key::q, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolZoom,
                    KeyCombo::new(gdk::Key::z, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPage,
                    KeyCombo::new(gdk::Key::p, false, true, false),
                );

                // Zoom levels
                self.set_shortcut(
                    ShortcutAction::Zoom100,
                    KeyCombo::new(gdk::Key::_1, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomPage,
                    KeyCombo::new(gdk::Key::_2, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomSelection,
                    KeyCombo::new(gdk::Key::_3, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomFitAll,
                    KeyCombo::new(gdk::Key::_4, false, false, false),
                );
            }
            ShortcutPreset::Figma => {
                self.set_shortcut(
                    ShortcutAction::ToolSelect,
                    KeyCombo::new(gdk::Key::v, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPathEditor,
                    KeyCombo::new(gdk::Key::v, true, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolRectangle,
                    KeyCombo::new(gdk::Key::r, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolCircle,
                    KeyCombo::new(gdk::Key::o, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolStar,
                    KeyCombo::new(gdk::Key::s, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolTriangle,
                    KeyCombo::new(gdk::Key::t, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolSpiral,
                    KeyCombo::new(gdk::Key::x, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPen,
                    KeyCombo::new(gdk::Key::p, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolBrush,
                    KeyCombo::new(gdk::Key::b, false, true, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolText,
                    KeyCombo::new(gdk::Key::t, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolImage,
                    KeyCombo::new(gdk::Key::I, false, true, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPaintBucket,
                    KeyCombo::new(gdk::Key::b, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolEyedropper,
                    KeyCombo::new(gdk::Key::i, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolGradient,
                    KeyCombo::new(gdk::Key::g, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolMeshGradient,
                    KeyCombo::new(gdk::Key::m, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolMeasure,
                    KeyCombo::new(gdk::Key::m, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolZoom,
                    KeyCombo::new(gdk::Key::z, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPage,
                    KeyCombo::new(gdk::Key::f, false, false, false),
                );

                self.set_shortcut(
                    ShortcutAction::Zoom100,
                    KeyCombo::new(gdk::Key::_0, false, true, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomFitAll,
                    KeyCombo::new(gdk::Key::_1, false, true, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomSelection,
                    KeyCombo::new(gdk::Key::_2, false, true, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomPage,
                    KeyCombo::new(gdk::Key::_2, false, false, false),
                );
            }
            ShortcutPreset::Illustrator => {
                self.set_shortcut(
                    ShortcutAction::ToolSelect,
                    KeyCombo::new(gdk::Key::v, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPathEditor,
                    KeyCombo::new(gdk::Key::a, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolRectangle,
                    KeyCombo::new(gdk::Key::m, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolCircle,
                    KeyCombo::new(gdk::Key::l, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolStar,
                    KeyCombo::new(gdk::Key::s, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolTriangle,
                    KeyCombo::new(gdk::Key::t, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolSpiral,
                    KeyCombo::new(gdk::Key::x, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPen,
                    KeyCombo::new(gdk::Key::p, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolBrush,
                    KeyCombo::new(gdk::Key::b, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolText,
                    KeyCombo::new(gdk::Key::t, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolImage,
                    KeyCombo::new(gdk::Key::I, false, true, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPaintBucket,
                    KeyCombo::new(gdk::Key::k, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolEyedropper,
                    KeyCombo::new(gdk::Key::i, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolGradient,
                    KeyCombo::new(gdk::Key::g, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolMeshGradient,
                    KeyCombo::new(gdk::Key::u, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolMeasure,
                    KeyCombo::new(gdk::Key::q, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolZoom,
                    KeyCombo::new(gdk::Key::z, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPage,
                    KeyCombo::new(gdk::Key::o, false, true, false),
                );

                self.set_shortcut(
                    ShortcutAction::Zoom100,
                    KeyCombo::new(gdk::Key::_1, true, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomPage,
                    KeyCombo::new(gdk::Key::_0, true, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomSelection,
                    KeyCombo::new(gdk::Key::_3, true, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomFitAll,
                    KeyCombo::new(gdk::Key::_0, true, false, true),
                );
            }
            ShortcutPreset::Inkscape => {
                self.set_shortcut(
                    ShortcutAction::ToolSelect,
                    KeyCombo::new(gdk::Key::s, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPathEditor,
                    KeyCombo::new(gdk::Key::n, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolRectangle,
                    KeyCombo::new(gdk::Key::r, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolCircle,
                    KeyCombo::new(gdk::Key::e, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolStar,
                    KeyCombo::new(gdk::Key::asterisk, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolTriangle,
                    KeyCombo::new(gdk::Key::t, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolSpiral,
                    KeyCombo::new(gdk::Key::i, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPen,
                    KeyCombo::new(gdk::Key::b, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolBrush,
                    KeyCombo::new(gdk::Key::p, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolText,
                    KeyCombo::new(gdk::Key::t, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolImage,
                    KeyCombo::new(gdk::Key::I, false, true, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPaintBucket,
                    KeyCombo::new(gdk::Key::u, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolEyedropper,
                    KeyCombo::new(gdk::Key::d, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolGradient,
                    KeyCombo::new(gdk::Key::g, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolMeshGradient,
                    KeyCombo::new(gdk::Key::m, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolMeasure,
                    KeyCombo::new(gdk::Key::m, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolZoom,
                    KeyCombo::new(gdk::Key::z, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ToolPage,
                    KeyCombo::new(gdk::Key::p, false, true, false),
                );

                self.set_shortcut(
                    ShortcutAction::Zoom100,
                    KeyCombo::new(gdk::Key::_1, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomPage,
                    KeyCombo::new(gdk::Key::_2, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomSelection,
                    KeyCombo::new(gdk::Key::_3, false, false, false),
                );
                self.set_shortcut(
                    ShortcutAction::ZoomFitAll,
                    KeyCombo::new(gdk::Key::_4, false, false, false),
                );
            }
        }
    }

    pub fn action_descriptor(&self, action: ShortcutAction) -> (String, ShortcutCategory) {
        match action {
            ShortcutAction::ToolSelect => (
                crate::core::gettext("Select and Transform"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolPathEditor => (
                crate::core::gettext("Path Node Editing"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolRectangle => {
                (crate::core::gettext("Rectangle"), ShortcutCategory::Tools)
            }
            ShortcutAction::ToolCircle => (
                crate::core::gettext("Circle / Ellipse"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolStar => (crate::core::gettext("Star"), ShortcutCategory::Tools),
            ShortcutAction::ToolTriangle => (
                crate::core::gettext("Polygon / Triangle"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolSpiral => (crate::core::gettext("Spiral"), ShortcutCategory::Tools),
            ShortcutAction::ToolPen => (crate::core::gettext("Pen Tool"), ShortcutCategory::Tools),
            ShortcutAction::ToolBrush => {
                (crate::core::gettext("Brush Tool"), ShortcutCategory::Tools)
            }
            ShortcutAction::ToolText => (
                crate::core::gettext("Typography Text"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolImage => (
                crate::core::gettext("Image Frame Tool"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolPaintBucket => (
                crate::core::gettext("Paint Bucket"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolEyedropper => {
                (crate::core::gettext("Eyedropper"), ShortcutCategory::Tools)
            }
            ShortcutAction::ToolGradient => (
                crate::core::gettext("Linear/Radial Gradient"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolMeshGradient => (
                crate::core::gettext("Mesh Gradient"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolMeasure => (
                crate::core::gettext("Measurement Ruler"),
                ShortcutCategory::Tools,
            ),
            ShortcutAction::ToolZoom => {
                (crate::core::gettext("Zoom Tool"), ShortcutCategory::Tools)
            }
            ShortcutAction::ToolPage => (
                crate::core::gettext("Artboard / Page"),
                ShortcutCategory::Tools,
            ),

            ShortcutAction::Zoom100 => (
                crate::core::gettext("Zoom 100% (1:1)"),
                ShortcutCategory::View,
            ),
            ShortcutAction::ZoomPage => {
                (crate::core::gettext("Zoom to Page"), ShortcutCategory::View)
            }
            ShortcutAction::ZoomSelection => (
                crate::core::gettext("Zoom to Selection"),
                ShortcutCategory::View,
            ),
            ShortcutAction::ZoomFitAll => (
                crate::core::gettext("Zoom to All Objects"),
                ShortcutCategory::View,
            ),
            ShortcutAction::ToggleRulers => (
                crate::core::gettext("Show/Hide Rulers"),
                ShortcutCategory::View,
            ),
            ShortcutAction::ToggleGuides => (
                crate::core::gettext("Show/Hide Guides"),
                ShortcutCategory::View,
            ),
            ShortcutAction::ToggleGrid => (
                crate::core::gettext("Show/Hide Grid"),
                ShortcutCategory::View,
            ),
            ShortcutAction::ToggleSnap => (
                crate::core::gettext("Toggle Magnetic Snapping"),
                ShortcutCategory::View,
            ),

            ShortcutAction::Save => (
                crate::core::gettext("Save Document"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::SaveAs => (crate::core::gettext("Save As..."), ShortcutCategory::Edit),
            ShortcutAction::Open => (
                crate::core::gettext("Open Document"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::NewDocument => {
                (crate::core::gettext("New Document"), ShortcutCategory::Edit)
            }
            ShortcutAction::Export => {
                (crate::core::gettext("Quick Export"), ShortcutCategory::Edit)
            }
            ShortcutAction::Undo => (crate::core::gettext("Undo"), ShortcutCategory::Edit),
            ShortcutAction::Redo => (crate::core::gettext("Redo"), ShortcutCategory::Edit),
            ShortcutAction::Copy => (crate::core::gettext("Copy"), ShortcutCategory::Edit),
            ShortcutAction::Cut => (crate::core::gettext("Cut"), ShortcutCategory::Edit),
            ShortcutAction::Paste => (crate::core::gettext("Paste"), ShortcutCategory::Edit),
            ShortcutAction::Duplicate => {
                (crate::core::gettext("Duplicate"), ShortcutCategory::Edit)
            }
            ShortcutAction::Delete => (
                crate::core::gettext("Delete Selected"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::SelectAll => {
                (crate::core::gettext("Select All"), ShortcutCategory::Edit)
            }
            ShortcutAction::Deselect => (crate::core::gettext("Deselect"), ShortcutCategory::Edit),
            ShortcutAction::Group => (
                crate::core::gettext("Group Elements"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::Ungroup => (crate::core::gettext("Ungroup"), ShortcutCategory::Edit),
            ShortcutAction::ConvertToPath => (
                crate::core::gettext("Convert to Vector Path"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::BringForward => (
                crate::core::gettext("Bring Forward"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::BringToFront => (
                crate::core::gettext("Bring to Front"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::SendBackward => (
                crate::core::gettext("Send Backward"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::SendToBack => {
                (crate::core::gettext("Send to Back"), ShortcutCategory::Edit)
            }
            ShortcutAction::ToggleLock => (
                crate::core::gettext("Lock / Unlock Selected"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::ToggleHide => (
                crate::core::gettext("Hide / Show Selected"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::CloneSelected => (
                crate::core::gettext("Create Linked Clone"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::UnlinkClone => (
                crate::core::gettext("Unlink Clone"),
                ShortcutCategory::Edit,
            ),
            ShortcutAction::SelectOriginal => (
                crate::core::gettext("Select Original"),
                ShortcutCategory::Edit,
            ),
        }
    }

    pub fn list_by_category(
        &self,
        category: ShortcutCategory,
    ) -> Vec<(ShortcutAction, String, KeyCombo)> {
        let all_actions = [
            ShortcutAction::ToolSelect,
            ShortcutAction::ToolPathEditor,
            ShortcutAction::ToolRectangle,
            ShortcutAction::ToolCircle,
            ShortcutAction::ToolStar,
            ShortcutAction::ToolTriangle,
            ShortcutAction::ToolSpiral,
            ShortcutAction::ToolPen,
            ShortcutAction::ToolBrush,
            ShortcutAction::ToolText,
            ShortcutAction::ToolImage,
            ShortcutAction::ToolPaintBucket,
            ShortcutAction::ToolEyedropper,
            ShortcutAction::ToolGradient,
            ShortcutAction::ToolMeshGradient,
            ShortcutAction::ToolMeasure,
            ShortcutAction::ToolZoom,
            ShortcutAction::ToolPage,
            ShortcutAction::Zoom100,
            ShortcutAction::ZoomPage,
            ShortcutAction::ZoomSelection,
            ShortcutAction::ZoomFitAll,
            ShortcutAction::ToggleRulers,
            ShortcutAction::ToggleGuides,
            ShortcutAction::ToggleGrid,
            ShortcutAction::ToggleSnap,
            ShortcutAction::Save,
            ShortcutAction::SaveAs,
            ShortcutAction::Open,
            ShortcutAction::NewDocument,
            ShortcutAction::Export,
            ShortcutAction::Undo,
            ShortcutAction::Redo,
            ShortcutAction::Copy,
            ShortcutAction::Cut,
            ShortcutAction::Paste,
            ShortcutAction::Duplicate,
            ShortcutAction::Delete,
            ShortcutAction::SelectAll,
            ShortcutAction::Deselect,
            ShortcutAction::Group,
            ShortcutAction::Ungroup,
            ShortcutAction::ConvertToPath,
            ShortcutAction::BringForward,
            ShortcutAction::BringToFront,
            ShortcutAction::SendBackward,
            ShortcutAction::SendToBack,
            ShortcutAction::ToggleLock,
            ShortcutAction::ToggleHide,
            ShortcutAction::CloneSelected,
            ShortcutAction::UnlinkClone,
            ShortcutAction::SelectOriginal,
        ];

        let mut list = Vec::new();
        for action in all_actions {
            let (desc, cat) = self.action_descriptor(action);
            if cat == category {
                if let Some(combo) = self.get_shortcut(action) {
                    list.push((action, desc, combo));
                }
            }
        }
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_matching() {
        let manager = ShortcutManager::default();
        let action = manager.action_for_event(gdk::Key::v, false, false, false);
        assert_eq!(action, Some(ShortcutAction::ToolSelect));

        let undo = manager.action_for_event(gdk::Key::z, true, false, false);
        assert_eq!(undo, Some(ShortcutAction::Undo));

        let redo = manager.action_for_event(gdk::Key::z, true, true, false);
        assert_eq!(redo, Some(ShortcutAction::Redo));
    }

    #[test]
    fn test_shortcut_presets() {
        let mut manager = ShortcutManager::default();
        manager.load_preset(ShortcutPreset::Figma);
        let circle = manager.action_for_event(gdk::Key::o, false, false, false);
        assert_eq!(circle, Some(ShortcutAction::ToolCircle));

        manager.load_preset(ShortcutPreset::Illustrator);
        let rect = manager.action_for_event(gdk::Key::m, false, false, false);
        assert_eq!(rect, Some(ShortcutAction::ToolRectangle));
    }
}
