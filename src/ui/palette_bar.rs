use gtk4::gdk;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::canvas::CanvasWidget;
use super::color_picker::ColorPickerPopover;
use crate::core::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PalettePreset {
    #[default]
    GnomeAdwaita,
    DocumentColors,
    Custom,
    PastelSoft,
    VibrantNeon,
    Monochrome,
    Material500,
}

impl PalettePreset {
    pub fn to_index(self) -> u32 {
        match self {
            Self::GnomeAdwaita => 0,
            Self::DocumentColors => 1,
            Self::Custom => 2,
            Self::PastelSoft => 3,
            Self::VibrantNeon => 4,
            Self::Monochrome => 5,
            Self::Material500 => 6,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            0 => Self::GnomeAdwaita,
            1 => Self::DocumentColors,
            2 => Self::Custom,
            3 => Self::PastelSoft,
            4 => Self::VibrantNeon,
            5 => Self::Monochrome,
            _ => Self::Material500,
        }
    }

    #[allow(dead_code)]
    pub fn label(self) -> String {
        match self {
            Self::GnomeAdwaita => crate::core::gettext("GNOME Adwaita"),
            Self::DocumentColors => crate::core::gettext("Document Colors"),
            Self::Custom => crate::core::gettext("Custom Palette"),
            Self::PastelSoft => crate::core::gettext("Pastel Soft"),
            Self::VibrantNeon => crate::core::gettext("Vibrant Neon"),
            Self::Monochrome => crate::core::gettext("Monochrome"),
            Self::Material500 => crate::core::gettext("Material 500"),
        }
    }
}

fn get_preset_colors(preset: PalettePreset, canvas: &CanvasWidget) -> Vec<Option<Color>> {
    match preset {
        PalettePreset::GnomeAdwaita => vec![
            None,
            Some(Color::new(0.0, 0.0, 0.0, 1.0)),       // Black
            Some(Color::new(1.0, 1.0, 1.0, 1.0)),       // White
            Some(Color::new(0.208, 0.518, 0.894, 1.0)), // GNOME Blue #3584e4
            Some(Color::new(0.129, 0.565, 0.643, 1.0)), // Cyan #2190a4
            Some(Color::new(0.149, 0.635, 0.412, 1.0)), // Green #26a269
            Some(Color::new(0.898, 0.647, 0.039, 1.0)), // Yellow #e5a50a
            Some(Color::new(0.902, 0.380, 0.0, 1.0)),   // Orange #e66100
            Some(Color::new(0.753, 0.110, 0.157, 1.0)), // Red #c01c28
            Some(Color::new(0.569, 0.255, 0.675, 1.0)), // Purple #9141ac
            Some(Color::new(0.835, 0.380, 0.600, 1.0)), // Pink #d56199
            Some(Color::new(0.141, 0.122, 0.192, 1.0)), // Dark Slate #241f31
            Some(Color::new(0.369, 0.361, 0.392, 1.0)), // Warm Gray #5e5c64
            Some(Color::new(0.871, 0.867, 0.855, 1.0)), // Light Gray #deddda
        ],
        PalettePreset::DocumentColors => get_document_colors(canvas),
        PalettePreset::Custom => vec![],
        PalettePreset::PastelSoft => vec![
            None,
            Some(Color::from_hex("#ffb3ba").unwrap_or(Color::BLACK)), // Coral
            Some(Color::from_hex("#ffdfba").unwrap_or(Color::BLACK)), // Peach
            Some(Color::from_hex("#ffffba").unwrap_or(Color::BLACK)), // Butter
            Some(Color::from_hex("#baffc9").unwrap_or(Color::BLACK)), // Mint
            Some(Color::from_hex("#bae1ff").unwrap_or(Color::BLACK)), // Sky
            Some(Color::from_hex("#d7baff").unwrap_or(Color::BLACK)), // Lavender
            Some(Color::from_hex("#ffbae1").unwrap_or(Color::BLACK)), // Rose
            Some(Color::from_hex("#ffd1dc").unwrap_or(Color::BLACK)), // Blush
            Some(Color::from_hex("#c1e1c1").unwrap_or(Color::BLACK)), // Sage
            Some(Color::from_hex("#b2bec3").unwrap_or(Color::BLACK)), // Slate
            Some(Color::from_hex("#2d3436").unwrap_or(Color::BLACK)), // Charcoal
        ],
        PalettePreset::VibrantNeon => vec![
            None,
            Some(Color::from_hex("#00f0ff").unwrap_or(Color::BLACK)), // Electric Cyan
            Some(Color::from_hex("#39ff14").unwrap_or(Color::BLACK)), // Laser Green
            Some(Color::from_hex("#fff01f").unwrap_or(Color::BLACK)), // Neon Yellow
            Some(Color::from_hex("#ff6700").unwrap_or(Color::BLACK)), // Sunburst Orange
            Some(Color::from_hex("#ff007f").unwrap_or(Color::BLACK)), // Hot Pink
            Some(Color::from_hex("#9900ff").unwrap_or(Color::BLACK)), // Electric Purple
            Some(Color::from_hex("#0022ff").unwrap_or(Color::BLACK)), // Ultramarine
            Some(Color::from_hex("#000000").unwrap_or(Color::BLACK)), // Black
            Some(Color::from_hex("#ffffff").unwrap_or(Color::BLACK)), // White
        ],
        PalettePreset::Monochrome => vec![
            None,
            Some(Color::from_hex("#000000").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#1a1a1a").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#333333").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#4d4d4d").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#666666").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#808080").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#999999").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#b3b3b3").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#cccccc").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#e6e6e6").unwrap_or(Color::BLACK)),
            Some(Color::from_hex("#ffffff").unwrap_or(Color::BLACK)),
        ],
        PalettePreset::Material500 => vec![
            None,
            Some(Color::from_hex("#f44336").unwrap_or(Color::BLACK)), // Red
            Some(Color::from_hex("#e91e63").unwrap_or(Color::BLACK)), // Pink
            Some(Color::from_hex("#9c27b0").unwrap_or(Color::BLACK)), // Purple
            Some(Color::from_hex("#673ab7").unwrap_or(Color::BLACK)), // Deep Purple
            Some(Color::from_hex("#3f51b5").unwrap_or(Color::BLACK)), // Indigo
            Some(Color::from_hex("#2196f3").unwrap_or(Color::BLACK)), // Blue
            Some(Color::from_hex("#00bcd4").unwrap_or(Color::BLACK)), // Cyan
            Some(Color::from_hex("#009688").unwrap_or(Color::BLACK)), // Teal
            Some(Color::from_hex("#4caf50").unwrap_or(Color::BLACK)), // Green
            Some(Color::from_hex("#ffeb3b").unwrap_or(Color::BLACK)), // Yellow
            Some(Color::from_hex("#ff9800").unwrap_or(Color::BLACK)), // Orange
            Some(Color::from_hex("#795548").unwrap_or(Color::BLACK)), // Brown
            Some(Color::from_hex("#607d8b").unwrap_or(Color::BLACK)), // Blue Grey
        ],
    }
}

fn get_document_colors(canvas: &CanvasWidget) -> Vec<Option<Color>> {
    canvas.get_document_colors()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteTargetMode {
    Fill,
    Stroke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteBarPosition {
    Left,
    Right,
    Bottom,
    Top,
}

#[derive(Clone)]
pub struct ColorPaletteBar {
    container: gtk4::Box,
    swatches_box: gtk4::Box,
    scrolled_win: gtk4::ScrolledWindow,
    swatches: Rc<RefCell<Vec<Option<Color>>>>,
    custom_colors: Rc<RefCell<Vec<Option<Color>>>>,
    active_color: Rc<Cell<Option<Color>>>,
    target_mode: Rc<Cell<PaletteTargetMode>>,
    current_preset: Rc<Cell<PalettePreset>>,
    position: Rc<Cell<PaletteBarPosition>>,
    on_reposition: Rc<RefCell<Option<Box<dyn Fn(PaletteBarPosition)>>>>,
    canvas: CanvasWidget,
}

impl ColorPaletteBar {
    pub fn new(canvas: CanvasWidget) -> Self {
        let initial_preset = PalettePreset::GnomeAdwaita;
        let initial_colors = get_preset_colors(initial_preset, &canvas);
        let custom_colors = Rc::new(RefCell::new(Vec::new()));

        let swatches = Rc::new(RefCell::new(initial_colors));
        let active_color = Rc::new(Cell::new(Some(Color::new(0.208, 0.518, 0.894, 1.0))));
        let target_mode = Rc::new(Cell::new(PaletteTargetMode::Fill));
        let current_preset = Rc::new(Cell::new(initial_preset));
        let position = Rc::new(Cell::new(PaletteBarPosition::Left));
        let on_reposition: Rc<RefCell<Option<Box<dyn Fn(PaletteBarPosition)>>>> =
            Rc::new(RefCell::new(None));

        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .css_classes(["toolbar", "card", "color-palette-capsule"])
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Center)
            .margin_start(56)
            .build();

        let swatches_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .build();

        let scrolled_win = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .overlay_scrolling(true)
            .propagate_natural_height(true)
            .propagate_natural_width(true)
            .max_content_height(440)
            .has_frame(false)
            .css_classes(["color-palette-scrolled"])
            .child(&swatches_box)
            .build();

        let bar = Self {
            container: container.clone(),
            swatches_box: swatches_box.clone(),
            scrolled_win: scrolled_win.clone(),
            swatches,
            custom_colors,
            active_color,
            target_mode,
            current_preset,
            position,
            on_reposition,
            canvas: canvas.clone(),
        };

        bar.rebuild_swatches();

        // Options / More Button (...)
        let more_btn = gtk4::Button::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text(crate::core::gettext("Palette Options"))
            .css_classes(["flat", "color-more-btn"])
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .build();

        let popover = gtk4::Popover::builder()
            .has_arrow(true)
            .css_classes(["app-menu-popover"])
            .build();

        let menu_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .width_request(220)
            .build();

        // 1. Preset Selector DropDown
        let preset_lbl = gtk4::Label::builder()
            .label(crate::core::gettext("Color Presets"))
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label", "caption"])
            .build();
        menu_box.append(&preset_lbl);

        let preset_names = [
            crate::core::gettext("GNOME Adwaita"),
            crate::core::gettext("Document Colors"),
            crate::core::gettext("Custom Palette"),
            crate::core::gettext("Pastel Soft"),
            crate::core::gettext("Vibrant Neon"),
            crate::core::gettext("Monochrome"),
            crate::core::gettext("Material 500"),
        ];
        let preset_str_list: Vec<&str> = preset_names.iter().map(|s| s.as_str()).collect();
        let preset_dd = gtk4::DropDown::from_strings(&preset_str_list);
        preset_dd.set_selected(initial_preset.to_index());

        {
            let b_clone = bar.clone();
            let c_c = canvas.clone();
            preset_dd.connect_selected_notify(move |dd| {
                let idx = dd.selected();
                let preset = PalettePreset::from_index(idx);
                b_clone.current_preset.set(preset);
                if preset == PalettePreset::Custom {
                    *b_clone.swatches.borrow_mut() = b_clone.custom_colors.borrow().clone();
                } else {
                    *b_clone.swatches.borrow_mut() = get_preset_colors(preset, &c_c);
                }
                b_clone.rebuild_swatches();
            });
        }
        menu_box.append(&preset_dd);

        let sep0 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        menu_box.append(&sep0);

        // 2. Target Mode Segmented Switch: Fill / Stroke
        let mode_lbl = gtk4::Label::builder()
            .label(crate::core::gettext("Target Mode"))
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label", "caption"])
            .build();
        menu_box.append(&mode_lbl);

        let seg_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(2)
            .css_classes(["linked"])
            .build();

        let btn_fill_mode = gtk4::ToggleButton::builder()
            .label(crate::core::gettext("Fill"))
            .hexpand(true)
            .active(true)
            .build();
        let btn_stroke_mode = gtk4::ToggleButton::builder()
            .label(crate::core::gettext("Stroke"))
            .hexpand(true)
            .group(&btn_fill_mode)
            .build();

        {
            let tmode = bar.target_mode.clone();
            let b_clone = bar.clone();
            btn_fill_mode.connect_toggled(move |btn| {
                if btn.is_active() {
                    tmode.set(PaletteTargetMode::Fill);
                    b_clone.rebuild_swatches();
                }
            });
        }
        {
            let tmode = bar.target_mode.clone();
            let b_clone = bar.clone();
            btn_stroke_mode.connect_toggled(move |btn| {
                if btn.is_active() {
                    tmode.set(PaletteTargetMode::Stroke);
                    b_clone.rebuild_swatches();
                }
            });
        }
        seg_box.append(&btn_fill_mode);
        seg_box.append(&btn_stroke_mode);
        menu_box.append(&seg_box);

        let sep1 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        menu_box.append(&sep1);

        // 3. Custom Color Picker Action
        let picker_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Start)
            .build();
        let picker_icon = gtk4::Image::from_icon_name("color-select-symbolic");
        let picker_lbl = gtk4::Label::new(Some(&crate::core::gettext("Color Picker...")));
        picker_box.append(&picker_icon);
        picker_box.append(&picker_lbl);

        let picker_btn = gtk4::Button::builder()
            .child(&picker_box)
            .css_classes(["flat"])
            .halign(gtk4::Align::Fill)
            .build();

        let color_popover = ColorPickerPopover::new(
            canvas.clone(),
            bar.active_color.get().unwrap_or(Color::BLACK),
            0,
        );
        color_popover.attach_to(&picker_btn);

        {
            let c_pop = color_popover.clone();
            let pop_close = popover.clone();
            picker_btn.connect_clicked(move |_| {
                pop_close.popdown();
                c_pop.popup();
            });
        }

        {
            let b_clone = bar.clone();
            let c_c = canvas.clone();
            color_popover.on_color_changed(move |col| {
                b_clone.apply_color(Some(col));
                c_c.widget().queue_draw();
            });
        }
        menu_box.append(&picker_btn);

        // 4. Add active color to palette
        let add_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Start)
            .build();
        let add_icon = gtk4::Image::from_icon_name("list-add-symbolic");
        let add_lbl = gtk4::Label::new(Some(&crate::core::gettext("Add Active Color")));
        add_box.append(&add_icon);
        add_box.append(&add_lbl);

        let add_btn = gtk4::Button::builder()
            .child(&add_box)
            .css_classes(["flat"])
            .halign(gtk4::Align::Fill)
            .build();

        {
            let b_clone = bar.clone();
            let dd_c = preset_dd.clone();
            let pop_close = popover.clone();
            add_btn.connect_clicked(move |_| {
                if let Some(col) = b_clone.active_color.get() {
                    let mut list = b_clone.custom_colors.borrow_mut();
                    if !list.contains(&Some(col)) {
                        list.push(Some(col));
                    }
                    drop(list);
                    b_clone.current_preset.set(PalettePreset::Custom);
                    *b_clone.swatches.borrow_mut() = b_clone.custom_colors.borrow().clone();
                    dd_c.set_selected(PalettePreset::Custom.to_index());
                    b_clone.rebuild_swatches();
                }
                pop_close.popdown();
            });
        }
        menu_box.append(&add_btn);

        // 5. Clear Custom Colors
        let clear_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Start)
            .build();
        let clear_icon = gtk4::Image::from_icon_name("edit-clear-symbolic");
        let clear_lbl = gtk4::Label::new(Some(&crate::core::gettext("Reset / Clear Palette")));
        clear_box.append(&clear_icon);
        clear_box.append(&clear_lbl);

        let clear_btn = gtk4::Button::builder()
            .child(&clear_box)
            .css_classes(["flat"])
            .halign(gtk4::Align::Fill)
            .build();

        {
            let b_clone = bar.clone();
            let c_c = canvas.clone();
            let pop_close = popover.clone();
            clear_btn.connect_clicked(move |_| {
                let preset = b_clone.current_preset.get();
                if preset == PalettePreset::Custom {
                    b_clone.custom_colors.borrow_mut().clear();
                    *b_clone.swatches.borrow_mut() = Vec::new();
                } else {
                    *b_clone.swatches.borrow_mut() = get_preset_colors(preset, &c_c);
                }
                b_clone.rebuild_swatches();
                pop_close.popdown();
            });
        }
        menu_box.append(&clear_btn);

        let sep2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        menu_box.append(&sep2);

        // 6. Reposition Buttons: Left, Right, Bottom, Top
        let pos_lbl = gtk4::Label::builder()
            .label(crate::core::gettext("Position"))
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label", "caption"])
            .build();
        menu_box.append(&pos_lbl);

        let pos_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::Center)
            .build();

        let positions = [
            (
                crate::core::gettext("Left"),
                PaletteBarPosition::Left,
                "sidebar-show-symbolic",
            ),
            (
                crate::core::gettext("Right"),
                PaletteBarPosition::Right,
                "sidebar-show-right-symbolic",
            ),
            (
                crate::core::gettext("Bottom"),
                PaletteBarPosition::Bottom,
                "go-down-symbolic",
            ),
            (
                crate::core::gettext("Top"),
                PaletteBarPosition::Top,
                "go-up-symbolic",
            ),
        ];

        for (name, pos, icon) in positions {
            let btn = gtk4::Button::builder()
                .tooltip_text(&name)
                .icon_name(icon)
                .css_classes(["flat"])
                .build();

            let b_clone = bar.clone();
            let pop_close = popover.clone();
            btn.connect_clicked(move |_| {
                b_clone.set_position(pos);
                pop_close.popdown();
            });
            pos_box.append(&btn);
        }
        menu_box.append(&pos_box);

        popover.set_child(Some(&menu_box));
        popover.set_parent(&more_btn);

        {
            let p_c = popover.clone();
            more_btn.connect_destroy(move |_| {
                if p_c.parent().is_some() {
                    p_c.unparent();
                }
            });
        }

        let pop_open = popover.clone();
        more_btn.connect_clicked(move |_| {
            pop_open.popup();
        });

        container.append(&scrolled_win);
        container.append(&more_btn);

        bar
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.container
    }

    #[allow(dead_code)]
    pub fn set_on_reposition<F: Fn(PaletteBarPosition) + 'static>(&self, f: F) {
        *self.on_reposition.borrow_mut() = Some(Box::new(f));
    }

    pub fn set_position(&self, pos: PaletteBarPosition) {
        self.position.set(pos);
        match pos {
            PaletteBarPosition::Left => {
                self.container.set_orientation(gtk4::Orientation::Vertical);
                self.swatches_box
                    .set_orientation(gtk4::Orientation::Vertical);
                self.scrolled_win
                    .set_hscrollbar_policy(gtk4::PolicyType::Never);
                self.scrolled_win
                    .set_vscrollbar_policy(gtk4::PolicyType::Automatic);
                self.scrolled_win.set_max_content_height(440);
                self.scrolled_win.set_max_content_width(-1);
                self.container.set_halign(gtk4::Align::Start);
                self.container.set_valign(gtk4::Align::Center);
                self.container.set_margin_start(56);
                self.container.set_margin_end(0);
                self.container.set_margin_top(0);
                self.container.set_margin_bottom(0);
            }
            PaletteBarPosition::Right => {
                self.container.set_orientation(gtk4::Orientation::Vertical);
                self.swatches_box
                    .set_orientation(gtk4::Orientation::Vertical);
                self.scrolled_win
                    .set_hscrollbar_policy(gtk4::PolicyType::Never);
                self.scrolled_win
                    .set_vscrollbar_policy(gtk4::PolicyType::Automatic);
                self.scrolled_win.set_max_content_height(440);
                self.scrolled_win.set_max_content_width(-1);
                self.container.set_halign(gtk4::Align::End);
                self.container.set_valign(gtk4::Align::Center);
                self.container.set_margin_end(16);
                self.container.set_margin_start(0);
                self.container.set_margin_top(0);
                self.container.set_margin_bottom(0);
            }
            PaletteBarPosition::Bottom => {
                self.container
                    .set_orientation(gtk4::Orientation::Horizontal);
                self.swatches_box
                    .set_orientation(gtk4::Orientation::Horizontal);
                self.scrolled_win
                    .set_hscrollbar_policy(gtk4::PolicyType::Automatic);
                self.scrolled_win
                    .set_vscrollbar_policy(gtk4::PolicyType::Never);
                self.scrolled_win.set_max_content_width(520);
                self.scrolled_win.set_max_content_height(-1);
                self.container.set_halign(gtk4::Align::Center);
                self.container.set_valign(gtk4::Align::End);
                self.container.set_margin_bottom(24);
                self.container.set_margin_top(0);
                self.container.set_margin_start(0);
                self.container.set_margin_end(0);
            }
            PaletteBarPosition::Top => {
                self.container
                    .set_orientation(gtk4::Orientation::Horizontal);
                self.swatches_box
                    .set_orientation(gtk4::Orientation::Horizontal);
                self.scrolled_win
                    .set_hscrollbar_policy(gtk4::PolicyType::Automatic);
                self.scrolled_win
                    .set_vscrollbar_policy(gtk4::PolicyType::Never);
                self.scrolled_win.set_max_content_width(520);
                self.scrolled_win.set_max_content_height(-1);
                self.container.set_halign(gtk4::Align::Center);
                self.container.set_valign(gtk4::Align::Start);
                self.container.set_margin_top(48);
                self.container.set_margin_bottom(0);
                self.container.set_margin_start(0);
                self.container.set_margin_end(0);
            }
        }
        if let Some(ref cb) = *self.on_reposition.borrow() {
            cb(pos);
        }
    }

    pub fn apply_color(&self, color: Option<Color>) {
        self.active_color.set(color);
        match self.target_mode.get() {
            PaletteTargetMode::Fill => {
                if let Some(col) = color {
                    self.canvas.set_fill_color(col);
                } else {
                    self.canvas.set_fill_color(Color::new(0.0, 0.0, 0.0, 0.0));
                }
            }
            PaletteTargetMode::Stroke => {
                self.canvas.set_stroke_color(color);
            }
        }
        self.rebuild_swatches();
    }

    pub fn update_state(
        &self,
        _selected_count: usize,
        style: (Option<Color>, Option<Color>, f32),
        doc_colors: &[Option<Color>],
    ) {
        let current_target = match self.target_mode.get() {
            PaletteTargetMode::Fill => style.0,
            PaletteTargetMode::Stroke => style.1,
        };

        if self.current_preset.get() == PalettePreset::DocumentColors {
            let mut unique = Vec::new();
            for c in doc_colors {
                if !unique.contains(c) {
                    unique.push(*c);
                }
            }
            if *self.swatches.borrow() != unique {
                *self.swatches.borrow_mut() = unique;
                self.rebuild_swatches();
            }
        }

        if self.active_color.get() != current_target {
            self.active_color.set(current_target);
            self.rebuild_swatches();
        }
    }

    pub fn rebuild_swatches(&self) {
        while let Some(child) = self.swatches_box.first_child() {
            self.swatches_box.remove(&child);
        }

        let is_custom = self.current_preset.get() == PalettePreset::Custom;
        let list = self.swatches.borrow().clone();
        let active = self.active_color.get();

        for (idx, color_opt) in list.iter().copied().enumerate() {
            let is_active = match (color_opt, active) {
                (Some(c1), Some(c2)) => c1 == c2,
                (None, None) => true,
                (None, Some(c2)) => c2.a < 0.01,
                _ => false,
            };

            // Enhanced larger drawing area (32x32 px)
            let swatch_area = gtk4::DrawingArea::builder()
                .content_width(32)
                .content_height(32)
                .halign(gtk4::Align::Center)
                .valign(gtk4::Align::Center)
                .build();

            let col_draw = color_opt;
            swatch_area.set_draw_func(move |_area, cr, width, height| {
                let w = width as f64;
                let h = height as f64;
                let cx = w / 2.0;
                let cy = h / 2.0;
                let radius = 13.5;

                if let Some(col) = col_draw {
                    let r = col.r as f64;
                    let g = col.g as f64;
                    let b = col.b as f64;
                    let a = col.a as f64;

                    // Fill circular swatch
                    cr.arc(cx, cy, radius, 0.0, 2.0 * std::f64::consts::PI);
                    cr.set_source_rgba(r, g, b, a);
                    let _ = cr.fill_preserve();

                    // Subtle outline for boundary definition
                    if r + g + b > 2.35 {
                        cr.set_source_rgba(0.0, 0.0, 0.0, 0.22);
                        cr.set_line_width(1.0);
                        let _ = cr.stroke();
                    } else {
                        cr.set_source_rgba(0.0, 0.0, 0.0, 0.12);
                        cr.set_line_width(0.8);
                        let _ = cr.stroke();
                    }

                    // Selected / Active concentric inner ring
                    if is_active {
                        let inner_r = radius - 4.0;
                        cr.arc(cx, cy, inner_r, 0.0, 2.0 * std::f64::consts::PI);
                        if r + g + b > 1.5 {
                            cr.set_source_rgba(0.1, 0.1, 0.1, 0.95);
                        } else {
                            cr.set_source_rgba(1.0, 1.0, 1.0, 0.95);
                        }
                        cr.set_line_width(2.8);
                        let _ = cr.stroke();
                    }
                } else {
                    // Transparent / None circle with red diagonal slash
                    cr.arc(cx, cy, radius - 0.5, 0.0, 2.0 * std::f64::consts::PI);
                    cr.set_source_rgba(0.5, 0.5, 0.5, 0.35);
                    cr.set_line_width(1.4);
                    let _ = cr.stroke();

                    let offset = radius * std::f64::consts::FRAC_1_SQRT_2;
                    cr.move_to(cx - offset, cy - offset);
                    cr.line_to(cx + offset, cy + offset);
                    cr.set_source_rgba(0.88, 0.2, 0.2, 0.95);
                    cr.set_line_width(2.2);
                    let _ = cr.stroke();

                    if is_active {
                        cr.arc(cx, cy, radius - 4.0, 0.0, 2.0 * std::f64::consts::PI);
                        cr.set_source_rgba(0.2, 0.2, 0.2, 0.9);
                        cr.set_line_width(2.8);
                        let _ = cr.stroke();
                    }
                }
            });

            let tooltip = match color_opt {
                Some(c) => c.to_hex(),
                None => crate::core::gettext("Transparent / None"),
            };

            let btn = gtk4::Button::builder()
                .child(&swatch_area)
                .tooltip_text(&tooltip)
                .css_classes(["flat", "color-swatch-circle-btn"])
                .focus_on_click(false)
                .build();

            let b_clone = self.clone();
            btn.connect_clicked(move |_| {
                b_clone.apply_color(color_opt);
            });

            // Right Click Context Menu on Swatch (Delete Color, Copy Hex, Edit)
            let swatch_popover = gtk4::Popover::builder().has_arrow(true).build();
            let pop_menu_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(4)
                .margin_top(4)
                .margin_bottom(4)
                .margin_start(4)
                .margin_end(4)
                .build();

            // 1. Copy Hex
            if let Some(col) = color_opt {
                let copy_btn = gtk4::Button::builder()
                    .label(&format!(
                        "{}: {}",
                        crate::core::gettext("Copy Hex"),
                        col.to_hex()
                    ))
                    .icon_name("edit-copy-symbolic")
                    .css_classes(["flat"])
                    .halign(gtk4::Align::Start)
                    .build();
                let col_hex = col.to_hex();
                let pop_c = swatch_popover.clone();
                copy_btn.connect_clicked(move |_| {
                    if let Some(display) = gdk::Display::default() {
                        display.clipboard().set_text(&col_hex);
                    }
                    pop_c.popdown();
                });
                pop_menu_box.append(&copy_btn);
            }

            // 2. Delete Color (available on Custom Palette)
            if is_custom {
                let del_btn = gtk4::Button::builder()
                    .label(&crate::core::gettext("Delete Color"))
                    .icon_name("user-trash-symbolic")
                    .css_classes(["flat", "destructive-action"])
                    .halign(gtk4::Align::Start)
                    .build();
                let b_del = self.clone();
                let pop_c = swatch_popover.clone();
                del_btn.connect_clicked(move |_| {
                    let mut list = b_del.custom_colors.borrow_mut();
                    if idx < list.len() {
                        list.remove(idx);
                    }
                    drop(list);
                    *b_del.swatches.borrow_mut() = b_del.custom_colors.borrow().clone();
                    b_del.rebuild_swatches();
                    pop_c.popdown();
                });
                pop_menu_box.append(&del_btn);
            }

            swatch_popover.set_child(Some(&pop_menu_box));
            swatch_popover.set_parent(&btn);

            {
                let pop_c = swatch_popover.clone();
                btn.connect_destroy(move |_| {
                    if pop_c.parent().is_some() {
                        pop_c.unparent();
                    }
                });
            }

            let gesture_right = gtk4::GestureClick::builder().button(3).build();
            let pop_r = swatch_popover.clone();
            gesture_right.connect_released(move |_, _, _, _| {
                pop_r.popup();
            });
            btn.add_controller(gesture_right);

            self.swatches_box.append(&btn);
        }

        // Dedicated "+" Button (Always visible on Custom Palette or at end)
        if is_custom {
            let add_btn = gtk4::Button::builder()
                .icon_name("list-add-symbolic")
                .tooltip_text(crate::core::gettext("Add Color to Palette (+)"))
                .css_classes(["flat", "color-add-btn"])
                .focus_on_click(false)
                .build();

            let initial_col = self
                .active_color
                .get()
                .unwrap_or(Color::new(0.2, 0.55, 0.95, 1.0));

            let color_popover = ColorPickerPopover::new(self.canvas.clone(), initial_col, 0);
            color_popover.attach_to(&add_btn);

            let chosen_color = Rc::new(Cell::new(initial_col));

            // Live preview on canvas while dragging WITHOUT closing popover or rebuilding swatches
            {
                let chosen_c = chosen_color.clone();
                let b_clone = self.clone();
                color_popover.on_color_changed(move |col| {
                    chosen_c.set(col);
                    b_clone.active_color.set(Some(col));
                    match b_clone.target_mode.get() {
                        PaletteTargetMode::Fill => {
                            b_clone.canvas.set_fill_color(col);
                        }
                        PaletteTargetMode::Stroke => {
                            b_clone.canvas.set_stroke_color(Some(col));
                        }
                    }
                });
            }

            // Dedicated "Add to Palette" Button inside the Popover
            let confirm_btn = gtk4::Button::builder()
                .label(crate::core::gettext("Add to Palette"))
                .icon_name("list-add-symbolic")
                .css_classes(["suggested-action", "pill"])
                .halign(gtk4::Align::Fill)
                .margin_top(4)
                .margin_bottom(2)
                .margin_start(4)
                .margin_end(4)
                .build();

            {
                let b_clone = self.clone();
                let chosen_c = chosen_color.clone();
                let pop_c = color_popover.popover().clone();
                confirm_btn.connect_clicked(move |_| {
                    let col = chosen_c.get();
                    let mut list = b_clone.custom_colors.borrow_mut();
                    if !list.contains(&Some(col)) {
                        list.push(Some(col));
                    }
                    drop(list);
                    *b_clone.swatches.borrow_mut() = b_clone.custom_colors.borrow().clone();
                    pop_c.popdown();
                    b_clone.apply_color(Some(col));
                });
            }

            color_popover.append_footer_widget(&confirm_btn);

            {
                let c_pop = color_popover.clone();
                add_btn.connect_clicked(move |_| {
                    c_pop.popup();
                });
            }

            self.swatches_box.append(&add_btn);
        }
    }
}
