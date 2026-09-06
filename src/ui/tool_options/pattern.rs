use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::helpers::create_resource_btn;
use crate::core::element::PatternType;
use crate::core::{Color, Point};
use crate::ui::canvas::CanvasWidget;
use crate::ui::color_picker::ColorPickerPopover;
use crate::ui::inspector::appearance::fills::{
    create_pattern_preview_tile, draw_pattern_preview_cairo,
};

#[derive(Clone)]
pub struct PatternControls {
    pub pattern_box: gtk4::Box,
    pub pattern_type_lbl: gtk4::Label,
    pub pattern_type_da: gtk4::DrawingArea,
    pub pattern_type_cell: Rc<Cell<PatternType>>,
    pub custom_path_cell: Rc<RefCell<Option<String>>>,
    pub scale_spin: gtk4::SpinButton,
    pub angle_spin: gtk4::SpinButton,
    pub offset_x_spin: gtk4::SpinButton,
    pub offset_y_spin: gtk4::SpinButton,
    pub c1_da: gtk4::DrawingArea,
    pub c1_cell: Rc<Cell<Color>>,
    pub c2_da: gtk4::DrawingArea,
    pub c2_cell: Rc<Cell<Color>>,
}

pub fn build_pattern_controls(
    canvas: &CanvasWidget,
    is_syncing: &Rc<Cell<bool>>,
) -> PatternControls {
    let pattern_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let pattern_type_cell = Rc::new(Cell::new(PatternType::Checkerboard));
    let custom_path_cell = Rc::new(RefCell::new(None::<String>));

    // 1. Pattern Type Button with Live Mini Preview & Dropdown Popover
    let btn_content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .build();

    let pattern_type_da = gtk4::DrawingArea::builder()
        .content_width(20)
        .content_height(14)
        .valign(gtk4::Align::Center)
        .build();

    {
        let pt_c = pattern_type_cell.clone();
        let cp_c = custom_path_cell.clone();
        pattern_type_da.set_draw_func(move |_, cr, w, h| {
            let pt = pt_c.get();
            let cp = cp_c.borrow();
            draw_pattern_preview_cairo(cr, w as f64, h as f64, pt, cp.as_deref());
        });
    }
    btn_content_box.append(&pattern_type_da);

    let pattern_type_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Checkerboard"))
        .valign(gtk4::Align::Center)
        .build();
    btn_content_box.append(&pattern_type_lbl);

    let arrow_icon = gtk4::Image::from_icon_name("pan-down-symbolic");
    arrow_icon.set_pixel_size(12);
    arrow_icon.set_valign(gtk4::Align::Center);
    btn_content_box.append(&arrow_icon);

    let pattern_btn = gtk4::Button::builder()
        .child(&btn_content_box)
        .css_classes(["flat", "pill-btn"])
        .tooltip_text(&crate::core::gettext("Choose Pattern Style & Preset"))
        .valign(gtk4::Align::Center)
        .build();

    // Visual Pattern Preview Popover
    let pattern_popover = gtk4::Popover::builder()
        .has_arrow(true)
        .position(gtk4::PositionType::Bottom)
        .css_classes(["color-picker-popover", "pattern-selector-popover"])
        .build();
    pattern_popover.set_parent(&pattern_btn);

    {
        let popover_c = pattern_popover.clone();
        pattern_btn.connect_destroy(move |_| {
            if popover_c.parent().is_some() {
                popover_c.unparent();
            }
        });
    }

    let pop_vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .width_request(260)
        .build();

    let title_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Patterns"))
        .css_classes(["heading", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    pop_vbox.append(&title_lbl);

    let scroller = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .max_content_height(320)
        .propagate_natural_height(true)
        .build();

    let pat_grid = gtk4::FlowBox::builder()
        .valign(gtk4::Align::Start)
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(2)
        .min_children_per_line(2)
        .homogeneous(true)
        .row_spacing(4)
        .column_spacing(4)
        .build();

    let pattern_types = [
        (PatternType::Grid, crate::core::gettext("Technical Grid")),
        (PatternType::Dots, crate::core::gettext("Halftone Dots")),
        (PatternType::Stripes, crate::core::gettext("Diagonal Stripes")),
        (PatternType::Checkerboard, crate::core::gettext("Checkerboard")),
        (PatternType::Hexagon, crate::core::gettext("Honeycomb")),
        (PatternType::Crosshatch, crate::core::gettext("Crosshatch")),
        (PatternType::Brick, crate::core::gettext("Brick Wall")),
        (PatternType::Scales, crate::core::gettext("Seigaiha Scales")),
        (PatternType::Houndstooth, crate::core::gettext("Houndstooth")),
        (PatternType::Basketweave, crate::core::gettext("Basketweave")),
    ];

    for (pt, pt_name) in pattern_types {
        let cv_p = canvas.clone();
        let pop_close = pattern_popover.clone();
        let pt_cell = pattern_type_cell.clone();
        let cp_cell = custom_path_cell.clone();
        let lbl_upd = pattern_type_lbl.clone();
        let da_upd = pattern_type_da.clone();
        let is_sync_p = is_syncing.clone();
        let name_str = pt_name.clone();

        let tile = create_pattern_preview_tile(
            pt,
            None,
            &pt_name,
            false,
            move || {
                pt_cell.set(pt);
                *cp_cell.borrow_mut() = None;
                lbl_upd.set_label(&name_str);
                da_upd.queue_draw();
                if !is_sync_p.get() {
                    cv_p.set_pattern_type_and_custom_path(pt, None);
                }
                pop_close.popdown();
            },
        );
        pat_grid.append(&tile);
    }

    let user_patterns = crate::core::scan_user_patterns();
    for cp in user_patterns {
        let cp_name = cp.name.clone();
        let cp_path = cp.file_path.clone();
        let cv_p = canvas.clone();
        let pop_close = pattern_popover.clone();
        let pt_cell = pattern_type_cell.clone();
        let cp_cell = custom_path_cell.clone();
        let lbl_upd = pattern_type_lbl.clone();
        let da_upd = pattern_type_da.clone();
        let is_sync_p = is_syncing.clone();
        let name_str = cp_name.clone();

        let tile = create_pattern_preview_tile(
            PatternType::Custom,
            Some(cp_path.clone()),
            &cp_name,
            false,
            move || {
                pt_cell.set(PatternType::Custom);
                *cp_cell.borrow_mut() = Some(cp_path.clone());
                lbl_upd.set_label(&name_str);
                da_upd.queue_draw();
                if !is_sync_p.get() {
                    cv_p.set_pattern_type_and_custom_path(PatternType::Custom, Some(cp_path.clone()));
                }
                pop_close.popdown();
            },
        );
        pat_grid.append(&tile);
    }

    scroller.set_child(Some(&pat_grid));
    pop_vbox.append(&scroller);
    pattern_popover.set_child(Some(&pop_vbox));

    {
        let pop_show = pattern_popover.clone();
        pattern_btn.connect_clicked(move |_| {
            pop_show.popup();
        });
    }

    pattern_box.append(&pattern_btn);

    let sep1 = gtk4::Separator::new(gtk4::Orientation::Vertical);
    sep1.set_valign(gtk4::Align::Center);
    pattern_box.append(&sep1);

    // 2. Primary & Secondary Colors
    let c1_cell = Rc::new(Cell::new(Color::BLACK));
    let c1_da = gtk4::DrawingArea::builder()
        .content_width(18)
        .content_height(18)
        .build();
    {
        let c1_c = c1_cell.clone();
        c1_da.set_draw_func(move |_, cr, w, h| {
            let col = c1_c.get();
            cr.arc(w as f64 * 0.5, h as f64 * 0.5, (w.min(h) as f64 * 0.5) - 1.0, 0.0, std::f64::consts::TAU);
            cr.set_source_rgba(col.r as f64, col.g as f64, col.b as f64, col.a as f64);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.3);
            cr.set_line_width(1.0);
            let _ = cr.stroke();
        });
    }
    let c1_btn = gtk4::Button::builder()
        .child(&c1_da)
        .css_classes(["flat", "circular"])
        .tooltip_text(&crate::core::gettext("Pattern Foreground Color"))
        .valign(gtk4::Align::Center)
        .build();

    {
        let cv_c1 = canvas.clone();
        let c1_btn_cl = c1_btn.clone();
        let c1_cell_cl = c1_cell.clone();
        let c1_da_cl = c1_da.clone();
        c1_btn.connect_clicked(move |_| {
            let cur = c1_cell_cl.get();
            let cv_apply = cv_c1.clone();
            let c_upd = c1_cell_cl.clone();
            let da_upd = c1_da_cl.clone();
            let pop = ColorPickerPopover::standalone_with_title(
                cv_c1.clone(),
                cur,
                Some(&crate::core::gettext("Pattern Foreground Color")),
            );
            pop.on_color_changed(move |new_col| {
                c_upd.set(new_col);
                da_upd.queue_draw();
                cv_apply.set_pattern_primary_color(new_col);
            });
            pop.attach_to(&c1_btn_cl);
            pop.popup();
        });
    }
    pattern_box.append(&c1_btn);

    let c2_cell = Rc::new(Cell::new(Color::WHITE));
    let c2_da = gtk4::DrawingArea::builder()
        .content_width(18)
        .content_height(18)
        .build();
    {
        let c2_c = c2_cell.clone();
        c2_da.set_draw_func(move |_, cr, w, h| {
            let col = c2_c.get();
            cr.arc(w as f64 * 0.5, h as f64 * 0.5, (w.min(h) as f64 * 0.5) - 1.0, 0.0, std::f64::consts::TAU);
            cr.set_source_rgba(col.r as f64, col.g as f64, col.b as f64, col.a as f64);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.3);
            cr.set_line_width(1.0);
            let _ = cr.stroke();
        });
    }
    let c2_btn = gtk4::Button::builder()
        .child(&c2_da)
        .css_classes(["flat", "circular"])
        .tooltip_text(&crate::core::gettext("Pattern Background Color"))
        .valign(gtk4::Align::Center)
        .build();

    {
        let cv_c2 = canvas.clone();
        let c2_btn_cl = c2_btn.clone();
        let c2_cell_cl = c2_cell.clone();
        let c2_da_cl = c2_da.clone();
        c2_btn.connect_clicked(move |_| {
            let cur = c2_cell_cl.get();
            let cv_apply = cv_c2.clone();
            let c_upd = c2_cell_cl.clone();
            let da_upd = c2_da_cl.clone();
            let pop = ColorPickerPopover::standalone_with_title(
                cv_c2.clone(),
                cur,
                Some(&crate::core::gettext("Pattern Background Color")),
            );
            pop.on_color_changed(move |new_col| {
                c_upd.set(new_col);
                da_upd.queue_draw();
                cv_apply.set_pattern_secondary_color(new_col);
            });
            pop.attach_to(&c2_btn_cl);
            pop.popup();
        });
    }
    pattern_box.append(&c2_btn);

    // 3. Swap Colors Button
    let btn_swap_colors = create_resource_btn(
        "object-flip-horizontal-symbolic",
        &crate::core::gettext("Swap Pattern Colors"),
    );
    {
        let cv_swap = canvas.clone();
        let is_sync_swap = is_syncing.clone();
        let c1_c = c1_cell.clone();
        let c2_c = c2_cell.clone();
        let da1_c = c1_da.clone();
        let da2_c = c2_da.clone();
        btn_swap_colors.connect_clicked(move |_| {
            if is_sync_swap.get() {
                return;
            }
            let col1 = c1_c.get();
            let col2 = c2_c.get();
            c1_c.set(col2);
            c2_c.set(col1);
            da1_c.queue_draw();
            da2_c.queue_draw();
            cv_swap.swap_pattern_colors();
        });
    }
    pattern_box.append(&btn_swap_colors);

    let sep2 = gtk4::Separator::new(gtk4::Orientation::Vertical);
    sep2.set_valign(gtk4::Align::Center);
    pattern_box.append(&sep2);

    // 4. Pattern Tile Size (Scale)
    let scale_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();
    let scale_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Scale:"))
        .css_classes(["caption"])
        .valign(gtk4::Align::Center)
        .build();
    let scale_spin = gtk4::SpinButton::with_range(4.0, 2048.0, 1.0);
    scale_spin.set_value(24.0);
    scale_spin.set_digits(0);
    scale_spin.set_css_classes(&["numeric-spin", "pill-spin"]);
    scale_spin.set_tooltip_text(Some(&crate::core::gettext("Pattern Tile Scale / Size in Pixels")));
    scale_spin.set_valign(gtk4::Align::Center);
    scale_box.append(&scale_lbl);
    scale_box.append(&scale_spin);

    {
        let cv_sc = canvas.clone();
        let is_sync_sc = is_syncing.clone();
        scale_spin.connect_value_changed(move |spin| {
            if is_sync_sc.get() {
                return;
            }
            cv_sc.set_pattern_scale(spin.value() as f32);
        });
    }
    pattern_box.append(&scale_box);

    // 5. Pattern Angle
    let angle_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();
    let angle_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Angle:"))
        .css_classes(["caption"])
        .valign(gtk4::Align::Center)
        .build();
    let angle_spin = gtk4::SpinButton::with_range(0.0, 360.0, 1.0);
    angle_spin.set_value(0.0);
    angle_spin.set_digits(0);
    angle_spin.set_css_classes(&["numeric-spin", "pill-spin"]);
    angle_spin.set_tooltip_text(Some(&crate::core::gettext("Pattern Rotation Angle in Degrees")));
    angle_spin.set_valign(gtk4::Align::Center);
    angle_box.append(&angle_lbl);
    angle_box.append(&angle_spin);

    {
        let cv_ang = canvas.clone();
        let is_sync_ang = is_syncing.clone();
        angle_spin.connect_value_changed(move |spin| {
            if is_sync_ang.get() {
                return;
            }
            cv_ang.set_pattern_angle(spin.value() as f32);
        });
    }
    pattern_box.append(&angle_box);

    let sep3 = gtk4::Separator::new(gtk4::Orientation::Vertical);
    sep3.set_valign(gtk4::Align::Center);
    pattern_box.append(&sep3);

    // 6. Pattern Offset X & Y
    let ox_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();
    let ox_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("X:"))
        .css_classes(["caption"])
        .valign(gtk4::Align::Center)
        .build();
    let offset_x_spin = gtk4::SpinButton::with_range(-10000.0, 10000.0, 1.0);
    offset_x_spin.set_value(0.0);
    offset_x_spin.set_digits(0);
    offset_x_spin.set_css_classes(&["numeric-spin", "pill-spin"]);
    offset_x_spin.set_tooltip_text(Some(&crate::core::gettext("Pattern Horizontal Offset")));
    offset_x_spin.set_valign(gtk4::Align::Center);
    ox_box.append(&ox_lbl);
    ox_box.append(&offset_x_spin);

    {
        let cv_ox = canvas.clone();
        let is_sync_ox = is_syncing.clone();
        offset_x_spin.connect_value_changed(move |spin| {
            if is_sync_ox.get() {
                return;
            }
            let cur = cv_ox.get_active_pattern_info().map(|(_, _, _, _, _, off, _)| off).unwrap_or(Point::ZERO);
            cv_ox.set_pattern_offset(Point::new(spin.value() as f32, cur.y));
        });
    }
    pattern_box.append(&ox_box);

    let oy_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();
    let oy_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Y:"))
        .css_classes(["caption"])
        .valign(gtk4::Align::Center)
        .build();
    let offset_y_spin = gtk4::SpinButton::with_range(-10000.0, 10000.0, 1.0);
    offset_y_spin.set_value(0.0);
    offset_y_spin.set_digits(0);
    offset_y_spin.set_css_classes(&["numeric-spin", "pill-spin"]);
    offset_y_spin.set_tooltip_text(Some(&crate::core::gettext("Pattern Vertical Offset")));
    offset_y_spin.set_valign(gtk4::Align::Center);
    oy_box.append(&oy_lbl);
    oy_box.append(&offset_y_spin);

    {
        let cv_oy = canvas.clone();
        let is_sync_oy = is_syncing.clone();
        offset_y_spin.connect_value_changed(move |spin| {
            if is_sync_oy.get() {
                return;
            }
            let cur = cv_oy.get_active_pattern_info().map(|(_, _, _, _, _, off, _)| off).unwrap_or(Point::ZERO);
            cv_oy.set_pattern_offset(Point::new(cur.x, spin.value() as f32));
        });
    }
    pattern_box.append(&oy_box);

    // 7. Reset Transform
    let btn_reset = create_resource_btn(
        "view-refresh-symbolic",
        &crate::core::gettext("Reset Pattern Transform & Alignment"),
    );
    {
        let cv_res = canvas.clone();
        let is_sync_res = is_syncing.clone();
        let sc_spin = scale_spin.clone();
        let an_spin = angle_spin.clone();
        let ox_sp = offset_x_spin.clone();
        let oy_sp = offset_y_spin.clone();
        btn_reset.connect_clicked(move |_| {
            if is_sync_res.get() {
                return;
            }
            cv_res.reset_pattern_transform();
            is_sync_res.set(true);
            sc_spin.set_value(24.0);
            an_spin.set_value(0.0);
            ox_sp.set_value(0.0);
            oy_sp.set_value(0.0);
            is_sync_res.set(false);
        });
    }
    pattern_box.append(&btn_reset);

    PatternControls {
        pattern_box,
        pattern_type_lbl,
        pattern_type_da,
        pattern_type_cell,
        custom_path_cell,
        scale_spin,
        angle_spin,
        offset_x_spin,
        offset_y_spin,
        c1_da,
        c1_cell,
        c2_da,
        c2_cell,
    }
}
