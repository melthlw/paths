use gtk4::prelude::*;

use super::helpers::{create_coord_entry, create_resource_btn, create_resource_toggle_btn};
use crate::ui::canvas::CanvasWidget;

pub struct SelectControls {
    pub general_box: gtk4::Box,
    pub layer_box: gtk4::Box,
    pub transform_modes_box: gtk4::Box,
    pub btn_convert_path: gtk4::Button,
    pub right_capsule: gtk4::Box,
    pub x_entry: gtk4::Entry,
    pub y_entry: gtk4::Entry,
    pub w_entry: gtk4::Entry,
    pub h_entry: gtk4::Entry,
    pub unit_dd: gtk4::DropDown,
}

pub fn build_select_controls(canvas: &CanvasWidget) -> SelectControls {
    // General selection tools container (Rotate, Layer Order, Convert to Path, Boolean Ops, Transform Modes)
    let general_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .visible(false)
        .build();

    // 2. Rotate CCW (90° Anti-horário)
    let btn_rot_ccw = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/edit-undo.svg",
        &crate::core::gettext("Rotate 90° CCW"),
    );
    let canvas_ccw = canvas.clone();
    btn_rot_ccw.connect_clicked(move |_| {
        canvas_ccw.rotate_selected_deg(-90.0);
    });
    general_box.append(&btn_rot_ccw);

    // 3. Rotate CW (90° Horário)
    let btn_rot_cw = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/edit-redo.svg",
        &crate::core::gettext("Rotate 90° CW"),
    );
    let canvas_cw = canvas.clone();
    btn_rot_cw.connect_clicked(move |_| {
        canvas_cw.rotate_selected_deg(90.0);
    });
    general_box.append(&btn_rot_cw);

    // 4-7. Layer Ordering Box (Visible when elements are selected)
    let layer_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .visible(false)
        .build();

    let btn_front = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/layer-bring-to-front.svg",
        &crate::core::gettext("Bring to Front"),
    );
    let canvas_front = canvas.clone();
    btn_front.connect_clicked(move |_| {
        canvas_front.bring_to_front();
    });
    layer_box.append(&btn_front);

    let btn_fwd = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/layer-bring-forward.svg",
        &crate::core::gettext("Bring Forward"),
    );
    let canvas_fwd = canvas.clone();
    btn_fwd.connect_clicked(move |_| {
        canvas_fwd.bring_forward();
    });
    layer_box.append(&btn_fwd);

    let btn_back = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/layer-send-backward.svg",
        &crate::core::gettext("Send Backward"),
    );
    let canvas_back = canvas.clone();
    btn_back.connect_clicked(move |_| {
        canvas_back.send_backward();
    });
    layer_box.append(&btn_back);

    let btn_bottom = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/layer-send-to-back.svg",
        &crate::core::gettext("Send to Back"),
    );
    let canvas_bottom = canvas.clone();
    btn_bottom.connect_clicked(move |_| {
        canvas_bottom.send_to_back();
    });
    layer_box.append(&btn_bottom);

    general_box.append(&layer_box);

    // Convert to Path button
    let btn_convert_path = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/object-to-path.svg",
        &crate::core::gettext("Convert to Path (Ctrl+Shift+C)"),
    );
    let canvas_conv = canvas.clone();
    btn_convert_path.connect_clicked(move |_| {
        canvas_conv.convert_selected_to_path();
    });
    btn_convert_path.set_visible(false);
    general_box.append(&btn_convert_path);

    // Transform Modes Box (Scale Stroke, Scale Corners, Move Gradients, Move Patterns)
    let transform_modes_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .visible(true)
        .build();

    let cur_opts = canvas.transform_options();

    // 1. Scale Stroke Width
    let btn_scale_stroke = create_resource_toggle_btn(
        "/io/github/lewis/GnomePaths/icons/corner-sharp.svg",
        &crate::core::gettext("When scaling objects, scale the stroke width in the same proportion"),
        cur_opts.scale_stroke_width,
    );
    let canvas_ss = canvas.clone();
    btn_scale_stroke.connect_toggled(move |btn| {
        canvas_ss.set_scale_stroke_width(btn.is_active());
    });
    transform_modes_box.append(&btn_scale_stroke);

    // 2. Scale Corner Radii
    let btn_scale_corners = create_resource_toggle_btn(
        "/io/github/lewis/GnomePaths/icons/corner-round.svg",
        &crate::core::gettext("When scaling rectangles, scale the radii of rounded corners in the same proportion"),
        cur_opts.scale_corner_radii,
    );
    let canvas_sc = canvas.clone();
    btn_scale_corners.connect_toggled(move |btn| {
        canvas_sc.set_scale_corner_radii(btn.is_active());
    });
    transform_modes_box.append(&btn_scale_corners);

    // 3. Move Gradients
    let btn_move_gradients = create_resource_toggle_btn(
        "/io/github/lewis/GnomePaths/icons/tool-gradient.svg",
        &crate::core::gettext("Move gradients (in fill or stroke) along with the objects"),
        cur_opts.move_gradients,
    );
    let canvas_mg = canvas.clone();
    btn_move_gradients.connect_toggled(move |btn| {
        canvas_mg.set_move_gradients(btn.is_active());
    });
    transform_modes_box.append(&btn_move_gradients);

    // 4. Move Patterns
    let btn_move_patterns = create_resource_toggle_btn(
        "/io/github/lewis/GnomePaths/icons/format-fill.svg",
        &crate::core::gettext("Move patterns (in fill or stroke) along with the objects"),
        cur_opts.move_patterns,
    );
    let canvas_mp = canvas.clone();
    btn_move_patterns.connect_toggled(move |btn| {
        canvas_mp.set_move_patterns(btn.is_active());
    });
    transform_modes_box.append(&btn_move_patterns);

    general_box.append(&transform_modes_box);

    // RIGHT CAPSULE: [X] [Y] [Flip H] [Flip V] [W] [Lock] [H]
    let right_capsule = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .css_classes(["toolbar", "card"])
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    // X Coordinate Entry
    let (x_box, x_entry) = create_coord_entry("X");
    let canvas_x = canvas.clone();
    x_entry.connect_activate(move |entry| {
        let text = entry.text();
        let unit = canvas_x.unit();
        let base = canvas_x
            .state()
            .try_borrow()
            .ok()
            .and_then(|s| s.document.selection_bounds())
            .map(|b| b.x);
        if let Ok(val) = crate::core::eval_math_expression(text.as_str(), unit, base) {
            canvas_x.set_selected_x(val);
            entry.set_text(&unit.format(val));
        }
    });
    right_capsule.append(&x_box);

    // Y Coordinate Entry
    let (y_box, y_entry) = create_coord_entry("Y");
    let canvas_y = canvas.clone();
    y_entry.connect_activate(move |entry| {
        let text = entry.text();
        let unit = canvas_y.unit();
        let base = canvas_y
            .state()
            .try_borrow()
            .ok()
            .and_then(|s| s.document.selection_bounds())
            .map(|b| b.y);
        if let Ok(val) = crate::core::eval_math_expression(text.as_str(), unit, base) {
            canvas_y.set_selected_y(val);
            entry.set_text(&unit.format(val));
        }
    });
    right_capsule.append(&y_box);

    // Espelhar Horizontalmente (Mirror Horizontal)
    let btn_flip_h = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/mirror-horizontal.svg",
        &crate::core::gettext("Flip Horizontal"),
    );
    let canvas_fliph = canvas.clone();
    btn_flip_h.connect_clicked(move |_| {
        canvas_fliph.flip_horizontal();
    });
    right_capsule.append(&btn_flip_h);

    // Espelhar Verticalmente (Mirror Vertical)
    let btn_flip_v = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/mirror-vertical.svg",
        &crate::core::gettext("Flip Vertical"),
    );
    let canvas_flipv = canvas.clone();
    btn_flip_v.connect_clicked(move |_| {
        canvas_flipv.flip_vertical();
    });
    right_capsule.append(&btn_flip_v);

    // Width Entry
    let (w_box, w_entry) = create_coord_entry("W");
    let canvas_w = canvas.clone();

    // Aspect Ratio Lock Toggle
    let img_lock = gtk4::Image::from_icon_name("changes-prevent-symbolic");
    img_lock.set_pixel_size(18);
    let lock_btn = gtk4::ToggleButton::builder()
        .child(&img_lock)
        .tooltip_text(crate::core::gettext("Lock Aspect Ratio"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let lock_clone_w = lock_btn.clone();
    w_entry.connect_activate(move |entry| {
        let text = entry.text();
        let unit = canvas_w.unit();
        let base = canvas_w
            .state()
            .try_borrow()
            .ok()
            .and_then(|s| s.document.selection_bounds())
            .map(|b| b.width);
        if let Ok(val) = crate::core::eval_math_expression(text.as_str(), unit, base) {
            canvas_w.set_selected_width(val, lock_clone_w.is_active());
            entry.set_text(&unit.format(val));
        }
    });
    right_capsule.append(&w_box);
    right_capsule.append(&lock_btn);

    // Height Entry
    let (h_box, h_entry) = create_coord_entry("H");
    let canvas_h = canvas.clone();
    let lock_clone_h = lock_btn;
    h_entry.connect_activate(move |entry| {
        let text = entry.text();
        let unit = canvas_h.unit();
        let base = canvas_h
            .state()
            .try_borrow()
            .ok()
            .and_then(|s| s.document.selection_bounds())
            .map(|b| b.height);
        if let Ok(val) = crate::core::eval_math_expression(text.as_str(), unit, base) {
            canvas_h.set_selected_height(val, lock_clone_h.is_active());
            entry.set_text(&unit.format(val));
        }
    });
    right_capsule.append(&h_box);

    let sep_u = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    right_capsule.append(&sep_u);

    let unit_suffixes: Vec<&str> = crate::core::Unit::ALL
        .iter()
        .map(|u| u.suffix())
        .collect();
    let unit_strings = gtk4::StringList::new(&unit_suffixes);
    let unit_dd = gtk4::DropDown::builder()
        .model(&unit_strings)
        .selected(canvas.unit().to_index())
        .tooltip_text(crate::core::gettext("Document Unit (px, mm, cm, m, in, pt, pc)"))
        .valign(gtk4::Align::Center)
        .css_classes(["flat"])
        .build();

    let canvas_u = canvas.clone();
    unit_dd.connect_selected_notify(move |dd| {
        let unit = crate::core::Unit::from_index(dd.selected());
        if canvas_u.unit() != unit {
            canvas_u.set_unit(unit);
        }
    });
    right_capsule.append(&unit_dd);

    SelectControls {
        general_box,
        layer_box,
        transform_modes_box,
        btn_convert_path,
        right_capsule,
        x_entry,
        y_entry,
        w_entry,
        h_entry,
        unit_dd,
    }
}
