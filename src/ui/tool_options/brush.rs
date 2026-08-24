use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::core::{BrushMode, BrushStyle, StrokeCap};
use crate::ui::canvas::CanvasWidget;

#[derive(Clone)]
pub struct BrushControls {
    pub brush_box: gtk4::Box,
    pub btn_mode_brush: gtk4::ToggleButton,
    pub btn_mode_pencil: gtk4::ToggleButton,
    pub style_dd: gtk4::DropDown,
    pub width_spin: gtk4::SpinButton,
    pub smoothing_spin: gtk4::SpinButton,
    pub calligraphy_box: gtk4::Box,
    pub calligraphy_angle_spin: gtk4::SpinButton,
    pub btn_pressure: gtk4::ToggleButton,
    pub btn_taper_start: gtk4::ToggleButton,
    pub btn_taper_end: gtk4::ToggleButton,
    pub btn_auto_close: gtk4::ToggleButton,
    pub cap_dd: gtk4::DropDown,
    #[allow(dead_code)]
    pub btn_convert_path: gtk4::Button,
    pub status_lbl: gtk4::Label,
}

pub fn build_brush_controls(canvas: &CanvasWidget, is_syncing: &Rc<Cell<bool>>) -> BrushControls {
    let brush_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    // 1. Dual Mode Segmented Switch: 🖌️ Brush vs ✏️ Pencil
    let mode_seg_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .css_classes(["linked"])
        .build();

    let btn_mode_brush = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Brush"))
        .tooltip_text(&crate::core::gettext(
            "Brush Mode (Freehand variable-width styled strokes)",
        ))
        .active(true)
        .css_classes(["flat"])
        .build();

    let btn_mode_pencil = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Pencil"))
        .tooltip_text(&crate::core::gettext(
            "Pencil Mode (Auto-fits smooth vector Bézier curve paths)",
        ))
        .group(&btn_mode_brush)
        .css_classes(["flat"])
        .build();

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_mode_brush.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            if btn.is_active() {
                let state_rc = canvas_c.state();
                if let Ok(mut state) = state_rc.try_borrow_mut() {
                    if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                        if let Some(brush) = feat.as_brush_feature_mut() {
                            brush.mode = BrushMode::Brush;
                        }
                    }
                };
            }
        });
    }

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_mode_pencil.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            if btn.is_active() {
                let state_rc = canvas_c.state();
                if let Ok(mut state) = state_rc.try_borrow_mut() {
                    if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                        if let Some(brush) = feat.as_brush_feature_mut() {
                            brush.mode = BrushMode::Pencil;
                        }
                    }
                };
            }
        });
    }

    mode_seg_box.append(&btn_mode_brush);
    mode_seg_box.append(&btn_mode_pencil);
    brush_box.append(&mode_seg_box);

    let sep1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    brush_box.append(&sep1);

    // 2. Brush Preset / Style DropDown
    let style_names = [
        crate::core::gettext("Solid Round"),
        crate::core::gettext("Pencil (Graphite)"),
        crate::core::gettext("Calligraphy (Chisel 45°)"),
        crate::core::gettext("Ink Pen (Tapered)"),
        crate::core::gettext("Marker (Highlighter)"),
        crate::core::gettext("Airbrush (Soft)"),
    ];
    let style_model = gtk4::StringList::new(&style_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let style_dd = gtk4::DropDown::builder()
        .model(&style_model)
        .selected(0)
        .tooltip_text(&crate::core::gettext("Brush Preset / Style"))
        .valign(gtk4::Align::Center)
        .build();

    let calligraphy_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let cal_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Angle:"))
        .css_classes(["dim-label", "caption"])
        .build();
    let calligraphy_angle_spin = gtk4::SpinButton::with_range(-180.0, 180.0, 5.0);
    calligraphy_angle_spin.set_value(45.0);
    calligraphy_angle_spin.set_digits(0);
    calligraphy_angle_spin.set_tooltip_text(Some(&crate::core::gettext("Calligraphy Chisel Nib Angle (degrees)")));
    calligraphy_box.append(&cal_lbl);
    calligraphy_box.append(&calligraphy_angle_spin);

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        let cal_box_clone = calligraphy_box.clone();
        style_dd.connect_selected_notify(move |dd| {
            if is_sync.get() {
                return;
            }
            let style = match dd.selected() {
                0 => BrushStyle::Round,
                1 => BrushStyle::Pencil,
                2 => BrushStyle::Calligraphy,
                3 => BrushStyle::Ink,
                4 => BrushStyle::Marker,
                5 => BrushStyle::Airbrush,
                _ => BrushStyle::Round,
            };
            cal_box_clone.set_visible(style == BrushStyle::Calligraphy);
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.style = style;
                    }
                }
            };
        });
    }

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        calligraphy_angle_spin.connect_value_changed(move |spin| {
            if is_sync.get() {
                return;
            }
            let val = spin.value() as f32;
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.calligraphy_angle = val;
                    }
                }
            };
        });
    }

    brush_box.append(&style_dd);
    brush_box.append(&calligraphy_box);

    let sep2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    brush_box.append(&sep2);

    // 3. Stroke Width SpinButton
    let width_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let width_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Size:"))
        .css_classes(["dim-label", "caption"])
        .build();
    let width_spin = gtk4::SpinButton::with_range(1.0, 200.0, 1.0);
    width_spin.set_digits(1);
    width_spin.set_value(6.0);
    width_spin.set_tooltip_text(Some(&crate::core::gettext("Brush Size / Stroke Width (px)")));
    width_spin.set_valign(gtk4::Align::Center);
    width_box.append(&width_lbl);
    width_box.append(&width_spin);

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        width_spin.connect_value_changed(move |spin| {
            if is_sync.get() {
                return;
            }
            let val = spin.value() as f32;
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                state.active_stroke_width = val;
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.width = val;
                    }
                }
            };
        });
    }
    brush_box.append(&width_box);

    let sep3 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    brush_box.append(&sep3);

    // 4. Smoothing / Stabilizer (Streamline)
    let smooth_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let smooth_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Smooth:"))
        .css_classes(["dim-label", "caption"])
        .build();
    let smoothing_spin = gtk4::SpinButton::with_range(0.0, 100.0, 5.0);
    smoothing_spin.set_digits(0);
    smoothing_spin.set_value(50.0);
    smoothing_spin.set_tooltip_text(Some(&crate::core::gettext(
        "Stroke Smoothing & Stabilizer (0% to 100%) - prevents jitter",
    )));
    smoothing_spin.set_valign(gtk4::Align::Center);
    smooth_box.append(&smooth_lbl);
    smooth_box.append(&smoothing_spin);

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        smoothing_spin.connect_value_changed(move |spin| {
            if is_sync.get() {
                return;
            }
            let factor = (spin.value() as f32) / 100.0;
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.smoothing = factor;
                    }
                }
            };
        });
    }
    brush_box.append(&smooth_box);

    let sep4 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    brush_box.append(&sep4);

    // 5. Dynamics & Taper Toggle Buttons
    let dynamics_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .css_classes(["linked"])
        .valign(gtk4::Align::Center)
        .build();

    let img_pressure = gtk4::Image::from_icon_name("input-tablet-symbolic");
    img_pressure.set_pixel_size(16);
    let btn_pressure = gtk4::ToggleButton::builder()
        .child(&img_pressure)
        .tooltip_text(&crate::core::gettext("Pressure / Velocity Dynamics (Width changes with speed)"))
        .active(true)
        .css_classes(["flat"])
        .build();

    let btn_taper_start = gtk4::ToggleButton::builder()
        .label("▶")
        .tooltip_text(&crate::core::gettext("Taper Start (Fine point at stroke beginning)"))
        .active(false)
        .css_classes(["flat"])
        .build();

    let btn_taper_end = gtk4::ToggleButton::builder()
        .label("◀")
        .tooltip_text(&crate::core::gettext("Taper End (Fine point at stroke end)"))
        .active(false)
        .css_classes(["flat"])
        .build();

    let img_close = gtk4::Image::from_icon_name("object-flip-horizontal-symbolic");
    img_close.set_pixel_size(16);
    let btn_auto_close = gtk4::ToggleButton::builder()
        .child(&img_close)
        .tooltip_text(&crate::core::gettext("Auto-Close Path (Create closed shape with fill)"))
        .active(false)
        .css_classes(["flat"])
        .build();

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_pressure.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            let active = btn.is_active();
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.pressure_dynamics = active;
                    }
                }
            };
        });
    }

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_taper_start.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            let active = btn.is_active();
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.taper_start = active;
                    }
                }
            };
        });
    }

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_taper_end.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            let active = btn.is_active();
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.taper_end = active;
                    }
                }
            };
        });
    }

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_auto_close.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            let active = btn.is_active();
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.auto_close = active;
                    }
                }
            };
        });
    }

    dynamics_box.append(&btn_pressure);
    dynamics_box.append(&btn_taper_start);
    dynamics_box.append(&btn_taper_end);
    dynamics_box.append(&btn_auto_close);
    brush_box.append(&dynamics_box);

    let sep5 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    brush_box.append(&sep5);

    // 6. Cap Style DropDown
    let cap_names = [
        crate::core::gettext("Round Cap"),
        crate::core::gettext("Square Cap"),
        crate::core::gettext("Flat Cap"),
    ];
    let cap_model = gtk4::StringList::new(&cap_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let cap_dd = gtk4::DropDown::builder()
        .model(&cap_model)
        .selected(0)
        .tooltip_text(&crate::core::gettext("Stroke Cap Style"))
        .valign(gtk4::Align::Center)
        .build();

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        cap_dd.connect_selected_notify(move |dd| {
            if is_sync.get() {
                return;
            }
            let cap = match dd.selected() {
                0 => StrokeCap::Round,
                1 => StrokeCap::Square,
                2 => StrokeCap::Butt,
                _ => StrokeCap::Round,
            };
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.cap_style = cap;
                    }
                }
            };
        });
    }
    brush_box.append(&cap_dd);

    let sep6 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    brush_box.append(&sep6);

    // 7. Action: Convert Selected Brush Stroke to Vector Path
    let btn_convert_path = gtk4::Button::builder()
        .label(&crate::core::gettext("To Path"))
        .tooltip_text(&crate::core::gettext(
            "Convert selected brush stroke to editable Bézier path nodes",
        ))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    {
        let canvas_c = canvas.clone();
        btn_convert_path.connect_clicked(move |_| {
            canvas_c.convert_selected_to_path();
        });
    }
    brush_box.append(&btn_convert_path);

    // 8. Status Label
    let status_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Ready to draw"))
        .css_classes(["dim-label", "caption"])
        .margin_start(4)
        .margin_end(4)
        .valign(gtk4::Align::Center)
        .build();
    brush_box.append(&status_lbl);

    BrushControls {
        brush_box,
        btn_mode_brush,
        btn_mode_pencil,
        style_dd,
        width_spin,
        smoothing_spin,
        calligraphy_box,
        calligraphy_angle_spin,
        btn_pressure,
        btn_taper_start,
        btn_taper_end,
        btn_auto_close,
        cap_dd,
        btn_convert_path,
        status_lbl,
    }
}
