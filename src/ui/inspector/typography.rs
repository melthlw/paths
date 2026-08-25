use gtk4::glib;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::core::element::{OpenTypeFeatures, TextAlign, TextCase};
use crate::ui::canvas::CanvasWidget;
use crate::ui::tool_options::helpers::{FONT_WEIGHT_NAMES, FONT_WEIGHT_VALUES};

pub fn build_typography_section(canvas: &CanvasWidget) -> gtk4::Box {
    let is_syncing = Rc::new(Cell::new(false));

    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(12)
        .margin_end(12)
        .build();

    // ─────────────────────────────────────────────────────────────
    // Card 1: Font Family & Primary Typography
    // ─────────────────────────────────────────────────────────────
    let font_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .css_classes(["card"])
        .margin_bottom(4)
        .build();

    let font_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();
    let font_icon = crate::ui::icons::make_symbolic_image("tool-text", 16);
    let font_title = gtk4::Label::builder()
        .label(crate::core::gettext("Typography & Font"))
        .css_classes(["heading", "caption"])
        .hexpand(true)
        .xalign(0.0)
        .build();
    let glyph_btn = gtk4::MenuButton::builder()
        .icon_name("preferences-desktop-font-symbolic")
        .tooltip_text(crate::core::gettext("Glyph & Symbol Catalog"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let glyph_pop = crate::ui::dialogs::glyph_map::build_glyph_map_popover(canvas);
    glyph_btn.set_popover(Some(&glyph_pop));

    font_header.append(&font_icon);
    font_header.append(&font_title);
    font_header.append(&glyph_btn);
    font_card.append(&font_header);

    let font_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    font_card.append(&font_sep);

    let font_content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(4)
        .margin_bottom(10)
        .build();

    // Font Family Dropdown with Live Typeface Preview
    let system_fonts = crate::core::get_system_font_families();
    let font_strs: Vec<&str> = system_fonts.iter().map(|s| s.as_str()).collect();
    let font_model = gtk4::StringList::new(&font_strs);

    let font_factory = gtk4::SignalListItemFactory::new();
    font_factory.connect_setup(|_, list_item| {
        let item = list_item
            .downcast_ref::<gtk4::ListItem>()
            .expect("ListItem expected");
        let label = gtk4::Label::builder()
            .xalign(0.0)
            .margin_start(6)
            .margin_end(6)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        item.set_child(Some(&label));
    });
    font_factory.connect_bind(|_, list_item| {
        let item = list_item
            .downcast_ref::<gtk4::ListItem>()
            .expect("ListItem expected");
        let label = item
            .child()
            .and_downcast::<gtk4::Label>()
            .expect("Label expected");
        let st_obj = item
            .item()
            .and_downcast::<gtk4::StringObject>()
            .expect("StringObject expected");
        let font_name = st_obj.string();

        let mut font_desc = gtk4::pango::FontDescription::from_string(&font_name);
        font_desc.set_size(13 * gtk4::pango::SCALE);
        let attr_list = gtk4::pango::AttrList::new();
        attr_list.insert(gtk4::pango::AttrFontDesc::new(&font_desc));

        label.set_attributes(Some(&attr_list));
        label.set_text(&font_name);
    });

    let font_dd = gtk4::DropDown::builder()
        .model(&font_model)
        .factory(&font_factory)
        .selected(0)
        .enable_search(true)
        .expression(gtk4::PropertyExpression::new(
            gtk4::StringObject::static_type(),
            None::<gtk4::Expression>,
            "string",
        ))
        .tooltip_text(crate::core::gettext("Font Family"))
        .build();

    let canvas_f = canvas.clone();
    let sys_fonts_c = system_fonts.clone();
    let sync_f = is_syncing.clone();
    font_dd.connect_selected_notify(move |dd| {
        if sync_f.get() {
            return;
        }
        let idx = dd.selected() as usize;
        if idx < sys_fonts_c.len() {
            canvas_f.set_selected_font_family(&sys_fonts_c[idx]);
        }
    });

    let font_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    let font_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Family"))
        .css_classes(["dim-label", "caption"])
        .width_request(60)
        .xalign(0.0)
        .build();
    font_row.append(&font_lbl);
    font_dd.set_hexpand(true);
    font_row.append(&font_dd);
    font_content.append(&font_row);

    // Font Weight & Size Row
    let weight_size_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();

    let weight_model = gtk4::StringList::new(FONT_WEIGHT_NAMES);
    let weight_dd = gtk4::DropDown::builder()
        .model(&weight_model)
        .selected(2)
        .tooltip_text(crate::core::gettext("Font Weight"))
        .hexpand(true)
        .build();
    let canvas_w = canvas.clone();
    let sync_w = is_syncing.clone();
    weight_dd.connect_selected_notify(move |dd| {
        if sync_w.get() {
            return;
        }
        let idx = dd.selected() as usize;
        if idx < FONT_WEIGHT_VALUES.len() {
            canvas_w.set_selected_font_weight(FONT_WEIGHT_VALUES[idx]);
        }
    });

    let size_spin = gtk4::SpinButton::with_range(8.0, 288.0, 1.0);
    size_spin.set_value(32.0);
    size_spin.set_tooltip_text(Some(&crate::core::gettext("Font Size (pt)")));
    size_spin.set_width_chars(5);
    let canvas_sz = canvas.clone();
    let sync_sz = is_syncing.clone();
    size_spin.connect_value_changed(move |spin| {
        if sync_sz.get() {
            return;
        }
        canvas_sz.set_selected_font_size(spin.value() as f32);
    });

    weight_size_row.append(&weight_dd);
    weight_size_row.append(&size_spin);
    font_content.append(&weight_size_row);

    // Fine Spacing Controls: Leading (Line Height) & Kerning / Tracking
    let spacing_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();

    // Leading (Line Height)
    let lh_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .hexpand(true)
        .build();
    let lh_lbl = gtk4::Label::builder()
        .label("↕")
        .tooltip_text(crate::core::gettext("Line Height (Leading)"))
        .css_classes(["dim-label", "heading"])
        .build();
    let lh_spin = gtk4::SpinButton::with_range(0.5, 4.0, 0.1);
    lh_spin.set_digits(2);
    lh_spin.set_value(1.2);
    lh_spin.set_hexpand(true);
    let canvas_lh = canvas.clone();
    let sync_lh = is_syncing.clone();
    lh_spin.connect_value_changed(move |spin| {
        if sync_lh.get() {
            return;
        }
        canvas_lh.set_selected_line_height(spin.value() as f32);
    });
    lh_box.append(&lh_lbl);
    lh_box.append(&lh_spin);

    // Tracking (Letter Spacing)
    let ls_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .hexpand(true)
        .build();
    let ls_lbl = gtk4::Label::builder()
        .label("AV")
        .tooltip_text(crate::core::gettext("Tracking / Letter Spacing"))
        .css_classes(["dim-label", "caption"])
        .build();
    let ls_spin = gtk4::SpinButton::with_range(-20.0, 200.0, 0.5);
    ls_spin.set_digits(1);
    ls_spin.set_value(0.0);
    ls_spin.set_hexpand(true);
    let canvas_ls = canvas.clone();
    let sync_ls = is_syncing.clone();
    ls_spin.connect_value_changed(move |spin| {
        if sync_ls.get() {
            return;
        }
        canvas_ls.set_selected_letter_spacing(spin.value() as f32);
    });
    ls_box.append(&ls_lbl);
    ls_box.append(&ls_spin);

    spacing_row.append(&lh_box);
    spacing_row.append(&ls_box);
    font_content.append(&spacing_row);

    // Alignment Buttons
    let align_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["linked"])
        .halign(gtk4::Align::Center)
        .margin_top(4)
        .build();

    let img_al = gtk4::Image::from_icon_name("text-align-left-symbolic");
    let btn_align_left = gtk4::ToggleButton::builder()
        .child(&img_al)
        .tooltip_text(crate::core::gettext("Align Left"))
        .active(true)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let img_ac = gtk4::Image::from_icon_name("text-align-center-symbolic");
    let btn_align_center = gtk4::ToggleButton::builder()
        .child(&img_ac)
        .tooltip_text(crate::core::gettext("Center"))
        .group(&btn_align_left)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let img_ar = gtk4::Image::from_icon_name("text-align-right-symbolic");
    let btn_align_right = gtk4::ToggleButton::builder()
        .child(&img_ar)
        .tooltip_text(crate::core::gettext("Align Right"))
        .group(&btn_align_left)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let img_aj = gtk4::Image::from_icon_name("text-align-justify-symbolic");
    let btn_align_justify = gtk4::ToggleButton::builder()
        .child(&img_aj)
        .tooltip_text(crate::core::gettext("Justify"))
        .group(&btn_align_left)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

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

    align_box.append(&btn_align_left);
    align_box.append(&btn_align_center);
    align_box.append(&btn_align_right);
    align_box.append(&btn_align_justify);
    font_content.append(&align_box);

    // Text Decorations Row (Underline, Strikethrough, Subscript, Superscript)
    let dec_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk4::Align::Center)
        .margin_top(4)
        .build();

    let btn_u = gtk4::ToggleButton::builder()
        .label("U̲")
        .tooltip_text(crate::core::gettext("Underline"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let canvas_u = canvas.clone();
    let sync_u = is_syncing.clone();
    btn_u.connect_toggled(move |btn| {
        if !sync_u.get() {
            canvas_u.set_selected_text_underline(btn.is_active());
        }
    });

    let btn_s = gtk4::ToggleButton::builder()
        .label("S̶")
        .tooltip_text(crate::core::gettext("Strikethrough"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let canvas_s = canvas.clone();
    let sync_s = is_syncing.clone();
    btn_s.connect_toggled(move |btn| {
        if !sync_s.get() {
            canvas_s.set_selected_text_strikethrough(btn.is_active());
        }
    });

    let btn_sub = gtk4::ToggleButton::builder()
        .label("X₂")
        .tooltip_text(crate::core::gettext("Subscript"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let btn_sup = gtk4::ToggleButton::builder()
        .label("X²")
        .tooltip_text(crate::core::gettext("Superscript"))
        .group(&btn_sub)
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let canvas_sub = canvas.clone();
    let sync_sub = is_syncing.clone();
    btn_sub.connect_toggled(move |btn| {
        if sync_sub.get() {
            return;
        }
        if btn.is_active() {
            canvas_sub.set_selected_text_baseline(crate::core::element::TextBaseline::Subscript);
        } else {
            canvas_sub.set_selected_text_baseline(crate::core::element::TextBaseline::Normal);
        }
    });

    let canvas_sup = canvas.clone();
    let sync_sup = is_syncing.clone();
    btn_sup.connect_toggled(move |btn| {
        if sync_sup.get() {
            return;
        }
        if btn.is_active() {
            canvas_sup.set_selected_text_baseline(crate::core::element::TextBaseline::Superscript);
        } else {
            canvas_sup.set_selected_text_baseline(crate::core::element::TextBaseline::Normal);
        }
    });

    dec_row.append(&btn_u);
    dec_row.append(&btn_s);
    dec_row.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
    dec_row.append(&btn_sub);
    dec_row.append(&btn_sup);

    font_content.append(&dec_row);

    font_card.append(&font_content);
    container.append(&font_card);

    // ─────────────────────────────────────────────────────────────
    // Card 2: OpenType Features Support
    // ─────────────────────────────────────────────────────────────
    let ot_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .css_classes(["card"])
        .margin_bottom(4)
        .build();

    let ot_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();
    let ot_title = gtk4::Label::builder()
        .label(crate::core::gettext("OpenType Features"))
        .css_classes(["heading", "caption"])
        .build();
    ot_header.append(&gtk4::Image::from_icon_name("preferences-desktop-font-symbolic"));
    ot_header.append(&ot_title);
    ot_card.append(&ot_header);

    let ot_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    ot_card.append(&ot_sep);

    let ot_content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_start(12)
        .margin_end(12)
        .margin_top(4)
        .margin_bottom(8)
        .build();

    let create_ot_row = |label_text: &str, default_val: bool| -> (gtk4::Box, gtk4::Switch) {
        let row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .valign(gtk4::Align::Center)
            .build();
        let lbl = gtk4::Label::builder()
            .label(label_text)
            .hexpand(true)
            .xalign(0.0)
            .css_classes(["caption"])
            .build();
        let sw = gtk4::Switch::builder()
            .active(default_val)
            .valign(gtk4::Align::Center)
            .build();
        row.append(&lbl);
        row.append(&sw);
        (row, sw)
    };

    let (row_liga, sw_liga) = create_ot_row(&crate::core::gettext("Ligatures (liga, clig)"), true);
    let (row_salt, sw_salt) = create_ot_row(&crate::core::gettext("Stylistic Alternates (salt)"), false);
    let (row_frac, sw_frac) = create_ot_row(&crate::core::gettext("Fractions (frac)"), false);
    let (row_tnum, sw_tnum) = create_ot_row(&crate::core::gettext("Tabular Numerals (tnum)"), false);
    let (row_kern, sw_kern) = create_ot_row(&crate::core::gettext("Auto Kerning (kern)"), true);

    ot_content.append(&row_liga);
    ot_content.append(&row_salt);
    ot_content.append(&row_frac);
    ot_content.append(&row_tnum);
    ot_content.append(&row_kern);

    let apply_ot_features = {
        let canvas_ot = canvas.clone();
        let sw_liga_c = sw_liga.clone();
        let sw_salt_c = sw_salt.clone();
        let sw_frac_c = sw_frac.clone();
        let sw_tnum_c = sw_tnum.clone();
        let sw_kern_c = sw_kern.clone();
        let sync_ot = is_syncing.clone();

        Rc::new(move || {
            if sync_ot.get() {
                return;
            }
            let features = OpenTypeFeatures {
                ligatures: sw_liga_c.is_active(),
                contextual_alt: sw_liga_c.is_active(),
                stylistic_alts: sw_salt_c.is_active(),
                fractions: sw_frac_c.is_active(),
                tabular_numerals: sw_tnum_c.is_active(),
                kerning: sw_kern_c.is_active(),
            };
            canvas_ot.set_selected_opentype_features(features);
        })
    };

    let a_ot1 = apply_ot_features.clone();
    sw_liga.connect_state_set(move |_, _| { a_ot1(); glib::Propagation::Proceed });
    let a_ot2 = apply_ot_features.clone();
    sw_salt.connect_state_set(move |_, _| { a_ot2(); glib::Propagation::Proceed });
    let a_ot3 = apply_ot_features.clone();
    sw_frac.connect_state_set(move |_, _| { a_ot3(); glib::Propagation::Proceed });
    let a_ot4 = apply_ot_features.clone();
    sw_tnum.connect_state_set(move |_, _| { a_ot4(); glib::Propagation::Proceed });
    let a_ot5 = apply_ot_features.clone();
    sw_kern.connect_state_set(move |_, _| { a_ot5(); glib::Propagation::Proceed });

    ot_card.append(&ot_content);
    container.append(&ot_card);

    // ─────────────────────────────────────────────────────────────
    // Card 3: Text Case & Capitalization
    // ─────────────────────────────────────────────────────────────
    let case_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .css_classes(["card"])
        .margin_bottom(4)
        .build();

    let case_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();
    case_header.append(&gtk4::Label::builder()
        .label(crate::core::gettext("Text Capitalization"))
        .css_classes(["heading", "caption"])
        .build());
    case_card.append(&case_header);

    let case_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    case_card.append(&case_sep);

    let case_content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_start(10)
        .margin_end(10)
        .margin_top(4)
        .margin_bottom(8)
        .build();

    let case_model = gtk4::StringList::new(&[
        &crate::core::gettext("Normal (As Typed)"),
        &crate::core::gettext("UPPERCASE"),
        &crate::core::gettext("lowercase"),
        &crate::core::gettext("Title Case"),
        &crate::core::gettext("SMALL CAPS"),
    ]);
    let case_dd = gtk4::DropDown::builder()
        .model(&case_model)
        .selected(0)
        .hexpand(true)
        .build();

    let canvas_case = canvas.clone();
    let sync_case = is_syncing.clone();
    case_dd.connect_selected_notify(move |dd| {
        if sync_case.get() {
            return;
        }
        let tc = match dd.selected() {
            1 => TextCase::Uppercase,
            2 => TextCase::Lowercase,
            3 => TextCase::TitleCase,
            4 => TextCase::SmallCaps,
            _ => TextCase::Normal,
        };
        canvas_case.set_selected_text_case(tc);
    });

    case_content.append(&case_dd);
    case_card.append(&case_content);
    container.append(&case_card);

    // ─────────────────────────────────────────────────────────────
    // Card 4: Text on Path Controls
    // ─────────────────────────────────────────────────────────────
    let path_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .css_classes(["card"])
        .margin_bottom(4)
        .build();

    let path_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();
    path_header.append(&crate::ui::icons::make_symbolic_image("segment-curve-symbolic", 16));
    path_header.append(&gtk4::Label::builder()
        .label(crate::core::gettext("Text on Path"))
        .css_classes(["heading", "caption"])
        .build());
    path_card.append(&path_header);

    let path_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    path_card.append(&path_sep);

    let path_content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(4)
        .margin_bottom(8)
        .build();

    // Attach / Detach Buttons
    let attach_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();

    let btn_attach = gtk4::Button::builder()
        .label(crate::core::gettext("Attach to Path"))
        .icon_name("insert-link-symbolic")
        .tooltip_text(crate::core::gettext("Attach selected Text element to selected Path element"))
        .hexpand(true)
        .css_classes(["suggested-action"])
        .build();
    let canvas_att = canvas.clone();
    btn_attach.connect_clicked(move |_| {
        canvas_att.attach_selected_text_to_path();
    });

    let btn_detach = gtk4::Button::builder()
        .icon_name("edit-delete-symbolic")
        .tooltip_text(crate::core::gettext("Detach Text from Path"))
        .css_classes(["flat"])
        .build();
    let canvas_det = canvas.clone();
    btn_detach.connect_clicked(move |_| {
        canvas_det.detach_selected_text_from_path();
    });

    attach_box.append(&btn_attach);
    attach_box.append(&btn_detach);
    path_content.append(&attach_box);

    // Path Offset & Spacing
    let po_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    let po_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Offset:"))
        .css_classes(["dim-label", "caption"])
        .build();
    let po_spin = gtk4::SpinButton::with_range(-1000.0, 5000.0, 5.0);
    po_spin.set_digits(1);
    po_spin.set_value(0.0);
    po_spin.set_hexpand(true);
    let canvas_po = canvas.clone();
    let sync_po = is_syncing.clone();
    po_spin.connect_value_changed(move |spin| {
        if sync_po.get() {
            return;
        }
        canvas_po.set_selected_text_path_offset(spin.value() as f32);
    });
    po_row.append(&po_lbl);
    po_row.append(&po_spin);
    path_content.append(&po_row);

    // Vertical Alignment & Glyph Orientation Dropdowns
    let valign_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    let valign_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("V-Align:"))
        .css_classes(["dim-label", "caption"])
        .build();
    let valign_model = gtk4::StringList::new(&[
        &crate::core::gettext("Baseline"),
        &crate::core::gettext("Ascender (Top)"),
        &crate::core::gettext("Descender (Bottom)"),
        &crate::core::gettext("Center"),
    ]);
    let valign_dd = gtk4::DropDown::builder()
        .model(&valign_model)
        .selected(0)
        .hexpand(true)
        .build();
    let canvas_valign = canvas.clone();
    let sync_valign = is_syncing.clone();
    valign_dd.connect_selected_notify(move |dd| {
        if sync_valign.get() {
            return;
        }
        let v = match dd.selected() {
            1 => crate::core::element::PathVerticalAlign::Ascender,
            2 => crate::core::element::PathVerticalAlign::Descender,
            3 => crate::core::element::PathVerticalAlign::Center,
            _ => crate::core::element::PathVerticalAlign::Baseline,
        };
        canvas_valign.set_selected_text_path_valign(v);
    });
    valign_row.append(&valign_lbl);
    valign_row.append(&valign_dd);
    path_content.append(&valign_row);

    let ori_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    let ori_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Orient:"))
        .css_classes(["dim-label", "caption"])
        .build();
    let ori_model = gtk4::StringList::new(&[
        &crate::core::gettext("Follow Curve (Tangent)"),
        &crate::core::gettext("Keep Upright (Vertical)"),
        &crate::core::gettext("Perpendicular"),
    ]);
    let ori_dd = gtk4::DropDown::builder()
        .model(&ori_model)
        .selected(0)
        .hexpand(true)
        .build();
    let canvas_g_ori = canvas.clone();
    let sync_g_ori = is_syncing.clone();
    ori_dd.connect_selected_notify(move |dd| {
        if sync_g_ori.get() {
            return;
        }
        let o = match dd.selected() {
            1 => crate::core::element::PathGlyphOrientation::Upright,
            2 => crate::core::element::PathGlyphOrientation::Perpendicular,
            _ => crate::core::element::PathGlyphOrientation::Tangent,
        };
        canvas_g_ori.set_selected_text_path_glyph_orientation(o);
    });
    ori_row.append(&ori_lbl);
    ori_row.append(&ori_dd);
    path_content.append(&ori_row);

    // Invert Side & Orientation Toggles
    let (row_inv, sw_inv) = create_ot_row(&crate::core::gettext("Invert Path Side"), false);
    let canvas_inv = canvas.clone();
    let sync_inv = is_syncing.clone();
    sw_inv.connect_state_set(move |_, state| {
        if !sync_inv.get() {
            canvas_inv.set_selected_text_path_inverted(state);
        }
        glib::Propagation::Proceed
    });
    path_content.append(&row_inv);

    let (row_ori, sw_ori) = create_ot_row(&crate::core::gettext("Follow Path Tangent"), true);
    let canvas_ori = canvas.clone();
    let sync_ori = is_syncing.clone();
    sw_ori.connect_state_set(move |_, state| {
        if !sync_ori.get() {
            canvas_ori.set_selected_text_path_orientation(state);
        }
        glib::Propagation::Proceed
    });
    path_content.append(&row_ori);

    let (row_rep, sw_rep) = create_ot_row(&crate::core::gettext("Repeat Along Path"), false);
    let canvas_rep = canvas.clone();
    let sync_rep = is_syncing.clone();
    sw_rep.connect_state_set(move |_, state| {
        if !sync_rep.get() {
            canvas_rep.set_selected_text_path_repeat(state);
        }
        glib::Propagation::Proceed
    });
    path_content.append(&row_rep);

    path_card.append(&path_content);

    // Reactive sensitivity update for Text-on-Path controls
    let canvas_sens = canvas.clone();
    let btn_att_c = btn_attach.clone();
    let btn_det_c = btn_detach.clone();
    let po_spin_c = po_spin.clone();
    let po_lbl_c = po_lbl.clone();
    let sw_inv_c = sw_inv.clone();
    let sw_ori_c = sw_ori.clone();
    let sw_rep_c = sw_rep.clone();
    let valign_dd_c = valign_dd.clone();
    let ori_dd_c = ori_dd.clone();

    path_card.add_tick_callback(move |_, _| {
        let state_rc = canvas_sens.state();
        let state = state_rc.borrow();
        let sel_ids = &state.document.selected_ids;
        let elems: Vec<&crate::core::Element> = sel_ids
            .iter()
            .filter_map(|id| state.document.find_element(*id))
            .collect();

        let has_text = elems.iter().any(|e| matches!(e, crate::core::Element::Text(_)));
        let has_path = elems.iter().any(|e| !matches!(e, crate::core::Element::Text(_)));
        let is_attached = elems.iter().any(|e| {
            if let crate::core::Element::Text(t) = e {
                t.path_id.is_some()
            } else {
                false
            }
        });

        let can_attach = (has_text && has_path) || is_attached;
        let can_detach = is_attached;
        let can_edit_path = is_attached || (has_text && has_path);

        if btn_att_c.is_sensitive() != can_attach {
            btn_att_c.set_sensitive(can_attach);
        }
        if btn_det_c.is_sensitive() != can_detach {
            btn_det_c.set_sensitive(can_detach);
        }
        if po_spin_c.is_sensitive() != can_edit_path {
            po_spin_c.set_sensitive(can_edit_path);
            po_lbl_c.set_sensitive(can_edit_path);
            valign_dd_c.set_sensitive(can_edit_path);
            ori_dd_c.set_sensitive(can_edit_path);
            sw_inv_c.set_sensitive(can_edit_path);
            sw_ori_c.set_sensitive(can_edit_path);
            sw_rep_c.set_sensitive(can_edit_path);
        }

        glib::ControlFlow::Continue
    });

    container.append(&path_card);

    container
}
