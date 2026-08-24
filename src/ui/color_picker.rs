use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::canvas::CanvasWidget;
use crate::core::Color;

#[derive(Clone)]
pub struct ColorPickerPopover {
    popover: gtk4::Popover,
    color: Rc<Cell<Color>>,
    hue: Rc<Cell<f32>>,
    sat: Rc<Cell<f32>>,
    val: Rc<Cell<f32>>,
    alpha: Rc<Cell<f32>>,
    on_change: Rc<RefCell<Option<Box<dyn Fn(Color)>>>>,
    on_mode_change: Rc<RefCell<Option<Box<dyn Fn(usize)>>>>,
    sv_area: gtk4::DrawingArea,
    hue_area: gtk4::DrawingArea,
    alpha_area: gtk4::DrawingArea,
    hex_entry: gtk4::Entry,
    is_updating: Rc<Cell<bool>>,

    mode_buttons: Vec<gtk4::Button>,

    title_lbl: gtk4::Label,

    mesh_panel: gtk4::Box,

    pattern_panel: gtk4::Box,
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

impl ColorPickerPopover {
    pub fn new(canvas: CanvasWidget, initial_color: Color, initial_mode: usize) -> Self {
        let (h, s, v) = initial_color.to_hsv();
        let color = Rc::new(Cell::new(initial_color));
        let hue = Rc::new(Cell::new(h));
        let sat = Rc::new(Cell::new(s));
        let val = Rc::new(Cell::new(v));
        let alpha = Rc::new(Cell::new(initial_color.a));
        let is_updating = Rc::new(Cell::new(false));
        let on_change: Rc<RefCell<Option<Box<dyn Fn(Color)>>>> = Rc::new(RefCell::new(None));
        let on_mode_change: Rc<RefCell<Option<Box<dyn Fn(usize)>>>> = Rc::new(RefCell::new(None));

        let popover = gtk4::Popover::builder()
            .has_arrow(true)
            .css_classes(["color-picker-popover"])
            .build();

        let root_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_start(4)
            .margin_end(4)
            .margin_top(4)
            .margin_bottom(4)
            .width_request(304)
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
                    .spacing(5)
                    .halign(gtk4::Align::Center)
                    .valign(gtk4::Align::Center)
                    .build();
                let img = gtk4::Image::from_icon_name(*icon);
                img.set_pixel_size(12);
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
            .spacing(6)
            .css_classes(["color-picker-card"])
            .build();

        // 2A. Header: [ Title ] ──────── [ RGBA ▾ ]
        let header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .margin_start(4)
            .margin_end(4)
            .margin_top(2)
            .margin_bottom(2)
            .valign(gtk4::Align::Center)
            .build();

        let initial_title = match initial_mode {
            0 => crate::core::gettext("Flat Color"),
            1 => crate::core::gettext("Gradient"),
            2 => crate::core::gettext("Mesh Gradient"),
            3 => crate::core::gettext("Patterns"),
            _ => crate::core::gettext("Flat Color"),
        };

        let title_lbl = gtk4::Label::builder()
            .label(&initial_title)
            .css_classes(["heading"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        header.append(&title_lbl);

        card.append(&header);

        // Top Divider
        let divider_top = gtk4::Box::builder()
            .css_classes(["color-picker-divider"])
            .hexpand(true)
            .build();
        card.append(&divider_top);

        // Pre-create DrawingAreas and Entry for reference by mode panels
        let hue_area = gtk4::DrawingArea::builder()
            .content_width(30)
            .content_height(154)
            .valign(gtk4::Align::Fill)
            .build();

        let sv_area = gtk4::DrawingArea::builder()
            .content_width(220)
            .content_height(154)
            .hexpand(true)
            .valign(gtk4::Align::Fill)
            .build();

        let alpha_area = gtk4::DrawingArea::builder()
            .content_width(260)
            .content_height(30)
            .margin_top(4)
            .margin_bottom(4)
            .valign(gtk4::Align::Center)
            .hexpand(true)
            .build();

        let hex_entry = gtk4::Entry::builder()
            .text(&initial_color.to_hex_rgba())
            .width_chars(9)
            .max_width_chars(10)
            .css_classes(["numeric", "flat"])
            .valign(gtk4::Align::Center)
            .build();

        // ── Mesh Panel (visible in Mesh mode) ──
        let mesh_panel = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_start(4)
            .margin_end(4)
            .margin_top(2)
            .margin_bottom(2)
            .visible(initial_mode == 2)
            .build();

        // 1. Mesh Tool Activation Button
        let mesh_tool_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .build();

        let edit_nodes_btn = gtk4::Button::builder()
            .css_classes(["pill-btn"])
            .hexpand(true)
            .tooltip_text(&crate::core::gettext("Edit Mesh Nodes on Screen"))
            .build();
        let edit_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::Center)
            .build();
        let edit_icon = gtk4::Image::from_icon_name("eyedropper-pick-symbolic");
        let edit_lbl = gtk4::Label::builder()
            .label(&crate::core::gettext("Edit Mesh Nodes on Screen"))
            .build();
        edit_box.append(&edit_icon);
        edit_box.append(&edit_lbl);
        edit_nodes_btn.set_child(Some(&edit_box));

        {
            let canvas_m = canvas.clone();
            let pop_c = popover.clone();
            edit_nodes_btn.connect_clicked(move |_| {
                canvas_m.set_active_tool("mesh_gradient");
                pop_c.popdown();
            });
        }
        mesh_tool_row.append(&edit_nodes_btn);
        mesh_panel.append(&mesh_tool_row);

        // 2. Mesh Themes Row
        let themes_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::Fill)
            .build();

        let mesh_presets = [
            ("Sunset", Color::from_hex("#ff5e3a").unwrap()),
            ("Aurora", Color::from_hex("#00f2fe").unwrap()),
            ("Ocean", Color::from_hex("#009efd").unwrap()),
            ("Neon", Color::from_hex("#f857a6").unwrap()),
        ];

        for (p_name, c1) in mesh_presets {
            let p_btn = gtk4::Button::builder()
                .label(&crate::core::gettext(p_name))
                .css_classes(["flat", "pill-btn"])
                .hexpand(true)
                .build();
            let on_ch = on_change.clone();
            let col_c = color.clone();
            let hue_c = hue.clone();
            let sat_c = sat.clone();
            let val_c = val.clone();
            let is_upd = is_updating.clone();
            let sv_draw = sv_area.clone();
            let hue_draw = hue_area.clone();
            let hex_e = hex_entry.clone();
            p_btn.connect_clicked(move |_| {
                col_c.set(c1);
                let (h, s, v) = c1.to_hsv();
                hue_c.set(h);
                sat_c.set(s);
                val_c.set(v);
                sv_draw.queue_draw();
                hue_draw.queue_draw();
                if !is_upd.get() {
                    is_upd.set(true);
                    hex_e.set_text(&c1.to_hex_rgba());
                    is_upd.set(false);
                }
                if let Some(cb) = on_ch.borrow().as_ref() {
                    cb(c1);
                }
            });
            themes_box.append(&p_btn);
        }
        mesh_panel.append(&themes_box);
        card.append(&mesh_panel);

        // ── Pattern Panel (visible in Pattern mode) ──
        let pattern_panel = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_start(4)
            .margin_end(4)
            .margin_top(2)
            .margin_bottom(2)
            .visible(initial_mode == 3)
            .build();

        let pat_types_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::Fill)
            .build();

        let pattern_types = [
            (crate::core::gettext("Checkerboard"), "view-grid-symbolic"),
            (crate::core::gettext("Dots"), "format-fill-symbolic"),
            (crate::core::gettext("Stripes"), "distribute-vertical-symbolic"),
            (crate::core::gettext("Grid"), "view-grid-symbolic"),
            (crate::core::gettext("Honeycomb"), "lib-patterns-symbolic"),
        ];

        for (pt_name, pt_icon) in pattern_types {
            let p_btn = gtk4::Button::builder()
                .css_classes(["flat", "pill-btn"])
                .hexpand(true)
                .tooltip_text(&pt_name)
                .build();
            let b_content = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(3)
                .halign(gtk4::Align::Center)
                .build();
            let img = gtk4::Image::from_icon_name(pt_icon);
            img.set_pixel_size(12);
            let lbl = gtk4::Label::builder().label(&pt_name).build();
            b_content.append(&img);
            b_content.append(&lbl);
            p_btn.set_child(Some(&b_content));
            pat_types_box.append(&p_btn);
        }
        pattern_panel.append(&pat_types_box);
        card.append(&pattern_panel);

        // Wire mode buttons
        let on_mode_clone = on_mode_change.clone();
        let mode_btns_clone = mode_buttons.clone();
        let canvas_mode = canvas.clone();
        for (i, btn) in mode_buttons.iter().enumerate() {
            let on_mode = on_mode_clone.clone();
            let all_btns = mode_btns_clone.clone();
            let title_c = title_lbl.clone();
            let mesh_p_c = mesh_panel.clone();
            let pat_p_c = pattern_panel.clone();
            let cv = canvas_mode.clone();
            btn.connect_clicked(move |_| {
                for (k, b) in all_btns.iter().enumerate() {
                    if k == i {
                        b.add_css_class("active");
                    } else {
                        b.remove_css_class("active");
                    }
                }
                match i {
                    0 => {
                        title_c.set_text(&crate::core::gettext("Flat Color"));
                        mesh_p_c.set_visible(false);
                        pat_p_c.set_visible(false);
                    }
                    1 => {
                        title_c.set_text(&crate::core::gettext("Gradient"));
                        mesh_p_c.set_visible(false);
                        pat_p_c.set_visible(false);
                        cv.set_active_tool("gradient");
                    }
                    2 => {
                        title_c.set_text(&crate::core::gettext("Mesh Gradient"));
                        mesh_p_c.set_visible(true);
                        pat_p_c.set_visible(false);
                        cv.set_active_tool("mesh_gradient");
                    }
                    3 => {
                        title_c.set_text(&crate::core::gettext("Patterns"));
                        mesh_p_c.set_visible(false);
                        pat_p_c.set_visible(true);
                    }
                    _ => {}
                }
                if let Some(cb) = on_mode.borrow().as_ref() {
                    cb(i);
                }
            });
        }

        root_box.append(&top_modes_box);

        // 2B. Body Row: [ Hue Slider (vertical) ] + [ SV Palette (2D Canvas) ]
        let body_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        // ── Vertical Hue Slider ──
        let hue_clone = hue.clone();
        hue_area.set_draw_func(move |_area, cr, width, height| {
            let w = width as f64;
            let h = height as f64;
            let radius = 10.0;

            cr.save().unwrap();
            draw_rounded_rect(cr, 0.0, 0.0, w, h, radius);
            cr.clip();

            // Rainbow vertical gradient (0° to 360°)
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

            // Subtle border
            draw_rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, radius);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
            cr.set_line_width(1.0);
            let _ = cr.stroke();

            // Thumb capsule handle
            let cur_h = hue_clone.get() as f64;
            let thumb_h = 14.0;
            let thumb_y = (cur_h / 360.0 * (h - thumb_h)).clamp(0.0, h - thumb_h);
            draw_rounded_rect(cr, 2.0, thumb_y, w - 4.0, thumb_h, 6.0);
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
            let radius = 10.0;

            cr.save().unwrap();
            draw_rounded_rect(cr, 0.0, 0.0, w, h, radius);
            cr.clip();

            // Base Pure Hue color
            let cur_h = hue_for_sv.get();
            let pure_hue = Color::from_hsv(cur_h, 1.0, 1.0, 1.0);
            cr.set_source_rgb(pure_hue.r as f64, pure_hue.g as f64, pure_hue.b as f64);
            let _ = cr.paint();

            // Horizontal white-to-transparent gradient (Saturation: Left = 0, Right = 1)
            let sat_grad = cairo::LinearGradient::new(0.0, 0.0, w, 0.0);
            sat_grad.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, 1.0);
            sat_grad.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.0);
            cr.set_source(&sat_grad).unwrap();
            let _ = cr.paint();

            // Vertical transparent-to-black gradient (Value: Top = 1, Bottom = 0)
            let val_grad = cairo::LinearGradient::new(0.0, 0.0, 0.0, h);
            val_grad.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.0);
            val_grad.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 1.0);
            cr.set_source(&val_grad).unwrap();
            let _ = cr.paint();

            cr.restore().unwrap();

            // Subtle border
            draw_rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, radius);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.25);
            cr.set_line_width(1.0);
            let _ = cr.stroke();

            // Draggable Circular Cursor Handle
            let cur_s = sat_for_sv.get() as f64;
            let cur_v = val_for_sv.get() as f64;
            let cursor_x = (cur_s * w).clamp(6.0, w - 6.0);
            let cursor_y = ((1.0 - cur_v) * h).clamp(6.0, h - 6.0);

            // Outer drop shadow ring
            cr.arc(cursor_x, cursor_y, 7.5, 0.0, std::f64::consts::TAU);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
            cr.set_line_width(2.5);
            let _ = cr.stroke();

            // Inner crisp white ring
            cr.arc(cursor_x, cursor_y, 6.0, 0.0, std::f64::consts::TAU);
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            cr.set_line_width(2.2);
            let _ = cr.stroke();
        });

        body_row.append(&sv_area);
        card.append(&body_row);

        // ── 2C. Alpha / Opacity Horizontal Slider ──
        let color_for_alpha = color.clone();
        let alpha_for_alpha = alpha.clone();

        alpha_area.set_draw_func(move |_area, cr, width, height| {
            let w = width as f64;
            let h = height as f64;
            let radius = 10.0;

            cr.save().unwrap();
            draw_rounded_rect(cr, 0.0, 0.0, w, h, radius);
            cr.clip();

            // Dark checkerboard transparency pattern
            let check_size = 5.0;
            let cols = (w / check_size).ceil() as usize;
            let rows = (h / check_size).ceil() as usize;
            for r in 0..rows {
                for c in 0..cols {
                    if (r + c) % 2 == 0 {
                        cr.set_source_rgb(0.75, 0.75, 0.75);
                    } else {
                        cr.set_source_rgb(0.55, 0.55, 0.55);
                    }
                    cr.rectangle(
                        c as f64 * check_size,
                        r as f64 * check_size,
                        check_size,
                        check_size,
                    );
                    let _ = cr.fill();
                }
            }

            // Alpha gradient of current color
            let cur_c = color_for_alpha.get();
            let alpha_grad = cairo::LinearGradient::new(0.0, 0.0, w, 0.0);
            alpha_grad.add_color_stop_rgba(
                0.0,
                cur_c.r as f64,
                cur_c.g as f64,
                cur_c.b as f64,
                0.0,
            );
            alpha_grad.add_color_stop_rgba(
                1.0,
                cur_c.r as f64,
                cur_c.g as f64,
                cur_c.b as f64,
                1.0,
            );
            cr.set_source(&alpha_grad).unwrap();
            let _ = cr.paint();

            cr.restore().unwrap();

            // Border
            draw_rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, radius);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.3);
            cr.set_line_width(1.0);
            let _ = cr.stroke();

            // Centered percentage text with shadow
            let cur_a = alpha_for_alpha.get();
            let pct_text = format!("{}%", (cur_a * 100.0).round() as i32);
            cr.set_font_size(12.0);
            let ext = cr.text_extents(&pct_text).unwrap();
            let tx = (w - ext.width()) / 2.0 - ext.x_bearing();
            let ty = (h - ext.height()) / 2.0 - ext.y_bearing();

            // Text shadow
            cr.move_to(tx + 1.0, ty + 1.0);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.7);
            let _ = cr.show_text(&pct_text);

            // Text main
            cr.move_to(tx, ty);
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            let _ = cr.show_text(&pct_text);

            // Thumb slider handle
            let thumb_w = 14.0;
            let thumb_x = (cur_a as f64 * (w - thumb_w)).clamp(0.0, w - thumb_w);
            draw_rounded_rect(cr, thumb_x, 2.0, thumb_w, h - 4.0, 6.0);
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
            cr.set_line_width(1.2);
            let _ = cr.stroke();
        });

        card.append(&alpha_area);

        // Bottom Divider
        let divider_bottom = gtk4::Box::builder()
            .css_classes(["color-picker-divider"])
            .hexpand(true)
            .build();
        card.append(&divider_bottom);

        // ── 2D. Footer: [ #000000ff 💉 ] ──
        let footer = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .margin_start(4)
            .margin_end(4)
            .margin_top(4)
            .margin_bottom(2)
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

        root_box.append(&card);
        popover.set_child(Some(&root_box));

        let instance = Self {
            popover,
            color,
            hue,
            sat,
            val,
            alpha,
            on_change,
            on_mode_change,
            sv_area,
            hue_area,
            alpha_area,
            hex_entry,
            is_updating,
            mode_buttons,
            title_lbl,
            mesh_panel,
            pattern_panel,
        };

        instance.wire_events();
        instance
    }

    fn wire_events(&self) {
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

            if let Some(cb) = on_ch3.borrow().as_ref() {
                cb(new_col);
            }
        };

        let ua_start = update_alpha_at.clone();
        let area_a_start = self.alpha_area.clone();
        alpha_drag.connect_drag_begin(move |_, x, _y| {
            ua_start(&area_a_start, x);
        });

        let ua_update = update_alpha_at.clone();
        let area_a_update = self.alpha_area.clone();
        alpha_drag.connect_drag_update(move |gesture, offset_x, _offset_y| {
            if let Some((start_x, _)) = gesture.start_point() {
                ua_update(&area_a_update, start_x + offset_x);
            }
        });

        self.alpha_area.add_controller(alpha_drag);

        // ── Hex Entry Live Updates & Activation ──
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

        let handle_hex = move |txt: &str| {
            if is_upd4.get() {
                return;
            }
            let trimmed = txt.trim();
            if let Some(parsed) = Color::from_hex(trimmed) {
                let (h, s, v) = parsed.to_hsv();
                hue_c4.set(h);
                sat_c4.set(s);
                val_c4.set(v);
                alpha_c4.set(parsed.a);
                col_c4.set(parsed);

                hue_redraw4.queue_draw();
                sv_redraw4.queue_draw();
                alpha_redraw4.queue_draw();

                if let Some(cb) = on_ch4.borrow().as_ref() {
                    cb(parsed);
                }
            }
        };

        let hh_act = handle_hex.clone();
        self.hex_entry.connect_activate(move |entry| {
            hh_act(&entry.text());
        });

        let hh_chg = handle_hex.clone();
        self.hex_entry.connect_changed(move |entry| {
            let txt = entry.text();
            let clean_len = txt.trim().trim_start_matches('#').len();
            if clean_len == 3 || clean_len == 4 || clean_len == 6 || clean_len == 8 {
                hh_chg(&txt);
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
    }

    pub fn on_color_changed<F: Fn(Color) + 'static>(&self, callback: F) {
        *self.on_change.borrow_mut() = Some(Box::new(callback));
    }

    #[allow(dead_code)]
    pub fn on_mode_changed<F: Fn(usize) + 'static>(&self, callback: F) {
        *self.on_mode_change.borrow_mut() = Some(Box::new(callback));
    }

    pub fn set_mode(&self, mode: usize) {
        for (k, b) in self.mode_buttons.iter().enumerate() {
            if k == mode {
                b.add_css_class("active");
            } else {
                b.remove_css_class("active");
            }
        }
        match mode {
            0 => {
                self.title_lbl.set_text(&crate::core::gettext("Flat Color"));
                self.mesh_panel.set_visible(false);
                self.pattern_panel.set_visible(false);
            }
            1 => {
                self.title_lbl.set_text(&crate::core::gettext("Gradient"));
                self.mesh_panel.set_visible(false);
                self.pattern_panel.set_visible(false);
            }
            2 => {
                self.title_lbl
                    .set_text(&crate::core::gettext("Mesh Gradient"));
                self.mesh_panel.set_visible(true);
                self.pattern_panel.set_visible(false);
            }
            3 => {
                self.title_lbl.set_text(&crate::core::gettext("Patterns"));
                self.mesh_panel.set_visible(false);
                self.pattern_panel.set_visible(true);
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

    #[allow(dead_code)]
    pub fn unparent(&self) {
        if self.popover.parent().is_some() {
            self.popover.unparent();
        }
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
