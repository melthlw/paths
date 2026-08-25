use gtk4::gdk;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::canvas::CanvasWidget;
use crate::core::{Color, GradientStop};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorPickerTarget {
    #[default]
    Fill,
    Stroke,
    Standalone,
}

#[derive(Clone)]
pub struct ColorPickerPopover {
    popover: gtk4::Popover,
    color: Rc<Cell<Color>>,
    hue: Rc<Cell<f32>>,
    sat: Rc<Cell<f32>>,
    val: Rc<Cell<f32>>,
    alpha: Rc<Cell<f32>>,
    current_mode: Rc<Cell<usize>>,
    active_grad_stop: Rc<Cell<usize>>,
    active_mesh_node: Rc<Cell<usize>>,
    on_change: Rc<RefCell<Option<Box<dyn Fn(Color)>>>>,
    on_mode_change: Rc<RefCell<Option<Box<dyn Fn(usize)>>>>,
    sv_area: gtk4::DrawingArea,
    hue_area: gtk4::DrawingArea,
    alpha_area: gtk4::DrawingArea,
    hex_entry: gtk4::Entry,
    is_updating: Rc<Cell<bool>>,

    mode_buttons: Vec<gtk4::Button>,
    title_lbl: gtk4::Label,
    solid_panel: gtk4::Box,
    gradient_panel: gtk4::Box,
    mesh_panel: gtk4::Box,
    pattern_panel: gtk4::Box,

    grad_track_da: gtk4::DrawingArea,
    grad_stops_ref: Rc<RefCell<Vec<GradientStop>>>,
    pat_c1_da: gtk4::DrawingArea,
    pat_c2_da: gtk4::DrawingArea,

    canvas: CanvasWidget,
    target: ColorPickerTarget,
}

fn draw_rounded_rect(cr: &cairo::Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
    let r = radius.min(width / 2.0).min(height / 2.0);
    cr.new_sub_path();
    cr.arc(x + width - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(
        x + width - r,
        y + height - r,
        r,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    cr.arc(
        x + r,
        y + height - r,
        r,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    cr.arc(
        x + r,
        y + r,
        r,
        std::f64::consts::PI,
        3.0 * std::f64::consts::FRAC_PI_2,
    );
    cr.close_path();
}

fn create_swatch_da(col_cell: Rc<Cell<Color>>, size: i32) -> gtk4::DrawingArea {
    let da = gtk4::DrawingArea::builder()
        .content_width(size)
        .content_height(size)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .build();
    da.set_draw_func(move |_, cr, w, h| {
        let c = col_cell.get();
        let r = (w.min(h) as f64 * 0.5) - 1.0;
        cr.arc(w as f64 * 0.5, h as f64 * 0.5, r, 0.0, std::f64::consts::TAU);
        cr.set_source_rgba(c.r as f64, c.g as f64, c.b as f64, c.a as f64);
        let _ = cr.fill_preserve();
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.3);
        cr.set_line_width(1.0);
        let _ = cr.stroke();
    });
    da
}

fn interpolate_stops(stops: &[GradientStop], t: f32) -> Color {
    if stops.is_empty() {
        return Color::WHITE;
    }
    if stops.len() == 1 || t <= stops[0].offset {
        return stops[0].color;
    }
    if t >= stops.last().unwrap().offset {
        return stops.last().unwrap().color;
    }
    for i in 0..stops.len() - 1 {
        let s0 = &stops[i];
        let s1 = &stops[i + 1];
        if t >= s0.offset && t <= s1.offset {
            let span = (s1.offset - s0.offset).max(0.001);
            let local_t = (t - s0.offset) / span;
            return Color::new(
                s0.color.r + local_t * (s1.color.r - s0.color.r),
                s0.color.g + local_t * (s1.color.g - s0.color.g),
                s0.color.b + local_t * (s1.color.b - s0.color.b),
                s0.color.a + local_t * (s1.color.a - s0.color.a),
            );
        }
    }
    stops.last().unwrap().color
}

impl ColorPickerPopover {
    pub fn new(canvas: CanvasWidget, initial_color: Color, initial_mode: usize) -> Self {
        Self::with_target(canvas, initial_color, initial_mode, true, ColorPickerTarget::Fill)
    }

    pub fn with_mode_switcher(
        canvas: CanvasWidget,
        initial_color: Color,
        initial_mode: usize,
        show_mode_switcher: bool,
    ) -> Self {
        Self::with_target(canvas, initial_color, initial_mode, show_mode_switcher, ColorPickerTarget::Fill)
    }

    pub fn for_stroke(canvas: CanvasWidget, initial_color: Color) -> Self {
        Self::with_target(canvas, initial_color, 0, false, ColorPickerTarget::Stroke)
    }

    pub fn standalone(canvas: CanvasWidget, initial_color: Color) -> Self {
        Self::with_target(canvas, initial_color, 0, false, ColorPickerTarget::Standalone)
    }

    pub fn with_target(
        canvas: CanvasWidget,
        initial_color: Color,
        initial_mode: usize,
        show_mode_switcher: bool,
        target: ColorPickerTarget,
    ) -> Self {
        let initial_mesh_info = if target == ColorPickerTarget::Fill { canvas.get_active_mesh_info() } else { None };
        let init_mesh_node = initial_mesh_info.as_ref().map(|(n, _, _, _, _)| *n).unwrap_or(0);
        let initial_grad_info = if target == ColorPickerTarget::Fill { canvas.get_active_gradient_info() } else { None };
        let init_grad_stop = initial_grad_info.as_ref().map(|(s, _, _, _, _)| *s).unwrap_or(0);
        let actual_init_color = if initial_mode == 2 && target == ColorPickerTarget::Fill {
            initial_mesh_info.as_ref().map(|(_, _, _, c, _)| *c).unwrap_or(initial_color)
        } else if initial_mode == 1 && target == ColorPickerTarget::Fill {
            initial_grad_info.as_ref().map(|(_, _, _, _, c)| *c).unwrap_or(initial_color)
        } else {
            initial_color
        };
        let (h, s, v) = actual_init_color.to_hsv();
        let color = Rc::new(Cell::new(actual_init_color));
        let hue = Rc::new(Cell::new(h));
        let sat = Rc::new(Cell::new(s));
        let val = Rc::new(Cell::new(v));
        let alpha = Rc::new(Cell::new(actual_init_color.a));
        let current_mode = Rc::new(Cell::new(initial_mode));
        let active_grad_stop = Rc::new(Cell::new(init_grad_stop));
        let active_mesh_node = Rc::new(Cell::new(init_mesh_node));
        let is_updating = Rc::new(Cell::new(false));
        let on_change: Rc<RefCell<Option<Box<dyn Fn(Color)>>>> = Rc::new(RefCell::new(None));
        let on_mode_change: Rc<RefCell<Option<Box<dyn Fn(usize)>>>> = Rc::new(RefCell::new(None));

        // Initial gradient stops & pattern colors from selection
        let (init_stops, init_sec_color, init_angle) = {
            let fills_opt = canvas.get_selected_fills_and_strokes();
            let fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
            if let Some(f0) = fills.first() {
                let stops = f0.effective_stops();
                (stops, f0.secondary_color, f0.angle)
            } else {
                (
                    vec![
                        GradientStop::new(0.0, initial_color),
                        GradientStop::new(1.0, Color::WHITE),
                    ],
                    Color::WHITE,
                    90.0,
                )
            }
        };

        let grad_stops_ref = Rc::new(RefCell::new(init_stops));
        let pat_c1_col_cell = Rc::new(Cell::new(initial_color));
        let pat_c2_col_cell = Rc::new(Cell::new(init_sec_color));

        let pat_c1_da = create_swatch_da(pat_c1_col_cell.clone(), 16);
        let pat_c2_da = create_swatch_da(pat_c2_col_cell.clone(), 16);

        let popover = gtk4::Popover::builder()
            .has_arrow(true)
            .css_classes(["color-picker-popover"])
            .build();

        let root_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .margin_start(4)
            .margin_end(4)
            .margin_top(4)
            .margin_bottom(4)
            .width_request(248)
            .build();

        // ── 1. Top Mode Switcher (Floating Tab Capsule) ──
        let top_modes_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(2)
            .css_classes(["color-picker-tab-bar"])
            .halign(gtk4::Align::Fill)
            .build();

        let mode_labels = [
            (crate::core::gettext("Solid"), "media-record-symbolic"),
            (
                crate::core::gettext("Gradient"),
                "media-playlist-consecutive-symbolic",
            ),
            (crate::core::gettext("Mesh"), "action-unavailable-symbolic"),
            (crate::core::gettext("Pattern"), "view-grid-symbolic"),
        ];

        let mode_buttons: Vec<gtk4::Button> = mode_labels
            .iter()
            .enumerate()
            .map(|(idx, (lbl, icon))| {
                let btn = gtk4::Button::builder()
                    .css_classes(["flat", "color-picker-tab-btn"])
                    .hexpand(true)
                    .build();
                let b_content = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(3)
                    .halign(gtk4::Align::Center)
                    .valign(gtk4::Align::Center)
                    .build();
                let img = gtk4::Image::from_icon_name(*icon);
                img.set_pixel_size(11);
                let label = gtk4::Label::builder().label(lbl).build();
                b_content.append(&img);
                b_content.append(&label);
                btn.set_child(Some(&b_content));

                if idx == initial_mode {
                    btn.add_css_class("active");
                }

                top_modes_box.append(&btn);
                btn
            })
            .collect();

        // ── 2. Main Color Card ──
        let card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .css_classes(["color-picker-card"])
            .build();

        // 2A. Header
        let header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .margin_start(2)
            .margin_end(2)
            .margin_top(1)
            .margin_bottom(1)
            .valign(gtk4::Align::Center)
            .build();

        let initial_title = match target {
            ColorPickerTarget::Stroke => crate::core::gettext("Stroke Color"),
            ColorPickerTarget::Fill => match initial_mode {
                0 => crate::core::gettext("Flat Color"),
                1 => crate::core::gettext("Gradient"),
                2 => crate::core::gettext("Mesh Gradient"),
                3 => crate::core::gettext("Patterns"),
                _ => crate::core::gettext("Flat Color"),
            },
            ColorPickerTarget::Standalone => crate::core::gettext("Flat Color"),
        };

        let title_lbl = gtk4::Label::builder()
            .label(&initial_title)
            .css_classes(["heading"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        header.append(&title_lbl);
        card.append(&header);

        // Pre-create DrawingAreas and Entry (Compact Dimensions)
        let hue_area = gtk4::DrawingArea::builder()
            .content_width(16)
            .content_height(115)
            .valign(gtk4::Align::Fill)
            .build();

        let sv_area = gtk4::DrawingArea::builder()
            .content_width(196)
            .content_height(115)
            .hexpand(false)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Fill)
            .build();

        let alpha_area = gtk4::DrawingArea::builder()
            .content_width(218)
            .content_height(18)
            .margin_top(2)
            .margin_bottom(2)
            .valign(gtk4::Align::Center)
            .hexpand(false)
            .halign(gtk4::Align::Center)
            .build();

        let hex_entry = gtk4::Entry::builder()
            .text(&initial_color.to_hex_rgba())
            .width_chars(9)
            .max_width_chars(10)
            .css_classes(["numeric", "flat"])
            .valign(gtk4::Align::Center)
            .build();

        // ── Helper to load color into tuner ──
        let load_tuner_color = {
            let color_c = color.clone();
            let hue_c = hue.clone();
            let sat_c = sat.clone();
            let val_c = val.clone();
            let alpha_c = alpha.clone();
            let is_upd = is_updating.clone();
            let hex_e = hex_entry.clone();
            let sv_draw = sv_area.clone();
            let hue_draw = hue_area.clone();
            let alpha_draw = alpha_area.clone();

            Rc::new(move |c: Color| {
                color_c.set(c);
                let (h, s, v) = c.to_hsv();
                hue_c.set(h);
                sat_c.set(s);
                val_c.set(v);
                alpha_c.set(c.a);

                if !is_upd.get() {
                    is_upd.set(true);
                    hex_e.set_text(&c.to_hex_rgba());
                    is_upd.set(false);
                }

                sv_draw.queue_draw();
                hue_draw.queue_draw();
                alpha_draw.queue_draw();
            })
        };

        // ── 1. SOLID PANEL (Minimal top area) ──
        let solid_panel = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .visible(initial_mode == 0)
            .build();
        card.append(&solid_panel);

        // ── 2. GRADIENT PANEL (Interactive Multi-Stop Track & Angle Dial) ──
        let gradient_panel = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .visible(initial_mode == 1)
            .build();

        // Type switcher: Linear / Radial
        let grad_type_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::Fill)
            .build();

        let lin_btn = gtk4::Button::builder()
            .label(&crate::core::gettext("Linear"))
            .icon_name("media-playlist-consecutive-symbolic")
            .css_classes(["flat", "pill-btn", "active"])
            .hexpand(true)
            .build();
        let rad_btn = gtk4::Button::builder()
            .label(&crate::core::gettext("Radial"))
            .icon_name("media-record-symbolic")
            .css_classes(["flat", "pill-btn"])
            .hexpand(true)
            .build();

        {
            let cv_l = canvas.clone();
            let l_b = lin_btn.clone();
            let r_b = rad_btn.clone();
            lin_btn.connect_clicked(move |_| {
                l_b.add_css_class("active");
                r_b.remove_css_class("active");
                let fills_opt = cv_l.get_selected_fills_and_strokes();
                let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                if let Some(f0) = fills.first_mut() {
                    f0.style = crate::core::FillStyle::LinearGradient;
                }
                cv_l.set_selected_fills(fills);
                cv_l.set_active_tool("gradient");
            });
        }
        {
            let cv_r = canvas.clone();
            let l_b = lin_btn.clone();
            let r_b = rad_btn.clone();
            rad_btn.connect_clicked(move |_| {
                r_b.add_css_class("active");
                l_b.remove_css_class("active");
                let fills_opt = cv_r.get_selected_fills_and_strokes();
                let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                if let Some(f0) = fills.first_mut() {
                    f0.style = crate::core::FillStyle::RadialGradient;
                }
                cv_r.set_selected_fills(fills);
                cv_r.set_active_tool("gradient");
            });
        }
        grad_type_box.append(&lin_btn);
        grad_type_box.append(&rad_btn);
        gradient_panel.append(&grad_type_box);

        // ── Interactive Gradient Track DrawingArea ──
        let grad_track_da = gtk4::DrawingArea::builder()
            .content_height(34)
            .hexpand(true)
            .margin_top(2)
            .margin_bottom(2)
            .build();

        {
            let g_stops = grad_stops_ref.clone();
            let act_s = active_grad_stop.clone();
            grad_track_da.set_draw_func(move |_, cr, width, height| {
                let w = width as f64;
                let _h = height as f64;
                let track_pad = 8.0;
                let track_w = (w - track_pad * 2.0).max(1.0);
                let bar_h = 14.0;
                let bar_y = 2.0;

                // 1. Draw Checkerboard background for transparency
                cr.save().unwrap();
                draw_rounded_rect(cr, track_pad, bar_y, track_w, bar_h, 6.0);
                cr.clip();
                let check_sz = 4.0;
                let cols = (track_w / check_sz).ceil() as usize;
                let rows = (bar_h / check_sz).ceil() as usize;
                for r in 0..rows {
                    for c in 0..cols {
                        if (r + c) % 2 == 0 {
                            cr.set_source_rgb(0.8, 0.8, 0.8);
                        } else {
                            cr.set_source_rgb(0.6, 0.6, 0.6);
                        }
                        cr.rectangle(
                            track_pad + c as f64 * check_sz,
                            bar_y + r as f64 * check_sz,
                            check_sz,
                            check_sz,
                        );
                        let _ = cr.fill();
                    }
                }

                // 2. Draw Multi-Stop Gradient Interpolation
                let stops = g_stops.borrow();
                let pat = cairo::LinearGradient::new(track_pad, 0.0, track_pad + track_w, 0.0);
                for s in stops.iter() {
                    pat.add_color_stop_rgba(
                        s.offset.clamp(0.0, 1.0) as f64,
                        s.color.r as f64,
                        s.color.g as f64,
                        s.color.b as f64,
                        s.color.a as f64,
                    );
                }
                cr.set_source(&pat).unwrap();
                let _ = cr.paint();
                cr.restore().unwrap();

                // Border of gradient bar
                draw_rounded_rect(cr, track_pad + 0.5, bar_y + 0.5, track_w - 1.0, bar_h - 1.0, 6.0);
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.3);
                cr.set_line_width(1.0);
                let _ = cr.stroke();

                // 3. Draw Stop Pins along the track
                let active_idx = act_s.get();
                for (idx, s) in stops.iter().enumerate() {
                    let pin_x = track_pad + s.offset.clamp(0.0, 1.0) as f64 * track_w;
                    let pin_y = 24.0;
                    let is_active = idx == active_idx;

                    // Triangle pointer pointing up
                    cr.new_sub_path();
                    cr.move_to(pin_x, bar_y + bar_h + 1.0);
                    cr.line_to(pin_x + 4.5, pin_y - 4.0);
                    cr.line_to(pin_x - 4.5, pin_y - 4.0);
                    cr.close_path();
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.95);
                    let _ = cr.fill_preserve();
                    cr.set_source_rgba(0.0, 0.0, 0.0, 0.4);
                    cr.set_line_width(1.0);
                    let _ = cr.stroke();

                    // Pin circle handle
                    let radius = if is_active { 6.5 } else { 5.5 };
                    if is_active {
                        // Halo glow for active pin
                        cr.arc(pin_x, pin_y, radius + 2.5, 0.0, std::f64::consts::TAU);
                        cr.set_source_rgba(0.2, 0.6, 1.0, 0.45);
                        let _ = cr.fill();
                    }

                    cr.arc(pin_x, pin_y, radius, 0.0, std::f64::consts::TAU);
                    cr.set_source_rgba(
                        s.color.r as f64,
                        s.color.g as f64,
                        s.color.b as f64,
                        s.color.a as f64,
                    );
                    let _ = cr.fill_preserve();
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    cr.set_line_width(1.8);
                    let _ = cr.stroke_preserve();
                    cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
                    cr.set_line_width(0.8);
                    let _ = cr.stroke();
                }
            });
        }
        gradient_panel.append(&grad_track_da);

        // ── Stop Pills List & Actions Bar ──
        let stops_actions_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::End)
            .build();

        // Actions: Invert, Add, Delete
        let grad_invert_btn = gtk4::Button::builder()
            .icon_name("object-flip-horizontal-symbolic")
            .css_classes(["flat", "pill-btn"])
            .tooltip_text(&crate::core::gettext("Invert Gradient"))
            .build();

        let grad_add_btn = gtk4::Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(["flat", "pill-btn"])
            .tooltip_text(&crate::core::gettext("Add Color Stop"))
            .build();

        let grad_del_btn = gtk4::Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(["flat", "pill-btn"])
            .tooltip_text(&crate::core::gettext("Delete Selected Stop"))
            .build();

        stops_actions_row.append(&grad_invert_btn);
        stops_actions_row.append(&grad_add_btn);
        stops_actions_row.append(&grad_del_btn);
        gradient_panel.append(&stops_actions_row);

        // ── Function to update stop actions sensitivity ──
        let rebuild_stop_pills = {
            let g_stops = grad_stops_ref.clone();
            let act_s = active_grad_stop.clone();
            let del_b = grad_del_btn.clone();

            Rc::new(move || {
                let stops = g_stops.borrow().clone();
                del_b.set_sensitive(stops.len() > 2);
                let active_idx = act_s.get().min(stops.len().saturating_sub(1));
                act_s.set(active_idx);
            })
        };

        // Wire Invert, Add, and Delete actions
        {
            let g_stops = grad_stops_ref.clone();
            let act_s = active_grad_stop.clone();
            let cv_inv = canvas.clone();
            let rbp_fn = rebuild_stop_pills.clone();
            let da_redraw = grad_track_da.clone();
            let ltc = load_tuner_color.clone();

            grad_invert_btn.connect_clicked(move |_| {
                {
                    let mut stops = g_stops.borrow_mut();
                    for s in stops.iter_mut() {
                        s.offset = (1.0f32 - s.offset).clamp(0.0f32, 1.0f32);
                    }
                    stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
                }
                let stops_clone = g_stops.borrow().clone();
                let fills_opt = cv_inv.get_selected_fills_and_strokes();
                let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                if let Some(f0) = fills.first_mut() {
                    f0.stops = stops_clone.clone();
                    if let Some(s0) = stops_clone.first() {
                        f0.color = s0.color;
                    }
                    if let Some(send) = stops_clone.last() {
                        f0.secondary_color = send.color;
                    }
                }
                cv_inv.set_selected_fills(fills);
                let cur_idx = act_s.get().min(stops_clone.len().saturating_sub(1));
                ltc(stops_clone[cur_idx].color);
                rbp_fn();
                da_redraw.queue_draw();
            });
        }
        {
            let g_stops = grad_stops_ref.clone();
            let act_s = active_grad_stop.clone();
            let cv_add = canvas.clone();
            let rbp_fn = rebuild_stop_pills.clone();
            let da_redraw = grad_track_da.clone();
            let ltc = load_tuner_color.clone();

            grad_add_btn.connect_clicked(move |_| {
                let new_offset = 0.5f32;
                let interp_col = interpolate_stops(&g_stops.borrow(), new_offset);
                let new_stop = GradientStop::new(new_offset, interp_col);

                let mut new_idx = 0;
                {
                    let mut stops = g_stops.borrow_mut();
                    stops.push(new_stop);
                    stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
                    if let Some(pos) = stops.iter().position(|s| (s.offset - new_offset).abs() < 0.001) {
                        new_idx = pos;
                    }
                }
                act_s.set(new_idx);
                let stops_clone = g_stops.borrow().clone();
                let fills_opt = cv_add.get_selected_fills_and_strokes();
                let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                if let Some(f0) = fills.first_mut() {
                    f0.stops = stops_clone;
                }
                cv_add.set_selected_fills(fills);
                ltc(interp_col);
                rbp_fn();
                da_redraw.queue_draw();
            });
        }
        {
            let g_stops = grad_stops_ref.clone();
            let act_s = active_grad_stop.clone();
            let cv_del = canvas.clone();
            let rbp_fn = rebuild_stop_pills.clone();
            let da_redraw = grad_track_da.clone();
            let ltc = load_tuner_color.clone();

            grad_del_btn.connect_clicked(move |_| {
                let cur_idx = act_s.get();
                {
                    let mut stops = g_stops.borrow_mut();
                    if stops.len() > 2 && cur_idx < stops.len() {
                        stops.remove(cur_idx);
                    }
                }
                let stops_clone = g_stops.borrow().clone();
                let new_active = cur_idx.min(stops_clone.len().saturating_sub(1));
                act_s.set(new_active);

                let fills_opt = cv_del.get_selected_fills_and_strokes();
                let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                if let Some(f0) = fills.first_mut() {
                    f0.stops = stops_clone.clone();
                    if let Some(s0) = stops_clone.first() {
                        f0.color = s0.color;
                    }
                    if let Some(send) = stops_clone.last() {
                        f0.secondary_color = send.color;
                    }
                }
                cv_del.set_selected_fills(fills);
                ltc(stops_clone[new_active].color);
                rbp_fn();
                da_redraw.queue_draw();
            });
        }

        // ── Drag Gesture on Interactive Gradient Track ──
        let track_drag = gtk4::GestureDrag::new();
        let is_dragging_pin = Rc::new(Cell::new(false));
        {
            let g_stops = grad_stops_ref.clone();
            let act_s = active_grad_stop.clone();
            let ltc = load_tuner_color.clone();
            let is_drag = is_dragging_pin.clone();
            let da_redraw = grad_track_da.clone();
            let rbp_fn = rebuild_stop_pills.clone();
            let cv = canvas.clone();
            let da_area = grad_track_da.clone();

            track_drag.connect_drag_begin(move |_, x, y| {
                let w = da_area.width() as f64;
                let track_pad = 8.0;
                let track_w = (w - track_pad * 2.0).max(1.0);

                let mut hit_idx = None;
                {
                    let stops = g_stops.borrow();
                    for (idx, s) in stops.iter().enumerate() {
                        let pin_x = track_pad + s.offset.clamp(0.0, 1.0) as f64 * track_w;
                        let pin_y = 24.0;
                        let dx = x - pin_x;
                        let dy = y - pin_y;
                        if (dx * dx + dy * dy).sqrt() <= 10.0 {
                            hit_idx = Some(idx);
                            break;
                        }
                    }
                }

                if let Some(idx) = hit_idx {
                    act_s.set(idx);
                    is_drag.set(true);
                    let stops = g_stops.borrow();
                    ltc(stops[idx].color);
                    rbp_fn();
                    da_redraw.queue_draw();
                } else if y <= 20.0 && x >= track_pad && x <= w - track_pad {
                    // Clicked on bar: Insert new stop at offset!
                    let offset = ((x - track_pad) / track_w).clamp(0.0, 1.0) as f32;
                    let interp_col = interpolate_stops(&g_stops.borrow(), offset);
                    let new_stop = GradientStop::new(offset, interp_col);

                    let mut new_idx = 0;
                    {
                        let mut stops = g_stops.borrow_mut();
                        stops.push(new_stop);
                        stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
                        if let Some(pos) = stops.iter().position(|s| (s.offset - offset).abs() < 0.001) {
                            new_idx = pos;
                        }
                    }
                    act_s.set(new_idx);
                    is_drag.set(true);

                    let stops_clone = g_stops.borrow().clone();
                    let fills_opt = cv.get_selected_fills_and_strokes();
                    let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                    if let Some(f0) = fills.first_mut() {
                        f0.stops = stops_clone;
                    }
                    cv.set_selected_fills(fills);
                    ltc(interp_col);
                    rbp_fn();
                    da_redraw.queue_draw();
                }
            });
        }
        {
            let g_stops = grad_stops_ref.clone();
            let act_s = active_grad_stop.clone();
            let is_drag = is_dragging_pin.clone();
            let da_redraw = grad_track_da.clone();
            let rbp_fn = rebuild_stop_pills.clone();
            let cv = canvas.clone();
            let da_area = grad_track_da.clone();

            track_drag.connect_drag_update(move |gesture, offset_x, _| {
                if !is_drag.get() {
                    return;
                }
                if let Some((start_x, _)) = gesture.start_point() {
                    let cur_x = start_x + offset_x;
                    let w = da_area.width() as f64;
                    let track_pad = 8.0;
                    let track_w = (w - track_pad * 2.0).max(1.0);
                    let new_offset = ((cur_x - track_pad) / track_w).clamp(0.0, 1.0) as f32;

                    let active_idx = act_s.get();
                    {
                        let mut stops = g_stops.borrow_mut();
                        if active_idx < stops.len() {
                            stops[active_idx].offset = new_offset;
                        }
                    }

                    let stops_clone = g_stops.borrow().clone();
                    let fills_opt = cv.get_selected_fills_and_strokes();
                    let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                    if let Some(f0) = fills.first_mut() {
                        f0.stops = stops_clone;
                    }
                    cv.set_selected_fills(fills);
                    rbp_fn();
                    da_redraw.queue_draw();
                }
            });
        }
        {
            let is_drag = is_dragging_pin.clone();
            let g_stops = grad_stops_ref.clone();
            let act_s = active_grad_stop.clone();
            let rbp_fn = rebuild_stop_pills.clone();
            let da_redraw = grad_track_da.clone();

            track_drag.connect_drag_end(move |_, _, _| {
                is_drag.set(false);
                let cur_idx = act_s.get();
                let cur_stop = {
                    let mut stops = g_stops.borrow_mut();
                    let s_copy = stops.get(cur_idx).cloned();
                    stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
                    s_copy
                };
                if let Some(s) = cur_stop {
                    let stops = g_stops.borrow();
                    if let Some(new_p) = stops.iter().position(|item| (item.offset - s.offset).abs() < 0.001) {
                        act_s.set(new_p);
                    }
                }
                rbp_fn();
                da_redraw.queue_draw();
            });
        }
        grad_track_da.add_controller(track_drag);

        // Initial build of stop pills
        rebuild_stop_pills();

        // ── Angle Control (Slider + Circular Compass Dial Knob + Badge, NO degree buttons!) ──
        let angle_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .margin_top(2)
            .margin_bottom(2)
            .build();

        let angle_lbl = gtk4::Label::builder()
            .label(&crate::core::gettext("Angle"))
            .css_classes(["caption"])
            .build();
        angle_row.append(&angle_lbl);

        let grad_angle_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 360.0, 1.0);
        grad_angle_scale.set_value(init_angle as f64);
        grad_angle_scale.set_hexpand(true);

        // Interactive Circular Angle Dial (26x26px)
        let angle_dial_da = gtk4::DrawingArea::builder()
            .content_width(26)
            .content_height(26)
            .css_classes(["angle-dial"])
            .valign(gtk4::Align::Center)
            .build();

        let current_angle = Rc::new(Cell::new(init_angle));

        {
            let cur_ang = current_angle.clone();
            angle_dial_da.set_draw_func(move |_, cr, w, h| {
                let cx = w as f64 * 0.5;
                let cy = h as f64 * 0.5;
                let radius = (w.min(h) as f64 * 0.5) - 1.5;
                let ang_rad = (cur_ang.get() as f64).to_radians();

                // Dial Background
                cr.arc(cx, cy, radius, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.08);
                let _ = cr.fill_preserve();
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.35);
                cr.set_line_width(1.2);
                let _ = cr.stroke();

                // Needle Line
                let nx = cx + ang_rad.cos() * (radius - 2.0);
                let ny = cy + ang_rad.sin() * (radius - 2.0);
                cr.move_to(cx, cy);
                cr.line_to(nx, ny);
                cr.set_source_rgba(0.2, 0.6, 1.0, 1.0);
                cr.set_line_width(2.2);
                let _ = cr.stroke();

                // Center pivot dot
                cr.arc(cx, cy, 2.5, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(0.2, 0.6, 1.0, 1.0);
                let _ = cr.fill();
            });
        }

        // Degree Badge
        let angle_badge = gtk4::Label::builder()
            .label(&format!("{}°", init_angle as i32))
            .css_classes(["angle-badge"])
            .valign(gtk4::Align::Center)
            .build();

        // Wire angle slider & dial
        {
            let cv_ga = canvas.clone();
            let cur_ang = current_angle.clone();
            let dial_draw = angle_dial_da.clone();
            let badge = angle_badge.clone();
            grad_angle_scale.connect_value_changed(move |sc| {
                let val = sc.value() as f32;
                cur_ang.set(val);
                badge.set_text(&format!("{}°", val as i32));
                dial_draw.queue_draw();

                let fills_opt = cv_ga.get_selected_fills_and_strokes();
                let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                if let Some(f0) = fills.first_mut() {
                    f0.angle = val;
                }
                cv_ga.set_selected_fills(fills);
            });
        }

        // Dial gesture
        let dial_drag = gtk4::GestureDrag::new();
        {
            let sc_clone = grad_angle_scale.clone();
            let dial_area = angle_dial_da.clone();
            let update_from_pos = move |x: f64, y: f64| {
                let w = dial_area.width() as f64;
                let h = dial_area.height() as f64;
                let dx = x - (w * 0.5);
                let dy = y - (h * 0.5);
                let mut deg = dy.atan2(dx).to_degrees();
                if deg < 0.0 {
                    deg += 360.0;
                }
                sc_clone.set_value(deg.round());
            };

            let ufp_start = update_from_pos.clone();
            dial_drag.connect_drag_begin(move |_, x, y| {
                ufp_start(x, y);
            });
            dial_drag.connect_drag_update(move |gesture, offset_x, offset_y| {
                if let Some((sx, sy)) = gesture.start_point() {
                    update_from_pos(sx + offset_x, sy + offset_y);
                }
            });
        }
        angle_dial_da.add_controller(dial_drag);

        angle_row.append(&grad_angle_scale);
        angle_row.append(&angle_dial_da);
        angle_row.append(&angle_badge);
        gradient_panel.append(&angle_row);

        card.append(&gradient_panel);

        // ── 3. MESH PANEL (Dynamic Mesh Palette) ──
        let mesh_panel = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .visible(initial_mode == 2)
            .build();

        card.append(&mesh_panel);

        // ── 4. PATTERN PANEL ──
        let pattern_panel = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .visible(initial_mode == 3)
            .build();

        let pat_scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .propagate_natural_height(true)
            .max_content_height(120)
            .min_content_height(100)
            .build();

        let pat_content_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .build();

        let pat_grid = gtk4::FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .max_children_per_line(2)
            .min_children_per_line(2)
            .homogeneous(true)
            .row_spacing(2)
            .column_spacing(2)
            .build();

        let pattern_types = [
            (crate::core::element::PatternType::Checkerboard, crate::core::gettext("Checkerboard")),
            (crate::core::element::PatternType::Dots, crate::core::gettext("Dots")),
            (crate::core::element::PatternType::Stripes, crate::core::gettext("Stripes")),
            (crate::core::element::PatternType::Grid, crate::core::gettext("Grid")),
            (crate::core::element::PatternType::Hexagon, crate::core::gettext("Honeycomb")),
            (crate::core::element::PatternType::Crosshatch, crate::core::gettext("Crosshatch")),
            (crate::core::element::PatternType::Brick, crate::core::gettext("Brick Wall")),
            (crate::core::element::PatternType::Scales, crate::core::gettext("Scales")),
            (crate::core::element::PatternType::Houndstooth, crate::core::gettext("Houndstooth")),
            (crate::core::element::PatternType::Basketweave, crate::core::gettext("Basketweave")),
        ];

        for (pt, pt_name) in pattern_types {
            let canvas_p = canvas.clone();
            let tile = crate::ui::inspector::appearance::fills::create_pattern_preview_tile(
                pt,
                None,
                &pt_name,
                false,
                move || {
                    let fills_opt = canvas_p.get_selected_fills_and_strokes();
                    let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                    if fills.is_empty() {
                        fills.push(crate::core::FillLayer::default());
                    }
                    if let Some(f0) = fills.first_mut() {
                        f0.style = crate::core::FillStyle::Pattern;
                        f0.pattern_type = pt;
                        f0.custom_pattern_path = None;
                    }
                    canvas_p.set_selected_fills(fills);
                    canvas_p.set_active_tool("pattern");
                },
            );
            pat_grid.append(&tile);
        }

        let user_patterns = crate::core::scan_user_patterns();
        for cp in user_patterns {
            let cp_name = cp.name.clone();
            let cp_path = cp.file_path.clone();
            let canvas_p = canvas.clone();
            let tile = crate::ui::inspector::appearance::fills::create_pattern_preview_tile(
                crate::core::element::PatternType::Custom,
                Some(cp_path.clone()),
                &cp_name,
                false,
                move || {
                    let fills_opt = canvas_p.get_selected_fills_and_strokes();
                    let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                    if fills.is_empty() {
                        fills.push(crate::core::FillLayer::default());
                    }
                    if let Some(f0) = fills.first_mut() {
                        f0.style = crate::core::FillStyle::Pattern;
                        f0.pattern_type = crate::core::element::PatternType::Custom;
                        f0.custom_pattern_path = Some(cp_path.clone());
                    }
                    canvas_p.set_selected_fills(fills);
                    canvas_p.set_active_tool("pattern");
                },
            );
            pat_grid.append(&tile);
        }

        pat_content_box.append(&pat_grid);
        pat_scrolled.set_child(Some(&pat_content_box));
        pattern_panel.append(&pat_scrolled);

        // Pattern Color Stops (Color 1 Foreground & Color 2 Background)
        let pat_stops_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::Fill)
            .build();

        let pat_c1_btn = gtk4::Button::builder()
            .css_classes(["flat", "color-stop-btn", "active"])
            .hexpand(true)
            .build();
        let pat_c1_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::Center)
            .build();
        pat_c1_box.append(&pat_c1_da);
        pat_c1_box.append(&gtk4::Label::new(Some(&crate::core::gettext("Pattern"))));
        pat_c1_btn.set_child(Some(&pat_c1_box));

        let pat_c2_btn = gtk4::Button::builder()
            .css_classes(["flat", "color-stop-btn"])
            .hexpand(true)
            .build();
        let pat_c2_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::Center)
            .build();
        pat_c2_box.append(&pat_c2_da);
        pat_c2_box.append(&gtk4::Label::new(Some(&crate::core::gettext("Background"))));
        pat_c2_btn.set_child(Some(&pat_c2_box));

        let pat_swap_btn = gtk4::Button::builder()
            .icon_name("object-flip-horizontal-symbolic")
            .css_classes(["flat", "pill-btn"])
            .tooltip_text(&crate::core::gettext("Swap Colors"))
            .build();

        let active_pat_stop = Rc::new(Cell::new(0usize));
        {
            let act_stop = active_pat_stop.clone();
            let s1_b = pat_c1_btn.clone();
            let s2_b = pat_c2_btn.clone();
            let s1_c = pat_c1_col_cell.clone();
            let ltc = load_tuner_color.clone();
            pat_c1_btn.connect_clicked(move |_| {
                act_stop.set(0);
                s1_b.add_css_class("active");
                s2_b.remove_css_class("active");
                ltc(s1_c.get());
            });
        }
        {
            let act_stop = active_pat_stop.clone();
            let s1_b = pat_c1_btn.clone();
            let s2_b = pat_c2_btn.clone();
            let s2_c = pat_c2_col_cell.clone();
            let ltc = load_tuner_color.clone();
            pat_c2_btn.connect_clicked(move |_| {
                act_stop.set(1);
                s2_b.add_css_class("active");
                s1_b.remove_css_class("active");
                ltc(s2_c.get());
            });
        }
        {
            let cv_swap = canvas.clone();
            let s1_c = pat_c1_col_cell.clone();
            let s2_c = pat_c2_col_cell.clone();
            let s1_da_c = pat_c1_da.clone();
            let s2_da_c = pat_c2_da.clone();
            let act_stop = active_pat_stop.clone();
            let ltc = load_tuner_color.clone();
            pat_swap_btn.connect_clicked(move |_| {
                let fills_opt = cv_swap.get_selected_fills_and_strokes();
                let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                if let Some(f0) = fills.first_mut() {
                    std::mem::swap(&mut f0.color, &mut f0.secondary_color);
                    s1_c.set(f0.color);
                    s2_c.set(f0.secondary_color);
                    if act_stop.get() == 0 {
                        ltc(f0.color);
                    } else {
                        ltc(f0.secondary_color);
                    }
                }
                cv_swap.set_selected_fills(fills);
                s1_da_c.queue_draw();
                s2_da_c.queue_draw();
            });
        }

        pat_stops_row.append(&pat_c1_btn);
        pat_stops_row.append(&pat_swap_btn);
        pat_stops_row.append(&pat_c2_btn);
        pattern_panel.append(&pat_stops_row);

        // Pattern Scale Controls
        let scale_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .valign(gtk4::Align::Center)
            .build();
        let scale_lbl = gtk4::Label::builder()
            .label(&crate::core::gettext("Scale"))
            .css_classes(["caption"])
            .build();
        scale_row.append(&scale_lbl);

        let pat_scale_slider = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 4.0, 128.0, 1.0);
        pat_scale_slider.set_value(20.0);
        pat_scale_slider.set_hexpand(true);
        let cv_scale = canvas.clone();
        pat_scale_slider.connect_value_changed(move |sc| {
            let val = sc.value() as f32;
            let fills_opt = cv_scale.get_selected_fills_and_strokes();
            let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
            if let Some(f0) = fills.first_mut() {
                f0.pattern_scale = val;
            }
            cv_scale.set_selected_fills(fills);
        });
        scale_row.append(&pat_scale_slider);
        pattern_panel.append(&scale_row);

        card.append(&pattern_panel);

        // ── 5. SHARED COLOR TUNING SECTION (Compact & Ergonomic) ──
        let divider_tuner = gtk4::Box::builder()
            .css_classes(["color-picker-divider"])
            .hexpand(true)
            .visible(initial_mode == 1 || initial_mode == 3)
            .build();
        card.append(&divider_tuner);

        let body_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::Center)
            .margin_top(2)
            .margin_bottom(2)
            .build();

        // ── Vertical Hue Slider ──
        let hue_clone = hue.clone();
        hue_area.set_draw_func(move |_area, cr, width, height| {
            let w = width as f64;
            let h = height as f64;
            let radius = 8.0;

            cr.save().unwrap();
            draw_rounded_rect(cr, 0.0, 0.0, w, h, radius);
            cr.clip();

            let pat = cairo::LinearGradient::new(0.0, 0.0, 0.0, h);
            pat.add_color_stop_rgb(0.0, 1.0, 0.0, 0.0);
            pat.add_color_stop_rgb(1.0 / 6.0, 1.0, 1.0, 0.0);
            pat.add_color_stop_rgb(2.0 / 6.0, 0.0, 1.0, 0.0);
            pat.add_color_stop_rgb(3.0 / 6.0, 0.0, 1.0, 1.0);
            pat.add_color_stop_rgb(4.0 / 6.0, 0.0, 0.0, 1.0);
            pat.add_color_stop_rgb(5.0 / 6.0, 1.0, 0.0, 1.0);
            pat.add_color_stop_rgb(1.0, 1.0, 0.0, 0.0);
            cr.set_source(&pat).unwrap();
            let _ = cr.paint();
            cr.restore().unwrap();

            draw_rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, radius);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
            cr.set_line_width(1.0);
            let _ = cr.stroke();

            let cur_h = hue_clone.get() as f64;
            let thumb_h = 10.0;
            let thumb_y = (cur_h / 360.0 * (h - thumb_h)).clamp(0.0, h - thumb_h);
            draw_rounded_rect(cr, 2.0, thumb_y, w - 4.0, thumb_h, 5.0);
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
            cr.set_line_width(1.2);
            let _ = cr.stroke();
        });
        body_row.append(&hue_area);

        // ── 2D Saturation/Value Palette ──
        let hue_for_sv = hue.clone();
        let sat_for_sv = sat.clone();
        let val_for_sv = val.clone();

        sv_area.set_draw_func(move |_area, cr, width, height| {
            let w = width as f64;
            let h = height as f64;
            let radius = 8.0;

            cr.save().unwrap();
            draw_rounded_rect(cr, 0.0, 0.0, w, h, radius);
            cr.clip();

            let cur_h = hue_for_sv.get();
            let pure_hue = Color::from_hsv(cur_h, 1.0, 1.0, 1.0);
            cr.set_source_rgb(pure_hue.r as f64, pure_hue.g as f64, pure_hue.b as f64);
            let _ = cr.paint();

            let sat_grad = cairo::LinearGradient::new(0.0, 0.0, w, 0.0);
            sat_grad.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, 1.0);
            sat_grad.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.0);
            cr.set_source(&sat_grad).unwrap();
            let _ = cr.paint();

            let val_grad = cairo::LinearGradient::new(0.0, 0.0, 0.0, h);
            val_grad.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.0);
            val_grad.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 1.0);
            cr.set_source(&val_grad).unwrap();
            let _ = cr.paint();
            cr.restore().unwrap();

            draw_rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, radius);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
            cr.set_line_width(1.0);
            let _ = cr.stroke();

            let cur_s = sat_for_sv.get() as f64;
            let cur_v = val_for_sv.get() as f64;
            let cursor_x = (cur_s * w).clamp(5.0, w - 5.0);
            let cursor_y = ((1.0 - cur_v) * h).clamp(5.0, h - 5.0);

            cr.arc(cursor_x, cursor_y, 6.5, 0.0, std::f64::consts::TAU);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
            cr.set_line_width(2.0);
            let _ = cr.stroke();

            cr.arc(cursor_x, cursor_y, 5.0, 0.0, std::f64::consts::TAU);
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            cr.set_line_width(1.8);
            let _ = cr.stroke();
        });
        body_row.append(&sv_area);
        card.append(&body_row);

        // ── Alpha / Opacity Horizontal Slider ──
        let color_for_alpha = color.clone();
        let alpha_for_alpha = alpha.clone();

        alpha_area.set_draw_func(move |_area, cr, width, height| {
            let w = width as f64;
            let h = height as f64;
            let radius = 8.0;

            cr.save().unwrap();
            draw_rounded_rect(cr, 0.0, 0.0, w, h, radius);
            cr.clip();

            let check_size = 4.0;
            let cols = (w / check_size).ceil() as usize;
            let rows = (h / check_size).ceil() as usize;
            for r in 0..rows {
                for c in 0..cols {
                    if (r + c) % 2 == 0 {
                        cr.set_source_rgb(0.75, 0.75, 0.75);
                    } else {
                        cr.set_source_rgb(0.55, 0.55, 0.55);
                    }
                    cr.rectangle(c as f64 * check_size, r as f64 * check_size, check_size, check_size);
                    let _ = cr.fill();
                }
            }

            let cur_c = color_for_alpha.get();
            let alpha_grad = cairo::LinearGradient::new(0.0, 0.0, w, 0.0);
            alpha_grad.add_color_stop_rgba(0.0, cur_c.r as f64, cur_c.g as f64, cur_c.b as f64, 0.0);
            alpha_grad.add_color_stop_rgba(1.0, cur_c.r as f64, cur_c.g as f64, cur_c.b as f64, 1.0);
            cr.set_source(&alpha_grad).unwrap();
            let _ = cr.paint();
            cr.restore().unwrap();

            draw_rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, radius);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.3);
            cr.set_line_width(1.0);
            let _ = cr.stroke();

            let cur_a = alpha_for_alpha.get();
            let pct_text = format!("{}%", (cur_a * 100.0).round() as i32);
            cr.set_font_size(10.0);
            let ext = cr.text_extents(&pct_text).unwrap();
            let tx = (w - ext.width()) / 2.0 - ext.x_bearing();
            let ty = (h - ext.height()) / 2.0 - ext.y_bearing();

            cr.move_to(tx + 1.0, ty + 1.0);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.7);
            let _ = cr.show_text(&pct_text);

            cr.move_to(tx, ty);
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            let _ = cr.show_text(&pct_text);

            let thumb_w = 12.0;
            let thumb_x = (cur_a as f64 * (w - thumb_w)).clamp(0.0, w - thumb_w);
            draw_rounded_rect(cr, thumb_x, 2.0, thumb_w, h - 4.0, 5.0);
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
            cr.set_line_width(1.2);
            let _ = cr.stroke();
        });
        card.append(&alpha_area);

        // ── Footer: [ Hex Input + Eyedropper ] ──
        let footer = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .margin_top(2)
            .margin_bottom(1)
            .valign(gtk4::Align::Center)
            .build();

        let footer_spacer = gtk4::Box::builder().hexpand(true).build();
        footer.append(&footer_spacer);

        let hex_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(2)
            .css_classes(["color-picker-hex-capsule"])
            .valign(gtk4::Align::Center)
            .build();
        hex_box.append(&hex_entry);

        let eyedropper_btn = gtk4::Button::builder()
            .icon_name("color-picker-symbolic")
            .css_classes(["flat", "circular", "color-picker-eyedropper"])
            .valign(gtk4::Align::Center)
            .tooltip_text(&crate::core::gettext("Eyedropper"))
            .build();

        {
            let canvas_eye = canvas.clone();
            let pop_close = popover.clone();
            eyedropper_btn.connect_clicked(move |_| {
                canvas_eye.set_active_tool("eyedropper");
                pop_close.popdown();
            });
        }
        hex_box.append(&eyedropper_btn);
        footer.append(&hex_box);
        card.append(&footer);

        // ── 2-Row Swatches Grid (Recent Document Colors + Defaults) ──
        let swatches_grid = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .halign(gtk4::Align::Center)
            .margin_top(3)
            .margin_bottom(1)
            .build();

        let row1 = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .build();
        let row2 = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .build();

        let mut palette_colors = Vec::new();
        for opt_c in canvas.get_document_colors() {
            if let Some(c) = opt_c {
                if !palette_colors.contains(&c) {
                    palette_colors.push(c);
                }
            }
        }

        let default_hexes = [
            "#e01b24", "#ff7800", "#f6d32d", "#33d17a", "#3584e4",
            "#9141ac", "#c061cb", "#241f31", "#77767b", "#ffffff",
        ];

        for hex in default_hexes {
            if palette_colors.len() >= 10 {
                break;
            }
            if let Some(c) = Color::from_hex(hex) {
                if !palette_colors.contains(&c) {
                    palette_colors.push(c);
                }
            }
        }
        palette_colors.truncate(10);

        for (idx, col) in palette_colors.into_iter().enumerate() {
            let btn = gtk4::Button::builder()
                .css_classes(["flat", "swatch-chip"])
                .tooltip_text(&col.to_hex_rgba())
                .build();
            let da = gtk4::DrawingArea::builder().content_width(20).content_height(20).build();
            da.set_draw_func(move |_, cr, w, h| {
                cr.arc(w as f64 * 0.5, h as f64 * 0.5, (w.min(h) as f64 * 0.5) - 1.0, 0.0, std::f64::consts::TAU);
                cr.set_source_rgba(col.r as f64, col.g as f64, col.b as f64, col.a as f64);
                let _ = cr.fill_preserve();
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
                cr.set_line_width(1.0);
                let _ = cr.stroke();
            });
            btn.set_child(Some(&da));

            let drag_swatch = gtk4::DragSource::builder()
                .actions(gdk::DragAction::COPY)
                .build();
            let paintable = gtk4::WidgetPaintable::new(Some(&da));
            drag_swatch.set_icon(Some(&paintable), 10, 10);
            let hex_val = col.to_hex();
            let tg = target;
            drag_swatch.connect_prepare(move |_, _, _| {
                let payload = match tg {
                    ColorPickerTarget::Stroke => format!("gnome-paths:stroke:{}", hex_val),
                    _ => format!("gnome-paths:fill:{}", hex_val),
                };
                Some(gdk::ContentProvider::for_value(&payload.to_value()))
            });
            btn.add_controller(drag_swatch);

            let ltc = load_tuner_color.clone();
            let cv_s = canvas.clone();
            let cur_m = current_mode.clone();
            let act_g = active_grad_stop.clone();
            let act_mesh_n = active_mesh_node.clone();
            let act_p = active_pat_stop.clone();
            let g_stops = grad_stops_ref.clone();
            let pc1_c = pat_c1_col_cell.clone();
            let pc2_c = pat_c2_col_cell.clone();
            let pc1_da_c = pat_c1_da.clone();
            let pc2_da_c = pat_c2_da.clone();
            let g_track_c = grad_track_da.clone();
            let rbp_c = rebuild_stop_pills.clone();
            let on_ch = on_change.clone();

            btn.connect_clicked(move |_| {
                ltc(col);
                if tg == ColorPickerTarget::Stroke {
                    cv_s.set_stroke_color(Some(col));
                } else if tg == ColorPickerTarget::Fill {
                    let mode = cur_m.get();

                    let fills_opt = cv_s.get_selected_fills_and_strokes();
                    let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                    if fills.is_empty() {
                        fills.push(crate::core::FillLayer::default());
                    }

                    if let Some(f0) = fills.first_mut() {
                        match mode {
                            0 => {
                                f0.style = crate::core::FillStyle::Solid;
                                f0.mesh = None;
                                f0.stops.clear();
                                f0.custom_pattern_path = None;
                                f0.color = col;
                                cv_s.set_fill_color(col);
                            }
                            1 => {
                                let stop_idx = act_g.get().min(g_stops.borrow().len().saturating_sub(1));
                                act_g.set(stop_idx);
                                {
                                    let mut stops = g_stops.borrow_mut();
                                    if stop_idx < stops.len() {
                                        stops[stop_idx].color = col;
                                    }
                                }
                                let stops_clone = g_stops.borrow().clone();
                                f0.stops = stops_clone.clone();
                                if let Some(s0) = stops_clone.first() {
                                    f0.color = s0.color;
                                }
                                if let Some(send) = stops_clone.last() {
                                    f0.secondary_color = send.color;
                                }
                                g_track_c.queue_draw();
                                rbp_c();
                            }
                            2 => {
                                let node_idx = cv_s.get_active_mesh_node().unwrap_or_else(|| act_mesh_n.get());
                                act_mesh_n.set(node_idx);
                                cv_s.set_mesh_node_color(node_idx, col);
                            }
                            3 => {
                                if act_p.get() == 0 {
                                    f0.color = col;
                                    pc1_c.set(col);
                                } else {
                                    f0.secondary_color = col;
                                    pc2_c.set(col);
                                }
                                pc1_da_c.queue_draw();
                                pc2_da_c.queue_draw();
                            }
                            _ => {}
                        }
                    }
                    if mode != 2 {
                        cv_s.set_selected_fills(fills);
                    }
                }

                if let Some(cb) = on_ch.borrow().as_ref() {
                    cb(col);
                }
            });

            if idx < 5 {
                row1.append(&btn);
            } else {
                row2.append(&btn);
            }
        }
        swatches_grid.append(&row1);
        swatches_grid.append(&row2);
        card.append(&swatches_grid);

        // Wire mode tab switcher
        let on_mode_clone = on_mode_change.clone();
        let mode_btns_clone = mode_buttons.clone();
        let canvas_mode = canvas.clone();
        let rbp_mode = rebuild_stop_pills.clone();
        let g_track_mode = grad_track_da.clone();
        let g_stops_mode = grad_stops_ref.clone();

        let div_tun_clone = divider_tuner.clone();

        for (i, btn) in mode_buttons.iter().enumerate() {
            let on_mode = on_mode_clone.clone();
            let all_btns = mode_btns_clone.clone();
            let title_c = title_lbl.clone();
            let solid_p_c = solid_panel.clone();
            let grad_p_c = gradient_panel.clone();
            let mesh_p_c = mesh_panel.clone();
            let pat_p_c = pattern_panel.clone();
            let div_tun_c = div_tun_clone.clone();
            let cur_m = current_mode.clone();
            let cv = canvas_mode.clone();
            let ltc = load_tuner_color.clone();
            let rbp_fn = rbp_mode.clone();
            let gt_draw = g_track_mode.clone();
            let gst = g_stops_mode.clone();

            btn.connect_clicked(move |_| {
                for (k, b) in all_btns.iter().enumerate() {
                    if k == i {
                        b.add_css_class("active");
                    } else {
                        b.remove_css_class("active");
                    }
                }
                cur_m.set(i);

                solid_p_c.set_visible(i == 0);
                grad_p_c.set_visible(i == 1);
                mesh_p_c.set_visible(i == 2);
                pat_p_c.set_visible(i == 3);
                div_tun_c.set_visible(i == 1 || i == 3);

                match i {
                    0 => {
                        title_c.set_text(&crate::core::gettext("Flat Color"));
                        let fills_opt = cv.get_selected_fills_and_strokes();
                        let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                        if fills.is_empty() {
                            fills.push(crate::core::FillLayer::default());
                        }
                        if let Some(f0) = fills.first_mut() {
                            f0.style = crate::core::FillStyle::Solid;
                            f0.mesh = None;
                            f0.stops.clear();
                            f0.custom_pattern_path = None;
                            ltc(f0.color);
                        }
                        cv.set_selected_fills(fills);
                    }
                    1 => {
                        title_c.set_text(&crate::core::gettext("Gradient"));
                        let fills_opt = cv.get_selected_fills_and_strokes();
                        let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                        if fills.is_empty() {
                            fills.push(crate::core::FillLayer::default());
                        }
                        if let Some(f0) = fills.first_mut() {
                            f0.style = crate::core::FillStyle::LinearGradient;
                            f0.mesh = None;
                            f0.custom_pattern_path = None;
                            if f0.stops.len() < 2 {
                                f0.stops = vec![
                                    crate::core::GradientStop::new(0.0, f0.color),
                                    crate::core::GradientStop::new(1.0, f0.secondary_color),
                                ];
                            }
                            let st = f0.effective_stops();
                            *gst.borrow_mut() = st.clone();
                            if let Some(s0) = st.first() {
                                ltc(s0.color);
                            }
                        }
                        cv.set_selected_fills(fills);
                        cv.set_active_tool("gradient");
                        rbp_fn();
                        gt_draw.queue_draw();
                    }
                    2 => {
                        title_c.set_text(&crate::core::gettext("Mesh Gradient"));
                        cv.reset_selected_mesh_grid(3, 3, None, None);
                        let active_mesh = cv.get_selected_mesh();
                        if let Some(m) = active_mesh {
                            if let Some(n0) = m.nodes.first() {
                                ltc(n0.color);
                            }
                        }
                        cv.set_active_tool("mesh_gradient");
                    }
                    3 => {
                        title_c.set_text(&crate::core::gettext("Patterns"));
                        let fills_opt = cv.get_selected_fills_and_strokes();
                        let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
                        if fills.is_empty() {
                            fills.push(crate::core::FillLayer::default());
                        }
                        if let Some(f0) = fills.first_mut() {
                            f0.style = crate::core::FillStyle::Pattern;
                            f0.mesh = None;
                            f0.stops.clear();
                            f0.pattern_type = crate::core::element::PatternType::Checkerboard;
                            f0.pattern_scale = 20.0;
                            f0.secondary_color = crate::core::Color::WHITE;
                            ltc(f0.color);
                        }
                        cv.set_selected_fills(fills);
                        cv.set_active_tool("pattern");
                    }
                    _ => {}
                }
                if let Some(cb) = on_mode.borrow().as_ref() {
                    cb(i);
                }
            });
        }

        let effective_show_mode = show_mode_switcher && target == ColorPickerTarget::Fill;
        if effective_show_mode {
            root_box.append(&top_modes_box);
        }

        root_box.append(&card);
        popover.set_child(Some(&root_box));

        let instance = Self {
            popover,
            color,
            hue,
            sat,
            val,
            alpha,
            current_mode,
            active_grad_stop,
            active_mesh_node,
            on_change,
            on_mode_change,
            sv_area,
            hue_area,
            alpha_area,
            hex_entry,
            is_updating,
            mode_buttons,
            title_lbl,
            solid_panel,
            gradient_panel,
            mesh_panel,
            pattern_panel,
            grad_track_da,
            grad_stops_ref,
            pat_c1_da,
            pat_c2_da,
            canvas,
            target,
        };

        instance.wire_events(rebuild_stop_pills, active_pat_stop);
        instance
    }

    fn wire_events(
        &self,
        rebuild_stop_pills: Rc<dyn Fn()>,
        active_pat_stop: Rc<Cell<usize>>,
    ) {
        let cv = self.canvas.clone();
        let cur_mode = self.current_mode.clone();
        let act_grad_stop = self.active_grad_stop.clone();
        let act_mesh_n = self.active_mesh_node.clone();
        let act_p = active_pat_stop;
        let g_stops = self.grad_stops_ref.clone();
        let pc1_da = self.pat_c1_da.clone();
        let pc2_da = self.pat_c2_da.clone();
        let g_track_da = self.grad_track_da.clone();
        let rbp_fn = rebuild_stop_pills;

        let tg = self.target;
        let apply_current_color = move |new_col: Color| {
            if tg == ColorPickerTarget::Stroke {
                cv.set_stroke_color(Some(new_col));
                return;
            } else if tg == ColorPickerTarget::Standalone {
                return;
            }

            let mode = cur_mode.get();

            let fills_opt = cv.get_selected_fills_and_strokes();
            let mut fills = fills_opt.map(|(f, _)| f).unwrap_or_default();
            if fills.is_empty() {
                fills.push(crate::core::FillLayer::default());
            }

            if let Some(f0) = fills.first_mut() {
                match mode {
                    0 => {
                        f0.style = crate::core::FillStyle::Solid;
                        f0.mesh = None;
                        f0.stops.clear();
                        f0.custom_pattern_path = None;
                        f0.color = new_col;
                        cv.set_fill_color(new_col);
                    }
                    1 => {
                        let stop_idx = act_grad_stop.get().min(g_stops.borrow().len().saturating_sub(1));
                        act_grad_stop.set(stop_idx);
                        {
                            let mut stops = g_stops.borrow_mut();
                            if stop_idx < stops.len() {
                                stops[stop_idx].color = new_col;
                            }
                        }
                        let stops_clone = g_stops.borrow().clone();
                        f0.stops = stops_clone.clone();
                        if let Some(s0) = stops_clone.first() {
                            f0.color = s0.color;
                        }
                        if let Some(send) = stops_clone.last() {
                            f0.secondary_color = send.color;
                        }
                        g_track_da.queue_draw();
                        rbp_fn();
                    }
                    2 => {
                        let node_idx = cv.get_active_mesh_node().unwrap_or_else(|| act_mesh_n.get());
                        act_mesh_n.set(node_idx);
                        cv.set_mesh_node_color(node_idx, new_col);
                    }
                    3 => {
                        if act_p.get() == 0 {
                            f0.color = new_col;
                        } else {
                            f0.secondary_color = new_col;
                        }
                        pc1_da.queue_draw();
                        pc2_da.queue_draw();
                    }
                    _ => {}
                }
            }
            if mode != 2 {
                cv.set_selected_fills(fills);
            }
        };

        // ── Hue Gesture ──
        let hue_drag = gtk4::GestureDrag::new();
        let hue_c = self.hue.clone();
        let sat_c = self.sat.clone();
        let val_c = self.val.clone();
        let alpha_c = self.alpha.clone();
        let col_c = self.color.clone();
        let on_ch = self.on_change.clone();
        let is_upd = self.is_updating.clone();
        let sv_redraw = self.sv_area.clone();
        let hue_redraw = self.hue_area.clone();
        let alpha_redraw = self.alpha_area.clone();
        let hex_e = self.hex_entry.clone();
        let app_col1 = apply_current_color.clone();

        let update_hue_at = move |area: &gtk4::DrawingArea, y: f64| {
            let h_total = area.height() as f64;
            let ratio = (y / h_total).clamp(0.0, 1.0) as f32;
            let new_h = ratio * 360.0;
            hue_c.set(new_h);

            let new_col = Color::from_hsv(new_h, sat_c.get(), val_c.get(), alpha_c.get());
            col_c.set(new_col);

            if !is_upd.get() {
                is_upd.set(true);
                hex_e.set_text(&new_col.to_hex_rgba());
                is_upd.set(false);
            }

            hue_redraw.queue_draw();
            sv_redraw.queue_draw();
            alpha_redraw.queue_draw();

            app_col1(new_col);

            if let Some(cb) = on_ch.borrow().as_ref() {
                cb(new_col);
            }
        };

        let uh_start = update_hue_at.clone();
        let area_start = self.hue_area.clone();
        hue_drag.connect_drag_begin(move |_, _x, y| {
            uh_start(&area_start, y);
        });

        let uh_update = update_hue_at.clone();
        let area_update = self.hue_area.clone();
        hue_drag.connect_drag_update(move |gesture, _, offset_y| {
            if let Some((_, start_y)) = gesture.start_point() {
                uh_update(&area_update, start_y + offset_y);
            }
        });

        self.hue_area.add_controller(hue_drag);

        // ── SV Palette Gesture ──
        let sv_drag = gtk4::GestureDrag::new();
        let hue_c2 = self.hue.clone();
        let sat_c2 = self.sat.clone();
        let val_c2 = self.val.clone();
        let alpha_c2 = self.alpha.clone();
        let col_c2 = self.color.clone();
        let on_ch2 = self.on_change.clone();
        let is_upd2 = self.is_updating.clone();
        let sv_redraw2 = self.sv_area.clone();
        let alpha_redraw2 = self.alpha_area.clone();
        let hex_e2 = self.hex_entry.clone();
        let app_col2 = apply_current_color.clone();

        let update_sv_at = move |area: &gtk4::DrawingArea, x: f64, y: f64| {
            let w = area.width() as f64;
            let h = area.height() as f64;
            let s = (x / w).clamp(0.0, 1.0) as f32;
            let v = (1.0 - (y / h)).clamp(0.0, 1.0) as f32;

            sat_c2.set(s);
            val_c2.set(v);

            let new_col = Color::from_hsv(hue_c2.get(), s, v, alpha_c2.get());
            col_c2.set(new_col);

            if !is_upd2.get() {
                is_upd2.set(true);
                hex_e2.set_text(&new_col.to_hex_rgba());
                is_upd2.set(false);
            }

            sv_redraw2.queue_draw();
            alpha_redraw2.queue_draw();

            app_col2(new_col);

            if let Some(cb) = on_ch2.borrow().as_ref() {
                cb(new_col);
            }
        };

        let usv_start = update_sv_at.clone();
        let area_sv_start = self.sv_area.clone();
        sv_drag.connect_drag_begin(move |_, x, y| {
            usv_start(&area_sv_start, x, y);
        });

        let usv_update = update_sv_at.clone();
        let area_sv_update = self.sv_area.clone();
        sv_drag.connect_drag_update(move |gesture, offset_x, offset_y| {
            if let Some((start_x, start_y)) = gesture.start_point() {
                usv_update(&area_sv_update, start_x + offset_x, start_y + offset_y);
            }
        });

        self.sv_area.add_controller(sv_drag);

        // ── Alpha Slider Gesture ──
        let alpha_drag = gtk4::GestureDrag::new();
        let hue_c3 = self.hue.clone();
        let sat_c3 = self.sat.clone();
        let val_c3 = self.val.clone();
        let alpha_c3 = self.alpha.clone();
        let col_c3 = self.color.clone();
        let on_ch3 = self.on_change.clone();
        let is_upd3 = self.is_updating.clone();
        let alpha_redraw3 = self.alpha_area.clone();
        let hex_e3 = self.hex_entry.clone();
        let app_col3 = apply_current_color.clone();

        let update_alpha_at = move |area: &gtk4::DrawingArea, x: f64| {
            let w = area.width() as f64;
            let a = (x / w).clamp(0.0, 1.0) as f32;
            alpha_c3.set(a);

            let new_col = Color::from_hsv(hue_c3.get(), sat_c3.get(), val_c3.get(), a);
            col_c3.set(new_col);

            if !is_upd3.get() {
                is_upd3.set(true);
                hex_e3.set_text(&new_col.to_hex_rgba());
                is_upd3.set(false);
            }

            alpha_redraw3.queue_draw();

            app_col3(new_col);

            if let Some(cb) = on_ch3.borrow().as_ref() {
                cb(new_col);
            }
        };

        let ua_start = update_alpha_at.clone();
        let area_alpha_start = self.alpha_area.clone();
        alpha_drag.connect_drag_begin(move |_, x, _y| {
            ua_start(&area_alpha_start, x);
        });

        let ua_update = update_alpha_at.clone();
        let area_alpha_update = self.alpha_area.clone();
        alpha_drag.connect_drag_update(move |gesture, offset_x, _offset_y| {
            if let Some((start_x, _)) = gesture.start_point() {
                ua_update(&area_alpha_update, start_x + offset_x);
            }
        });

        self.alpha_area.add_controller(alpha_drag);

        // ── Hex Entry Changes ──
        let hue_c4 = self.hue.clone();
        let sat_c4 = self.sat.clone();
        let val_c4 = self.val.clone();
        let alpha_c4 = self.alpha.clone();
        let col_c4 = self.color.clone();
        let on_ch4 = self.on_change.clone();
        let is_upd4 = self.is_updating.clone();
        let sv_redraw4 = self.sv_area.clone();
        let hue_redraw4 = self.hue_area.clone();
        let alpha_redraw4 = self.alpha_area.clone();
        let app_col4 = apply_current_color;

        self.hex_entry.connect_changed(move |entry| {
            if is_upd4.get() {
                return;
            }
            let text = entry.text();
            if let Some(parsed) = Color::from_hex(&text) {
                col_c4.set(parsed);
                let (h, s, v) = parsed.to_hsv();
                hue_c4.set(h);
                sat_c4.set(s);
                val_c4.set(v);
                alpha_c4.set(parsed.a);

                sv_redraw4.queue_draw();
                hue_redraw4.queue_draw();
                alpha_redraw4.queue_draw();

                app_col4(parsed);

                if let Some(cb) = on_ch4.borrow().as_ref() {
                    cb(parsed);
                }
            }
        });
    }

    pub fn set_color(&self, c: Color) {
        let (h, s, v) = c.to_hsv();
        self.hue.set(h);
        self.sat.set(s);
        self.val.set(v);
        self.alpha.set(c.a);
        self.color.set(c);

        if !self.is_updating.get() {
            self.is_updating.set(true);
            self.hex_entry.set_text(&c.to_hex_rgba());
            self.is_updating.set(false);
        }

        self.hue_area.queue_draw();
        self.sv_area.queue_draw();
        self.alpha_area.queue_draw();
        self.grad_track_da.queue_draw();
        self.pat_c1_da.queue_draw();
        self.pat_c2_da.queue_draw();
    }

    pub fn on_color_changed<F: Fn(Color) + 'static>(&self, callback: F) {
        *self.on_change.borrow_mut() = Some(Box::new(callback));
    }

    pub fn set_mode(&self, mode: usize) {
        for (k, b) in self.mode_buttons.iter().enumerate() {
            if k == mode {
                b.add_css_class("active");
            } else {
                b.remove_css_class("active");
            }
        }
        self.current_mode.set(mode);

        self.solid_panel.set_visible(mode == 0);
        self.gradient_panel.set_visible(mode == 1);
        self.mesh_panel.set_visible(mode == 2);
        self.pattern_panel.set_visible(mode == 3);

        match mode {
            0 => {
                self.title_lbl.set_text(&crate::core::gettext("Flat Color"));
            }
            1 => {
                self.title_lbl.set_text(&crate::core::gettext("Gradient"));
            }
            2 => {
                self.title_lbl.set_text(&crate::core::gettext("Mesh Gradient"));
            }
            3 => {
                self.title_lbl.set_text(&crate::core::gettext("Patterns"));
            }
            _ => {}
        }
        if let Some(cb) = self.on_mode_change.borrow().as_ref() {
            cb(mode);
        }
    }

    pub fn popover(&self) -> &gtk4::Popover {
        &self.popover
    }

    pub fn attach_to(&self, widget: &impl IsA<gtk4::Widget>) {
        if self.popover.parent().is_some() {
            self.popover.unparent();
        }
        self.popover.set_parent(widget);
    }

    pub fn popup(&self) {
        self.popover.popup();
    }

    pub fn append_footer_widget(&self, widget: &impl IsA<gtk4::Widget>) {
        if let Some(child) = self.popover.child() {
            if let Some(root_box) = child.downcast_ref::<gtk4::Box>() {
                root_box.append(widget);
            }
        }
    }
}
