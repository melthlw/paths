use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use super::helpers::{create_resource_btn, create_resource_toggle_btn};
use crate::core::{Color, GradientType};
use crate::ui::canvas::CanvasWidget;
use crate::ui::color_picker::ColorPickerPopover;

#[derive(Clone)]
pub struct GradientControls {
    pub grad_box: gtk4::Box,
    pub btn_type_linear: gtk4::ToggleButton,
    pub btn_type_radial: gtk4::ToggleButton,
    pub angle_box: gtk4::Box,
    pub angle_spin: gtk4::SpinButton,
    pub btn_delete_stop: gtk4::Button,
    pub stop_pos_spin: gtk4::SpinButton,
    pub color_btn: gtk4::Button,
    pub color_da: gtk4::DrawingArea,
    pub stop_color_cell: Rc<Cell<Color>>,
}

pub fn build_gradient_controls(canvas: &CanvasWidget, is_syncing: &Rc<Cell<bool>>) -> GradientControls {
    let grad_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    // 1. Gradient Type Switcher (Linear vs Radial)
    let type_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["linked"])
        .valign(gtk4::Align::Center)
        .build();

    let btn_type_linear = create_resource_toggle_btn(
        "media-playlist-consecutive-symbolic",
        &crate::core::gettext("Linear Gradient"),
        true,
    );
    let btn_type_radial = create_resource_toggle_btn(
        "media-record-symbolic",
        &crate::core::gettext("Radial Gradient"),
        false,
    );

    {
        let cv_lin = canvas.clone();
        let sync_lin = is_syncing.clone();
        let btn_rad_c = btn_type_radial.clone();
        btn_type_linear.connect_toggled(move |b| {
            if sync_lin.get() {
                return;
            }
            if b.is_active() {
                sync_lin.set(true);
                btn_rad_c.set_active(false);
                sync_lin.set(false);
                cv_lin.set_gradient_type(GradientType::Linear);
            } else if !btn_rad_c.is_active() {
                b.set_active(true);
            }
        });
    }

    {
        let cv_rad = canvas.clone();
        let sync_rad = is_syncing.clone();
        let btn_lin_c = btn_type_linear.clone();
        btn_type_radial.connect_toggled(move |b| {
            if sync_rad.get() {
                return;
            }
            if b.is_active() {
                sync_rad.set(true);
                btn_lin_c.set_active(false);
                sync_rad.set(false);
                cv_rad.set_gradient_type(GradientType::Radial);
            } else if !btn_lin_c.is_active() {
                b.set_active(true);
            }
        });
    }

    type_box.append(&btn_type_linear);
    type_box.append(&btn_type_radial);
    grad_box.append(&type_box);

    let sep1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    grad_box.append(&sep1);

    // 2. Angle Box (Angle Spin for Linear)
    let angle_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();

    let angle_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Angle:"))
        .css_classes(["caption", "dim-label"])
        .valign(gtk4::Align::Center)
        .build();
    angle_box.append(&angle_lbl);

    let angle_adj = gtk4::Adjustment::new(90.0, 0.0, 360.0, 5.0, 15.0, 0.0);
    let angle_spin = gtk4::SpinButton::builder()
        .adjustment(&angle_adj)
        .climb_rate(1.0)
        .digits(0)
        .width_chars(4)
        .max_width_chars(5)
        .valign(gtk4::Align::Center)
        .tooltip_text(&crate::core::gettext("Gradient Angle (0° - 360°)"))
        .build();

    {
        let cv_ang = canvas.clone();
        let sync_ang = is_syncing.clone();
        angle_spin.connect_value_changed(move |spin| {
            if sync_ang.get() {
                return;
            }
            cv_ang.set_gradient_angle(spin.value() as f32);
        });
    }
    angle_box.append(&angle_spin);
    grad_box.append(&angle_box);

    // 3. Reverse Stops Button
    let btn_reverse = create_resource_btn(
        "object-flip-horizontal-symbolic",
        &crate::core::gettext("Reverse Stops (Swap start, middle, and end colors)"),
    );
    let cv_rev = canvas.clone();
    btn_reverse.connect_clicked(move |_| {
        cv_rev.reverse_selected_gradient_stops();
    });
    grad_box.append(&btn_reverse);

    let sep2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    grad_box.append(&sep2);

    // 4. Add Color Stop (+)
    let btn_add_stop = create_resource_btn(
        "list-add-symbolic",
        &crate::core::gettext("Add Intermediate Color Stop"),
    );
    let cv_as = canvas.clone();
    btn_add_stop.connect_clicked(move |_| {
        cv_as.add_selected_gradient_stop();
    });
    grad_box.append(&btn_add_stop);

    // 5. Delete Color Stop (Trash)
    let btn_delete_stop = create_resource_btn(
        "user-trash-symbolic",
        &crate::core::gettext("Delete Selected Color Stop"),
    );
    let cv_ds = canvas.clone();
    btn_delete_stop.connect_clicked(move |_| {
        cv_ds.delete_selected_gradient_stop();
    });
    grad_box.append(&btn_delete_stop);

    let sep3 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    grad_box.append(&sep3);

    // 6. Stop Position Spin (0% - 100%)
    let stop_pos_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let pos_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Pos:"))
        .css_classes(["caption", "dim-label"])
        .valign(gtk4::Align::Center)
        .build();
    stop_pos_box.append(&pos_lbl);

    let pos_adj = gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 0.0);
    let stop_pos_spin = gtk4::SpinButton::builder()
        .adjustment(&pos_adj)
        .climb_rate(1.0)
        .digits(0)
        .width_chars(3)
        .max_width_chars(4)
        .valign(gtk4::Align::Center)
        .tooltip_text(&crate::core::gettext("Selected Stop Position (0% - 100%)"))
        .build();

    {
        let cv_pos = canvas.clone();
        let sync_pos = is_syncing.clone();
        stop_pos_spin.connect_value_changed(move |spin| {
            if sync_pos.get() {
                return;
            }
            let val = (spin.value() / 100.0) as f32;
            let info = cv_pos.get_active_gradient_info();
            if let Some((stop_idx, _, _, _, _)) = info {
                let fills_opt = cv_pos.get_selected_fills_and_strokes();
                let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                if let Some(f0) = fills.first_mut() {
                    let mut eff = f0.effective_stops();
                    if stop_idx < eff.len() {
                        eff[stop_idx].offset = val.clamp(0.0, 1.0);
                        eff.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
                        f0.stops = eff;
                        cv_pos.set_selected_fills(fills);
                    }
                }
            }
        });
    }
    stop_pos_box.append(&stop_pos_spin);
    grad_box.append(&stop_pos_box);

    let sep4 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    grad_box.append(&sep4);

    // 7. Active Stop Color Swatch Button with Popover
    let color_btn = gtk4::Button::builder()
        .css_classes(["flat", "circular"])
        .tooltip_text(&crate::core::gettext("Selected Gradient Stop Color"))
        .valign(gtk4::Align::Center)
        .build();

    let stop_color_cell = Rc::new(Cell::new(Color::BLACK));
    let stop_col_c = stop_color_cell.clone();

    let color_da = gtk4::DrawingArea::builder()
        .content_width(18)
        .content_height(18)
        .build();
    {
        let s_c = stop_col_c.clone();
        color_da.set_draw_func(move |_, cr, w, h| {
            let col = s_c.get();
            cr.arc(
                w as f64 * 0.5,
                h as f64 * 0.5,
                (w.min(h) as f64 * 0.5) - 1.0,
                0.0,
                std::f64::consts::TAU,
            );
            cr.set_source_rgba(col.r as f64, col.g as f64, col.b as f64, col.a as f64);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.3);
            cr.set_line_width(1.0);
            let _ = cr.stroke();
        });
    }
    color_btn.set_child(Some(&color_da));

    {
        let cv_col = canvas.clone();
        let col_b = color_btn.clone();
        let col_cell_c = stop_color_cell.clone();
        let da_c = color_da.clone();
        color_btn.connect_clicked(move |_| {
            let info = cv_col.get_active_gradient_info();
            let cur_col = info.as_ref().map(|(_, _, _, _, c)| *c).unwrap_or(Color::BLACK);
            let active_stop = info.as_ref().map(|(s, _, _, _, _)| *s).unwrap_or(0);
            let cv_apply = cv_col.clone();
            let cell_upd = col_cell_c.clone();
            let da_upd = da_c.clone();
            let pop = ColorPickerPopover::with_mode_switcher(
                cv_col.clone(),
                cur_col,
                1, // gradient mode
                true,
            );
            pop.on_color_changed(move |new_c| {
                cell_upd.set(new_c);
                da_upd.queue_draw();
                cv_apply.set_gradient_stop_color(active_stop, new_c);
            });
            pop.attach_to(&col_b);
            pop.popup();
        });
    }
    grad_box.append(&color_btn);

    GradientControls {
        grad_box,
        btn_type_linear,
        btn_type_radial,
        angle_box,
        angle_spin,
        btn_delete_stop,
        stop_pos_spin,
        color_btn,
        color_da,
        stop_color_cell,
    }
}
