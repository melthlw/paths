use gtk4::glib;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use super::helpers::{create_resource_btn, FONT_WEIGHT_NAMES, FONT_WEIGHT_VALUES};
use crate::core::TextAlign;
use crate::ui::canvas::CanvasWidget;

pub struct PenBrushTextControls {
    pub path_editor_box: gtk4::Box,
    pub text_capsule: gtk4::Box,
    pub system_fonts: Vec<String>,
    pub font_dd: gtk4::DropDown,
    pub weight_dd: gtk4::DropDown,
    pub size_spin: gtk4::SpinButton,
    pub line_height_spin: gtk4::SpinButton,
    pub btn_align_left: gtk4::ToggleButton,
    pub btn_align_center: gtk4::ToggleButton,
    pub btn_align_right: gtk4::ToggleButton,
    pub btn_align_justify: gtk4::ToggleButton,
    pub text_box_w_spin: gtk4::SpinButton,
    pub letter_spacing_spin: gtk4::SpinButton,
    pub word_spacing_spin: gtk4::SpinButton,
    pub page_capsule: gtk4::Box,
    pub page_name_entry: gtk4::Entry,
    pub page_preset_dd: gtk4::DropDown,
    pub page_w_spin: gtk4::SpinButton,
    pub page_h_spin: gtk4::SpinButton,
    pub btn_del_page: gtk4::Button,
}

pub fn build_pen_brush_text_controls(
    canvas: &CanvasWidget,
    is_syncing: &Rc<Cell<bool>>,
) -> PenBrushTextControls {
    // ── Path Editor Options Box ──
    let path_editor_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    // 1. Node Types: Corner, Smooth, Symmetric, Auto
    let btn_node_corner = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/node-corner.svg",
        &crate::core::gettext("Corner Node (Cusp)"),
    );
    let canvas_nc = canvas.clone();
    btn_node_corner.connect_clicked(move |_| {
        canvas_nc.set_selected_nodes_type(crate::core::NodeType::Corner);
    });
    path_editor_box.append(&btn_node_corner);

    let btn_node_smooth = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/node-smooth.svg",
        &crate::core::gettext("Smooth Node"),
    );
    let canvas_ns = canvas.clone();
    btn_node_smooth.connect_clicked(move |_| {
        canvas_ns.set_selected_nodes_type(crate::core::NodeType::Smooth);
    });
    path_editor_box.append(&btn_node_smooth);

    let btn_node_symmetric = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/node-symmetric.svg",
        &crate::core::gettext("Symmetric Node"),
    );
    let canvas_nsym = canvas.clone();
    btn_node_symmetric.connect_clicked(move |_| {
        canvas_nsym.set_selected_nodes_type(crate::core::NodeType::Symmetric);
    });
    path_editor_box.append(&btn_node_symmetric);

    let btn_node_auto = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/node-auto.svg",
        &crate::core::gettext("Auto-Smooth Node"),
    );
    let canvas_na = canvas.clone();
    btn_node_auto.connect_clicked(move |_| {
        canvas_na.set_selected_nodes_type(crate::core::NodeType::Auto);
    });
    path_editor_box.append(&btn_node_auto);

    let sep_node1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    path_editor_box.append(&sep_node1);

    // 2. Segment Tools: Make Straight, Make Curve
    let btn_seg_line = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/segment-line.svg",
        &crate::core::gettext("Make Segment Straight"),
    );
    let canvas_sl = canvas.clone();
    btn_seg_line.connect_clicked(move |_| {
        canvas_sl.make_selected_segments_straight();
    });
    path_editor_box.append(&btn_seg_line);

    let btn_seg_curve = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/segment-curve.svg",
        &crate::core::gettext("Make Segment Curve"),
    );
    let canvas_sc = canvas.clone();
    btn_seg_curve.connect_clicked(move |_| {
        canvas_sc.make_selected_segments_curve();
    });
    path_editor_box.append(&btn_seg_curve);

    let sep_node2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    path_editor_box.append(&sep_node2);

    // 3. Topology: Insert, Delete, Close, Reverse
    let btn_node_add = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/node-add.svg",
        &crate::core::gettext("Insert Node into Segment"),
    );
    let canvas_na2 = canvas.clone();
    btn_node_add.connect_clicked(move |_| {
        canvas_na2.insert_nodes_between_selected();
    });
    path_editor_box.append(&btn_node_add);

    let btn_node_del = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/node-delete.svg",
        &crate::core::gettext("Delete Selected Node(s) (Delete/Backspace)"),
    );
    let canvas_nd = canvas.clone();
    btn_node_del.connect_clicked(move |_| {
        canvas_nd.delete_selected_nodes();
    });
    path_editor_box.append(&btn_node_del);

    let btn_node_close = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/path-close.svg",
        &crate::core::gettext("Close / Open Path"),
    );
    let canvas_ncl = canvas.clone();
    btn_node_close.connect_clicked(move |_| {
        canvas_ncl.toggle_selected_paths_closed();
    });
    path_editor_box.append(&btn_node_close);

    let btn_node_rev = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/path-reverse.svg",
        &crate::core::gettext("Reverse Path Direction"),
    );
    let canvas_nr = canvas.clone();
    btn_node_rev.connect_clicked(move |_| {
        canvas_nr.reverse_selected_paths_direction();
    });
    path_editor_box.append(&btn_node_rev);

    let sep_node3 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    path_editor_box.append(&sep_node3);

    // 4. Node Alignment
    let btn_align_h = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/align-center-v.svg",
        &crate::core::gettext("Align Nodes Horizontally"),
    );
    let canvas_ah = canvas.clone();
    btn_align_h.connect_clicked(move |_| {
        canvas_ah.align_selected_nodes_horizontal();
    });
    path_editor_box.append(&btn_align_h);

    let btn_align_v = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/align-center-h.svg",
        &crate::core::gettext("Align Nodes Vertically"),
    );
    let canvas_av = canvas.clone();
    btn_align_v.connect_clicked(move |_| {
        canvas_av.align_selected_nodes_vertical();
    });
    path_editor_box.append(&btn_align_v);

    let btn_dist_h = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/distribute-horizontal.svg",
        &crate::core::gettext("Distribute Nodes Horizontally"),
    );
    let canvas_dh = canvas.clone();
    btn_dist_h.connect_clicked(move |_| {
        canvas_dh.distribute_selected_nodes_horizontal();
    });
    path_editor_box.append(&btn_dist_h);

    let btn_dist_v = create_resource_btn(
        "/io/github/lewis/GnomePaths/icons/distribute-vertical.svg",
        &crate::core::gettext("Distribute Nodes Vertically"),
    );
    let canvas_dv = canvas.clone();
    btn_dist_v.connect_clicked(move |_| {
        canvas_dv.distribute_selected_nodes_vertical();
    });
    path_editor_box.append(&btn_dist_v);

    // 2. TEXT CAPSULE
    let text_capsule = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["toolbar", "card"])
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let text_mode_img = crate::ui::icons::make_symbolic_image("tool-text", 18);
    text_mode_img.set_valign(gtk4::Align::Center);
    text_mode_img.set_margin_start(8);
    text_mode_img.set_margin_end(4);
    text_mode_img.set_opacity(0.5);
    text_mode_img.add_css_class("dim-label");
    text_mode_img.set_tooltip_text(Some(&crate::core::gettext("Active Tool: Text")));
    text_capsule.append(&text_mode_img);

    let text_mode_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    text_capsule.append(&text_mode_sep);

    let system_fonts = crate::core::get_system_font_families();
    let font_strs: Vec<&str> = system_fonts.iter().map(|s| s.as_str()).collect();
    let font_model = gtk4::StringList::new(&font_strs);
    let font_dd = gtk4::DropDown::builder()
        .model(&font_model)
        .selected(0)
        .enable_search(true)
        .expression(gtk4::PropertyExpression::new(
            gtk4::StringObject::static_type(),
            None::<gtk4::Expression>,
            "string",
        ))
        .css_classes(["flat"])
        .tooltip_text(crate::core::gettext("Font Family (Search or use ↑ / ↓)"))
        .valign(gtk4::Align::Center)
        .build();

    let canvas_font = canvas.clone();
    let syncing_font = is_syncing.clone();
    let sys_fonts_clone = system_fonts.clone();
    font_dd.connect_selected_notify(move |dd| {
        if syncing_font.get() {
            return;
        }
        let idx = dd.selected() as usize;
        if idx < sys_fonts_clone.len() {
            canvas_font.set_selected_font_family(&sys_fonts_clone[idx]);
        }
    });

    let font_dd_key = font_dd.clone();
    let sys_fonts_key = system_fonts.clone();
    let prefix_buf = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
    let last_type_time = std::rc::Rc::new(std::cell::Cell::new(glib::monotonic_time()));
    let key_ctrl = gtk4::EventControllerKey::new();
    key_ctrl.connect_key_pressed(move |_ctrl, keyval, _keycode, _state| {
        let current = font_dd_key.selected() as usize;
        match keyval {
            gtk4::gdk::Key::Down | gtk4::gdk::Key::KP_Down => {
                if current + 1 < sys_fonts_key.len() {
                    font_dd_key.set_selected((current + 1) as u32);
                    return glib::Propagation::Stop;
                }
            }
            gtk4::gdk::Key::Up | gtk4::gdk::Key::KP_Up => {
                if current > 0 {
                    font_dd_key.set_selected((current - 1) as u32);
                    return glib::Propagation::Stop;
                }
            }
            _ => {
                if let Some(ch) = keyval.to_unicode() {
                    if !ch.is_control() && (ch.is_alphanumeric() || ch == ' ') {
                        let now = glib::monotonic_time();
                        let mut buf = prefix_buf.borrow_mut();
                        if now - last_type_time.get() > 800_000 {
                            buf.clear();
                        }
                        last_type_time.set(now);
                        buf.push(ch);

                        let search_term = buf.to_lowercase();
                        let match_idx = sys_fonts_key
                            .iter()
                            .position(|f| f.to_lowercase().starts_with(&search_term));

                        if let Some(idx) = match_idx {
                            font_dd_key.set_selected(idx as u32);
                            return glib::Propagation::Stop;
                        } else if buf.chars().count() > 1 {
                            buf.clear();
                            buf.push(ch);
                            let single = ch.to_lowercase().to_string();
                            if let Some(idx) = sys_fonts_key
                                .iter()
                                .position(|f| f.to_lowercase().starts_with(&single))
                            {
                                font_dd_key.set_selected(idx as u32);
                                return glib::Propagation::Stop;
                            }
                        }
                    }
                }
            }
        }
        glib::Propagation::Proceed
    });
    font_dd.add_controller(key_ctrl);

    let font_dd_scroll = font_dd.clone();
    let sys_fonts_scroll = system_fonts.clone();
    let scroll_ctrl =
        gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
    scroll_ctrl.connect_scroll(move |_ctrl, _dx, dy| {
        let current = font_dd_scroll.selected() as usize;
        if dy > 0.0 {
            if current + 1 < sys_fonts_scroll.len() {
                font_dd_scroll.set_selected((current + 1) as u32);
                return glib::Propagation::Stop;
            }
        } else if dy < 0.0 {
            if current > 0 {
                font_dd_scroll.set_selected((current - 1) as u32);
                return glib::Propagation::Stop;
            }
        }
        glib::Propagation::Proceed
    });
    font_dd.add_controller(scroll_ctrl);

    text_capsule.append(&font_dd);

    let sep_text_1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    text_capsule.append(&sep_text_1);

    let weight_model = gtk4::StringList::new(FONT_WEIGHT_NAMES);
    let weight_dd = gtk4::DropDown::builder()
        .model(&weight_model)
        .selected(2)
        .css_classes(["flat"])
        .tooltip_text(crate::core::gettext("Font Weight"))
        .valign(gtk4::Align::Center)
        .build();

    let canvas_weight = canvas.clone();
    let syncing_weight = is_syncing.clone();
    weight_dd.connect_selected_notify(move |dd| {
        if syncing_weight.get() {
            return;
        }
        let idx = dd.selected() as usize;
        if idx < FONT_WEIGHT_VALUES.len() {
            canvas_weight.set_selected_font_weight(FONT_WEIGHT_VALUES[idx]);
        }
    });
    text_capsule.append(&weight_dd);

    let sep_text_2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    text_capsule.append(&sep_text_2);

    let size_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let size_lbl = gtk4::Label::builder()
        .label("T")
        .css_classes(["heading", "dim-label"])
        .build();
    let size_spin = gtk4::SpinButton::with_range(8.0, 288.0, 1.0);
    size_spin.set_value(32.0);
    size_spin.set_tooltip_text(Some(&crate::core::gettext("Font Size (pt)")));
    size_spin.set_valign(gtk4::Align::Center);
    size_box.append(&size_lbl);
    size_box.append(&size_spin);

    let canvas_size = canvas.clone();
    let syncing_size = is_syncing.clone();
    size_spin.connect_value_changed(move |spin| {
        if syncing_size.get() {
            return;
        }
        canvas_size.set_selected_font_size(spin.value() as f32);
    });
    text_capsule.append(&size_box);

    let sep_text_3 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    text_capsule.append(&sep_text_3);

    let lh_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let lh_lbl = gtk4::Label::builder()
        .label("↕")
        .css_classes(["heading", "dim-label"])
        .build();
    let line_height_spin = gtk4::SpinButton::with_range(0.5, 4.0, 0.1);
    line_height_spin.set_digits(2);
    line_height_spin.set_value(1.2);
    line_height_spin.set_tooltip_text(Some(&crate::core::gettext("Line Height")));
    line_height_spin.set_valign(gtk4::Align::Center);
    lh_box.append(&lh_lbl);
    lh_box.append(&line_height_spin);

    let canvas_lh = canvas.clone();
    let syncing_lh = is_syncing.clone();
    line_height_spin.connect_value_changed(move |spin| {
        if syncing_lh.get() {
            return;
        }
        canvas_lh.set_selected_line_height(spin.value() as f32);
    });
    text_capsule.append(&lh_box);

    let sep_text_4 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    text_capsule.append(&sep_text_4);

    let align_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["linked"])
        .valign(gtk4::Align::Center)
        .build();

    let img_al = gtk4::Image::from_icon_name("format-justify-left-symbolic");
    img_al.set_pixel_size(16);
    let btn_align_left = gtk4::ToggleButton::builder()
        .child(&img_al)
        .tooltip_text(crate::core::gettext("Align Left"))
        .active(true)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let img_ac = gtk4::Image::from_icon_name("format-justify-center-symbolic");
    img_ac.set_pixel_size(16);
    let btn_align_center = gtk4::ToggleButton::builder()
        .child(&img_ac)
        .tooltip_text(crate::core::gettext("Center"))
        .group(&btn_align_left)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let img_ar = gtk4::Image::from_icon_name("format-justify-right-symbolic");
    img_ar.set_pixel_size(16);
    let btn_align_right = gtk4::ToggleButton::builder()
        .child(&img_ar)
        .tooltip_text(crate::core::gettext("Align Right"))
        .group(&btn_align_left)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let img_aj = gtk4::Image::from_icon_name("format-justify-fill-symbolic");
    img_aj.set_pixel_size(16);
    let btn_align_justify = gtk4::ToggleButton::builder()
        .child(&img_aj)
        .tooltip_text(crate::core::gettext("Justify"))
        .group(&btn_align_left)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    align_box.append(&btn_align_left);
    align_box.append(&btn_align_center);
    align_box.append(&btn_align_right);
    align_box.append(&btn_align_justify);
    text_capsule.append(&align_box);

    let canvas_al = canvas.clone();
    let sync_al = is_syncing.clone();
    btn_align_left.connect_toggled(move |btn| {
        if sync_al.get() {
            return;
        }
        if btn.is_active() {
            canvas_al.set_selected_text_align(TextAlign::Left);
        }
    });

    let canvas_ac = canvas.clone();
    let sync_ac = is_syncing.clone();
    btn_align_center.connect_toggled(move |btn| {
        if sync_ac.get() {
            return;
        }
        if btn.is_active() {
            canvas_ac.set_selected_text_align(TextAlign::Center);
        }
    });

    let canvas_ar = canvas.clone();
    let sync_ar = is_syncing.clone();
    btn_align_right.connect_toggled(move |btn| {
        if sync_ar.get() {
            return;
        }
        if btn.is_active() {
            canvas_ar.set_selected_text_align(TextAlign::Right);
        }
    });

    let canvas_aj = canvas.clone();
    let sync_aj = is_syncing.clone();
    btn_align_justify.connect_toggled(move |btn| {
        if sync_aj.get() {
            return;
        }
        if btn.is_active() {
            canvas_aj.set_selected_text_align(TextAlign::Justify);
        }
    });

    let sep_text_5 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    text_capsule.append(&sep_text_5);

    let box_w_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let box_w_lbl = gtk4::Label::builder()
        .label("⤢")
        .css_classes(["heading", "dim-label"])
        .build();
    let text_box_w_spin = gtk4::SpinButton::with_range(0.0, 5000.0, 10.0);
    text_box_w_spin.set_digits(0);
    text_box_w_spin.set_value(0.0);
    text_box_w_spin.set_tooltip_text(Some(&crate::core::gettext("Text Box Width (0 = Auto)")));
    text_box_w_spin.set_valign(gtk4::Align::Center);
    box_w_box.append(&box_w_lbl);
    box_w_box.append(&text_box_w_spin);

    let canvas_bw = canvas.clone();
    let syncing_bw = is_syncing.clone();
    text_box_w_spin.connect_value_changed(move |spin| {
        if syncing_bw.get() {
            return;
        }
        let val = spin.value();
        if val <= 0.0 {
            canvas_bw.set_selected_text_box_width(None);
        } else {
            canvas_bw.set_selected_text_box_width(Some(val as f32));
        }
    });

    let canvas_box_reset = canvas.clone();
    let box_w_gesture = gtk4::GestureClick::new();
    box_w_gesture.connect_pressed(move |_gesture, n_press, _x, _y| {
        if n_press >= 2 {
            canvas_box_reset.set_selected_text_box_width(None);
        }
    });
    box_w_box.add_controller(box_w_gesture);
    text_capsule.append(&box_w_box);

    let sep_text_6 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    text_capsule.append(&sep_text_6);

    let ls_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let ls_lbl = gtk4::Label::builder()
        .label("AV")
        .css_classes(["heading", "dim-label"])
        .build();
    let letter_spacing_spin = gtk4::SpinButton::with_range(-20.0, 200.0, 0.5);
    letter_spacing_spin.set_digits(1);
    letter_spacing_spin.set_value(0.0);
    letter_spacing_spin.set_tooltip_text(Some(&crate::core::gettext(
        "Letter Spacing (Tracking / px)",
    )));
    letter_spacing_spin.set_valign(gtk4::Align::Center);
    ls_box.append(&ls_lbl);
    ls_box.append(&letter_spacing_spin);

    let canvas_ls = canvas.clone();
    let syncing_ls = is_syncing.clone();
    letter_spacing_spin.connect_value_changed(move |spin| {
        if syncing_ls.get() {
            return;
        }
        canvas_ls.set_selected_letter_spacing(spin.value() as f32);
    });
    text_capsule.append(&ls_box);

    let ws_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let ws_lbl = gtk4::Label::builder()
        .label("W↔")
        .css_classes(["heading", "dim-label"])
        .build();
    let word_spacing_spin = gtk4::SpinButton::with_range(-20.0, 200.0, 1.0);
    word_spacing_spin.set_digits(1);
    word_spacing_spin.set_value(0.0);
    word_spacing_spin.set_tooltip_text(Some(&crate::core::gettext("Word Spacing (px)")));
    word_spacing_spin.set_valign(gtk4::Align::Center);
    ws_box.append(&ws_lbl);
    ws_box.append(&word_spacing_spin);

    let canvas_ws = canvas.clone();
    let syncing_ws = is_syncing.clone();
    word_spacing_spin.connect_value_changed(move |spin| {
        if syncing_ws.get() {
            return;
        }
        canvas_ws.set_selected_word_spacing(spin.value() as f32);
    });
    text_capsule.append(&ws_box);

    // 4. PAGE CAPSULE: Page Tool Options
    let page_capsule = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["toolbar", "card"])
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    let page_mode_img = crate::ui::icons::make_symbolic_image("tool-page", 20);
    page_mode_img.set_opacity(0.5);
    page_mode_img.add_css_class("dim-label");
    page_mode_img.set_tooltip_text(Some(&crate::core::gettext("Active Tool: Artboard / Page")));
    page_capsule.append(&page_mode_img);

    let sep_p1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    page_capsule.append(&sep_p1);

    let page_name_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();
    let page_name_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Name:"))
        .css_classes(["dim-label", "caption"])
        .build();
    let page_name_entry = gtk4::Entry::builder()
        .placeholder_text(crate::core::gettext("Page 1"))
        .width_chars(12)
        .valign(gtk4::Align::Center)
        .build();
    page_name_box.append(&page_name_lbl);
    page_name_box.append(&page_name_entry);
    page_capsule.append(&page_name_box);

    let canvas_pname = canvas.clone();
    let sync_pname = is_syncing.clone();
    page_name_entry.connect_activate(move |entry| {
        if sync_pname.get() {
            return;
        }
        let text = entry.text();
        if !text.trim().is_empty() {
            canvas_pname.set_active_page_name(text.trim().to_string());
        }
    });

    let sep_p2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    page_capsule.append(&sep_p2);

    let preset_names = [
        "A4 (794 × 1123)",
        "A3 (1123 × 1587)",
        "A5 (559 × 794)",
        "Full HD 1080p (1920 × 1080)",
        "4K UHD (3840 × 2160)",
        "Instagram Post (1080 × 1080)",
        "Instagram Story (1080 × 1920)",
        &crate::core::gettext("Custom"),
    ];
    let preset_model = gtk4::StringList::new(&preset_names);
    let page_preset_dd = gtk4::DropDown::builder()
        .model(&preset_model)
        .selected(0)
        .valign(gtk4::Align::Center)
        .build();
    page_capsule.append(&page_preset_dd);

    let canvas_preset = canvas.clone();
    let sync_preset = is_syncing.clone();
    page_preset_dd.connect_selected_notify(move |dd| {
        if sync_preset.get() {
            return;
        }
        let idx = dd.selected();
        let (w, h) = match idx {
            0 => (794.0, 1123.0),
            1 => (1123.0, 1587.0),
            2 => (559.0, 794.0),
            3 => (1920.0, 1080.0),
            4 => (3840.0, 2160.0),
            5 => (1080.0, 1080.0),
            6 => (1080.0, 1920.0),
            _ => return,
        };
        canvas_preset.set_active_page_size(w, h);
    });

    let btn_orientation = gtk4::Button::builder()
        .icon_name("object-rotate-right-symbolic")
        .tooltip_text(crate::core::gettext(
            "Toggle Orientation (Portrait / Landscape)",
        ))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let canvas_orient = canvas.clone();
    btn_orientation.connect_clicked(move |_| {
        canvas_orient.toggle_active_page_orientation();
    });
    page_capsule.append(&btn_orientation);

    let sep_p3 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    page_capsule.append(&sep_p3);

    let pw_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let pw_lbl = gtk4::Label::builder()
        .label("W")
        .css_classes(["dim-label", "caption"])
        .build();
    let page_w_spin = gtk4::SpinButton::with_range(50.0, 20000.0, 10.0);
    page_w_spin.set_digits(0);
    page_w_spin.set_value(794.0);
    page_w_spin.set_valign(gtk4::Align::Center);
    pw_box.append(&pw_lbl);
    pw_box.append(&page_w_spin);
    page_capsule.append(&pw_box);

    let ph_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();
    let ph_lbl = gtk4::Label::builder()
        .label("H")
        .css_classes(["dim-label", "caption"])
        .build();
    let page_h_spin = gtk4::SpinButton::with_range(50.0, 20000.0, 10.0);
    page_h_spin.set_digits(0);
    page_h_spin.set_value(1123.0);
    page_h_spin.set_valign(gtk4::Align::Center);
    ph_box.append(&ph_lbl);
    ph_box.append(&page_h_spin);
    page_capsule.append(&ph_box);

    let canvas_pw = canvas.clone();
    let page_h_spin_clone = page_h_spin.clone();
    let sync_pw = is_syncing.clone();
    page_w_spin.connect_value_changed(move |spin| {
        if sync_pw.get() {
            return;
        }
        canvas_pw.set_active_page_size(spin.value() as f32, page_h_spin_clone.value() as f32);
    });

    let canvas_ph = canvas.clone();
    let page_w_spin_clone = page_w_spin.clone();
    let sync_ph = is_syncing.clone();
    page_h_spin.connect_value_changed(move |spin| {
        if sync_ph.get() {
            return;
        }
        canvas_ph.set_active_page_size(page_w_spin_clone.value() as f32, spin.value() as f32);
    });

    let sep_p4 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    page_capsule.append(&sep_p4);

    let btn_add_page = gtk4::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(crate::core::gettext("Create New Page"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let canvas_add_page = canvas.clone();
    btn_add_page.connect_clicked(move |_| {
        canvas_add_page.add_new_page();
    });
    page_capsule.append(&btn_add_page);

    let btn_del_page = gtk4::Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text(crate::core::gettext("Delete Current Page"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let canvas_del_page = canvas.clone();
    btn_del_page.connect_clicked(move |_| {
        canvas_del_page.delete_active_page();
    });
    page_capsule.append(&btn_del_page);

    PenBrushTextControls {
        path_editor_box,
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
        page_capsule,
        page_name_entry,
        page_preset_dd,
        page_w_spin,
        page_h_spin,
        btn_del_page,
    }
}
