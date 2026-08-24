//! Visual Themes, Interface Scaling, Toolbar Icon Sizes and Canvas Backgrounds
//! Inspired by GNOME Text Editor, GNOME Builder, and modern GNOME ergonomics.

use gtk4::prelude::*;
use std::cell::Cell;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisualThemePreset {
    #[default]
    Adwaita,
    SolarizedLight,
    SolarizedDark,
    Cobalt,
    Oblivion,
    Nord,
    Dracula,
    Monokai,
    Darker,
    Catppuccin,
}

impl VisualThemePreset {
    pub const ALL: [VisualThemePreset; 10] = [
        VisualThemePreset::Adwaita,
        VisualThemePreset::SolarizedLight,
        VisualThemePreset::SolarizedDark,
        VisualThemePreset::Cobalt,
        VisualThemePreset::Oblivion,
        VisualThemePreset::Nord,
        VisualThemePreset::Dracula,
        VisualThemePreset::Monokai,
        VisualThemePreset::Darker,
        VisualThemePreset::Catppuccin,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Adwaita => "adwaita",
            Self::SolarizedLight => "solarized-light",
            Self::SolarizedDark => "solarized-dark",
            Self::Cobalt => "cobalt",
            Self::Oblivion => "oblivion",
            Self::Nord => "nord",
            Self::Dracula => "dracula",
            Self::Monokai => "monokai",
            Self::Darker => "darker",
            Self::Catppuccin => "catppuccin",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id.trim().to_lowercase().as_str() {
            "solarized-light" | "solarized_light" => Self::SolarizedLight,
            "solarized-dark" | "solarized_dark" => Self::SolarizedDark,
            "cobalt" => Self::Cobalt,
            "oblivion" => Self::Oblivion,
            "nord" => Self::Nord,
            "dracula" => Self::Dracula,
            "monokai" => Self::Monokai,
            "darker" => Self::Darker,
            "catppuccin" => Self::Catppuccin,
            _ => Self::Adwaita,
        }
    }

    pub fn title(self) -> String {
        match self {
            Self::Adwaita => crate::core::gettext("Adwaita (System)"),
            Self::SolarizedLight => crate::core::gettext("Solarized Light"),
            Self::SolarizedDark => crate::core::gettext("Solarized Dark"),
            Self::Cobalt => crate::core::gettext("Cobalt (Builder)"),
            Self::Oblivion => crate::core::gettext("Oblivion (Classic)"),
            Self::Nord => crate::core::gettext("Nord (Arctic)"),
            Self::Dracula => crate::core::gettext("Dracula"),
            Self::Monokai => crate::core::gettext("Monokai Pro"),
            Self::Darker => crate::core::gettext("GNOME Darker"),
            Self::Catppuccin => crate::core::gettext("Catppuccin (Mocha)"),
        }
    }

    pub fn subtitle(self) -> String {
        match self {
            Self::Adwaita => crate::core::gettext("Default GNOME HIG palette"),
            Self::SolarizedLight => crate::core::gettext("Warm parchment tones"),
            Self::SolarizedDark => crate::core::gettext("Deep solarized teal"),
            Self::Cobalt => crate::core::gettext("GNOME Builder dark navy"),
            Self::Oblivion => crate::core::gettext("GNOME classic dark slate"),
            Self::Nord => crate::core::gettext("Arctic icy blue and snow"),
            Self::Dracula => crate::core::gettext("Vibrant dark purple"),
            Self::Monokai => crate::core::gettext("High-contrast dark charcoal"),
            Self::Darker => crate::core::gettext("Deeper GNOME dark tone"),
            Self::Catppuccin => crate::core::gettext("Soothing warm pastel theme"),
        }
    }

    pub fn preview_colors(self) -> (&'static str, &'static str, &'static str) {
        // (Background, Card/View, Accent)
        match self {
            Self::Adwaita => ("#303030", "#242424", "#3584e4"),
            Self::SolarizedLight => ("#fdf6e3", "#eee8d5", "#b58900"),
            Self::SolarizedDark => ("#002b36", "#073642", "#268bd2"),
            Self::Cobalt => ("#001b33", "#002240", "#0088ff"),
            Self::Oblivion => ("#24292b", "#2e3436", "#e66100"),
            Self::Nord => ("#242933", "#2e3440", "#88c0d0"),
            Self::Dracula => ("#1e1f29", "#282a36", "#ff79c6"),
            Self::Monokai => ("#1e1e1e", "#272822", "#a6e22e"),
            Self::Darker => ("#222224", "#2a2a2e", "#3584e4"),
            Self::Catppuccin => ("#1e1e2e", "#313244", "#cba6f7"),
        }
    }

    pub fn accent_color(self) -> crate::core::Color {
        if let Some(custom) = current_custom_accent() {
            if let Some(c) = crate::core::Color::from_hex(&custom) {
                return c;
            }
        }
        match self {
            Self::Adwaita => crate::core::Color::new(0.208, 0.518, 0.894, 1.0),
            Self::SolarizedLight => crate::core::Color::from_hex("#b58900").unwrap(),
            Self::SolarizedDark => crate::core::Color::from_hex("#268bd2").unwrap(),
            Self::Cobalt => crate::core::Color::from_hex("#0088ff").unwrap(),
            Self::Oblivion => crate::core::Color::from_hex("#e66100").unwrap(),
            Self::Nord => crate::core::Color::from_hex("#88c0d0").unwrap(),
            Self::Dracula => crate::core::Color::from_hex("#ff79c6").unwrap(),
            Self::Monokai => crate::core::Color::from_hex("#a6e22e").unwrap(),
            Self::Darker => crate::core::Color::from_hex("#3584e4").unwrap(),
            Self::Catppuccin => crate::core::Color::from_hex("#cba6f7").unwrap(),
        }
    }

    pub fn workspace_bg_color(self) -> Option<crate::core::Color> {
        match self {
            Self::Adwaita => None,
            Self::SolarizedLight => Some(crate::core::Color::from_hex("#eee8d5").unwrap()),
            Self::SolarizedDark => Some(crate::core::Color::from_hex("#00212b").unwrap()),
            Self::Cobalt => Some(crate::core::Color::from_hex("#001324").unwrap()),
            Self::Oblivion => Some(crate::core::Color::from_hex("#1d2122").unwrap()),
            Self::Nord => Some(crate::core::Color::from_hex("#1d2129").unwrap()),
            Self::Dracula => Some(crate::core::Color::from_hex("#181920").unwrap()),
            Self::Monokai => Some(crate::core::Color::from_hex("#181818").unwrap()),
            Self::Darker => Some(crate::core::Color::from_hex("#1c1c1e").unwrap()),
            Self::Catppuccin => Some(crate::core::Color::from_hex("#181825").unwrap()),
        }
    }

    pub fn ruler_colors(self) -> Option<(crate::core::Color, crate::core::Color)> {
        match self {
            Self::Adwaita => None,
            Self::SolarizedLight => Some((
                crate::core::Color::from_hex("#fdf6e3").unwrap(),
                crate::core::Color::from_hex("#657b83").unwrap(),
            )),
            Self::SolarizedDark => Some((
                crate::core::Color::from_hex("#073642").unwrap(),
                crate::core::Color::from_hex("#93a1a1").unwrap(),
            )),
            Self::Cobalt => Some((
                crate::core::Color::from_hex("#001b33").unwrap(),
                crate::core::Color::from_hex("#c0d8f0").unwrap(),
            )),
            Self::Oblivion => Some((
                crate::core::Color::from_hex("#24292b").unwrap(),
                crate::core::Color::from_hex("#d3d7cf").unwrap(),
            )),
            Self::Nord => Some((
                crate::core::Color::from_hex("#242933").unwrap(),
                crate::core::Color::from_hex("#d8dee9").unwrap(),
            )),
            Self::Dracula => Some((
                crate::core::Color::from_hex("#21222c").unwrap(),
                crate::core::Color::from_hex("#f8f8f2").unwrap(),
            )),
            Self::Monokai => Some((
                crate::core::Color::from_hex("#22231e").unwrap(),
                crate::core::Color::from_hex("#f8f8f2").unwrap(),
            )),
            Self::Darker => Some((
                crate::core::Color::from_hex("#202022").unwrap(),
                crate::core::Color::from_hex("#d0d0d4").unwrap(),
            )),
            Self::Catppuccin => Some((
                crate::core::Color::from_hex("#1e1e2e").unwrap(),
                crate::core::Color::from_hex("#cdd6f4").unwrap(),
            )),
        }
    }
}

struct ThemeTokens {
    window_bg: &'static str,
    window_fg: &'static str,
    view_bg: &'static str,
    view_fg: &'static str,
    headerbar_bg: &'static str,
    headerbar_fg: &'static str,
    card_bg: &'static str,
    card_fg: &'static str,
    popover_bg: &'static str,
    popover_fg: &'static str,
    dialog_bg: &'static str,
    dialog_fg: &'static str,
    sidebar_bg: &'static str,
    sidebar_fg: &'static str,
    accent: &'static str,
    accent_fg: &'static str,
}

fn generate_theme_css(t: &ThemeTokens) -> String {
    format!(
        "@define-color window_bg_color {w_bg}; \
         @define-color window_backdrop_color {w_bg}; \
         @define-color window_fg_color {w_fg}; \
         @define-color window_fg_backdrop_color {w_fg}; \
         @define-color view_bg_color {v_bg}; \
         @define-color view_backdrop_color {v_bg}; \
         @define-color view_fg_color {v_fg}; \
         @define-color view_fg_backdrop_color {v_fg}; \
         @define-color headerbar_bg_color {h_bg}; \
         @define-color headerbar_backdrop_color {h_bg}; \
         @define-color headerbar_fg_color {h_fg}; \
         @define-color headerbar_fg_backdrop_color {h_fg}; \
         @define-color card_bg_color {c_bg}; \
         @define-color card_backdrop_color {c_bg}; \
         @define-color card_fg_color {c_fg}; \
         @define-color card_fg_backdrop_color {c_fg}; \
         @define-color popover_bg_color {p_bg}; \
         @define-color popover_backdrop_color {p_bg}; \
         @define-color popover_fg_color {p_fg}; \
         @define-color dialog_bg_color {d_bg}; \
         @define-color dialog_backdrop_color {d_bg}; \
         @define-color dialog_fg_color {d_fg}; \
         @define-color sidebar_bg_color {s_bg}; \
         @define-color sidebar_backdrop_color {s_bg}; \
         @define-color sidebar_fg_color {s_fg}; \
         @define-color sidebar_fg_backdrop_color {s_fg}; \
         @define-color secondary_sidebar_bg_color {s_bg}; \
         @define-color secondary_sidebar_backdrop_color {s_bg}; \
         @define-color secondary_sidebar_fg_color {s_fg}; \
         @define-color accent_color {acc}; \
         @define-color accent_bg_color {acc}; \
         @define-color accent_fg_color {acc_fg}; \
         \
         window, window:backdrop, .background, .background:backdrop {{ background-color: @window_bg_color; color: @window_fg_color; }} \
         headerbar, headerbar:backdrop {{ background-color: @headerbar_bg_color; color: @headerbar_fg_color; }} \
         \
         split-view > sidebar, split-view > sidebar:backdrop, navigation-split-view > sidebar, navigation-split-view > sidebar:backdrop, .sidebar, .sidebar:backdrop {{ background-color: @sidebar_bg_color; color: @sidebar_fg_color; }} \
         split-view > sidebar headerbar, split-view > sidebar headerbar:backdrop, navigation-split-view > sidebar headerbar, navigation-split-view > sidebar headerbar:backdrop, .sidebar headerbar, .sidebar headerbar:backdrop, headerbar.flat, headerbar.flat:backdrop {{ background-color: @sidebar_bg_color; color: @sidebar_fg_color; border-bottom: none; box-shadow: none; }} \
         preferences-window > split-view > sidebar, preferences-window > split-view > sidebar:backdrop, preferences-window split-view > sidebar, preferences-window split-view > sidebar:backdrop {{ background-color: @sidebar_bg_color; color: @sidebar_fg_color; }} \
         preferences-window > split-view > sidebar headerbar, preferences-window > split-view > sidebar headerbar:backdrop, preferences-window split-view > sidebar headerbar, preferences-window split-view > sidebar headerbar:backdrop {{ background-color: @sidebar_bg_color; color: @sidebar_fg_color; border-bottom: none; box-shadow: none; }} \
         \
         .boxed-list, .boxed-list:backdrop, card, card:backdrop, preferencesgroup > listbox, preferencesgroup > listbox:backdrop {{ background-color: @card_bg_color; color: @card_fg_color; }} \
         .floating-panel, .color-palette-capsule, .toolbar-floating, .tool-capsule, .inspector-panel {{ background-color: @card_bg_color; color: @card_fg_color; border-color: alpha(currentColor, 0.12); }} \
         button.suggested-action, .suggested-action, .pill.suggested-action {{ background-color: @accent_bg_color; color: @accent_fg_color; }} \
         button.suggested-action:hover, .suggested-action:hover {{ background-color: alpha(@accent_bg_color, 0.88); }} \
         .linked button.active, .toolbar button.active, button.toggle.active {{ background-color: alpha(@accent_color, 0.20); color: @accent_color; }} \
         scale highlight {{ background-color: @accent_bg_color; }} \
         tab.selected, .tab-bar tab.active {{ background-color: alpha(@accent_color, 0.16); border-bottom: 2px solid @accent_color; color: @accent_color; }}",
        w_bg = t.window_bg,
        w_fg = t.window_fg,
        v_bg = t.view_bg,
        v_fg = t.view_fg,
        h_bg = t.headerbar_bg,
        h_fg = t.headerbar_fg,
        c_bg = t.card_bg,
        c_fg = t.card_fg,
        p_bg = t.popover_bg,
        p_fg = t.popover_fg,
        d_bg = t.dialog_bg,
        d_fg = t.dialog_fg,
        s_bg = t.sidebar_bg,
        s_fg = t.sidebar_fg,
        acc = t.accent,
        acc_fg = t.accent_fg,
    )
}

impl VisualThemePreset {
    pub fn css_definitions(self) -> String {
        match self {
            Self::Adwaita => String::new(),
            Self::SolarizedLight => generate_theme_css(&ThemeTokens {
                window_bg: "#fdf6e3",
                window_fg: "#586e75",
                view_bg: "#eee8d5",
                view_fg: "#657b83",
                headerbar_bg: "#eee8d5",
                headerbar_fg: "#586e75",
                card_bg: "#f5eed9",
                card_fg: "#586e75",
                popover_bg: "#fdf6e3",
                popover_fg: "#586e75",
                dialog_bg: "#fdf6e3",
                dialog_fg: "#586e75",
                sidebar_bg: "#eee8d5",
                sidebar_fg: "#586e75",
                accent: "#b58900",
                accent_fg: "#fdf6e3",
            }),
            Self::SolarizedDark => generate_theme_css(&ThemeTokens {
                window_bg: "#002b36",
                window_fg: "#93a1a1",
                view_bg: "#073642",
                view_fg: "#839496",
                headerbar_bg: "#002129",
                headerbar_fg: "#93a1a1",
                card_bg: "#073642",
                card_fg: "#93a1a1",
                popover_bg: "#002b36",
                popover_fg: "#93a1a1",
                dialog_bg: "#002b36",
                dialog_fg: "#93a1a1",
                sidebar_bg: "#002129",
                sidebar_fg: "#93a1a1",
                accent: "#268bd2",
                accent_fg: "#ffffff",
            }),
            Self::Cobalt => generate_theme_css(&ThemeTokens {
                window_bg: "#00172c",
                window_fg: "#e0e8f0",
                view_bg: "#002240",
                view_fg: "#ffffff",
                headerbar_bg: "#001324",
                headerbar_fg: "#ffffff",
                card_bg: "#002240",
                card_fg: "#ffffff",
                popover_bg: "#001f3b",
                popover_fg: "#ffffff",
                dialog_bg: "#00172c",
                dialog_fg: "#ffffff",
                sidebar_bg: "#001324",
                sidebar_fg: "#ffffff",
                accent: "#0088ff",
                accent_fg: "#ffffff",
            }),
            Self::Oblivion => generate_theme_css(&ThemeTokens {
                window_bg: "#24292b",
                window_fg: "#d3d7cf",
                view_bg: "#2e3436",
                view_fg: "#eeeeec",
                headerbar_bg: "#1d2122",
                headerbar_fg: "#eeeeec",
                card_bg: "#2e3436",
                card_fg: "#eeeeec",
                popover_bg: "#272c2e",
                popover_fg: "#eeeeec",
                dialog_bg: "#24292b",
                dialog_fg: "#eeeeec",
                sidebar_bg: "#1d2122",
                sidebar_fg: "#eeeeec",
                accent: "#e66100",
                accent_fg: "#ffffff",
            }),
            Self::Nord => generate_theme_css(&ThemeTokens {
                window_bg: "#242933",
                window_fg: "#d8dee9",
                view_bg: "#2e3440",
                view_fg: "#eceff4",
                headerbar_bg: "#1d2129",
                headerbar_fg: "#eceff4",
                card_bg: "#2e3440",
                card_fg: "#eceff4",
                popover_bg: "#2e3440",
                popover_fg: "#eceff4",
                dialog_bg: "#242933",
                dialog_fg: "#eceff4",
                sidebar_bg: "#1d2129",
                sidebar_fg: "#eceff4",
                accent: "#88c0d0",
                accent_fg: "#2e3440",
            }),
            Self::Dracula => generate_theme_css(&ThemeTokens {
                window_bg: "#1e1f29",
                window_fg: "#f8f8f2",
                view_bg: "#282a36",
                view_fg: "#f8f8f2",
                headerbar_bg: "#181920",
                headerbar_fg: "#f8f8f2",
                card_bg: "#282a36",
                card_fg: "#f8f8f2",
                popover_bg: "#21222c",
                popover_fg: "#f8f8f2",
                dialog_bg: "#1e1f29",
                dialog_fg: "#f8f8f2",
                sidebar_bg: "#181920",
                sidebar_fg: "#f8f8f2",
                accent: "#ff79c6",
                accent_fg: "#282a36",
            }),
            Self::Monokai => generate_theme_css(&ThemeTokens {
                window_bg: "#1e1e1e",
                window_fg: "#fcfcfa",
                view_bg: "#272822",
                view_fg: "#f8f8f2",
                headerbar_bg: "#181818",
                headerbar_fg: "#f8f8f2",
                card_bg: "#272822",
                card_fg: "#f8f8f2",
                popover_bg: "#22231e",
                popover_fg: "#f8f8f2",
                dialog_bg: "#1e1e1e",
                dialog_fg: "#f8f8f2",
                sidebar_bg: "#181818",
                sidebar_fg: "#f8f8f2",
                accent: "#a6e22e",
                accent_fg: "#272822",
            }),
            Self::Darker => generate_theme_css(&ThemeTokens {
                window_bg: "#222224",
                window_fg: "#eeeeec",
                view_bg: "#1c1c1e",
                view_fg: "#ffffff",
                headerbar_bg: "#19191b",
                headerbar_fg: "#ffffff",
                card_bg: "#2a2a2e",
                card_fg: "#ffffff",
                popover_bg: "#26262a",
                popover_fg: "#ffffff",
                dialog_bg: "#222224",
                dialog_fg: "#ffffff",
                sidebar_bg: "#1c1c1e",
                sidebar_fg: "#ffffff",
                accent: "#3584e4",
                accent_fg: "#ffffff",
            }),
            Self::Catppuccin => generate_theme_css(&ThemeTokens {
                window_bg: "#1e1e2e",
                window_fg: "#cdd6f4",
                view_bg: "#181825",
                view_fg: "#cdd6f4",
                headerbar_bg: "#11111b",
                headerbar_fg: "#cdd6f4",
                card_bg: "#313244",
                card_fg: "#cdd6f4",
                popover_bg: "#1e1e2e",
                popover_fg: "#cdd6f4",
                dialog_bg: "#1e1e2e",
                dialog_fg: "#cdd6f4",
                sidebar_bg: "#181825",
                sidebar_fg: "#cdd6f4",
                accent: "#cba6f7",
                accent_fg: "#1e1e2e",
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolbarIconSize {
    Small, // 16px
    #[default]
    Medium, // 20px
    Large, // 24px
}

impl ToolbarIconSize {
    pub fn id(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id.trim().to_lowercase().as_str() {
            "small" => Self::Small,
            "large" => Self::Large,
            _ => Self::Medium,
        }
    }

    pub fn to_index(self) -> u32 {
        match self {
            Self::Small => 0,
            Self::Medium => 1,
            Self::Large => 2,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => Self::Small,
            1 => Self::Medium,
            _ => Self::Large,
        }
    }

    #[allow(dead_code)]
    pub fn label(self) -> String {
        match self {
            Self::Small => crate::core::gettext("Small (16px)"),
            Self::Medium => crate::core::gettext("Medium (20px)"),
            Self::Large => crate::core::gettext("Large (24px)"),
        }
    }

    pub fn css(self) -> &'static str {
        match self {
            Self::Small => {
                ".toolbar button image, .toolbar-btn image, .toolbar-button image { min-width: 16px; min-height: 16px; -gtk-icon-size: 16px; padding: 0; }"
            }
            Self::Medium => {
                ".toolbar button image, .toolbar-btn image, .toolbar-button image { min-width: 20px; min-height: 20px; -gtk-icon-size: 20px; }"
            }
            Self::Large => {
                ".toolbar button image, .toolbar-btn image, .toolbar-button image { min-width: 24px; min-height: 24px; -gtk-icon-size: 24px; padding: 2px; }"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterfaceScale {
    Compact, // 90%
    #[default]
    Default, // 100%
    Comfortable, // 110%
    Large,   // 125%
}

impl InterfaceScale {
    pub fn id(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Default => "default",
            Self::Comfortable => "comfortable",
            Self::Large => "large",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id.trim().to_lowercase().as_str() {
            "compact" => Self::Compact,
            "comfortable" => Self::Comfortable,
            "large" => Self::Large,
            _ => Self::Default,
        }
    }

    pub fn to_index(self) -> u32 {
        match self {
            Self::Compact => 0,
            Self::Default => 1,
            Self::Comfortable => 2,
            Self::Large => 3,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => Self::Compact,
            1 => Self::Default,
            2 => Self::Comfortable,
            _ => Self::Large,
        }
    }

    #[allow(dead_code)]
    pub fn label(self) -> String {
        match self {
            Self::Compact => crate::core::gettext("Compact (90%)"),
            Self::Default => crate::core::gettext("Default (100%)"),
            Self::Comfortable => crate::core::gettext("Comfortable (110%)"),
            Self::Large => crate::core::gettext("Large (125%)"),
        }
    }

    pub fn css(self) -> &'static str {
        match self {
            Self::Compact => {
                "window { font-size: 12px; } .toolbar { padding: 3px 6px; } entry.numeric { min-height: 28px; } button { min-height: 28px; }"
            }
            Self::Default => "",
            Self::Comfortable => {
                "window { font-size: 14.5px; } .toolbar { padding: 6px 10px; } entry.numeric { min-height: 36px; } button { min-height: 36px; }"
            }
            Self::Large => {
                "window { font-size: 16px; } .toolbar { padding: 8px 14px; } entry.numeric { min-height: 40px; } button { min-height: 40px; }"
            }
        }
    }
}

thread_local! {
    static CURRENT_THEME: Cell<VisualThemePreset> = const { Cell::new(VisualThemePreset::Adwaita) };
    static THEME_CSS_PROVIDER: RefCell<Option<gtk4::CssProvider>> = const { RefCell::new(None) };

    static CURRENT_CUSTOM_ACCENT: RefCell<Option<String>> = const { RefCell::new(None) };
    static ACCENT_CSS_PROVIDER: RefCell<Option<gtk4::CssProvider>> = const { RefCell::new(None) };

    static CURRENT_ICON_SIZE: Cell<ToolbarIconSize> = const { Cell::new(ToolbarIconSize::Medium) };
    static ICON_SIZE_CSS_PROVIDER: RefCell<Option<gtk4::CssProvider>> = const { RefCell::new(None) };

    static CURRENT_SCALE: Cell<InterfaceScale> = const { Cell::new(InterfaceScale::Default) };
    static SCALE_CSS_PROVIDER: RefCell<Option<gtk4::CssProvider>> = const { RefCell::new(None) };
}

pub fn current_custom_accent() -> Option<String> {
    CURRENT_CUSTOM_ACCENT.with(|a| a.borrow().clone())
}

pub fn set_custom_accent(accent: Option<String>) {
    CURRENT_CUSTOM_ACCENT.with(|a| {
        *a.borrow_mut() = accent.clone();
    });

    ACCENT_CSS_PROVIDER.with(|p| {
        let mut p_borrow = p.borrow_mut();
        if p_borrow.is_none() {
            let provider = gtk4::CssProvider::new();
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_USER + 10,
                );
            }
            *p_borrow = Some(provider);
        }

        if let Some(ref provider) = *p_borrow {
            if let Some(ref hex) = accent {
                let css = format!(
                    "@define-color accent_color {hex}; \
                     @define-color accent_bg_color {hex}; \
                     @define-color accent_fg_color #ffffff;"
                );
                provider.load_from_string(&css);
            } else {
                provider.load_from_string("");
            }
        }
    });

    if let Some(app) = gtk4::gio::Application::default() {
        if let Some(gtk_app) = app.downcast_ref::<gtk4::Application>() {
            for win in gtk_app.windows() {
                win.queue_draw();
            }
        }
    }
}

pub fn current_visual_theme() -> VisualThemePreset {
    CURRENT_THEME.with(|t| t.get())
}

pub fn set_visual_theme(theme: VisualThemePreset) {
    CURRENT_THEME.with(|t| t.set(theme));

    let style_mgr = libadwaita::StyleManager::default();
    match theme {
        VisualThemePreset::Adwaita => {
            style_mgr.set_color_scheme(libadwaita::ColorScheme::Default);
        }
        VisualThemePreset::SolarizedLight => {
            style_mgr.set_color_scheme(libadwaita::ColorScheme::ForceLight);
        }
        _ => {
            style_mgr.set_color_scheme(libadwaita::ColorScheme::ForceDark);
        }
    }

    THEME_CSS_PROVIDER.with(|p| {
        let mut p_borrow = p.borrow_mut();
        if p_borrow.is_none() {
            let provider = gtk4::CssProvider::new();
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_USER,
                );
            }
            *p_borrow = Some(provider);
        }

        if let Some(ref provider) = *p_borrow {
            provider.load_from_string(&theme.css_definitions());
        }
    });

    if let Some(app) = gtk4::gio::Application::default() {
        if let Some(gtk_app) = app.downcast_ref::<gtk4::Application>() {
            for win in gtk_app.windows() {
                win.queue_draw();
            }
        }
    }
}

pub fn current_toolbar_icon_size() -> ToolbarIconSize {
    CURRENT_ICON_SIZE.with(|s| s.get())
}

pub fn set_toolbar_icon_size(size: ToolbarIconSize) {
    CURRENT_ICON_SIZE.with(|s| s.set(size));

    ICON_SIZE_CSS_PROVIDER.with(|p| {
        let mut p_borrow = p.borrow_mut();
        if p_borrow.is_none() {
            let provider = gtk4::CssProvider::new();
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 22,
                );
            }
            *p_borrow = Some(provider);
        }

        if let Some(ref provider) = *p_borrow {
            provider.load_from_string(size.css());
        }
    });
}

pub fn current_interface_scale() -> InterfaceScale {
    CURRENT_SCALE.with(|s| s.get())
}

pub fn set_interface_scale(scale: InterfaceScale) {
    CURRENT_SCALE.with(|s| s.set(scale));

    SCALE_CSS_PROVIDER.with(|p| {
        let mut p_borrow = p.borrow_mut();
        if p_borrow.is_none() {
            let provider = gtk4::CssProvider::new();
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 23,
                );
            }
            *p_borrow = Some(provider);
        }

        if let Some(ref provider) = *p_borrow {
            provider.load_from_string(scale.css());
        }
    });
}
