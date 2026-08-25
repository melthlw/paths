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
}

pub fn build_brush_controls(canvas: &CanvasWidget, is_syncing: &Rc<Cell<bool>>) -> BrushControls {
    let brush_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    // 1. Dual Mode Segmented Switch: Brush vs Pencil
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
    let style_dd = gtk4::DropDown::builder()
        .selected(0)
        .tooltip_text(&crate::core::gettext("Brush Preset / Style"))
        .valign(gtk4::Align::Center)
        .build();

    fn update_brush_style_model(style_dd: &gtk4::DropDown) {
        let mut style_names = vec![
            crate::core::gettext("Solid Round"),
            crate::core::gettext("Pencil Graphite"),
            crate::core::gettext("Calligraphy Ribbon"),
            crate::core::gettext("Ink Pen"),
            crate::core::gettext("Highlighter Marker"),
            crate::core::gettext("Airbrush Soft"),
            crate::core::gettext("Charcoal Sketch"),
            crate::core::gettext("Watercolor Wash"),
            crate::core::gettext("Neon Glow Ribbon"),
            crate::core::gettext("Grainy Chalk"),
            crate::core::gettext("Spray Splatter"),
            crate::core::gettext("Star Trail"),
            crate::core::gettext("Pearl Bead Chain"),
            crate::core::gettext("Vector Arrow Trail"),
        ];

        let custom_presets = crate::core::brush_store::load_custom_brush_presets();
        for preset in &custom_presets {
            style_names.push(format!("Custom: {}", preset.name));
        }

        let refs: Vec<&str> = style_names.iter().map(|s| s.as_str()).collect();
        let model = gtk4::StringList::new(&refs);
        style_dd.set_model(Some(&model));
    }

    update_brush_style_model(&style_dd);

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
    calligraphy_angle_spin.set_tooltip_text(Some(&crate::core::gettext(
        "Calligraphy Chisel Nib Angle (degrees)",
    )));
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
            let idx = dd.selected() as usize;
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        if idx < 14 {
                            let style = match idx {
                                0 => BrushStyle::Round,
                                1 => BrushStyle::Pencil,
                                2 => BrushStyle::Calligraphy,
                                3 => BrushStyle::Ink,
                                4 => BrushStyle::Marker,
                                5 => BrushStyle::Airbrush,
                                6 => BrushStyle::Charcoal,
                                7 => BrushStyle::Watercolor,
                                8 => BrushStyle::NeonGlow,
                                9 => BrushStyle::Chalk,
                                10 => BrushStyle::SprayPaint,
                                11 => BrushStyle::StarTrail,
                                12 => BrushStyle::BeadChain,
                                13 => BrushStyle::ArrowTrail,
                                _ => BrushStyle::Round,
                            };
                            brush.style = style;
                            cal_box_clone.set_visible(style == BrushStyle::Calligraphy);

                            if style == BrushStyle::StarTrail {
                                brush.body_marker = crate::core::MarkerShape::Star;
                                brush.body_spacing = 3.0;
                            } else if style == BrushStyle::BeadChain {
                                brush.body_marker = crate::core::MarkerShape::Circle;
                                brush.body_spacing = 2.5;
                            } else if style == BrushStyle::ArrowTrail {
                                brush.body_marker = crate::core::MarkerShape::StealthArrow;
                                brush.end_marker = crate::core::MarkerShape::Arrow;
                                brush.body_spacing = 3.5;
                            } else {
                                brush.start_marker = crate::core::MarkerShape::None;
                                brush.body_marker = crate::core::MarkerShape::None;
                                brush.end_marker = crate::core::MarkerShape::None;
                            }
                        } else {
                            let custom_presets =
                                crate::core::brush_store::load_custom_brush_presets();
                            let custom_idx = idx - 14;
                            if custom_idx < custom_presets.len() {
                                let preset = &custom_presets[custom_idx];
                                brush.style = preset.style;
                                brush.width = preset.width;
                                brush.smoothing = preset.smoothing;
                                brush.calligraphy_angle = preset.calligraphy_angle;
                                brush.auto_close = preset.auto_close;
                                brush.cap_style = preset.cap_style;
                                brush.join_style = preset.join_style;
                                brush.taper_start = preset.taper_start;
                                brush.taper_end = preset.taper_end;
                                brush.start_marker = preset.start_marker.clone();
                                brush.body_marker = if let Some(d) = &preset.svg_path_d {
                                    crate::core::MarkerShape::CustomPath(d.clone())
                                } else {
                                    preset.body_marker.clone()
                                };
                                brush.end_marker = preset.end_marker.clone();
                                brush.body_spacing = preset.body_spacing;
                                brush.marker_scale = preset.marker_scale;

                                cal_box_clone.set_visible(preset.style == BrushStyle::Calligraphy);
                            }
                        }
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
    width_spin.set_tooltip_text(Some(&crate::core::gettext(
        "Brush Size / Stroke Width (px)",
    )));
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

    // ── 5. Advanced Stroke & Contour Options Popover ──
    let btn_options_popover = gtk4::MenuButton::builder()
        .icon_name("preferences-other-symbolic")
        .tooltip_text(&crate::core::gettext(
            "Contour & Stroke Options (Markers, Caps, Taper)",
        ))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    let popover = gtk4::Popover::builder().autohide(true).build();

    let pop_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .width_request(240)
        .build();

    // Section 1: Dynamics & Taper Toggles
    let dynamics_title = gtk4::Label::builder()
        .label(&crate::core::gettext("Stroke Dynamics"))
        .css_classes(["heading", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    pop_box.append(&dynamics_title);

    let dynamics_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .css_classes(["linked"])
        .halign(gtk4::Align::Fill)
        .build();

    let img_pressure = gtk4::Image::from_icon_name("brush-pressure-symbolic");
    img_pressure.set_pixel_size(16);
    let btn_pressure = gtk4::ToggleButton::builder()
        .child(&img_pressure)
        .tooltip_text(&crate::core::gettext("Pressure Dynamics"))
        .active(true)
        .css_classes(["flat"])
        .build();

    let btn_taper_start = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Taper In"))
        .tooltip_text(&crate::core::gettext("Taper Start"))
        .active(false)
        .css_classes(["flat"])
        .build();

    let btn_taper_end = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Taper Out"))
        .tooltip_text(&crate::core::gettext("Taper End"))
        .active(false)
        .css_classes(["flat"])
        .build();

    let img_close = gtk4::Image::from_icon_name("flip-horizontal-symbolic");
    img_close.set_pixel_size(16);
    let btn_auto_close = gtk4::ToggleButton::builder()
        .child(&img_close)
        .tooltip_text(&crate::core::gettext("Auto-Close Path"))
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
    pop_box.append(&dynamics_box);

    let pop_sep1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    pop_box.append(&pop_sep1);

    // Section 2: Stroke Cap Style
    let cap_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    let cap_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Cap Style:"))
        .css_classes(["dim-label", "caption"])
        .build();

    let cap_names = [
        crate::core::gettext("Round Cap"),
        crate::core::gettext("Square Cap"),
        crate::core::gettext("Flat Cap"),
    ];
    let cap_model =
        gtk4::StringList::new(&cap_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let cap_dd = gtk4::DropDown::builder()
        .model(&cap_model)
        .selected(0)
        .hexpand(true)
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
    cap_box.append(&cap_lbl);
    cap_box.append(&cap_dd);
    pop_box.append(&cap_box);

    let pop_sep2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    pop_box.append(&pop_sep2);

    // Section 3: Contour Markers (Start, Middle, End)
    let markers_title = gtk4::Label::builder()
        .label(&crate::core::gettext("Contour Markers"))
        .css_classes(["heading", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    pop_box.append(&markers_title);

    let marker_names = [
        crate::core::gettext("None"),
        crate::core::gettext("Arrow"),
        crate::core::gettext("Stealth Arrow"),
        crate::core::gettext("Circle Cap"),
        crate::core::gettext("Diamond"),
        crate::core::gettext("Square Block"),
        crate::core::gettext("Triangle Point"),
        crate::core::gettext("Star Motif"),
    ];

    let index_to_marker_shape = |idx: u32| match idx {
        0 => crate::core::MarkerShape::None,
        1 => crate::core::MarkerShape::Arrow,
        2 => crate::core::MarkerShape::StealthArrow,
        3 => crate::core::MarkerShape::Circle,
        4 => crate::core::MarkerShape::Diamond,
        5 => crate::core::MarkerShape::Square,
        6 => crate::core::MarkerShape::Triangle,
        7 => crate::core::MarkerShape::Star,
        _ => crate::core::MarkerShape::None,
    };

    let marker_model =
        gtk4::StringList::new(&marker_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    // Start Marker Row
    let start_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    let start_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Start:"))
        .css_classes(["dim-label", "caption"])
        .width_request(55)
        .halign(gtk4::Align::Start)
        .build();
    let start_marker_dd = gtk4::DropDown::builder()
        .model(&marker_model)
        .selected(0)
        .hexpand(true)
        .build();
    start_row.append(&start_lbl);
    start_row.append(&start_marker_dd);

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        start_marker_dd.connect_selected_notify(move |dd| {
            if is_sync.get() {
                return;
            }
            let marker = index_to_marker_shape(dd.selected());
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.start_marker = marker;
                    }
                }
            }
        });
    }

    // Middle Marker Row
    let body_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    let body_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Middle:"))
        .css_classes(["dim-label", "caption"])
        .width_request(55)
        .halign(gtk4::Align::Start)
        .build();
    let body_marker_dd = gtk4::DropDown::builder()
        .model(&marker_model)
        .selected(0)
        .hexpand(true)
        .build();
    body_row.append(&body_lbl);
    body_row.append(&body_marker_dd);

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        body_marker_dd.connect_selected_notify(move |dd| {
            if is_sync.get() {
                return;
            }
            let marker = index_to_marker_shape(dd.selected());
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.body_marker = marker;
                    }
                }
            }
        });
    }

    // End Marker Row
    let end_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    let end_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("End:"))
        .css_classes(["dim-label", "caption"])
        .width_request(55)
        .halign(gtk4::Align::Start)
        .build();
    let end_marker_dd = gtk4::DropDown::builder()
        .model(&marker_model)
        .selected(0)
        .hexpand(true)
        .build();
    end_row.append(&end_lbl);
    end_row.append(&end_marker_dd);

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        end_marker_dd.connect_selected_notify(move |dd| {
            if is_sync.get() {
                return;
            }
            let marker = index_to_marker_shape(dd.selected());
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.end_marker = marker;
                    }
                }
            }
        });
    }

    pop_box.append(&start_row);
    pop_box.append(&body_row);
    pop_box.append(&end_row);

    // Marker Scale Row
    let scale_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    let scale_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Scale:"))
        .css_classes(["dim-label", "caption"])
        .width_request(55)
        .halign(gtk4::Align::Start)
        .build();
    let marker_scale_spin = gtk4::SpinButton::with_range(0.2, 5.0, 0.1);
    marker_scale_spin.set_digits(1);
    marker_scale_spin.set_value(1.0);
    marker_scale_spin.set_hexpand(true);
    scale_row.append(&scale_lbl);
    scale_row.append(&marker_scale_spin);

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        marker_scale_spin.connect_value_changed(move |spin| {
            if is_sync.get() {
                return;
            }
            let val = spin.value() as f32;
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.marker_scale = val;
                    }
                }
            }
        });
    }

    // Body Spacing Row
    let spacing_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    let spacing_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Spacing:"))
        .css_classes(["dim-label", "caption"])
        .width_request(55)
        .halign(gtk4::Align::Start)
        .build();
    let body_spacing_spin = gtk4::SpinButton::with_range(1.0, 10.0, 0.5);
    body_spacing_spin.set_digits(1);
    body_spacing_spin.set_value(2.0);
    body_spacing_spin.set_hexpand(true);
    spacing_row.append(&spacing_lbl);
    spacing_row.append(&body_spacing_spin);

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        body_spacing_spin.connect_value_changed(move |spin| {
            if is_sync.get() {
                return;
            }
            let val = spin.value() as f32;
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("brush") {
                    if let Some(brush) = feat.as_brush_feature_mut() {
                        brush.body_spacing = val;
                    }
                }
            }
        });
    }

    pop_box.append(&scale_row);
    pop_box.append(&spacing_row);

    popover.set_child(Some(&pop_box));
    btn_options_popover.set_popover(Some(&popover));
    brush_box.append(&btn_options_popover);

    let sep5 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    brush_box.append(&sep5);

    // 6. Action: Convert Selected Brush Stroke to Vector Path
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

    // 7. Clipboard to Brush Preset Button
    let btn_clipboard_to_brush = gtk4::Button::builder()
        .icon_name("edit-paste-symbolic")
        .tooltip_text(&crate::core::gettext(
            "Clipboard to Brush (Create custom brush preset from clipboard path)",
        ))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    {
        let canvas_c = canvas.clone();
        let style_dd_c = style_dd.clone();
        btn_clipboard_to_brush.connect_clicked(move |btn| {
            let display = gtk4::gdk::Display::default();
            if let Some(disp) = display {
                let clipboard = disp.clipboard();
                let canvas_cc = canvas_c.clone();
                let style_dd_cc = style_dd_c.clone();
                let btn_c = btn.clone();
                clipboard.read_text_async(None::<&gtk4::gio::Cancellable>, move |res| {
                    let path_text = match res {
                        Ok(Some(text)) => text.to_string(),
                        _ => String::new(),
                    };

                    let current_width = canvas_cc.active_stroke_width();
                    let (current_style, current_smoothing) = {
                        let state_rc = canvas_cc.state();
                        if let Ok(state) = state_rc.try_borrow() {
                            if let Some(feat) = state.plugin_manager.feature_by_id("brush") {
                                if let Some(brush) = feat.as_brush_feature() {
                                    (brush.style, brush.smoothing)
                                } else {
                                    (crate::core::BrushStyle::Round, 0.5)
                                }
                            } else {
                                (crate::core::BrushStyle::Round, 0.5)
                            }
                        } else {
                            (crate::core::BrushStyle::Round, 0.5)
                        }
                    };

                    let preset_res = crate::core::brush_store::create_brush_from_clipboard_or_path(
                        "Clipboard Brush",
                        &path_text,
                        current_style,
                        current_width,
                        current_smoothing,
                    );

                    match preset_res {
                        Ok(preset) => {
                            update_brush_style_model(&style_dd_cc);
                            let custom_presets =
                                crate::core::brush_store::load_custom_brush_presets();
                            if let Some(pos) =
                                custom_presets.iter().position(|p| p.id == preset.id)
                            {
                                style_dd_cc.set_selected((14 + pos) as u32);
                            }
                            btn_c.set_tooltip_text(Some(&format!(
                                "{} ('{}')",
                                crate::core::gettext("Brush saved to ~/.config/paths/brushes/"),
                                preset.name
                            )));
                        }
                        Err(e) => {
                            eprintln!("Failed to save brush preset: {}", e);
                        }
                    }
                });
            }
        });
    }
    brush_box.append(&btn_clipboard_to_brush);

    // 8. Selection to Brush Preset Button
    let btn_selection_to_brush = gtk4::Button::builder()
        .icon_name("object-select-symbolic")
        .tooltip_text(&crate::core::gettext(
            "Selection to Brush (Create custom brush preset from active vector selection)",
        ))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    {
        let canvas_c = canvas.clone();
        let style_dd_c = style_dd.clone();
        btn_selection_to_brush.connect_clicked(move |btn| {
            let res = canvas_c.create_brush_preset_from_selected("Selected Brush");
            match res {
                Ok(preset) => {
                    update_brush_style_model(&style_dd_c);
                    let custom_presets = crate::core::brush_store::load_custom_brush_presets();
                    if let Some(pos) = custom_presets.iter().position(|p| p.id == preset.id) {
                        style_dd_c.set_selected((14 + pos) as u32);
                    }
                    btn.set_tooltip_text(Some(&format!(
                        "{} ('{}')",
                        crate::core::gettext("Brush saved to ~/.config/paths/brushes/"),
                        preset.name
                    )));
                }
                Err(e) => {
                    eprintln!("Failed to save selection as brush: {}", e);
                }
            }
        });
    }
    brush_box.append(&btn_selection_to_brush);

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
    }
}
