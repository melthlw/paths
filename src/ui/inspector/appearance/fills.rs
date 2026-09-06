use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::super::swatch::{create_gradient_ramp_button, create_swatch_button};
use crate::core::{Color, FillLayer, FillStyle, PatternType};
use crate::ui::canvas::CanvasWidget;
use crate::ui::color_picker::ColorPickerPopover;

pub struct FillRow {
    pub container: gtk4::Box,
}

impl FillRow {
    pub fn new(
        entry: &FillLayer,
        canvas: CanvasWidget,
        is_updating: Rc<Cell<bool>>,
        list_ref: Rc<RefCell<Vec<FillLayer>>>,
        idx: usize,
        rebuild_cb: Rc<dyn Fn()>,
    ) -> Self {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let popovers_to_cleanup: Rc<RefCell<Vec<gtk4::Popover>>> =
            Rc::new(RefCell::new(Vec::new()));

        let line1 = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .build();

        let drag_handle = gtk4::Image::builder()
            .icon_name("drag-handle-symbolic")
            .opacity(0.35)
            .valign(gtk4::Align::Center)
            .build();
        line1.append(&drag_handle);

        let style_label = match entry.style {
            FillStyle::Solid => crate::core::gettext("Solid"),
            FillStyle::LinearGradient => crate::core::gettext("Linear"),
            FillStyle::RadialGradient => crate::core::gettext("Radial"),
            FillStyle::Mesh => crate::core::gettext("Mesh"),
            FillStyle::Pattern => crate::core::gettext("Pattern"),
        };
        let style_btn = gtk4::Button::builder()
            .label(style_label)
            .css_classes(["pill-btn"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Fill Style"))
            .hexpand(entry.style != FillStyle::Solid)
            .build();
        line1.append(&style_btn);

        let style_popover = gtk4::Popover::builder().has_arrow(true).build();
        popovers_to_cleanup.borrow_mut().push(style_popover.clone());
        let style_menu = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .width_request(170)
            .build();

        let fill_styles = [
            (
                FillStyle::Solid,
                crate::core::gettext("Solid"),
                "tool-paint-bucket-symbolic",
            ),
            (
                FillStyle::LinearGradient,
                crate::core::gettext("Linear Gradient"),
                "tool-gradient-symbolic",
            ),
            (
                FillStyle::RadialGradient,
                crate::core::gettext("Radial Gradient"),
                "tool-gradient-radial-symbolic",
            ),
            (
                FillStyle::Mesh,
                crate::core::gettext("Mesh Gradient"),
                "tool-mesh-symbolic",
            ),
            (
                FillStyle::Pattern,
                crate::core::gettext("Geometric Pattern"),
                "tool-pattern-symbolic",
            ),
        ];

        for (st, st_lbl, st_icon) in fill_styles {
            let item_btn = gtk4::Button::builder().css_classes(["flat"]).build();
            let item_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(8)
                .margin_start(6)
                .margin_end(6)
                .margin_top(4)
                .margin_bottom(4)
                .valign(gtk4::Align::Center)
                .build();
            let item_img = crate::ui::icons::make_symbolic_image(st_icon, 18);
            let item_label = gtk4::Label::builder()
                .label(st_lbl)
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .build();
            item_box.append(&item_img);
            item_box.append(&item_label);
            item_btn.set_child(Some(&item_box));

            let list = list_ref.clone();
            let canvas_s = canvas.clone();
            let pop_c = style_popover.clone();
            let rebuild = rebuild_cb.clone();
            item_btn.connect_clicked(move |_| {
                let Ok(mut l) = list.try_borrow_mut() else {
                    pop_c.popdown();
                    return;
                };
                if let Some(e) = l.get_mut(idx) {
                    e.style = st;
                    match st {
                        FillStyle::Solid => {
                            e.mesh = None;
                            e.stops.clear();
                            e.custom_pattern_path = None;
                        }
                        FillStyle::LinearGradient => {
                            e.mesh = None;
                            e.custom_pattern_path = None;
                            if e.stops.len() < 2 {
                                e.stops = vec![
                                    crate::core::GradientStop::new(0.0, e.color),
                                    crate::core::GradientStop::new(1.0, e.secondary_color),
                                ];
                            }
                            canvas_s.set_active_tool("gradient");
                        }
                        FillStyle::RadialGradient => {
                            e.mesh = None;
                            e.custom_pattern_path = None;
                            if e.stops.len() < 2 {
                                e.stops = vec![
                                    crate::core::GradientStop::new(0.0, e.color),
                                    crate::core::GradientStop::new(1.0, e.secondary_color),
                                ];
                            }
                            canvas_s.set_active_tool("gradient");
                        }
                        FillStyle::Mesh => {
                            e.custom_pattern_path = None;
                            e.stops.clear();
                            drop(l);
                            canvas_s.reset_selected_mesh_grid(3, 3, None, None);
                            canvas_s.set_active_tool("mesh_gradient");
                            pop_c.popdown();
                            rebuild();
                            return;
                        }
                        FillStyle::Pattern => {
                            e.mesh = None;
                            e.stops.clear();
                            if e.pattern_scale <= 0.0 {
                                e.pattern_scale = 20.0;
                            }
                            canvas_s.set_active_tool("pattern");
                        }
                    }
                }
                let cloned = l.clone();
                drop(l);
                canvas_s.set_selected_fills(cloned);
                pop_c.popdown();
                rebuild();
            });
            style_menu.append(&item_btn);
        }

        style_popover.set_child(Some(&style_menu));
        style_popover.set_parent(&style_btn);
        let pop_open = style_popover.clone();
        style_btn.connect_clicked(move |_| {
            pop_open.popup();
        });

        // Style-specific widgets on Line 1 & Line 2
        match entry.style {
            FillStyle::Solid => {
                let (color_btn1, swatch1_area, col1_cell) = create_swatch_button(entry.color);
                let picker1 = ColorPickerPopover::standalone_with_title(
                    canvas.clone(),
                    entry.color,
                    Some(&crate::core::gettext("Solid Color")),
                );
                popovers_to_cleanup.borrow_mut().push(picker1.popover().clone());
                picker1.attach_to(&color_btn1);
                let p1_open = picker1.clone();
                color_btn1.connect_clicked(move |_| {
                    p1_open.popup();
                });

                let hex_entry1 = gtk4::Entry::builder()
                    .text(entry.color.to_hex())
                    .width_chars(8)
                    .max_width_chars(9)
                    .css_classes(["numeric", "pill-entry"])
                    .valign(gtk4::Align::Center)
                    .hexpand(true)
                    .build();

                line1.append(&color_btn1);
                line1.append(&hex_entry1);

                // Wire Color 1
                {
                    let list = list_ref.clone();
                    let canvas = canvas.clone();
                    let u = is_updating.clone();
                    let hex = hex_entry1.clone();
                    let col1_c = col1_cell.clone();
                    let swatch1_a = swatch1_area.clone();
                    picker1.on_color_changed(move |col| {
                        col1_c.set(col);
                        swatch1_a.queue_draw();
                        u.set(true);
                        hex.set_text(&col.to_hex());
                        let mut l = list.borrow_mut();
                        if let Some(e) = l.get_mut(idx) {
                            e.color = col;
                        }
                        let cloned = l.clone();
                        drop(l);
                        canvas.set_selected_fills(cloned);
                        u.set(false);
                    });
                }
                {
                    let list = list_ref.clone();
                    let canvas = canvas.clone();
                    let u = is_updating.clone();
                    let pick1 = picker1.clone();
                    let col1_c = col1_cell.clone();
                    let swatch1_a = swatch1_area.clone();
                    hex_entry1.connect_activate(move |entry| {
                        if u.get() {
                            return;
                        }
                        let text = entry.text();
                        let hex_str = text.trim();
                        if let Some(col) = Color::from_hex(hex_str) {
                            u.set(true);
                            pick1.set_color(col);
                            col1_c.set(col);
                            swatch1_a.queue_draw();
                            let mut l = list.borrow_mut();
                            if let Some(e) = l.get_mut(idx) {
                                e.color = col;
                            }
                            let cloned = l.clone();
                            drop(l);
                            canvas.set_selected_fills(cloned);
                            u.set(false);
                        }
                    });
                }
            }
            FillStyle::LinearGradient | FillStyle::RadialGradient => {
                let is_radial = entry.style == FillStyle::RadialGradient;
                if !is_radial {
                    let angle_btn = gtk4::Button::builder()
                        .label(format!("{}°", entry.angle.round() as i32))
                        .css_classes(["pill-btn", "numeric"])
                        .valign(gtk4::Align::Center)
                        .tooltip_text(crate::core::gettext("Angle"))
                        .build();

                    let angle_popover = gtk4::Popover::builder().has_arrow(true).build();
                    popovers_to_cleanup.borrow_mut().push(angle_popover.clone());
                    let angle_menu = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Vertical)
                        .spacing(4)
                        .margin_top(6)
                        .margin_bottom(6)
                        .margin_start(6)
                        .margin_end(6)
                        .width_request(140)
                        .build();

                    let angle_presets = [
                        (0.0f32, crate::core::gettext("0° (Horizontal →)")),
                        (45.0f32, crate::core::gettext("45° (Diagonal ↘)")),
                        (90.0f32, crate::core::gettext("90° (Vertical ↓)")),
                        (135.0f32, crate::core::gettext("135° (Diagonal ↙)")),
                        (180.0f32, crate::core::gettext("180° (Horizontal ←)")),
                        (270.0f32, crate::core::gettext("270° (Vertical ↑)")),
                    ];

                    for (ang, ang_lbl) in angle_presets {
                        let p_btn = gtk4::Button::builder()
                            .label(ang_lbl)
                            .css_classes(["flat"])
                            .halign(gtk4::Align::Fill)
                            .build();

                        let list = list_ref.clone();
                        let canvas_a = canvas.clone();
                        let ap_c = angle_popover.clone();
                        let rebuild = rebuild_cb.clone();
                        p_btn.connect_clicked(move |_| {
                            let mut l = list.borrow_mut();
                            if let Some(e) = l.get_mut(idx) {
                                e.angle = ang;
                            }
                            let cloned = l.clone();
                            drop(l);
                            canvas_a.set_selected_fills(cloned);
                            ap_c.popdown();
                            rebuild();
                        });
                        angle_menu.append(&p_btn);
                    }

                    angle_popover.set_child(Some(&angle_menu));
                    angle_popover.set_parent(&angle_btn);

                    let ap_open = angle_popover.clone();
                    angle_btn.connect_clicked(move |_| {
                        ap_open.popup();
                    });
                    line1.append(&angle_btn);
                }
            }
            FillStyle::Pattern => {
                let cur_label = if let Some(ref cp) = entry.custom_pattern_path {
                    std::path::Path::new(cp)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.replace(['-', '_'], " "))
                        .unwrap_or_else(|| entry.pattern_type.label())
                } else {
                    entry.pattern_type.label()
                };
                let pat_btn = gtk4::Button::builder()
                    .label(&cur_label)
                    .css_classes(["pill-btn"])
                    .valign(gtk4::Align::Center)
                    .tooltip_text(crate::core::gettext("Pattern Type"))
                    .build();

                let pat_pop = gtk4::Popover::builder().has_arrow(true).build();
                popovers_to_cleanup.borrow_mut().push(pat_pop.clone());

                let scrolled_window = gtk4::ScrolledWindow::builder()
                    .hscrollbar_policy(gtk4::PolicyType::Never)
                    .vscrollbar_policy(gtk4::PolicyType::Automatic)
                    .propagate_natural_height(true)
                    .max_content_height(380)
                    .min_content_height(180)
                    .width_request(240)
                    .build();

                let pat_menu = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(8)
                    .margin_top(8)
                    .margin_bottom(8)
                    .margin_start(8)
                    .margin_end(8)
                    .build();

                let pat_options = [
                    (PatternType::Checkerboard, crate::core::gettext("Checkerboard")),
                    (PatternType::Dots, crate::core::gettext("Dots")),
                    (PatternType::Stripes, crate::core::gettext("Stripes")),
                    (PatternType::Grid, crate::core::gettext("Technical Grid")),
                    (PatternType::Hexagon, crate::core::gettext("Honeycomb")),
                    (PatternType::Crosshatch, crate::core::gettext("Crosshatch")),
                    (PatternType::Brick, crate::core::gettext("Brick Wall")),
                    (PatternType::Scales, crate::core::gettext("Seigaiha Scales")),
                    (PatternType::Houndstooth, crate::core::gettext("Houndstooth")),
                    (PatternType::Basketweave, crate::core::gettext("Basketweave")),
                ];

                let sec1_lbl = gtk4::Label::builder()
                    .label(&crate::core::gettext("Built-in Patterns"))
                    .css_classes(["dim-label", "caption", "heading"])
                    .halign(gtk4::Align::Start)
                    .margin_start(4)
                    .build();
                pat_menu.append(&sec1_lbl);

                let builtin_flow = gtk4::FlowBox::builder()
                    .selection_mode(gtk4::SelectionMode::None)
                    .max_children_per_line(2)
                    .min_children_per_line(2)
                    .homogeneous(true)
                    .row_spacing(6)
                    .column_spacing(6)
                    .build();

                for (pt, pt_lbl) in pat_options {
                    let is_active = entry.pattern_type == pt && entry.custom_pattern_path.is_none();
                    let list = list_ref.clone();
                    let canvas_p = canvas.clone();
                    let pp_c = pat_pop.clone();
                    let rebuild = rebuild_cb.clone();
                    let tile = create_pattern_preview_tile(
                        pt,
                        None,
                        &pt_lbl,
                        is_active,
                        move || {
                            let mut l = list.borrow_mut();
                            if let Some(e) = l.get_mut(idx) {
                                e.pattern_type = pt;
                                e.custom_pattern_path = None;
                            }
                            let cloned = l.clone();
                            drop(l);
                            canvas_p.set_selected_fills(cloned);
                            pp_c.popdown();
                            rebuild();
                        },
                    );
                    builtin_flow.append(&tile);
                }
                pat_menu.append(&builtin_flow);

                let user_patterns = crate::core::scan_user_patterns();
                if !user_patterns.is_empty() {
                    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
                    sep.set_margin_top(4);
                    sep.set_margin_bottom(4);
                    pat_menu.append(&sep);

                    let sec2_lbl = gtk4::Label::builder()
                        .label(&crate::core::gettext("Custom Patterns"))
                        .css_classes(["dim-label", "caption", "heading"])
                        .halign(gtk4::Align::Start)
                        .margin_start(4)
                        .build();
                    pat_menu.append(&sec2_lbl);

                    let custom_flow = gtk4::FlowBox::builder()
                        .selection_mode(gtk4::SelectionMode::None)
                        .max_children_per_line(2)
                        .min_children_per_line(2)
                        .homogeneous(true)
                        .row_spacing(6)
                        .column_spacing(6)
                        .build();

                    for cp in user_patterns {
                        let cp_name = cp.name.clone();
                        let cp_path = cp.file_path.clone();
                        let is_active = entry.pattern_type == PatternType::Custom
                            && entry.custom_pattern_path.as_deref() == Some(&cp_path);

                        let list = list_ref.clone();
                        let canvas_p = canvas.clone();
                        let pp_c = pat_pop.clone();
                        let rebuild = rebuild_cb.clone();
                        let cp_path_c = cp_path.clone();

                        let tile = create_pattern_preview_tile(
                            PatternType::Custom,
                            Some(cp_path),
                            &cp_name,
                            is_active,
                            move || {
                                let mut l = list.borrow_mut();
                                if let Some(e) = l.get_mut(idx) {
                                    e.pattern_type = PatternType::Custom;
                                    e.custom_pattern_path = Some(cp_path_c.clone());
                                }
                                let cloned = l.clone();
                                drop(l);
                                canvas_p.set_selected_fills(cloned);
                                pp_c.popdown();
                                rebuild();
                            },
                        );
                        custom_flow.append(&tile);
                    }
                    pat_menu.append(&custom_flow);
                }

                let sep2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
                sep2.set_margin_top(4);
                sep2.set_margin_bottom(4);
                pat_menu.append(&sep2);

                let action_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(6)
                    .halign(gtk4::Align::Fill)
                    .build();
                let edit_gizmo_btn = gtk4::Button::builder()
                    .label(&crate::core::gettext("Gizmo Tool"))
                    .icon_name("transform-move-pattern-symbolic")
                    .css_classes(["flat", "pill-btn"])
                    .hexpand(true)
                    .tooltip_text(&crate::core::gettext("Edit Pattern Handles on Canvas"))
                    .build();
                let canvas_gz = canvas.clone();
                let pp_c2 = pat_pop.clone();
                edit_gizmo_btn.connect_clicked(move |_| {
                    canvas_gz.set_active_tool("pattern");
                    pp_c2.popdown();
                });
                action_box.append(&edit_gizmo_btn);
                pat_menu.append(&action_box);

                scrolled_window.set_child(Some(&pat_menu));
                pat_pop.set_child(Some(&scrolled_window));
                pat_pop.set_parent(&pat_btn);

                let pop_c = pat_pop.clone();
                pat_btn.connect_clicked(move |_| {
                    pop_c.popup();
                });

                let pat_tool_btn = gtk4::Button::builder()
                    .icon_name("transform-move-pattern-symbolic")
                    .css_classes(["pill-btn"])
                    .valign(gtk4::Align::Center)
                    .tooltip_text(crate::core::gettext("Edit Pattern Gizmo on Canvas"))
                    .build();
                let canvas_pt = canvas.clone();
                pat_tool_btn.connect_clicked(move |_| {
                    canvas_pt.set_active_tool("pattern");
                });

                let angle_btn = gtk4::Button::builder()
                    .label(format!("{}°", entry.angle.round() as i32))
                    .css_classes(["pill-btn", "numeric"])
                    .valign(gtk4::Align::Center)
                    .tooltip_text(crate::core::gettext("Angle"))
                    .build();

                let angle_popover = gtk4::Popover::builder().has_arrow(true).build();
                popovers_to_cleanup.borrow_mut().push(angle_popover.clone());
                let angle_menu = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(4)
                    .margin_top(6)
                    .margin_bottom(6)
                    .margin_start(6)
                    .margin_end(6)
                    .width_request(140)
                    .build();

                let angle_presets = [
                    (0.0f32, crate::core::gettext("0° (Horizontal →)")),
                    (45.0f32, crate::core::gettext("45° (Diagonal ↘)")),
                    (90.0f32, crate::core::gettext("90° (Vertical ↓)")),
                    (135.0f32, crate::core::gettext("135° (Diagonal ↙)")),
                    (180.0f32, crate::core::gettext("180° (Horizontal ←)")),
                    (270.0f32, crate::core::gettext("270° (Vertical ↑)")),
                ];

                for (ang, ang_lbl) in angle_presets {
                    let p_btn = gtk4::Button::builder()
                        .label(ang_lbl)
                        .css_classes(["flat"])
                        .halign(gtk4::Align::Fill)
                        .build();

                    let list = list_ref.clone();
                    let canvas_a = canvas.clone();
                    let ap_c = angle_popover.clone();
                    let rebuild = rebuild_cb.clone();
                    p_btn.connect_clicked(move |_| {
                        let mut l = list.borrow_mut();
                        if let Some(e) = l.get_mut(idx) {
                            e.angle = ang;
                        }
                        let cloned = l.clone();
                        drop(l);
                        canvas_a.set_selected_fills(cloned);
                        ap_c.popdown();
                        rebuild();
                    });
                    angle_menu.append(&p_btn);
                }

                angle_popover.set_child(Some(&angle_menu));
                angle_popover.set_parent(&angle_btn);

                let ap_open = angle_popover.clone();
                angle_btn.connect_clicked(move |_| {
                    ap_open.popup();
                });

                line1.append(&pat_btn);
                line1.append(&pat_tool_btn);
                line1.append(&angle_btn);
            }
            FillStyle::Mesh => {
                let mesh_tool_btn = gtk4::Button::builder()
                    .label(crate::core::gettext("Mesh Tool"))
                    .icon_name("tool-mesh-symbolic")
                    .css_classes(["pill-btn"])
                    .valign(gtk4::Align::Center)
                    .tooltip_text(crate::core::gettext("Edit Mesh Nodes on Canvas (Mesh Tool)"))
                    .build();

                let canvas_m = canvas.clone();
                mesh_tool_btn.connect_clicked(move |_| {
                    canvas_m.set_active_tool("mesh_gradient");
                });
                line1.append(&mesh_tool_btn);
            }
        }

        let op_pct = (entry.opacity * 100.0).round() as i32;
        let op_entry = gtk4::Entry::builder()
            .text(format!("{}%", op_pct))
            .width_chars(5)
            .max_width_chars(5)
            .css_classes(["numeric", "pill-entry"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Opacity"))
            .hexpand(false)
            .build();
        line1.append(&op_entry);

        let eye_icon_name = if entry.enabled {
            "view-reveal-symbolic"
        } else {
            "view-conceal-symbolic"
        };
        let fill_vis_tip = if entry.enabled {
            crate::core::gettext("Hide fill")
        } else {
            crate::core::gettext("Show fill")
        };
        let vis_btn = gtk4::Button::builder()
            .icon_name(eye_icon_name)
            .css_classes(["flat", "circular"])
            .valign(gtk4::Align::Center)
            .tooltip_text(fill_vis_tip)
            .opacity(if entry.enabled { 1.0 } else { 0.5 })
            .build();
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let rebuild = rebuild_cb.clone();
            vis_btn.connect_clicked(move |_| {
                let mut l = list.borrow_mut();
                if let Some(e) = l.get_mut(idx) {
                    e.enabled = !e.enabled;
                }
                let cloned = l.clone();
                drop(l);
                canvas.set_selected_fills(cloned);
                rebuild();
            });
        }
        line1.append(&vis_btn);

        let del_btn = gtk4::Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(["flat", "circular"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Remove fill"))
            .build();
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let rebuild = rebuild_cb.clone();
            del_btn.connect_clicked(move |_| {
                let mut l = list.borrow_mut();
                if idx < l.len() {
                    l.remove(idx);
                }
                let cloned = l.clone();
                drop(l);
                canvas.set_selected_fills(cloned);
                rebuild();
            });
        }
        line1.append(&del_btn);

        container.append(&line1);

        // Line 2 for multi-control fill styles (gradients, patterns, mesh)
        match entry.style {
            FillStyle::LinearGradient | FillStyle::RadialGradient => {
                let is_radial = entry.style == FillStyle::RadialGradient;
                let line2 = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(6)
                    .valign(gtk4::Align::Center)
                    .hexpand(true)
                    .build();

                let (grad_btn, _grad_da) = create_gradient_ramp_button(&entry.effective_stops(), is_radial);
                let grad_picker = ColorPickerPopover::with_mode_switcher(canvas.clone(), entry.color, 1, false);
                popovers_to_cleanup.borrow_mut().push(grad_picker.popover().clone());
                grad_picker.attach_to(&grad_btn);

                let gp_open = grad_picker.clone();
                grad_btn.connect_clicked(move |_| {
                    gp_open.popup();
                });

                line2.append(&grad_btn);
                container.append(&line2);
            }
            FillStyle::Pattern => {
                let (color_btn1, swatch1_area, col1_cell) = create_swatch_button(entry.color);
                let picker1 = ColorPickerPopover::standalone_with_title(
                    canvas.clone(),
                    entry.color,
                    Some(&crate::core::gettext("Pattern Color 1")),
                );
                popovers_to_cleanup.borrow_mut().push(picker1.popover().clone());
                picker1.attach_to(&color_btn1);
                let p1_open = picker1.clone();
                color_btn1.connect_clicked(move |_| {
                    p1_open.popup();
                });

                let hex_entry1 = gtk4::Entry::builder()
                    .text(entry.color.to_hex())
                    .width_chars(8)
                    .max_width_chars(9)
                    .css_classes(["numeric", "pill-entry"])
                    .valign(gtk4::Align::Center)
                    .hexpand(true)
                    .build();

                let (color_btn2, swatch2_area, col2_cell) = create_swatch_button(entry.secondary_color);
                let picker2 = ColorPickerPopover::standalone_with_title(
                    canvas.clone(),
                    entry.secondary_color,
                    Some(&crate::core::gettext("Pattern Color 2")),
                );
                popovers_to_cleanup.borrow_mut().push(picker2.popover().clone());
                picker2.attach_to(&color_btn2);
                let p2_open = picker2.clone();
                color_btn2.connect_clicked(move |_| {
                    p2_open.popup();
                });

                let hex_entry2 = gtk4::Entry::builder()
                    .text(entry.secondary_color.to_hex())
                    .width_chars(8)
                    .max_width_chars(9)
                    .css_classes(["numeric", "pill-entry"])
                    .valign(gtk4::Align::Center)
                    .hexpand(true)
                    .build();

                let line2 = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(6)
                    .valign(gtk4::Align::Center)
                    .build();

                line2.append(&color_btn1);
                line2.append(&hex_entry1);
                line2.append(&color_btn2);
                line2.append(&hex_entry2);
                container.append(&line2);

                // Wire controls for Pattern Color 1
                {
                    let list = list_ref.clone();
                    let canvas = canvas.clone();
                    let u = is_updating.clone();
                    let hex = hex_entry1.clone();
                    let col1_c = col1_cell.clone();
                    let swatch1_a = swatch1_area.clone();
                    picker1.on_color_changed(move |col| {
                        col1_c.set(col);
                        swatch1_a.queue_draw();
                        u.set(true);
                        hex.set_text(&col.to_hex());
                        let mut l = list.borrow_mut();
                        if let Some(e) = l.get_mut(idx) {
                            e.color = col;
                        }
                        let cloned = l.clone();
                        drop(l);
                        canvas.set_selected_fills(cloned);
                        u.set(false);
                    });
                }
                {
                    let list = list_ref.clone();
                    let canvas = canvas.clone();
                    let u = is_updating.clone();
                    let pick1 = picker1.clone();
                    let col1_c = col1_cell.clone();
                    let swatch1_a = swatch1_area.clone();
                    hex_entry1.connect_activate(move |entry| {
                        if u.get() {
                            return;
                        }
                        let text = entry.text();
                        let hex_str = text.trim();
                        if let Some(col) = Color::from_hex(hex_str) {
                            u.set(true);
                            pick1.set_color(col);
                            col1_c.set(col);
                            swatch1_a.queue_draw();
                            let mut l = list.borrow_mut();
                            if let Some(e) = l.get_mut(idx) {
                                e.color = col;
                            }
                            let cloned = l.clone();
                            drop(l);
                            canvas.set_selected_fills(cloned);
                            u.set(false);
                        }
                    });
                }

                // Wire controls for Pattern Color 2
                {
                    let list = list_ref.clone();
                    let canvas = canvas.clone();
                    let u = is_updating.clone();
                    let hex = hex_entry2.clone();
                    let col2_cell_c = col2_cell.clone();
                    let swatch2_area_c = swatch2_area.clone();
                    picker2.on_color_changed(move |col| {
                        col2_cell_c.set(col);
                        swatch2_area_c.queue_draw();
                        u.set(true);
                        hex.set_text(&col.to_hex());
                        let mut l = list.borrow_mut();
                        if let Some(e) = l.get_mut(idx) {
                            e.secondary_color = col;
                        }
                        let cloned = l.clone();
                        drop(l);
                        canvas.set_selected_fills(cloned);
                        u.set(false);
                    });
                }
                {
                    let list = list_ref.clone();
                    let canvas = canvas.clone();
                    let u = is_updating.clone();
                    let pick2 = picker2.clone();
                    let col2_cell_c = col2_cell.clone();
                    let swatch2_area_c = swatch2_area.clone();
                    hex_entry2.connect_activate(move |entry| {
                        if u.get() {
                            return;
                        }
                        let text = entry.text();
                        let hex_str = text.trim();
                        if let Some(col) = Color::from_hex(hex_str) {
                            u.set(true);
                            pick2.set_color(col);
                            col2_cell_c.set(col);
                            swatch2_area_c.queue_draw();
                            let mut l = list.borrow_mut();
                            if let Some(e) = l.get_mut(idx) {
                                e.secondary_color = col;
                            }
                            let cloned = l.clone();
                            drop(l);
                            canvas.set_selected_fills(cloned);
                            u.set(false);
                        }
                    });
                }
            }
            FillStyle::Mesh => {
                let line2 = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(6)
                    .valign(gtk4::Align::Center)
                    .hexpand(true)
                    .build();

                let mesh_btn = gtk4::Button::builder()
                    .label(crate::core::gettext("Mesh Gradient Palette"))
                    .css_classes(["pill-btn"])
                    .hexpand(true)
                    .valign(gtk4::Align::Center)
                    .build();

                let mesh_picker = ColorPickerPopover::with_mode_switcher(canvas.clone(), entry.color, 2, false);
                popovers_to_cleanup.borrow_mut().push(mesh_picker.popover().clone());
                mesh_picker.attach_to(&mesh_btn);

                let mp_open = mesh_picker.clone();
                mesh_btn.connect_clicked(move |_| {
                    mp_open.popup();
                });

                line2.append(&mesh_btn);
                container.append(&line2);
            }
            FillStyle::Solid => {}
        }

        // Wire opacity control
        {
            let list = list_ref.clone();
            let canvas = canvas.clone();
            let u = is_updating.clone();
            op_entry.connect_activate(move |entry| {
                if u.get() {
                    return;
                }
                let cur_op = list.borrow().get(idx).map(|e| e.opacity * 100.0).unwrap_or(100.0);
                if let Ok(val) = crate::core::eval_math_expression(
                    entry.text().as_str(),
                    crate::core::Unit::Px,
                    Some(cur_op),
                ) {
                    let alpha = (val / 100.0).clamp(0.0, 1.0);
                    u.set(true);
                    entry.set_text(&format!("{}%", (alpha * 100.0).round() as i32));
                    let mut l = list.borrow_mut();
                    if let Some(e) = l.get_mut(idx) {
                        e.opacity = alpha;
                    }
                    let cloned = l.clone();
                    drop(l);
                    canvas.set_selected_fills(cloned);
                    u.set(false);
                }
            });
        }

        {
            let pop_clean = popovers_to_cleanup.clone();
            let r_cb = rebuild_cb.clone();
            for p in pop_clean.borrow().iter() {
                let r = r_cb.clone();
                p.connect_closed(move |_| {
                    r();
                });
            }
            container.connect_destroy(move |_| {
                for p in pop_clean.borrow().iter() {
                    if p.parent().is_some() {
                        p.unparent();
                    }
                }
            });
        }

        Self { container }
    }
}

pub fn create_pattern_preview_tile(
    pt: PatternType,
    custom_path: Option<String>,
    name: &str,
    is_active: bool,
    on_select: impl Fn() + 'static,
) -> gtk4::Button {
    let tile_btn = gtk4::Button::builder()
        .css_classes(["flat", "pattern-popover-tile"])
        .tooltip_text(name)
        .build();

    if is_active {
        tile_btn.add_css_class("active");
    }

    let vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(3)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let preview_area = gtk4::DrawingArea::builder()
        .width_request(100)
        .height_request(42)
        .build();

    let cp_opt = custom_path;
    preview_area.set_draw_func(move |_, cr, w, h| {
        draw_pattern_preview_cairo(cr, w as f64, h as f64, pt, cp_opt.as_deref());
    });

    let lbl = gtk4::Label::builder()
        .label(name)
        .css_classes(["caption"])
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .max_width_chars(13)
        .halign(gtk4::Align::Center)
        .build();

    vbox.append(&preview_area);
    vbox.append(&lbl);
    tile_btn.set_child(Some(&vbox));

    tile_btn.connect_clicked(move |_| {
        on_select();
    });

    tile_btn
}

pub fn draw_pattern_preview_cairo(
    cr: &gtk4::cairo::Context,
    w: f64,
    h: f64,
    pt: PatternType,
    custom_path: Option<&str>,
) {
    let r = 5.0;
    // 1. Clip to rounded rectangle
    cr.new_sub_path();
    cr.arc(w - r, r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(w - r, h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    cr.arc(r, h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
    cr.arc(r, r, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
    cr.close_path();
    let _ = cr.clip();

    // 2. Base Tile Background
    cr.set_source_rgb(0.95, 0.96, 0.98);
    cr.rectangle(0.0, 0.0, w, h);
    let _ = cr.fill();

    // 3. Draw Pattern Geometry
    let color_accent = (0.21, 0.52, 0.89, 0.90); // GNOME blue

    match pt {
        PatternType::Checkerboard => {
            let step = 9.0;
            let cols = (w / step).ceil() as usize;
            let rows = (h / step).ceil() as usize;
            for cx in 0..cols {
                for ry in 0..rows {
                    if (cx + ry) % 2 == 0 {
                        cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
                        cr.rectangle(cx as f64 * step, ry as f64 * step, step, step);
                        let _ = cr.fill();
                    }
                }
            }
        }
        PatternType::Dots => {
            cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
            let step = 8.5;
            let mut x = 4.5;
            while x < w {
                let mut y = 4.5;
                while y < h {
                    cr.arc(x, y, 2.0, 0.0, std::f64::consts::TAU);
                    cr.close_path();
                    let _ = cr.fill();
                    y += step;
                }
                x += step;
            }
        }
        PatternType::Stripes => {
            cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
            cr.set_line_width(3.2);
            let step = 8.0;
            let mut x = -h;
            while x < w + h {
                cr.move_to(x, 0.0);
                cr.line_to(x + h, h);
                x += step;
            }
            let _ = cr.stroke();
        }
        PatternType::Grid => {
            cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
            cr.set_line_width(1.0);
            let step = 7.5;
            let mut x = step;
            while x < w {
                cr.move_to(x, 0.0);
                cr.line_to(x, h);
                x += step;
            }
            let mut y = step;
            while y < h {
                cr.move_to(0.0, y);
                cr.line_to(w, y);
                y += step;
            }
            let _ = cr.stroke();
        }
        PatternType::Hexagon => {
            cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
            cr.set_line_width(1.0);
            let r_hex = 6.0;
            let w_step = r_hex * 3.0f64.sqrt();
            let h_step = r_hex * 1.5;
            let mut col = 0;
            let mut x = 0.0;
            while x < w + w_step {
                let offset_y = if col % 2 == 1 { h_step * 0.5 } else { 0.0 };
                let mut y = offset_y;
                while y < h + h_step {
                    let cx = x;
                    let cy = y;
                    for i in 0..6 {
                        let angle = std::f64::consts::FRAC_PI_3 * i as f64 + std::f64::consts::FRAC_PI_6;
                        let px = cx + r_hex * angle.cos();
                        let py = cy + r_hex * angle.sin();
                        if i == 0 {
                            cr.move_to(px, py);
                        } else {
                            cr.line_to(px, py);
                        }
                    }
                    cr.close_path();
                    let _ = cr.stroke();
                    y += h_step;
                }
                x += w_step * 0.5;
                col += 1;
            }
        }
        PatternType::Crosshatch => {
            cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
            cr.set_line_width(1.0);
            let step = 7.0;
            let mut x = -h;
            while x < w + h {
                cr.move_to(x, 0.0);
                cr.line_to(x + h, h);
                cr.move_to(x + h, 0.0);
                cr.line_to(x, h);
                x += step;
            }
            let _ = cr.stroke();
        }
        PatternType::Brick => {
            cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
            cr.set_line_width(1.0);
            let bw = 15.0;
            let bh = 7.5;
            let mut row = 0;
            let mut y = 0.0;
            while y < h + bh {
                let offset = if row % 2 == 1 { bw / 2.0 } else { 0.0 };
                cr.move_to(0.0, y);
                cr.line_to(w, y);
                let mut x = offset;
                while x < w + bw {
                    cr.move_to(x, y);
                    cr.line_to(x, y + bh);
                    x += bw;
                }
                y += bh;
                row += 1;
            }
            let _ = cr.stroke();
        }
        PatternType::Scales => {
            cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
            cr.set_line_width(1.0);
            let step = 14.0;
            let mut row = 0;
            let mut y = 0.0;
            while y < h + step {
                let offset = if row % 2 == 1 { step / 2.0 } else { 0.0 };
                let mut x = offset;
                while x < w + step {
                    cr.arc(x, y, 7.0, 0.0, std::f64::consts::PI);
                    let _ = cr.stroke();
                    cr.arc(x, y, 4.5, 0.0, std::f64::consts::PI);
                    let _ = cr.stroke();
                    x += step;
                }
                y += step * 0.5;
                row += 1;
            }
        }
        PatternType::Houndstooth => {
            cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
            let step = 13.0;
            let half = step * 0.5;
            let mut x = 0.0;
            while x < w + step {
                let mut y = 0.0;
                while y < h + step {
                    cr.move_to(x, y);
                    cr.line_to(x + half, y);
                    cr.line_to(x + step, y + half);
                    cr.line_to(x + half, y + half);
                    cr.line_to(x + half, y + step);
                    cr.line_to(x, y + half);
                    cr.close_path();
                    let _ = cr.fill();
                    y += step;
                }
                x += step;
            }
        }
        PatternType::Basketweave => {
            cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
            cr.set_line_width(1.4);
            let step = 14.0;
            let half = step * 0.5;
            let mut x = 0.0;
            while x < w + step {
                let mut y = 0.0;
                while y < h + step {
                    cr.move_to(x, y + half * 0.5);
                    cr.line_to(x + half, y + half * 0.5);
                    cr.move_to(x + half * 0.5, y + half);
                    cr.line_to(x + half * 0.5, y + step);
                    cr.move_to(x + half, y + step * 0.75);
                    cr.line_to(x + step, y + step * 0.75);
                    cr.move_to(x + step * 0.75, y);
                    cr.line_to(x + step * 0.75, y + half);
                    let _ = cr.stroke();
                    y += step;
                }
                x += step;
            }
        }
        PatternType::Custom => {
            // Draw custom SVG or stylized thematic motif based on filename
            let path_str = custom_path.unwrap_or("");
            let file_stem = std::path::Path::new(path_str)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            if file_stem.contains("wave") {
                // Japanese Waves (Traditional Seigaiha) multi-arc ripple motif
                cr.set_source_rgba(0.11, 0.44, 0.85, 0.95);
                cr.set_line_width(1.1);
                let step = 16.0;
                let mut y = 0.0;
                let mut row = 0;
                while y < h + step {
                    let offset = if row % 2 == 1 { step * 0.5 } else { 0.0 };
                    let mut x = offset;
                    while x < w + step {
                        cr.arc(x, y, 7.5, 0.0, std::f64::consts::PI);
                        let _ = cr.stroke();
                        cr.arc(x, y, 5.5, 0.0, std::f64::consts::PI);
                        let _ = cr.stroke();
                        cr.arc(x, y, 3.5, 0.0, std::f64::consts::PI);
                        let _ = cr.stroke();
                        cr.arc(x, y, 1.8, 0.0, std::f64::consts::PI);
                        let _ = cr.stroke();
                        x += step;
                    }
                    y += step * 0.45;
                    row += 1;
                }
            } else if file_stem.contains("carbon") {
                // Carbon Weave motif
                cr.set_source_rgb(0.2, 0.22, 0.26);
                cr.rectangle(0.0, 0.0, w, h);
                let _ = cr.fill();
                cr.set_source_rgba(0.35, 0.38, 0.45, 0.85);
                let step = 8.0;
                let mut x = 0.0;
                while x < w + step {
                    let mut y = 0.0;
                    while y < h + step {
                        cr.rectangle(x, y, 3.5, 7.0);
                        let _ = cr.fill();
                        cr.rectangle(x + 4.0, y + 4.0, 3.5, 7.0);
                        let _ = cr.fill();
                        y += step;
                    }
                    x += step;
                }
            } else if file_stem.contains("star") || file_stem.contains("moroccan") {
                // Moroccan 8-pointed star motif
                cr.set_source_rgba(0.85, 0.45, 0.15, 0.9);
                cr.set_line_width(1.2);
                let cx = w * 0.5;
                let cy = h * 0.5;
                let sz = 12.0;
                cr.rectangle(cx - sz * 0.5, cy - sz * 0.5, sz, sz);
                let _ = cr.stroke();
                let _ = cr.save();
                cr.translate(cx, cy);
                cr.rotate(std::f64::consts::FRAC_PI_4);
                cr.rectangle(-sz * 0.5, -sz * 0.5, sz, sz);
                let _ = cr.stroke();
                let _ = cr.restore();
            } else if file_stem.contains("lattice") || file_stem.contains("geometric") {
                // Geometric Lattice motif
                cr.set_source_rgba(0.65, 0.25, 0.85, 0.9);
                cr.set_line_width(1.1);
                let step = 10.0;
                let mut x = 0.0;
                while x < w + step {
                    let mut y = 0.0;
                    while y < h + step {
                        cr.move_to(x + step * 0.5, y);
                        cr.line_to(x + step, y + step * 0.5);
                        cr.line_to(x + step * 0.5, y + step);
                        cr.line_to(x, y + step * 0.5);
                        cr.close_path();
                        let _ = cr.stroke();
                        y += step;
                    }
                    x += step;
                }
            } else {
                // Generic Custom pattern: clean geometric motif
                cr.set_source_rgba(color_accent.0, color_accent.1, color_accent.2, color_accent.3);
                cr.set_line_width(1.2);
                cr.move_to(w * 0.5, 6.0);
                cr.line_to(w - 12.0, h * 0.5);
                cr.line_to(w * 0.5, h - 6.0);
                cr.line_to(12.0, h * 0.5);
                cr.close_path();
                let _ = cr.stroke();
                cr.arc(w * 0.5, h * 0.5, 3.5, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();
            }
        }
    }

    // 4. Subtle Border Ring
    cr.reset_clip();
    cr.new_sub_path();
    cr.arc(w - r - 0.5, r + 0.5, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(w - r - 0.5, h - r - 0.5, r, 0.0, std::f64::consts::FRAC_PI_2);
    cr.arc(r + 0.5, h - r - 0.5, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
    cr.arc(r + 0.5, r + 0.5, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
    cr.close_path();
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.15);
    cr.set_line_width(1.0);
    let _ = cr.stroke();
}
