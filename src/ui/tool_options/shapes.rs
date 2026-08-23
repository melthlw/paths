use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use super::helpers::create_resource_btn;
use crate::ui::canvas::CanvasWidget;

pub struct ShapeControls {
    pub shape_options_box: gtk4::Box,
    pub shape_sep: gtk4::Separator,
    pub rect_radius_box: gtk4::Box,
    pub rect_r_lbl: gtk4::Label,
    pub rect_radius_spin: gtk4::SpinButton,
    pub rect_split_btn: gtk4::ToggleButton,
    pub rect_indiv_box: gtk4::Box,
    pub rect_tl_spin: gtk4::SpinButton,
    pub rect_tr_spin: gtk4::SpinButton,
    pub rect_br_spin: gtk4::SpinButton,
    pub rect_bl_spin: gtk4::SpinButton,
    pub rect_style_dd: gtk4::DropDown,
    pub star_box: gtk4::Box,
    pub star_points_spin: gtk4::SpinButton,
    pub star_ratio_spin: gtk4::SpinButton,
    pub star_radius_spin: gtk4::SpinButton,
    pub polygon_box: gtk4::Box,
    pub polygon_sides_spin: gtk4::SpinButton,
    pub polygon_radius_spin: gtk4::SpinButton,
    pub circle_box: gtk4::Box,
    pub arc_btn_full: gtk4::ToggleButton,
    pub arc_btn_arc: gtk4::ToggleButton,
    pub arc_btn_seg: gtk4::ToggleButton,
    pub arc_btn_chord: gtk4::ToggleButton,
    pub circle_start_spin: gtk4::SpinButton,
    pub circle_end_spin: gtk4::SpinButton,
    pub spiral_box: gtk4::Box,
    pub spiral_turns_spin: gtk4::SpinButton,
    pub spiral_div_spin: gtk4::SpinButton,
    pub spiral_inner_spin: gtk4::SpinButton,
    pub boolean_box: gtk4::Box,
}

pub fn build_shape_controls(canvas: &CanvasWidget, is_syncing: &Rc<Cell<bool>>) -> ShapeControls {
    let shape_options_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let shape_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    shape_options_box.append(&shape_sep);

    // ── Rectangle: Corner Radius, Independent 4-Corners, Corner Style ──
    let rect_radius_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let rect_r_lbl = gtk4::Label::builder()
        .label("◰")
        .css_classes(["dim-label"])
        .build();
    let rect_radius_spin = gtk4::SpinButton::with_range(0.0, 200.0, 1.0);
    rect_radius_spin.set_value(0.0);
    rect_radius_spin.set_tooltip_text(Some(&crate::core::gettext("Corner Radius")));
    rect_radius_spin.set_valign(gtk4::Align::Center);

    let rect_split_btn = gtk4::ToggleButton::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text(crate::core::gettext("Edit corners individually"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .focus_on_click(false)
        .build();

    let rect_indiv_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let rect_tl_spin = gtk4::SpinButton::with_range(0.0, 200.0, 1.0);
    rect_tl_spin.set_value(0.0);
    rect_tl_spin.set_tooltip_text(Some(&format!("◤ {}", crate::core::gettext("Top-Left Radius"))));
    rect_tl_spin.set_valign(gtk4::Align::Center);

    let rect_tr_spin = gtk4::SpinButton::with_range(0.0, 200.0, 1.0);
    rect_tr_spin.set_value(0.0);
    rect_tr_spin.set_tooltip_text(Some(&format!("◥ {}", crate::core::gettext("Top-Right Radius"))));
    rect_tr_spin.set_valign(gtk4::Align::Center);

    let rect_br_spin = gtk4::SpinButton::with_range(0.0, 200.0, 1.0);
    rect_br_spin.set_value(0.0);
    rect_br_spin.set_tooltip_text(Some(&format!("◢ {}", crate::core::gettext("Bottom-Right Radius"))));
    rect_br_spin.set_valign(gtk4::Align::Center);

    let rect_bl_spin = gtk4::SpinButton::with_range(0.0, 200.0, 1.0);
    rect_bl_spin.set_value(0.0);
    rect_bl_spin.set_tooltip_text(Some(&format!("◣ {}", crate::core::gettext("Bottom-Left Radius"))));
    rect_bl_spin.set_valign(gtk4::Align::Center);

    let tl_lbl = gtk4::Label::builder().label("◤").css_classes(["dim-label", "caption"]).build();
    let tr_lbl = gtk4::Label::builder().label("◥").css_classes(["dim-label", "caption"]).build();
    let br_lbl = gtk4::Label::builder().label("◢").css_classes(["dim-label", "caption"]).build();
    let bl_lbl = gtk4::Label::builder().label("◣").css_classes(["dim-label", "caption"]).build();

    rect_indiv_box.append(&tl_lbl);
    rect_indiv_box.append(&rect_tl_spin);
    rect_indiv_box.append(&tr_lbl);
    rect_indiv_box.append(&rect_tr_spin);
    rect_indiv_box.append(&br_lbl);
    rect_indiv_box.append(&rect_br_spin);
    rect_indiv_box.append(&bl_lbl);
    rect_indiv_box.append(&rect_bl_spin);

    let style_model = gtk4::StringList::new(&[
        &crate::core::gettext("Round"),
        &crate::core::gettext("Chamfer"),
        &crate::core::gettext("Concave"),
    ]);
    let rect_style_dd = gtk4::DropDown::builder()
        .model(&style_model)
        .selected(0)
        .tooltip_text(crate::core::gettext("Corner Type"))
        .valign(gtk4::Align::Center)
        .build();

    rect_radius_box.append(&rect_r_lbl);
    rect_radius_box.append(&rect_radius_spin);
    rect_radius_box.append(&rect_split_btn);
    rect_radius_box.append(&rect_indiv_box);
    rect_radius_box.append(&rect_style_dd);

    // Wiring signals
    {
        let indiv_b = rect_indiv_box.clone();
        let r_spin = rect_radius_spin.clone();
        let r_lbl = rect_r_lbl.clone();
        rect_split_btn.connect_toggled(move |btn| {
            let is_indiv = btn.is_active();
            indiv_b.set_visible(is_indiv);
            r_spin.set_visible(!is_indiv);
            r_lbl.set_visible(!is_indiv);
        });
    }

    {
        let canvas_rr = canvas.clone();
        let syncing_rr = is_syncing.clone();
        let tl_s = rect_tl_spin.clone();
        let tr_s = rect_tr_spin.clone();
        let br_s = rect_br_spin.clone();
        let bl_s = rect_bl_spin.clone();
        rect_radius_spin.connect_value_changed(move |spin| {
            if syncing_rr.get() {
                return;
            }
            let val = spin.value() as f32;
            syncing_rr.set(true);
            tl_s.set_value(val as f64);
            tr_s.set_value(val as f64);
            br_s.set_value(val as f64);
            bl_s.set_value(val as f64);
            syncing_rr.set(false);
            canvas_rr.set_rect_corner_radius(val);
        });
    }

    {
        let canvas_indiv = canvas.clone();
        let syncing_indiv = is_syncing.clone();
        let tl_s = rect_tl_spin.clone();
        let tr_s = rect_tr_spin.clone();
        let br_s = rect_br_spin.clone();
        let bl_s = rect_bl_spin.clone();
        let r_spin = rect_radius_spin.clone();

        let update_indiv = move || {
            if syncing_indiv.get() {
                return;
            }
            let tl = tl_s.value() as f32;
            let tr = tr_s.value() as f32;
            let br = br_s.value() as f32;
            let bl = bl_s.value() as f32;
            let radii = crate::core::CornerRadii::new(tl, tr, br, bl);
            syncing_indiv.set(true);
            r_spin.set_value(radii.max_radius() as f64);
            syncing_indiv.set(false);
            canvas_indiv.set_rect_corner_radii(radii);
        };

        let cb1 = update_indiv.clone();
        rect_tl_spin.connect_value_changed(move |_| cb1());
        let cb2 = update_indiv.clone();
        rect_tr_spin.connect_value_changed(move |_| cb2());
        let cb3 = update_indiv.clone();
        rect_br_spin.connect_value_changed(move |_| cb3());
        let cb4 = update_indiv;
        rect_bl_spin.connect_value_changed(move |_| cb4());
    }

    {
        let canvas_style = canvas.clone();
        let syncing_st = is_syncing.clone();
        rect_style_dd.connect_selected_notify(move |dd| {
            if syncing_st.get() {
                return;
            }
            let style = match dd.selected() {
                1 => crate::core::CornerStyle::Chamfer,
                2 => crate::core::CornerStyle::Concave,
                _ => crate::core::CornerStyle::Round,
            };
            canvas_style.set_rect_corner_style(style);
        });
    }
    shape_options_box.append(&rect_radius_box);

    // ── Star: Points, Inner Ratio, Corner Radius ──
    let star_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let star_pts_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Points"))
        .css_classes(["dim-label"])
        .build();
    let star_points_spin = gtk4::SpinButton::with_range(3.0, 20.0, 1.0);
    star_points_spin.set_value(5.0);
    star_points_spin.set_tooltip_text(Some(&crate::core::gettext("Points")));
    star_points_spin.set_valign(gtk4::Align::Center);
    star_box.append(&star_pts_lbl);
    star_box.append(&star_points_spin);

    let star_ratio_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Ratio"))
        .css_classes(["dim-label"])
        .build();
    let star_ratio_spin = gtk4::SpinButton::with_range(0.1, 0.95, 0.05);
    star_ratio_spin.set_digits(2);
    star_ratio_spin.set_value(0.45);
    star_ratio_spin.set_tooltip_text(Some(&crate::core::gettext("Radius Ratio")));
    star_ratio_spin.set_valign(gtk4::Align::Center);
    star_box.append(&star_ratio_lbl);
    star_box.append(&star_ratio_spin);

    let star_r_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Radius"))
        .css_classes(["dim-label"])
        .build();
    let star_radius_spin = gtk4::SpinButton::with_range(0.0, 100.0, 1.0);
    star_radius_spin.set_value(0.0);
    star_radius_spin.set_tooltip_text(Some(&crate::core::gettext("Corner Rounding")));
    star_radius_spin.set_valign(gtk4::Align::Center);
    star_box.append(&star_r_lbl);
    star_box.append(&star_radius_spin);

    let canvas_star = canvas.clone();
    let syncing_star = is_syncing.clone();
    let star_pts_s = star_points_spin.clone();
    let star_ratio_s = star_ratio_spin.clone();
    let star_rad_s = star_radius_spin.clone();
    let star_cb = move || {
        if syncing_star.get() {
            return;
        }
        canvas_star.update_shape_origin(crate::core::ShapeOrigin::Star {
            points: star_pts_s.value() as u32,
            inner_ratio: star_ratio_s.value() as f32,
            corner_radius: star_rad_s.value() as f32,
        });
    };
    let cb1 = star_cb.clone();
    star_points_spin.connect_value_changed(move |_| {
        cb1();
    });
    let cb2 = star_cb.clone();
    star_ratio_spin.connect_value_changed(move |_| {
        cb2();
    });
    let cb3 = star_cb;
    star_radius_spin.connect_value_changed(move |_| {
        cb3();
    });

    shape_options_box.append(&star_box);

    // ── Polygon/Triangle: Sides, Corner Radius ──
    let polygon_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let poly_sides_lbl = gtk4::Label::builder()
        .label("⬠")
        .css_classes(["dim-label"])
        .build();
    let polygon_sides_spin = gtk4::SpinButton::with_range(3.0, 12.0, 1.0);
    polygon_sides_spin.set_value(3.0);
    polygon_sides_spin.set_tooltip_text(Some(&crate::core::gettext("Sides")));
    polygon_sides_spin.set_valign(gtk4::Align::Center);
    polygon_box.append(&poly_sides_lbl);
    polygon_box.append(&polygon_sides_spin);

    let poly_r_lbl = gtk4::Label::builder()
        .label("◰")
        .css_classes(["dim-label"])
        .build();
    let polygon_radius_spin = gtk4::SpinButton::with_range(0.0, 100.0, 1.0);
    polygon_radius_spin.set_value(0.0);
    polygon_radius_spin.set_tooltip_text(Some(&crate::core::gettext("Corner Rounding")));
    polygon_radius_spin.set_valign(gtk4::Align::Center);
    polygon_box.append(&poly_r_lbl);
    polygon_box.append(&polygon_radius_spin);

    let canvas_poly = canvas.clone();
    let syncing_poly = is_syncing.clone();
    let poly_sides_s = polygon_sides_spin.clone();
    let poly_rad_s = polygon_radius_spin.clone();
    let poly_cb = move || {
        if syncing_poly.get() {
            return;
        }
        canvas_poly.update_shape_origin(crate::core::ShapeOrigin::Triangle {
            sides: poly_sides_s.value() as u32,
            corner_radius: poly_rad_s.value() as f32,
        });
    };
    let pcb1 = poly_cb.clone();
    polygon_sides_spin.connect_value_changed(move |_| {
        pcb1();
    });
    let pcb2 = poly_cb;
    polygon_radius_spin.connect_value_changed(move |_| {
        pcb2();
    });

    shape_options_box.append(&polygon_box);

    // ── Circle: Arc Mode, Start/End Angles ──
    let circle_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let arc_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["linked"])
        .valign(gtk4::Align::Center)
        .build();

    let arc_btn_full = gtk4::ToggleButton::builder()
        .label(crate::core::gettext("Full"))
        .tooltip_text(crate::core::gettext("Full Circle"))
        .active(true)
        .build();

    let arc_btn_arc = gtk4::ToggleButton::builder()
        .label(crate::core::gettext("Arc"))
        .tooltip_text(crate::core::gettext("Arc"))
        .group(&arc_btn_full)
        .build();

    let arc_btn_seg = gtk4::ToggleButton::builder()
        .label(crate::core::gettext("Slice"))
        .tooltip_text(crate::core::gettext("Slice"))
        .group(&arc_btn_full)
        .build();

    let arc_btn_chord = gtk4::ToggleButton::builder()
        .label(crate::core::gettext("Chord"))
        .tooltip_text(crate::core::gettext("Chord"))
        .group(&arc_btn_full)
        .build();

    arc_box.append(&arc_btn_full);
    arc_box.append(&arc_btn_arc);
    arc_box.append(&arc_btn_seg);
    arc_box.append(&arc_btn_chord);
    circle_box.append(&arc_box);

    let circ_start_lbl = gtk4::Label::builder()
        .label("↻")
        .css_classes(["dim-label"])
        .build();
    let circle_start_spin = gtk4::SpinButton::with_range(0.0, 360.0, 1.0);
    circle_start_spin.set_value(0.0);
    circle_start_spin.set_tooltip_text(Some(&crate::core::gettext("Start Angle (°)")));
    circle_start_spin.set_valign(gtk4::Align::Center);
    circle_box.append(&circ_start_lbl);
    circle_box.append(&circle_start_spin);

    let circ_end_lbl = gtk4::Label::builder()
        .label("↺")
        .css_classes(["dim-label"])
        .build();
    let circle_end_spin = gtk4::SpinButton::with_range(0.0, 360.0, 1.0);
    circle_end_spin.set_value(360.0);
    circle_end_spin.set_tooltip_text(Some(&crate::core::gettext("End Angle (°)")));
    circle_end_spin.set_valign(gtk4::Align::Center);
    circle_box.append(&circ_end_lbl);
    circle_box.append(&circle_end_spin);

    let canvas_circ = canvas.clone();
    let syncing_circ = is_syncing.clone();
    let btn_f = arc_btn_full.clone();
    let btn_a = arc_btn_arc.clone();
    let btn_s = arc_btn_seg.clone();
    let circ_start_s = circle_start_spin.clone();
    let circ_end_s = circle_end_spin.clone();
    let circ_cb = move || {
        if syncing_circ.get() {
            return;
        }
        let mode = if btn_f.is_active() {
            crate::core::ArcMode::Full
        } else if btn_a.is_active() {
            crate::core::ArcMode::Arc
        } else if btn_s.is_active() {
            crate::core::ArcMode::Segment
        } else {
            crate::core::ArcMode::Chord
        };
        canvas_circ.update_shape_origin(crate::core::ShapeOrigin::Circle {
            arc_mode: mode,
            start_angle: circ_start_s.value() as f32,
            end_angle: circ_end_s.value() as f32,
        });
    };

    let c1 = circ_cb.clone();
    arc_btn_full.connect_toggled(move |b| {
        if b.is_active() {
            c1();
        }
    });
    let c2 = circ_cb.clone();
    arc_btn_arc.connect_toggled(move |b| {
        if b.is_active() {
            c2();
        }
    });
    let c3 = circ_cb.clone();
    arc_btn_seg.connect_toggled(move |b| {
        if b.is_active() {
            c3();
        }
    });
    let c4 = circ_cb.clone();
    arc_btn_chord.connect_toggled(move |b| {
        if b.is_active() {
            c4();
        }
    });
    let c5 = circ_cb.clone();
    circle_start_spin.connect_value_changed(move |_| {
        c5();
    });
    let c6 = circ_cb;
    circle_end_spin.connect_value_changed(move |_| {
        c6();
    });

    shape_options_box.append(&circle_box);

    // ── Spiral: Turns, Divergence, Inner Radius ──
    let spiral_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let spiral_turns_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Turns"))
        .css_classes(["dim-label"])
        .build();
    let spiral_turns_spin = gtk4::SpinButton::with_range(0.5, 20.0, 0.5);
    spiral_turns_spin.set_digits(1);
    spiral_turns_spin.set_value(3.0);
    spiral_turns_spin.set_tooltip_text(Some(&crate::core::gettext("Number of Turns")));
    spiral_turns_spin.set_valign(gtk4::Align::Center);
    spiral_box.append(&spiral_turns_lbl);
    spiral_box.append(&spiral_turns_spin);

    let spiral_div_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Divergence"))
        .css_classes(["dim-label"])
        .build();
    let spiral_div_spin = gtk4::SpinButton::with_range(0.1, 5.0, 0.1);
    spiral_div_spin.set_digits(1);
    spiral_div_spin.set_value(1.0);
    spiral_div_spin.set_tooltip_text(Some(&crate::core::gettext("Divergence Factor")));
    spiral_div_spin.set_valign(gtk4::Align::Center);
    spiral_box.append(&spiral_div_lbl);
    spiral_box.append(&spiral_div_spin);

    let spiral_inner_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Inner Radius"))
        .css_classes(["dim-label"])
        .build();
    let spiral_inner_spin = gtk4::SpinButton::with_range(0.0, 0.95, 0.05);
    spiral_inner_spin.set_digits(2);
    spiral_inner_spin.set_value(0.0);
    spiral_inner_spin.set_tooltip_text(Some(&crate::core::gettext("Inner Radius")));
    spiral_inner_spin.set_valign(gtk4::Align::Center);
    spiral_box.append(&spiral_inner_lbl);
    spiral_box.append(&spiral_inner_spin);

    let canvas_spiral = canvas.clone();
    let syncing_spiral = is_syncing.clone();
    let sp_turns_s = spiral_turns_spin.clone();
    let sp_div_s = spiral_div_spin.clone();
    let sp_inner_s = spiral_inner_spin.clone();
    let spiral_cb = move || {
        if syncing_spiral.get() {
            return;
        }
        canvas_spiral.update_shape_origin(crate::core::ShapeOrigin::Spiral {
            turns: sp_turns_s.value() as f32,
            divergence: sp_div_s.value() as f32,
            inner_radius: sp_inner_s.value() as f32,
        });
    };
    let sp_c1 = spiral_cb.clone();
    spiral_turns_spin.connect_value_changed(move |_| {
        sp_c1();
    });
    let sp_c2 = spiral_cb.clone();
    spiral_div_spin.connect_value_changed(move |_| {
        sp_c2();
    });
    let sp_c3 = spiral_cb;
    spiral_inner_spin.connect_value_changed(move |_| {
        sp_c3();
    });

    shape_options_box.append(&spiral_box);

    // 10-15. Boolean / Path Operations Box (Visible when 2+ elements are selected)
    let boolean_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .visible(false)
        .build();

    let sep_bool = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    boolean_box.append(&sep_bool);

    // 1. União
    let btn_union = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/bool-union.svg",
        &crate::core::gettext("Union (Ctrl++)"),
    );
    let canvas_union = canvas.clone();
    btn_union.connect_clicked(move |_| {
        canvas_union.apply_boolean_operation(crate::core::document::BooleanOperation::Union);
    });
    boolean_box.append(&btn_union);

    // 2. Diferença
    let btn_diff = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/bool-difference.svg",
        &crate::core::gettext("Difference (Ctrl+-)"),
    );
    let canvas_diff = canvas.clone();
    btn_diff.connect_clicked(move |_| {
        canvas_diff
            .apply_boolean_operation(crate::core::document::BooleanOperation::Difference);
    });
    boolean_box.append(&btn_diff);

    // 3. Interseção
    let btn_intersect = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/bool-intersection.svg",
        &crate::core::gettext("Intersection (Ctrl+*)"),
    );
    let canvas_inter = canvas.clone();
    btn_intersect.connect_clicked(move |_| {
        canvas_inter
            .apply_boolean_operation(crate::core::document::BooleanOperation::Intersection);
    });
    boolean_box.append(&btn_intersect);

    // 4. Exclusão
    let btn_excl = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/bool-exclusion.svg",
        &crate::core::gettext("Exclusion (Ctrl+^)"),
    );
    let canvas_excl = canvas.clone();
    btn_excl.connect_clicked(move |_| {
        canvas_excl.apply_boolean_operation(crate::core::document::BooleanOperation::Exclusion);
    });
    boolean_box.append(&btn_excl);

    // 5. Divisão
    let btn_div = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/bool-division.svg",
        &crate::core::gettext("Division (Ctrl+/)"),
    );
    let canvas_div = canvas.clone();
    btn_div.connect_clicked(move |_| {
        canvas_div.apply_boolean_operation(crate::core::document::BooleanOperation::Division);
    });
    boolean_box.append(&btn_div);

    // 6. Cortar
    let btn_cut = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/bool-cut.svg",
        &crate::core::gettext("Cut / Slice (Ctrl+Alt+/)"),
    );
    let canvas_cut = canvas.clone();
    btn_cut.connect_clicked(move |_| {
        canvas_cut.apply_boolean_operation(crate::core::document::BooleanOperation::Cut);
    });
    boolean_box.append(&btn_cut);

    ShapeControls {
        shape_options_box,
        shape_sep,
        rect_radius_box,
        rect_r_lbl,
        rect_radius_spin,
        rect_split_btn,
        rect_indiv_box,
        rect_tl_spin,
        rect_tr_spin,
        rect_br_spin,
        rect_bl_spin,
        rect_style_dd,
        star_box,
        star_points_spin,
        star_ratio_spin,
        star_radius_spin,
        polygon_box,
        polygon_sides_spin,
        polygon_radius_spin,
        circle_box,
        arc_btn_full,
        arc_btn_arc,
        arc_btn_seg,
        arc_btn_chord,
        circle_start_spin,
        circle_end_spin,
        spiral_box,
        spiral_turns_spin,
        spiral_div_spin,
        spiral_inner_spin,
        boolean_box,
    }
}
