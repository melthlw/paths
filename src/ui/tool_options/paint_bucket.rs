use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::plugins::features::paint_bucket::{PaintBucketMode, PaintBucketTarget};
use crate::ui::canvas::CanvasWidget;

#[derive(Clone)]
pub struct PaintBucketControls {
    pub paint_bucket_box: gtk4::Box,
    pub btn_mode_create: gtk4::ToggleButton,
    pub btn_mode_paint: gtk4::ToggleButton,
    pub btn_target_fill: gtk4::ToggleButton,
    pub btn_target_stroke: gtk4::ToggleButton,
    pub btn_target_both: gtk4::ToggleButton,
    pub btn_detect_intersection: gtk4::ToggleButton,
    pub btn_auto_select: gtk4::ToggleButton,
    pub offset_spin: gtk4::SpinButton,
}

pub fn build_paint_bucket_controls(
    canvas: &CanvasWidget,
    is_syncing: &Rc<Cell<bool>>,
) -> PaintBucketControls {
    let paint_bucket_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    // 1. Mode Segmented Capsule (Create New Object vs Paint Existing)
    let mode_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["linked"])
        .valign(gtk4::Align::Center)
        .build();

    let btn_mode_create = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("New Object"))
        .tooltip_text(&crate::core::gettext(
            "Create New Object (Fill bounded area as a new shape)",
        ))
        .active(true)
        .css_classes(["flat"])
        .build();

    let btn_mode_paint = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Paint Object"))
        .tooltip_text(&crate::core::gettext(
            "Paint Existing Object (Apply color to clicked shape)",
        ))
        .group(&btn_mode_create)
        .css_classes(["flat"])
        .build();

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_mode_create.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            if btn.is_active() {
                let state_rc = canvas_c.state();
                if let Ok(mut state) = state_rc.try_borrow_mut() {
                    if let Some(feat) = state.plugin_manager.feature_by_id_mut("paint_bucket") {
                        if let Some(bucket) = feat.as_paint_bucket_feature_mut() {
                            bucket.mode = PaintBucketMode::CreateNew;
                        }
                    }
                }
            }
        });
    }

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_mode_paint.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            if btn.is_active() {
                let state_rc = canvas_c.state();
                if let Ok(mut state) = state_rc.try_borrow_mut() {
                    if let Some(feat) = state.plugin_manager.feature_by_id_mut("paint_bucket") {
                        if let Some(bucket) = feat.as_paint_bucket_feature_mut() {
                            bucket.mode = PaintBucketMode::PaintExisting;
                        }
                    }
                }
            }
        });
    }

    mode_box.append(&btn_mode_create);
    mode_box.append(&btn_mode_paint);
    paint_bucket_box.append(&mode_box);

    let sep1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    paint_bucket_box.append(&sep1);

    // 2. Target Segmented Capsule (Fill vs Stroke vs Both)
    let target_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["linked"])
        .valign(gtk4::Align::Center)
        .build();

    let btn_target_fill = gtk4::ToggleButton::builder()
        .icon_name("format-fill-symbolic")
        .tooltip_text(&crate::core::gettext("Fill Only (Preenchimento)"))
        .active(true)
        .css_classes(["flat"])
        .build();

    let btn_target_stroke = gtk4::ToggleButton::builder()
        .icon_name("format-stroke-symbolic")
        .tooltip_text(&crate::core::gettext("Stroke Only (Contorno)"))
        .group(&btn_target_fill)
        .css_classes(["flat"])
        .build();

    let btn_target_both = gtk4::ToggleButton::builder()
        .icon_name("duplicate-symbolic")
        .tooltip_text(&crate::core::gettext("Both Fill & Stroke (Ambos)"))
        .group(&btn_target_fill)
        .css_classes(["flat"])
        .build();

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_target_fill.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            if btn.is_active() {
                let state_rc = canvas_c.state();
                if let Ok(mut state) = state_rc.try_borrow_mut() {
                    if let Some(feat) = state.plugin_manager.feature_by_id_mut("paint_bucket") {
                        if let Some(bucket) = feat.as_paint_bucket_feature_mut() {
                            bucket.target = PaintBucketTarget::Fill;
                        }
                    }
                }
            }
        });
    }

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_target_stroke.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            if btn.is_active() {
                let state_rc = canvas_c.state();
                if let Ok(mut state) = state_rc.try_borrow_mut() {
                    if let Some(feat) = state.plugin_manager.feature_by_id_mut("paint_bucket") {
                        if let Some(bucket) = feat.as_paint_bucket_feature_mut() {
                            bucket.target = PaintBucketTarget::Stroke;
                        }
                    }
                }
            }
        });
    }

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_target_both.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            if btn.is_active() {
                let state_rc = canvas_c.state();
                if let Ok(mut state) = state_rc.try_borrow_mut() {
                    if let Some(feat) = state.plugin_manager.feature_by_id_mut("paint_bucket") {
                        if let Some(bucket) = feat.as_paint_bucket_feature_mut() {
                            bucket.target = PaintBucketTarget::Both;
                        }
                    }
                }
            }
        });
    }

    target_box.append(&btn_target_fill);
    target_box.append(&btn_target_stroke);
    target_box.append(&btn_target_both);
    paint_bucket_box.append(&target_box);

    let sep2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    paint_bucket_box.append(&sep2);

    // 3. Detect Intersecting Regions Toggle Button
    let btn_detect_intersection = gtk4::ToggleButton::builder()
        .icon_name("bool-intersection-symbolic")
        .tooltip_text(&crate::core::gettext(
            "Detect Intersecting Regions (Create shape from bounded sub-areas)",
        ))
        .active(true)
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_detect_intersection.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("paint_bucket") {
                    if let Some(bucket) = feat.as_paint_bucket_feature_mut() {
                        bucket.detect_intersection = btn.is_active();
                    }
                }
            }
        });
    }
    paint_bucket_box.append(&btn_detect_intersection);

    // 4. Auto-select Created Shape Toggle Button
    let btn_auto_select = gtk4::ToggleButton::builder()
        .icon_name("object-select-symbolic")
        .tooltip_text(&crate::core::gettext(
            "Auto-select Created Shape (Selecionar automaticamente novo objeto)",
        ))
        .active(true)
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_auto_select.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("paint_bucket") {
                    if let Some(bucket) = feat.as_paint_bucket_feature_mut() {
                        bucket.auto_select = btn.is_active();
                    }
                }
            }
        });
    }
    paint_bucket_box.append(&btn_auto_select);

    let sep3 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    paint_bucket_box.append(&sep3);

    // 5. Offset / Inset / Outset adjustment SpinButton
    let offset_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();

    let offset_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Offset:"))
        .tooltip_text(&crate::core::gettext(
            "Area Inset / Outset Offset (Grow or shrink filled area in px)",
        ))
        .css_classes(["dim-label", "caption"])
        .build();
    offset_box.append(&offset_lbl);

    let offset_spin = gtk4::SpinButton::builder()
        .adjustment(&gtk4::Adjustment::new(0.0, -100.0, 100.0, 1.0, 5.0, 0.0))
        .climb_rate(1.0)
        .digits(1)
        .tooltip_text(&crate::core::gettext(
            "Area Inset / Outset Offset (Grow or shrink filled area in px)",
        ))
        .valign(gtk4::Align::Center)
        .width_chars(5)
        .build();

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        offset_spin.connect_value_changed(move |spin| {
            if is_sync.get() {
                return;
            }
            let state_rc = canvas_c.state();
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                if let Some(feat) = state.plugin_manager.feature_by_id_mut("paint_bucket") {
                    if let Some(bucket) = feat.as_paint_bucket_feature_mut() {
                        bucket.offset = spin.value() as f32;
                    }
                }
            }
        });
    }
    offset_box.append(&offset_spin);
    paint_bucket_box.append(&offset_box);

    PaintBucketControls {
        paint_bucket_box,
        btn_mode_create,
        btn_mode_paint,
        btn_target_fill,
        btn_target_stroke,
        btn_target_both,
        btn_detect_intersection,
        btn_auto_select,
        offset_spin,
    }
}
