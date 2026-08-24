use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::plugins::features::pen::PenMode;
use crate::ui::canvas::CanvasWidget;

#[derive(Clone)]
#[allow(dead_code)]
pub struct PenControls {
    pub pen_box: gtk4::Box,
    pub btn_mode_bezier: gtk4::ToggleButton,
    pub btn_mode_lines: gtk4::ToggleButton,
    pub btn_undo_node: gtk4::Button,
    pub btn_close_path: gtk4::Button,
    pub btn_finish_path: gtk4::Button,
    pub btn_resume_path: gtk4::Button,
    pub status_lbl: gtk4::Label,
}

pub fn build_pen_controls(canvas: &CanvasWidget, is_syncing: &Rc<Cell<bool>>) -> PenControls {
    let pen_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    // 1. Mode Segmented Capsule (Bézier vs Linhas Retas)
    let mode_seg_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .css_classes(["linked"])
        .build();

    let btn_mode_bezier = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Bézier"))
        .tooltip_text(&crate::core::gettext("Bézier Curves (Click & Drag for tangent handles)"))
        .active(true)
        .css_classes(["flat"])
        .build();

    let btn_mode_lines = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Lines"))
        .tooltip_text(&crate::core::gettext("Straight Line Segments"))
        .group(&btn_mode_bezier)
        .css_classes(["flat"])
        .build();

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_mode_bezier.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            if btn.is_active() {
                let state_rc = canvas_c.state();
                if let Ok(mut state) = state_rc.try_borrow_mut() {
                    if let Some(feat) = state.plugin_manager.feature_by_id_mut("pen") {
                        if let Some(pen) = feat.as_pen_feature_mut() {
                            pen.mode = PenMode::Bezier;
                        }
                    }
                };
            }
        });
    }

    {
        let canvas_c = canvas.clone();
        let is_sync = is_syncing.clone();
        btn_mode_lines.connect_toggled(move |btn| {
            if is_sync.get() {
                return;
            }
            if btn.is_active() {
                let state_rc = canvas_c.state();
                if let Ok(mut state) = state_rc.try_borrow_mut() {
                    if let Some(feat) = state.plugin_manager.feature_by_id_mut("pen") {
                        if let Some(pen) = feat.as_pen_feature_mut() {
                            pen.mode = PenMode::Lines;
                        }
                    }
                };
            }
        });
    }

    mode_seg_box.append(&btn_mode_bezier);
    mode_seg_box.append(&btn_mode_lines);
    pen_box.append(&mode_seg_box);

    let sep1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    pen_box.append(&sep1);

    // 2. Undo Last Node Button
    let btn_undo_node = gtk4::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text(&crate::core::gettext("Undo Last Node (Ctrl+Z / Backspace)"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    {
        let c = canvas.clone();
        btn_undo_node.connect_clicked(move |_| {
            c.pen_undo_node();
        });
    }
    pen_box.append(&btn_undo_node);

    // 3. Close Path Button
    let btn_close_path = gtk4::Button::builder()
        .icon_name("media-playlist-repeat-symbolic")
        .tooltip_text(&crate::core::gettext("Close & Finish Path (C)"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    {
        let c = canvas.clone();
        btn_close_path.connect_clicked(move |_| {
            c.pen_close_path();
        });
    }
    pen_box.append(&btn_close_path);

    // 4. Finish Path Button
    let btn_finish_path = gtk4::Button::builder()
        .icon_name("emblem-ok-symbolic")
        .tooltip_text(&crate::core::gettext("Finish Open Path (Enter / Right-Click)"))
        .css_classes(["flat", "suggested-action"])
        .valign(gtk4::Align::Center)
        .build();

    {
        let c = canvas.clone();
        btn_finish_path.connect_clicked(move |_| {
            c.pen_finish_path();
        });
    }
    pen_box.append(&btn_finish_path);

    let sep2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    pen_box.append(&sep2);

    // 5. Resume Path Button
    let btn_resume_path = gtk4::Button::builder()
        .icon_name("edit-find-replace-symbolic")
        .tooltip_text(&crate::core::gettext("Continue / Resume Selected Open Path"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    {
        let c = canvas.clone();
        btn_resume_path.connect_clicked(move |_| {
            c.pen_resume_path();
        });
    }
    pen_box.append(&btn_resume_path);

    // 6. Status Label
    let status_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Ready to draw"))
        .css_classes(["dim-label", "caption"])
        .margin_start(6)
        .margin_end(4)
        .valign(gtk4::Align::Center)
        .build();
    pen_box.append(&status_lbl);

    PenControls {
        pen_box,
        btn_mode_bezier,
        btn_mode_lines,
        btn_undo_node,
        btn_close_path,
        btn_finish_path,
        btn_resume_path,
        status_lbl,
    }
}
