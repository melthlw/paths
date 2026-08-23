pub mod helpers;
pub mod pen_brush_text;
pub mod select;
pub mod shapes;

use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use self::pen_brush_text::build_pen_brush_text_controls;
use self::select::build_select_controls;
use self::shapes::build_shape_controls;
use crate::core::TextAlign;
use crate::plugins::manifest::BarPosition;
use crate::ui::canvas::CanvasWidget;

#[derive(Clone)]
pub struct ToolOptionsBar {
    container: gtk4::Box,
    left_capsule: gtk4::Box,
    mode_img: gtk4::Image,
    general_box: gtk4::Box,
    layer_box: gtk4::Box,
    corner_box: gtk4::Box,
    btn_convert_path: gtk4::Button,
    shape_options_box: gtk4::Box,
    shape_sep: gtk4::Separator,
    // Shape-specific controls
    rect_radius_box: gtk4::Box,
    rect_r_lbl: gtk4::Label,
    rect_radius_spin: gtk4::SpinButton,
    rect_split_btn: gtk4::ToggleButton,
    rect_indiv_box: gtk4::Box,
    rect_tl_spin: gtk4::SpinButton,
    rect_tr_spin: gtk4::SpinButton,
    rect_br_spin: gtk4::SpinButton,
    rect_bl_spin: gtk4::SpinButton,
    rect_style_dd: gtk4::DropDown,
    star_box: gtk4::Box,
    star_points_spin: gtk4::SpinButton,
    star_ratio_spin: gtk4::SpinButton,
    star_radius_spin: gtk4::SpinButton,
    polygon_box: gtk4::Box,
    polygon_sides_spin: gtk4::SpinButton,
    polygon_radius_spin: gtk4::SpinButton,
    circle_box: gtk4::Box,
    arc_btn_full: gtk4::ToggleButton,
    arc_btn_arc: gtk4::ToggleButton,
    arc_btn_seg: gtk4::ToggleButton,
    arc_btn_chord: gtk4::ToggleButton,
    circle_start_spin: gtk4::SpinButton,
    circle_end_spin: gtk4::SpinButton,
    spiral_box: gtk4::Box,
    spiral_turns_spin: gtk4::SpinButton,
    spiral_div_spin: gtk4::SpinButton,
    spiral_inner_spin: gtk4::SpinButton,
    boolean_box: gtk4::Box,
    right_capsule: gtk4::Box,
    page_capsule: gtk4::Box,
    page_name_entry: gtk4::Entry,
    page_preset_dd: gtk4::DropDown,
    page_w_spin: gtk4::SpinButton,
    page_h_spin: gtk4::SpinButton,
    btn_del_page: gtk4::Button,
    text_capsule: gtk4::Box,
    system_fonts: Vec<String>,
    font_dd: gtk4::DropDown,
    weight_dd: gtk4::DropDown,
    size_spin: gtk4::SpinButton,
    line_height_spin: gtk4::SpinButton,
    btn_align_left: gtk4::ToggleButton,
    btn_align_center: gtk4::ToggleButton,
    btn_align_right: gtk4::ToggleButton,
    btn_align_justify: gtk4::ToggleButton,
    text_box_w_spin: gtk4::SpinButton,
    letter_spacing_spin: gtk4::SpinButton,
    word_spacing_spin: gtk4::SpinButton,
    x_entry: gtk4::Entry,
    y_entry: gtk4::Entry,
    w_entry: gtk4::Entry,
    h_entry: gtk4::Entry,
    path_editor_box: gtk4::Box,
    is_syncing: Rc<Cell<bool>>,
    is_top: Rc<Cell<bool>>,
}

impl ToolOptionsBar {
    pub fn new(canvas: CanvasWidget) -> Self {
        let is_syncing = Rc::new(Cell::new(false));
        let is_top = Rc::new(Cell::new(true));

        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Start)
            .margin_top(32)
            .build();

        let left_capsule = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(2)
            .css_classes(["toolbar", "card"])
            .valign(gtk4::Align::Center)
            .visible(false)
            .build();

        let mode_img = crate::ui::icons::make_symbolic_image("tool-selection-options", 18);
        mode_img.set_valign(gtk4::Align::Center);
        mode_img.set_margin_start(8);
        mode_img.set_margin_end(4);
        mode_img.set_opacity(0.5);
        mode_img.add_css_class("dim-label");
        mode_img.set_tooltip_text(Some(&crate::core::gettext("Active Tool: Selection")));
        left_capsule.append(&mode_img);

        let mode_sep = gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_start(2)
            .margin_end(2)
            .build();
        left_capsule.append(&mode_sep);

        let select_controls = build_select_controls(&canvas);
        let general_box = select_controls.general_box;
        let layer_box = select_controls.layer_box;
        let corner_box = select_controls.corner_box;
        let btn_convert_path = select_controls.btn_convert_path;
        let right_capsule = select_controls.right_capsule;
        let x_entry = select_controls.x_entry;
        let y_entry = select_controls.y_entry;
        let w_entry = select_controls.w_entry;
        let h_entry = select_controls.h_entry;

        let shape_controls = build_shape_controls(&canvas, &is_syncing);
        let shape_options_box = shape_controls.shape_options_box;
        let shape_sep = shape_controls.shape_sep;
        let rect_radius_box = shape_controls.rect_radius_box;
        let rect_r_lbl = shape_controls.rect_r_lbl;
        let rect_radius_spin = shape_controls.rect_radius_spin;
        let rect_split_btn = shape_controls.rect_split_btn;
        let rect_indiv_box = shape_controls.rect_indiv_box;
        let rect_tl_spin = shape_controls.rect_tl_spin;
        let rect_tr_spin = shape_controls.rect_tr_spin;
        let rect_br_spin = shape_controls.rect_br_spin;
        let rect_bl_spin = shape_controls.rect_bl_spin;
        let rect_style_dd = shape_controls.rect_style_dd;
        let star_box = shape_controls.star_box;
        let star_points_spin = shape_controls.star_points_spin;
        let star_ratio_spin = shape_controls.star_ratio_spin;
        let star_radius_spin = shape_controls.star_radius_spin;
        let polygon_box = shape_controls.polygon_box;
        let polygon_sides_spin = shape_controls.polygon_sides_spin;
        let polygon_radius_spin = shape_controls.polygon_radius_spin;
        let circle_box = shape_controls.circle_box;
        let arc_btn_full = shape_controls.arc_btn_full;
        let arc_btn_arc = shape_controls.arc_btn_arc;
        let arc_btn_seg = shape_controls.arc_btn_seg;
        let arc_btn_chord = shape_controls.arc_btn_chord;
        let circle_start_spin = shape_controls.circle_start_spin;
        let circle_end_spin = shape_controls.circle_end_spin;
        let spiral_box = shape_controls.spiral_box;
        let spiral_turns_spin = shape_controls.spiral_turns_spin;
        let spiral_div_spin = shape_controls.spiral_div_spin;
        let spiral_inner_spin = shape_controls.spiral_inner_spin;
        let boolean_box = shape_controls.boolean_box;

        general_box.append(&boolean_box);

        let pen_text_controls = build_pen_brush_text_controls(&canvas, &is_syncing);
        let path_editor_box = pen_text_controls.path_editor_box;
        let text_capsule = pen_text_controls.text_capsule;
        let system_fonts = pen_text_controls.system_fonts;
        let font_dd = pen_text_controls.font_dd;
        let weight_dd = pen_text_controls.weight_dd;
        let size_spin = pen_text_controls.size_spin;
        let line_height_spin = pen_text_controls.line_height_spin;
        let btn_align_left = pen_text_controls.btn_align_left;
        let btn_align_center = pen_text_controls.btn_align_center;
        let btn_align_right = pen_text_controls.btn_align_right;
        let btn_align_justify = pen_text_controls.btn_align_justify;
        let text_box_w_spin = pen_text_controls.text_box_w_spin;
        let letter_spacing_spin = pen_text_controls.letter_spacing_spin;
        let word_spacing_spin = pen_text_controls.word_spacing_spin;
        let page_capsule = pen_text_controls.page_capsule;
        let page_name_entry = pen_text_controls.page_name_entry;
        let page_preset_dd = pen_text_controls.page_preset_dd;
        let page_w_spin = pen_text_controls.page_w_spin;
        let page_h_spin = pen_text_controls.page_h_spin;
        let btn_del_page = pen_text_controls.btn_del_page;

        left_capsule.append(&general_box);
        left_capsule.append(&shape_options_box);
        left_capsule.append(&path_editor_box);

        container.append(&left_capsule);
        container.append(&text_capsule);
        container.append(&page_capsule);
        container.append(&right_capsule);

        // Position Popover
        let options_popover = gtk4::Popover::builder().has_arrow(true).build();
        let options_menu_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(8)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .width_request(210)
            .build();

        let opt_pos_title = gtk4::Label::builder()
            .label(crate::core::gettext("Tool Options Position"))
            .css_classes(["dim-label", "caption"])
            .halign(gtk4::Align::Start)
            .build();
        options_menu_box.append(&opt_pos_title);

        let opt_pos_linked = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .css_classes(["linked"])
            .homogeneous(true)
            .build();

        let container_pos = container.clone();
        let pop_close_opt = options_popover.clone();

        let make_opt_pos_btn = |icon_name: &'static str, tooltip: String, is_top_val: bool| {
            let btn = gtk4::Button::builder()
                .icon_name(icon_name)
                .tooltip_text(&tooltip)
                .css_classes(["flat"])
                .build();
            let c = container_pos.clone();
            let p = pop_close_opt.clone();
            let is_top_cell = is_top.clone();
            btn.connect_clicked(move |_| {
                is_top_cell.set(is_top_val);
                if is_top_val {
                    c.set_valign(gtk4::Align::Start);
                    c.set_margin_top(32);
                    c.set_margin_bottom(0);
                } else {
                    c.set_valign(gtk4::Align::End);
                    c.set_margin_bottom(96);
                    c.set_margin_top(0);
                }
                p.popdown();
            });
            btn
        };

        let btn_opt_top = make_opt_pos_btn(
            "pan-up-symbolic",
            crate::core::gettext("Top (Default)"),
            true,
        );
        let btn_opt_bottom =
            make_opt_pos_btn("pan-down-symbolic", crate::core::gettext("Bottom"), false);

        opt_pos_linked.append(&btn_opt_top);
        opt_pos_linked.append(&btn_opt_bottom);
        options_menu_box.append(&opt_pos_linked);

        options_popover.set_child(Some(&options_menu_box));
        options_popover.set_parent(&mode_img);
        {
            let pop_c = options_popover.clone();
            mode_img.connect_destroy(move |_| {
                if pop_c.parent().is_some() {
                    pop_c.unparent();
                }
            });
        }

        let gesture_mode_click = gtk4::GestureClick::builder().build();
        let pop_mc = options_popover.clone();
        gesture_mode_click.connect_released(move |_, _, _, _| {
            pop_mc.popup();
        });
        mode_img.add_controller(gesture_mode_click);

        for cap in [&left_capsule, &right_capsule, &page_capsule, &text_capsule] {
            let gesture_right = gtk4::GestureClick::builder().button(3).build();
            let pop_r = options_popover.clone();
            gesture_right.connect_released(move |_, _, _, _| {
                pop_r.popup();
            });
            cap.add_controller(gesture_right);
        }

        Self {
            container,
            left_capsule,
            mode_img,
            general_box,
            layer_box,
            corner_box,
            btn_convert_path,
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
            right_capsule,
            page_capsule,
            page_name_entry,
            page_preset_dd,
            page_w_spin,
            page_h_spin,
            btn_del_page,
            text_capsule,
            system_fonts,
            font_dd,
            weight_dd,
            size_spin,
            line_height_spin,
            btn_align_left,
            btn_align_center,
            btn_align_right,
            btn_align_justify,
            text_box_w_spin,
            letter_spacing_spin,
            word_spacing_spin,
            x_entry,
            y_entry,
            w_entry,
            h_entry,
            path_editor_box,
            is_syncing,
            is_top,
        }
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.container
    }

    pub fn update_state(
        &self,
        tool_id: &str,
        selected_count: usize,
        bounds: Option<crate::core::Rect>,
        is_editing_text: bool,
        text_info: Option<(String, u32, f32, f32, TextAlign, f32, f32)>,
        can_convert_to_path: bool,
        shape_origin: Option<crate::core::ShapeOrigin>,
        page_info: Option<(String, f32, f32, f32, f32, usize)>,
        node_coord: Option<crate::core::Point>,
    ) {
        let has_selection = selected_count > 0;
        let is_shape_tool = matches!(
            tool_id,
            "rectangle" | "square" | "star" | "triangle" | "circle" | "spiral"
        );

        if is_editing_text || tool_id == "text" || (has_selection && text_info.is_some()) {
            self.container.set_visible(true);
            self.left_capsule.set_visible(false);
            self.page_capsule.set_visible(false);
            self.right_capsule.set_visible(false);
            self.text_capsule.set_visible(true);
        } else if tool_id == "page" {
            self.container.set_visible(true);
            self.left_capsule.set_visible(false);
            self.text_capsule.set_visible(false);
            self.right_capsule.set_visible(false);
            self.page_capsule.set_visible(true);

            if let Some((name, _x, _y, w, h, pages_count)) = page_info {
                self.is_syncing.set(true);
                if !self.page_name_entry.has_focus()
                    && self.page_name_entry.text().as_str() != name.as_str()
                {
                    self.page_name_entry.set_text(&name);
                }
                if (self.page_w_spin.value() - w as f64).abs() > 0.5
                    && !self.page_w_spin.has_focus()
                {
                    self.page_w_spin.set_value(w as f64);
                }
                if (self.page_h_spin.value() - h as f64).abs() > 0.5
                    && !self.page_h_spin.has_focus()
                {
                    self.page_h_spin.set_value(h as f64);
                }

                let (cw, ch) = (w.round(), h.round());
                let matched_preset =
                    if (cw == 794.0 && ch == 1123.0) || (cw == 1123.0 && ch == 794.0) {
                        0
                    } else if (cw == 1123.0 && ch == 1587.0) || (cw == 1587.0 && ch == 1123.0) {
                        1
                    } else if (cw == 559.0 && ch == 794.0) || (cw == 794.0 && ch == 559.0) {
                        2
                    } else if (cw == 1920.0 && ch == 1080.0) || (cw == 1080.0 && ch == 1920.0) {
                        3
                    } else if (cw == 3840.0 && ch == 2160.0) || (cw == 2160.0 && ch == 3840.0) {
                        4
                    } else if cw == 1080.0 && ch == 1080.0 {
                        5
                    } else if cw == 1080.0 && ch == 1920.0 {
                        6
                    } else {
                        7
                    };

                if self.page_preset_dd.selected() != matched_preset {
                    self.page_preset_dd.set_selected(matched_preset);
                }

                self.btn_del_page.set_sensitive(pages_count > 1);
                self.is_syncing.set(false);
            }
        } else if is_shape_tool {
            self.container.set_visible(true);
            self.text_capsule.set_visible(false);
            self.page_capsule.set_visible(false);
            self.left_capsule.set_visible(true);
            self.right_capsule.set_visible(false);
            self.general_box.set_visible(false);
            self.shape_options_box.set_visible(true);
            self.shape_sep.set_visible(false);

            self.rect_radius_box.set_visible(false);
            self.star_box.set_visible(false);
            self.polygon_box.set_visible(false);
            self.circle_box.set_visible(false);
            self.spiral_box.set_visible(false);

            self.is_syncing.set(true);
            match tool_id {
                "rectangle" | "square" => {
                    self.rect_radius_box.set_visible(true);
                    if let Some(crate::core::ShapeOrigin::Rectangle {
                        corner_radius,
                        corner_radii,
                        corner_style,
                    }) = &shape_origin
                    {
                        if (self.rect_radius_spin.value() - *corner_radius as f64).abs() > 0.001
                            && !self.rect_radius_spin.has_focus()
                        {
                            self.rect_radius_spin.set_value(*corner_radius as f64);
                        }
                        if (self.rect_tl_spin.value() - corner_radii.top_left as f64).abs() > 0.001
                            && !self.rect_tl_spin.has_focus()
                        {
                            self.rect_tl_spin.set_value(corner_radii.top_left as f64);
                        }
                        if (self.rect_tr_spin.value() - corner_radii.top_right as f64).abs() > 0.001
                            && !self.rect_tr_spin.has_focus()
                        {
                            self.rect_tr_spin.set_value(corner_radii.top_right as f64);
                        }
                        if (self.rect_br_spin.value() - corner_radii.bottom_right as f64).abs() > 0.001
                            && !self.rect_br_spin.has_focus()
                        {
                            self.rect_br_spin.set_value(corner_radii.bottom_right as f64);
                        }
                        if (self.rect_bl_spin.value() - corner_radii.bottom_left as f64).abs() > 0.001
                            && !self.rect_bl_spin.has_focus()
                        {
                            self.rect_bl_spin.set_value(corner_radii.bottom_left as f64);
                        }
                        let style_idx = match corner_style {
                            crate::core::CornerStyle::Round => 0,
                            crate::core::CornerStyle::Chamfer => 1,
                            crate::core::CornerStyle::Concave => 2,
                        };
                        if self.rect_style_dd.selected() != style_idx {
                            self.rect_style_dd.set_selected(style_idx);
                        }
                        let is_indiv = !corner_radii.is_uniform();
                        if self.rect_split_btn.is_active() != is_indiv {
                            self.rect_split_btn.set_active(is_indiv);
                        }
                        self.rect_indiv_box.set_visible(is_indiv);
                        self.rect_r_lbl.set_visible(!is_indiv);
                        self.rect_radius_spin.set_visible(!is_indiv);
                    }
                }
                "star" => {
                    self.star_box.set_visible(true);
                    if let Some(crate::core::ShapeOrigin::Star {
                        corner_radius,
                        points,
                        inner_ratio,
                    }) = &shape_origin
                    {
                        if (self.star_points_spin.value() - *points as f64).abs() > 0.001
                            && !self.star_points_spin.has_focus()
                        {
                            self.star_points_spin.set_value(*points as f64);
                        }
                        if (self.star_ratio_spin.value() - *inner_ratio as f64).abs() > 0.001
                            && !self.star_ratio_spin.has_focus()
                        {
                            self.star_ratio_spin.set_value(*inner_ratio as f64);
                        }
                        if (self.star_radius_spin.value() - *corner_radius as f64).abs() > 0.001
                            && !self.star_radius_spin.has_focus()
                        {
                            self.star_radius_spin.set_value(*corner_radius as f64);
                        }
                    }
                }
                "triangle" => {
                    self.polygon_box.set_visible(true);
                    if let Some(crate::core::ShapeOrigin::Triangle {
                        corner_radius,
                        sides,
                    }) = &shape_origin
                    {
                        if (self.polygon_sides_spin.value() - *sides as f64).abs() > 0.001
                            && !self.polygon_sides_spin.has_focus()
                        {
                            self.polygon_sides_spin.set_value(*sides as f64);
                        }
                        if (self.polygon_radius_spin.value() - *corner_radius as f64).abs() > 0.001
                            && !self.polygon_radius_spin.has_focus()
                        {
                            self.polygon_radius_spin.set_value(*corner_radius as f64);
                        }
                    }
                }
                "circle" => {
                    self.circle_box.set_visible(true);
                    if let Some(crate::core::ShapeOrigin::Circle {
                        arc_mode,
                        start_angle,
                        end_angle,
                    }) = &shape_origin
                    {
                        match arc_mode {
                            crate::core::ArcMode::Full => self.arc_btn_full.set_active(true),
                            crate::core::ArcMode::Arc => self.arc_btn_arc.set_active(true),
                            crate::core::ArcMode::Segment => self.arc_btn_seg.set_active(true),
                            crate::core::ArcMode::Chord => self.arc_btn_chord.set_active(true),
                        }
                        if (self.circle_start_spin.value() - *start_angle as f64).abs() > 0.001
                            && !self.circle_start_spin.has_focus()
                        {
                            self.circle_start_spin.set_value(*start_angle as f64);
                        }
                        if (self.circle_end_spin.value() - *end_angle as f64).abs() > 0.001
                            && !self.circle_end_spin.has_focus()
                        {
                            self.circle_end_spin.set_value(*end_angle as f64);
                        }
                    }
                }
                "spiral" => {
                    self.spiral_box.set_visible(true);
                    if let Some(crate::core::ShapeOrigin::Spiral {
                        turns,
                        divergence,
                        inner_radius,
                    }) = &shape_origin
                    {
                        if (self.spiral_turns_spin.value() - *turns as f64).abs() > 0.001
                            && !self.spiral_turns_spin.has_focus()
                        {
                            self.spiral_turns_spin.set_value(*turns as f64);
                        }
                        if (self.spiral_div_spin.value() - *divergence as f64).abs() > 0.001
                            && !self.spiral_div_spin.has_focus()
                        {
                            self.spiral_div_spin.set_value(*divergence as f64);
                        }
                        if (self.spiral_inner_spin.value() - *inner_radius as f64).abs() > 0.001
                            && !self.spiral_inner_spin.has_focus()
                        {
                            self.spiral_inner_spin.set_value(*inner_radius as f64);
                        }
                    }
                }
                _ => {}
            }
            self.is_syncing.set(false);
        } else if tool_id == "path_editor" || tool_id == "path-editor" {
            self.container.set_visible(true);
            self.text_capsule.set_visible(false);
            self.page_capsule.set_visible(false);
            self.left_capsule.set_visible(true);
            self.right_capsule.set_visible(true);
            self.general_box.set_visible(false);
            self.shape_options_box.set_visible(false);
            self.path_editor_box.set_visible(true);
        } else if has_selection {
            self.container.set_visible(true);
            self.text_capsule.set_visible(false);
            self.page_capsule.set_visible(false);
            self.left_capsule.set_visible(true);
            self.right_capsule.set_visible(has_selection);
            self.general_box.set_visible(true);
            self.shape_options_box.set_visible(false);
            self.path_editor_box.set_visible(false);

            self.layer_box.set_visible(selected_count > 0);
            self.corner_box.set_visible(can_convert_to_path);
            self.btn_convert_path.set_visible(can_convert_to_path);
            self.boolean_box.set_visible(selected_count >= 2);
        } else {
            self.container.set_visible(false);
            self.left_capsule.set_visible(false);
            self.right_capsule.set_visible(false);
            self.text_capsule.set_visible(false);
            self.page_capsule.set_visible(false);
        }

        let (icon_res, tool_name) = match tool_id {
            "select" => (
                "/io/github/lewis/GnomePaths/icons/tool-selection-options.svg",
                crate::core::gettext("Selection"),
            ),
            "path-editor" | "path_editor" => (
                "/io/github/lewis/GnomePaths/icons/tool-path-editor.svg",
                crate::core::gettext("Node Editor"),
            ),
            "page" => (
                "/io/github/lewis/GnomePaths/icons/tool-page.svg",
                crate::core::gettext("Page Tool"),
            ),
            "rectangle" | "square" => (
                "/io/github/lewis/GnomePaths/icons/tool-square.svg",
                crate::core::gettext("Rectangle"),
            ),
            "circle" => (
                "/io/github/lewis/GnomePaths/icons/tool-circle.svg",
                crate::core::gettext("Circle"),
            ),
            "star" => (
                "/io/github/lewis/GnomePaths/icons/tool-star.svg",
                crate::core::gettext("Star"),
            ),
            "triangle" => (
                "/io/github/lewis/GnomePaths/icons/tool-triangle.svg",
                crate::core::gettext("Polygon"),
            ),
            "spiral" => (
                "/io/github/lewis/GnomePaths/icons/tool-spiral.svg",
                crate::core::gettext("Spiral"),
            ),
            "vector-pen" | "vector_pen" => (
                "/io/github/lewis/GnomePaths/icons/tool-vector-pen.svg",
                crate::core::gettext("Vector Pen"),
            ),
            "pen" => (
                "/io/github/lewis/GnomePaths/icons/tool-pen.svg",
                crate::core::gettext("Pen"),
            ),
            "brush" => (
                "/io/github/lewis/GnomePaths/icons/tool-vector-pen.svg",
                crate::core::gettext("Brush"),
            ),
            "text" => (
                "/io/github/lewis/GnomePaths/icons/tool-text.svg",
                crate::core::gettext("Text"),
            ),
            "paint_bucket" => (
                "/io/github/lewis/GnomePaths/icons/tool-paint-bucket.svg",
                crate::core::gettext("Paint Bucket"),
            ),
            "gradient" => (
                "/io/github/lewis/GnomePaths/icons/tool-gradient.svg",
                crate::core::gettext("Gradient"),
            ),
            "mesh_gradient" => (
                "/io/github/lewis/GnomePaths/icons/tool-mesh.svg",
                crate::core::gettext("Mesh Gradient"),
            ),
            "eyedropper" => (
                "/io/github/lewis/GnomePaths/icons/tool-eyedropper.svg",
                crate::core::gettext("Eyedropper"),
            ),
            "measure" => (
                "/io/github/lewis/GnomePaths/icons/tool-measure.svg",
                crate::core::gettext("Ruler / Measure"),
            ),
            "zoom" => (
                "/io/github/lewis/GnomePaths/icons/tool-zoom.svg",
                crate::core::gettext("Zoom"),
            ),
            "zoom_selection" => (
                "/io/github/lewis/GnomePaths/icons/tool-zoom-selection.svg",
                crate::core::gettext("Zoom Selection"),
            ),
            "zoom_fit_all" => (
                "/io/github/lewis/GnomePaths/icons/tool-zoom-fit-all.svg",
                crate::core::gettext("Zoom All"),
            ),
            "zoom_100" => (
                "/io/github/lewis/GnomePaths/icons/tool-zoom-100.svg",
                crate::core::gettext("Zoom 100%"),
            ),
            "zoom_fit_page" => (
                "/io/github/lewis/GnomePaths/icons/tool-zoom-page.svg",
                crate::core::gettext("Zoom Page"),
            ),
            "drag" => (
                "/io/github/lewis/GnomePaths/icons/tool-drag.svg",
                crate::core::gettext("Pan Canvas"),
            ),
            _ => (
                "/io/github/lewis/GnomePaths/icons/tool-selection-options.svg",
                crate::core::gettext("Selection"),
            ),
        };
        self.mode_img.set_icon_name(Some(&crate::ui::icons::symbolic_icon_name(icon_res)));
        let active_tool_label = crate::i18n!("Active Tool: {}", tool_name);
        self.mode_img.set_tooltip_text(Some(&active_tool_label));

        if let Some((family, weight, size, line_height, alignment, letter_spacing, word_spacing)) =
            text_info
        {
            self.is_syncing.set(true);

            if let Some(pos) = self
                .system_fonts
                .iter()
                .position(|f| f.eq_ignore_ascii_case(&family))
            {
                if self.font_dd.selected() != pos as u32 {
                    self.font_dd.set_selected(pos as u32);
                }
            }

            let weight_idx = self::helpers::FONT_WEIGHT_VALUES
                .iter()
                .position(|&w| w == weight)
                .unwrap_or(2) as u32;
            if self.weight_dd.selected() != weight_idx {
                self.weight_dd.set_selected(weight_idx);
            }

            if (self.size_spin.value() - size as f64).abs() > 0.01 && !self.size_spin.has_focus() {
                self.size_spin.set_value(size as f64);
            }

            if (self.line_height_spin.value() - line_height as f64).abs() > 0.01
                && !self.line_height_spin.has_focus()
            {
                self.line_height_spin.set_value(line_height as f64);
            }

            if (self.letter_spacing_spin.value() - letter_spacing as f64).abs() > 0.01
                && !self.letter_spacing_spin.has_focus()
            {
                self.letter_spacing_spin.set_value(letter_spacing as f64);
            }

            if (self.word_spacing_spin.value() - word_spacing as f64).abs() > 0.01
                && !self.word_spacing_spin.has_focus()
            {
                self.word_spacing_spin.set_value(word_spacing as f64);
            }

            if let Some(r) = bounds {
                if (self.text_box_w_spin.value() - r.width as f64).abs() > 0.5
                    && !self.text_box_w_spin.has_focus()
                {
                    self.text_box_w_spin.set_value(r.width.round() as f64);
                }
            }

            match alignment {
                TextAlign::Left => {
                    if !self.btn_align_left.is_active() {
                        self.btn_align_left.set_active(true);
                    }
                }
                TextAlign::Center => {
                    if !self.btn_align_center.is_active() {
                        self.btn_align_center.set_active(true);
                    }
                }
                TextAlign::Right => {
                    if !self.btn_align_right.is_active() {
                        self.btn_align_right.set_active(true);
                    }
                }
                TextAlign::Justify => {
                    if !self.btn_align_justify.is_active() {
                        self.btn_align_justify.set_active(true);
                    }
                }
            }

            self.is_syncing.set(false);
        }

        if tool_id == "path_editor" || tool_id == "path-editor" {
            if let Some(pt) = node_coord {
                let x_str = format!("{:.1}", pt.x);
                if !self.x_entry.has_focus() && self.x_entry.text().as_str() != x_str {
                    self.x_entry.set_text(&x_str);
                }
                let y_str = format!("{:.1}", pt.y);
                if !self.y_entry.has_focus() && self.y_entry.text().as_str() != y_str {
                    self.y_entry.set_text(&y_str);
                }
            } else if let Some(r) = bounds {
                let x_str = format!("{:.1}", r.x);
                if !self.x_entry.has_focus() && self.x_entry.text().as_str() != x_str {
                    self.x_entry.set_text(&x_str);
                }
                let y_str = format!("{:.1}", r.y);
                if !self.y_entry.has_focus() && self.y_entry.text().as_str() != y_str {
                    self.y_entry.set_text(&y_str);
                }
            } else {
                if !self.x_entry.has_focus() && !self.x_entry.text().is_empty() {
                    self.x_entry.set_text("");
                }
                if !self.y_entry.has_focus() && !self.y_entry.text().is_empty() {
                    self.y_entry.set_text("");
                }
            }
            if !self.w_entry.has_focus() && !self.w_entry.text().is_empty() {
                self.w_entry.set_text("");
            }
            if !self.h_entry.has_focus() && !self.h_entry.text().is_empty() {
                self.h_entry.set_text("");
            }
        } else if let Some(r) = bounds {
            let x_str = format!("{:.1}", r.x);
            if !self.x_entry.has_focus() && self.x_entry.text().as_str() != x_str {
                self.x_entry.set_text(&x_str);
            }
            let y_str = format!("{:.1}", r.y);
            if !self.y_entry.has_focus() && self.y_entry.text().as_str() != y_str {
                self.y_entry.set_text(&y_str);
            }
            let w_str = format!("{:.1}", r.width);
            if !self.w_entry.has_focus() && self.w_entry.text().as_str() != w_str {
                self.w_entry.set_text(&w_str);
            }
            let h_str = format!("{:.1}", r.height);
            if !self.h_entry.has_focus() && self.h_entry.text().as_str() != h_str {
                self.h_entry.set_text(&h_str);
            }
        } else {
            if !self.x_entry.has_focus() && !self.x_entry.text().is_empty() {
                self.x_entry.set_text("");
            }
            if !self.y_entry.has_focus() && !self.y_entry.text().is_empty() {
                self.y_entry.set_text("");
            }
            if !self.w_entry.has_focus() && !self.w_entry.text().is_empty() {
                self.w_entry.set_text("");
            }
            if !self.h_entry.has_focus() && !self.h_entry.text().is_empty() {
                self.h_entry.set_text("");
            }
        }
    }

    pub fn update_margin_for_toolbar(&self, toolbar_pos: BarPosition) {
        if self.is_top.get() {
            self.container.set_valign(gtk4::Align::Start);
            if toolbar_pos == BarPosition::Top {
                self.container.set_margin_top(104);
            } else {
                self.container.set_margin_top(32);
            }
            self.container.set_margin_bottom(0);
        } else {
            self.container.set_valign(gtk4::Align::End);
            if toolbar_pos == BarPosition::Bottom {
                self.container.set_margin_bottom(96);
            } else {
                self.container.set_margin_bottom(24);
            }
            self.container.set_margin_top(0);
        }
    }
}
