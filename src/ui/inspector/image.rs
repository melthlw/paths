use gtk4::gio;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::ui::canvas::ops_document::ImageAdjustments;
use crate::ui::canvas::CanvasWidget;

#[allow(dead_code)]
pub struct ImageSection {
    pub container: gtk4::Box,
    pub content_box: gtk4::Box,
    pub empty_state_box: gtk4::Box,
    pub name_label: gtk4::Label,
    pub dim_label: gtk4::Label,
    pub brightness_scale: gtk4::Scale,
    pub contrast_scale: gtk4::Scale,
    pub saturation_scale: gtk4::Scale,
    pub hue_scale: gtk4::Scale,
    pub blur_scale: gtk4::Scale,
    pub invert_switch: gtk4::Switch,
    pub grayscale_switch: gtk4::Switch,
    pub sepia_switch: gtk4::Switch,
    pub btn_reset_adjustments: gtk4::Button,
    pub btn_reset_aspect: gtk4::Button,
    pub is_updating: Rc<Cell<bool>>,
    pub update_fn: Rc<dyn Fn(Option<(String, crate::core::Rect, Option<(f32, f32)>, f32)>, Option<ImageAdjustments>)>,
}

/// Create a sleek inline compact slider row: Label on left, compact slider + value badge on right
fn create_compact_slider_row(
    icon_name: &str,
    label_text: &str,
    min: f64,
    max: f64,
    step: f64,
    default_val: f64,
    suffix: &str,
) -> (gtk4::Box, gtk4::Scale, gtk4::Label) {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(3)
        .margin_bottom(3)
        .valign(gtk4::Align::Center)
        .build();

    let left_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let icon = crate::ui::icons::make_symbolic_image(icon_name, 14);
    icon.add_css_class("dim-label");
    left_box.append(&icon);

    let lbl = gtk4::Label::builder()
        .label(label_text)
        .css_classes(["body"])
        .halign(gtk4::Align::Start)
        .build();
    left_box.append(&lbl);
    row.append(&left_box);

    let scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&gtk4::Adjustment::new(default_val, min, max, step, step * 5.0, 0.0))
        .draw_value(false)
        .width_request(100)
        .css_classes(["fine-tune"])
        .valign(gtk4::Align::Center)
        .build();
    row.append(&scale);

    let val_lbl = gtk4::Label::builder()
        .label(&format!("{:.0}{}", default_val, suffix))
        .css_classes(["caption", "numeric", "dim-label"])
        .width_request(38)
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .build();
    row.append(&val_lbl);

    (row, scale, val_lbl)
}

/// Create a circular stepper row (like Padding 10 [-] [+] in modern HIG design)
fn create_stepper_row<F: Fn(i64) + 'static>(
    icon_name: &str,
    label_text: &str,
    init_val: i64,
    min_val: i64,
    max_val: i64,
    step: i64,
    suffix: &str,
    on_change: F,
) -> (gtk4::Box, gtk4::Label, Rc<Cell<i64>>) {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(3)
        .margin_bottom(3)
        .valign(gtk4::Align::Center)
        .build();

    let left_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let icon = crate::ui::icons::make_symbolic_image(icon_name, 14);
    icon.add_css_class("dim-label");
    left_box.append(&icon);

    let lbl = gtk4::Label::builder()
        .label(label_text)
        .css_classes(["body"])
        .halign(gtk4::Align::Start)
        .build();
    left_box.append(&lbl);
    row.append(&left_box);

    let val_cell = Rc::new(Cell::new(init_val));
    let val_lbl = gtk4::Label::builder()
        .label(&format!("{}{}", init_val, suffix))
        .css_classes(["image-stepper-val", "numeric"])
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .build();
    row.append(&val_lbl);

    let stepper_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(3)
        .valign(gtk4::Align::Center)
        .build();

    let btn_minus = gtk4::Button::builder()
        .icon_name("list-remove-symbolic")
        .css_classes(["flat", "circular"])
        .valign(gtk4::Align::Center)
        .build();

    let btn_plus = gtk4::Button::builder()
        .icon_name("list-add-symbolic")
        .css_classes(["flat", "circular"])
        .valign(gtk4::Align::Center)
        .build();

    stepper_box.append(&btn_minus);
    stepper_box.append(&btn_plus);
    row.append(&stepper_box);

    let on_ch_rc = Rc::new(on_change);

    {
        let vc = val_cell.clone();
        let vl = val_lbl.clone();
        let suf = suffix.to_string();
        let oc = on_ch_rc.clone();
        btn_minus.connect_clicked(move |_| {
            let cur = vc.get();
            if cur > min_val {
                let next = (cur - step).max(min_val);
                vc.set(next);
                vl.set_text(&format!("{}{}", next, suf));
                oc(next);
            }
        });
    }

    {
        let vc = val_cell.clone();
        let vl = val_lbl.clone();
        let suf = suffix.to_string();
        let oc = on_ch_rc;
        btn_plus.connect_clicked(move |_| {
            let cur = vc.get();
            if cur < max_val {
                let next = (cur + step).min(max_val);
                vc.set(next);
                vl.set_text(&format!("{}{}", next, suf));
                oc(next);
            }
        });
    }

    (row, val_lbl, val_cell)
}

/// Create a clean switch row
fn create_switch_row(
    label_text: &str,
    tooltip: &str,
) -> (gtk4::Box, gtk4::Switch) {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(3)
        .margin_bottom(3)
        .tooltip_text(tooltip)
        .valign(gtk4::Align::Center)
        .build();

    let lbl = gtk4::Label::builder()
        .label(label_text)
        .css_classes(["body"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    row.append(&lbl);

    let sw = gtk4::Switch::builder()
        .valign(gtk4::Align::Center)
        .build();
    row.append(&sw);

    (row, sw)
}

pub fn build_image_section(canvas: &CanvasWidget) -> ImageSection {
    let is_updating = Rc::new(Cell::new(false));

    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(0)
        .hexpand(true)
        .vexpand(true)
        .build();

    // ─────────────────────────────────────────────────────────────
    // Empty State (when no image element is selected)
    // ─────────────────────────────────────────────────────────────
    let empty_state_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_top(40)
        .margin_bottom(40)
        .margin_start(20)
        .margin_end(20)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .build();

    let empty_icon = crate::ui::icons::make_symbolic_image("tool-image-symbolic", 48);
    empty_icon.add_css_class("dim-label");
    empty_state_box.append(&empty_icon);

    let empty_title = gtk4::Label::builder()
        .label(&crate::core::gettext("No Image Selected"))
        .css_classes(["title-4"])
        .halign(gtk4::Align::Center)
        .build();
    empty_state_box.append(&empty_title);

    let empty_subtitle = gtk4::Label::builder()
        .label(&crate::core::gettext(
            "Select a bitmap image to adjust colors or vectorize to paths, or rasterize selected vectors into an image.",
        ))
        .css_classes(["body", "dim-label"])
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .halign(gtk4::Align::Center)
        .build();
    empty_state_box.append(&empty_subtitle);

    let btn_activate_tool = gtk4::Button::builder()
        .label(&crate::core::gettext("Create Image Frame"))
        .css_classes(["suggested-action", "pill"])
        .halign(gtk4::Align::Center)
        .margin_top(8)
        .build();
    {
        let cv = canvas.clone();
        btn_activate_tool.connect_clicked(move |_| {
            cv.set_active_tool("image");
        });
    }
    empty_state_box.append(&btn_activate_tool);

    let btn_rasterize = gtk4::Button::builder()
        .label(&crate::core::gettext("Rasterize Selection to Bitmap"))
        .css_classes(["pill"])
        .halign(gtk4::Align::Center)
        .margin_top(4)
        .build();
    let btn_rasterize_c = btn_rasterize.clone();
    {
        let cv = canvas.clone();
        btn_rasterize.connect_clicked(move |_| {
            cv.rasterize_selected_to_image();
        });
    }
    empty_state_box.append(&btn_rasterize);

    let btn_import = gtk4::Button::builder()
        .label(&crate::core::gettext("Import Image File..."))
        .css_classes(["flat", "pill"])
        .halign(gtk4::Align::Center)
        .margin_top(2)
        .build();
    {
        let cv = canvas.clone();
        btn_import.connect_clicked(move |btn| {
            let file_dialog = gtk4::FileDialog::builder()
                .title(&crate::core::gettext("Choose Image File"))
                .modal(true)
                .build();
            let filter = gtk4::FileFilter::new();
            filter.set_name(Some(&crate::core::gettext("Image Files (*.png, *.jpg, *.webp)")));
            filter.add_pattern("*.png");
            filter.add_pattern("*.PNG");
            filter.add_pattern("*.jpg");
            filter.add_pattern("*.JPG");
            filter.add_pattern("*.jpeg");
            filter.add_pattern("*.JPEG");
            filter.add_pattern("*.webp");
            filter.add_pattern("*.WEBP");
            let filters = gio::ListStore::new::<gtk4::FileFilter>();
            filters.append(&filter);
            file_dialog.set_filters(Some(&filters));

            let root_win = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
            let cv_inner = cv.clone();
            file_dialog.open(root_win.as_ref(), gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        if let Some(path_str) = path.to_str() {
                            let _ = cv_inner.import_file(path_str);
                        }
                    }
                }
            });
        });
    }
    empty_state_box.append(&btn_import);
    container.append(&empty_state_box);

    // ─────────────────────────────────────────────────────────────
    // Active Content Box
    // ─────────────────────────────────────────────────────────────
    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(10)
        .margin_start(10)
        .margin_end(10)
        .margin_top(8)
        .margin_bottom(16)
        .visible(false)
        .build();

    // ─────────────────────────────────────────────────────────────
    // TOP SEGMENTED VIEW SWITCHER: [ 🎨 Ajustes ] [ ⚡ Vetorização ] [ ☰ Ambos ]
    // ─────────────────────────────────────────────────────────────
    let view_switch_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(0)
        .homogeneous(true)
        .css_classes(["linked", "image-segmented-pills"])
        .margin_bottom(2)
        .build();

    let tab_adj_btn = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Adjustments"))
        .active(true)
        .build();
    let tab_trace_btn = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Vectorize"))
        .build();
    let tab_all_btn = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("All"))
        .build();

    view_switch_box.append(&tab_adj_btn);
    view_switch_box.append(&tab_trace_btn);
    view_switch_box.append(&tab_all_btn);
    content_box.append(&view_switch_box);

    // ─────────────────────────────────────────────────────────────
    // 1. Arquivo Atual Card (Metadata, Replace & Quick Transforms)
    // ─────────────────────────────────────────────────────────────
    let meta_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(0)
        .css_classes(["card", "image-meta-card"])
        .build();

    // Header: Icon + "Arquivo Atual" + Replace Image button
    let meta_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();

    let img_icon = crate::ui::icons::make_symbolic_image("tool-image-symbolic", 16);
    meta_header.append(&img_icon);

    let meta_title = gtk4::Label::builder()
        .label(&crate::core::gettext("Current File"))
        .css_classes(["heading", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    meta_header.append(&meta_title);

    let meta_spacer = gtk4::Box::builder().hexpand(true).build();
    meta_header.append(&meta_spacer);

    let btn_replace = gtk4::Button::builder()
        .css_classes(["flat", "pill"])
        .tooltip_text(crate::core::gettext("Replace image file..."))
        .valign(gtk4::Align::Center)
        .build();
    let replace_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .build();
    replace_box.append(&crate::ui::icons::make_symbolic_image("document-open-symbolic", 13));
    replace_box.append(&gtk4::Label::builder().label(&crate::core::gettext("Replace")).css_classes(["caption"]).build());
    btn_replace.set_child(Some(&replace_box));

    {
        let cv = canvas.clone();
        btn_replace.connect_clicked(move |btn| {
            let file_dialog = gtk4::FileDialog::builder()
                .title(&crate::core::gettext("Choose Image File"))
                .modal(true)
                .build();
            let filter = gtk4::FileFilter::new();
            filter.set_name(Some(&crate::core::gettext("Image Files (*.png, *.jpg, *.webp, *.svg, *.gif)")));
            filter.add_pattern("*.png");
            filter.add_pattern("*.PNG");
            filter.add_pattern("*.jpg");
            filter.add_pattern("*.JPG");
            filter.add_pattern("*.jpeg");
            filter.add_pattern("*.JPEG");
            filter.add_pattern("*.webp");
            filter.add_pattern("*.WEBP");
            filter.add_pattern("*.svg");
            filter.add_pattern("*.SVG");
            filter.add_pattern("*.gif");
            filter.add_pattern("*.GIF");
            let filters = gio::ListStore::new::<gtk4::FileFilter>();
            filters.append(&filter);
            file_dialog.set_filters(Some(&filters));

            let root_win = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
            let cv_inner = cv.clone();
            file_dialog.open(root_win.as_ref(), gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        if let Some(path_str) = path.to_str() {
                            let _ = cv_inner.replace_selected_image(path_str);
                        }
                    }
                }
            });
        });
    }
    meta_header.append(&btn_replace);
    meta_card.append(&meta_header);

    meta_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // Row 1: File Name
    let name_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(1)
        .css_classes(["image-meta-row"])
        .build();
    let name_cap = gtk4::Label::builder()
        .label(&crate::core::gettext("Name"))
        .css_classes(["image-meta-caption"])
        .halign(gtk4::Align::Start)
        .build();
    name_row.append(&name_cap);
    let name_label = gtk4::Label::builder()
        .label(&crate::core::gettext("Image"))
        .css_classes(["image-meta-val"])
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .halign(gtk4::Align::Start)
        .build();
    name_row.append(&name_label);
    meta_card.append(&name_row);

    meta_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // Row 2: Dimensions & Aspect Ratio
    let dim_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .css_classes(["image-meta-row"])
        .valign(gtk4::Align::Center)
        .build();

    let dim_info_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(1)
        .hexpand(true)
        .halign(gtk4::Align::Start)
        .build();
    let dim_cap = gtk4::Label::builder()
        .label(&crate::core::gettext("Dimensions"))
        .css_classes(["image-meta-caption"])
        .halign(gtk4::Align::Start)
        .build();
    dim_info_box.append(&dim_cap);
    let dim_label = gtk4::Label::builder()
        .label("—")
        .css_classes(["image-meta-val"])
        .halign(gtk4::Align::Start)
        .build();
    dim_info_box.append(&dim_label);
    dim_row.append(&dim_info_box);

    let btn_reset_aspect = gtk4::Button::builder()
        .css_classes(["flat", "pill"])
        .tooltip_text(crate::core::gettext("Restore original aspect ratio"))
        .valign(gtk4::Align::Center)
        .build();
    let aspect_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .build();
    aspect_box.append(&crate::ui::icons::make_symbolic_image("lock-aspect-ratio-symbolic", 13));
    aspect_box.append(&gtk4::Label::builder().label(&crate::core::gettext("Ratio")).css_classes(["caption"]).build());
    btn_reset_aspect.set_child(Some(&aspect_box));
    {
        let cv = canvas.clone();
        btn_reset_aspect.connect_clicked(move |_| {
            cv.reset_selected_image_aspect_ratio();
        });
    }
    dim_row.append(&btn_reset_aspect);
    meta_card.append(&dim_row);

    meta_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // Row 3: Rotation and Flip Controls (Matching reference: Rotation [ ↶ ] [ ↷ ])
    let transform_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .css_classes(["image-meta-row"])
        .valign(gtk4::Align::Center)
        .build();

    let trans_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Rotation & Flip"))
        .css_classes(["body"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    transform_row.append(&trans_lbl);

    let trans_actions = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .css_classes(["linked"])
        .valign(gtk4::Align::Center)
        .build();

    let btn_rot_l = gtk4::Button::builder()
        .icon_name("object-rotate-left-symbolic")
        .css_classes(["flat"])
        .tooltip_text(crate::core::gettext("Rotate -90° counter-clockwise"))
        .build();
    {
        let cv = canvas.clone();
        btn_rot_l.connect_clicked(move |_| {
            cv.rotate_selected_deg(-90.0);
        });
    }
    trans_actions.append(&btn_rot_l);

    let btn_rot_r = gtk4::Button::builder()
        .icon_name("object-rotate-right-symbolic")
        .css_classes(["flat"])
        .tooltip_text(crate::core::gettext("Rotate +90° clockwise"))
        .build();
    {
        let cv = canvas.clone();
        btn_rot_r.connect_clicked(move |_| {
            cv.rotate_selected_deg(90.0);
        });
    }
    trans_actions.append(&btn_rot_r);

    let btn_flip_h = gtk4::Button::builder()
        .icon_name("object-flip-horizontal-symbolic")
        .css_classes(["flat"])
        .tooltip_text(crate::core::gettext("Flip horizontally"))
        .build();
    {
        let cv = canvas.clone();
        btn_flip_h.connect_clicked(move |_| {
            cv.flip_horizontal();
        });
    }
    trans_actions.append(&btn_flip_h);

    let btn_flip_v = gtk4::Button::builder()
        .icon_name("object-flip-vertical-symbolic")
        .css_classes(["flat"])
        .tooltip_text(crate::core::gettext("Flip vertically"))
        .build();
    {
        let cv = canvas.clone();
        btn_flip_v.connect_clicked(move |_| {
            cv.flip_vertical();
        });
    }
    trans_actions.append(&btn_flip_v);

    transform_row.append(&trans_actions);
    meta_card.append(&transform_row);

    content_box.append(&meta_card);

    // ─────────────────────────────────────────────────────────────
    // 2. Ajustes de Cor Card (Compact sliders, presets & switches)
    // ─────────────────────────────────────────────────────────────
    let adj_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(0)
        .css_classes(["card"])
        .build();

    let adj_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();

    let adj_icon = crate::ui::icons::make_symbolic_image("applications-graphics-symbolic", 15);
    adj_header.append(&adj_icon);

    let adj_title = gtk4::Label::builder()
        .label(&crate::core::gettext("Color Adjustments"))
        .css_classes(["heading", "caption"])
        .halign(gtk4::Align::Start)
        .build();
    adj_header.append(&adj_title);

    let adj_spacer = gtk4::Box::builder().hexpand(true).build();
    adj_header.append(&adj_spacer);

    let btn_reset_adjustments = gtk4::Button::builder()
        .icon_name("view-refresh-symbolic")
        .css_classes(["flat", "circular"])
        .tooltip_text(crate::core::gettext("Reset all adjustments"))
        .valign(gtk4::Align::Center)
        .build();
    {
        let cv = canvas.clone();
        btn_reset_adjustments.connect_clicked(move |_| {
            cv.reset_selected_image_adjustments();
        });
    }
    adj_header.append(&btn_reset_adjustments);
    adj_card.append(&adj_header);

    adj_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // ── Presets Chips Flow Bar ──
    let presets_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .margin_start(8)
        .margin_end(8)
        .margin_top(6)
        .margin_bottom(6)
        .halign(gtk4::Align::Center)
        .build();

    let preset_chips = [
        ("Normal", 0.0, 1.0, 1.0, 0.0, 0.0, false, false, false),
        ("Vibrant", 0.04, 1.18, 1.45, 0.0, 0.0, false, false, false),
        ("B&W", -0.02, 1.35, 0.0, 0.0, 0.0, false, true, false),
        ("Vintage", -0.04, 1.12, 0.85, 0.0, 0.0, false, false, true),
        ("Warm", 0.03, 1.05, 1.15, 15.0, 0.0, false, false, false),
        ("Cool", 0.02, 1.08, 1.10, -15.0, 0.0, false, false, false),
    ];

    for (p_name, br, ct, st, hr, bl, inv, gs, sp) in preset_chips {
        let chip = gtk4::Button::builder()
            .label(&crate::core::gettext(p_name))
            .css_classes(["image-preset-chip", "flat"])
            .build();
        let cv = canvas.clone();
        chip.connect_clicked(move |_| {
            cv.set_selected_image_adjustments(
                ImageAdjustments {
                    brightness: br,
                    contrast: ct,
                    saturation: st,
                    hue_rotate: hr,
                    blur: bl,
                    invert: inv,
                    grayscale: gs,
                    sepia: sp,
                },
                true,
            );
        });
        presets_row.append(&chip);
    }
    adj_card.append(&presets_row);

    adj_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // ── High Density Compact Sliders ──
    let (b_row, brightness_scale, b_val_lbl) = create_compact_slider_row(
        "weather-clear-symbolic",
        &crate::core::gettext("Brightness"),
        -100.0,
        100.0,
        1.0,
        0.0,
        "%",
    );
    adj_card.append(&b_row);

    let (c_row, contrast_scale, c_val_lbl) = create_compact_slider_row(
        "contrast-symbolic",
        &crate::core::gettext("Contrast"),
        -100.0,
        100.0,
        1.0,
        0.0,
        "%",
    );
    adj_card.append(&c_row);

    let (s_row, saturation_scale, s_val_lbl) = create_compact_slider_row(
        "color-select-symbolic",
        &crate::core::gettext("Saturation"),
        -100.0,
        100.0,
        1.0,
        0.0,
        "%",
    );
    adj_card.append(&s_row);

    let (h_row, hue_scale, h_val_lbl) = create_compact_slider_row(
        "rotate-right-symbolic",
        &crate::core::gettext("Hue Rotate"),
        -180.0,
        180.0,
        1.0,
        0.0,
        "°",
    );
    adj_card.append(&h_row);

    let (blur_row, blur_scale, blur_val_lbl) = create_compact_slider_row(
        "edit-find-symbolic",
        &crate::core::gettext("Blur"),
        0.0,
        50.0,
        0.5,
        0.0,
        " px",
    );
    adj_card.append(&blur_row);

    adj_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // ── Switches ──
    let (inv_row, invert_switch) = create_switch_row(
        &crate::core::gettext("Invert Colors"),
        &crate::core::gettext("Inverts the RGB color spectrum"),
    );
    adj_card.append(&inv_row);

    let (gray_row, grayscale_switch) = create_switch_row(
        &crate::core::gettext("Black & White (Grayscale)"),
        &crate::core::gettext("Converts image to monochrome grayscale"),
    );
    adj_card.append(&gray_row);

    let (sepia_row, sepia_switch) = create_switch_row(
        &crate::core::gettext("Sepia Tone"),
        &crate::core::gettext("Applies classic warm photographic tint"),
    );
    adj_card.append(&sepia_row);

    content_box.append(&adj_card);

    // ─────────────────────────────────────────────────────────────
    // 3. Vetorização de Bitmap Card (Trace Engine, Steppers, Mode Pills)
    // ─────────────────────────────────────────────────────────────
    let trace_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(0)
        .css_classes(["card"])
        .build();

    let trace_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();

    let trace_icon = crate::ui::icons::make_symbolic_image("object-to-path-symbolic", 15);
    trace_header.append(&trace_icon);

    let trace_title = gtk4::Label::builder()
        .label(&crate::core::gettext("Bitmap Vectorization"))
        .css_classes(["heading", "caption"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    trace_header.append(&trace_title);
    trace_card.append(&trace_header);

    trace_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // Mode Segmented Pill Bar: [ Monocromático ] [ Cores ] [ Contornos ]
    let mode_pill_bar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(0)
        .homogeneous(true)
        .css_classes(["linked", "image-segmented-pills"])
        .margin_start(10)
        .margin_end(10)
        .margin_top(8)
        .margin_bottom(6)
        .build();

    let btn_mode_mono = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Monochrome"))
        .active(true)
        .build();
    let btn_mode_color = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Colors"))
        .build();
    let btn_mode_edge = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Outlines"))
        .build();

    mode_pill_bar.append(&btn_mode_mono);
    mode_pill_bar.append(&btn_mode_color);
    mode_pill_bar.append(&btn_mode_edge);
    trace_card.append(&mode_pill_bar);

    let current_trace_mode = Rc::new(Cell::new(0usize)); // 0: Mono, 1: Color, 2: Outlines

    trace_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // Parameter: Threshold Slider (Monochrome / Outlines)
    let (thresh_row, thresh_scale, thresh_lbl) = create_compact_slider_row(
        "weather-clear-night-symbolic",
        &crate::core::gettext("Threshold"),
        1.0,
        99.0,
        1.0,
        50.0,
        "%",
    );
    trace_card.append(&thresh_row);

    // Parameter: Color Layers (Stepper Control: 4 [-] [+])
    let colors_val_cell = Rc::new(Cell::new(4i64));
    let (colors_row, _colors_lbl, colors_val_ref) = create_stepper_row(
        "color-select-symbolic",
        &crate::core::gettext("Color Layers"),
        4,
        2,
        16,
        1,
        "",
        {
            let cv_c = colors_val_cell.clone();
            move |v| {
                cv_c.set(v);
            }
        },
    );
    colors_row.set_visible(false);
    trace_card.append(&colors_row);

    // Parameter: Noise Filter (Stepper Control: 8 px [-] [+])
    let noise_val_cell = Rc::new(Cell::new(8i64));
    let (noise_row, _noise_lbl, noise_val_ref) = create_stepper_row(
        "view-grid-symbolic",
        &crate::core::gettext("Noise Filter"),
        8,
        0,
        50,
        1,
        " px",
        {
            let nv_c = noise_val_cell.clone();
            move |v| {
                nv_c.set(v);
            }
        },
    );
    trace_card.append(&noise_row);

    // Parameter: Detail / RDP Simplification
    let (detail_row, detail_scale, detail_lbl) = create_compact_slider_row(
        "pen-simplify-symbolic",
        &crate::core::gettext("Detail (RDP)"),
        0.2,
        4.0,
        0.1,
        1.0,
        "",
    );
    trace_card.append(&detail_row);

    // Parameter: Curve Smoothness
    let (smooth_row, smooth_scale, smooth_lbl) = create_compact_slider_row(
        "node-smooth-symbolic",
        &crate::core::gettext("Smoothness"),
        0.0,
        100.0,
        5.0,
        65.0,
        "%",
    );
    trace_card.append(&smooth_row);

    // Connect mode pill toggles with radio behavior
    {
        let m_cell = current_trace_mode.clone();
        let b_mono = btn_mode_mono.clone();
        let b_color = btn_mode_color.clone();
        let b_edge = btn_mode_edge.clone();
        let r_thresh = thresh_row.clone();
        let r_colors = colors_row.clone();

        b_mono.connect_toggled({
            let m_c = m_cell.clone();
            let b_col = b_color.clone();
            let b_ed = b_edge.clone();
            let r_th = r_thresh.clone();
            let r_co = r_colors.clone();
            move |b| {
                if b.is_active() {
                    m_c.set(0);
                    b_col.set_active(false);
                    b_ed.set_active(false);
                    r_th.set_visible(true);
                    r_co.set_visible(false);
                }
            }
        });

        b_color.connect_toggled({
            let m_c = m_cell.clone();
            let b_mo = b_mono.clone();
            let b_ed = b_edge.clone();
            let r_th = r_thresh.clone();
            let r_co = r_colors.clone();
            move |b| {
                if b.is_active() {
                    m_c.set(1);
                    b_mo.set_active(false);
                    b_ed.set_active(false);
                    r_th.set_visible(false);
                    r_co.set_visible(true);
                }
            }
        });

        b_edge.connect_toggled({
            let m_c = m_cell;
            let b_mo = b_mono;
            let b_col = b_color;
            let r_th = r_thresh;
            let r_co = r_colors;
            move |b| {
                if b.is_active() {
                    m_c.set(2);
                    b_mo.set_active(false);
                    b_col.set_active(false);
                    r_th.set_visible(true);
                    r_co.set_visible(false);
                }
            }
        });
    }

    // Connect slider readout labels
    {
        let lbl = thresh_lbl;
        thresh_scale.connect_value_changed(move |sc| {
            lbl.set_text(&format!("{:.0}%", sc.value()));
        });
    }
    {
        let lbl = detail_lbl;
        detail_scale.connect_value_changed(move |sc| {
            lbl.set_text(&format!("{:.1}", sc.value()));
        });
    }
    {
        let lbl = smooth_lbl;
        smooth_scale.connect_value_changed(move |sc| {
            lbl.set_text(&format!("{:.0}%", sc.value()));
        });
    }

    trace_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // Switches
    let (inv_trace_row, inv_trace_sw) = create_switch_row(
        &crate::core::gettext("Invert Selection"),
        &crate::core::gettext("Invert foreground/background vector cutout"),
    );
    trace_card.append(&inv_trace_row);

    let (keep_trace_row, keep_trace_sw) = create_switch_row(
        &crate::core::gettext("Keep Original Image"),
        &crate::core::gettext("Keep bitmap image and place vector paths on top"),
    );
    trace_card.append(&keep_trace_row);

    trace_card.append(&gtk4::Separator::builder().orientation(gtk4::Orientation::Horizontal).build());

    // Trace Card Action Row (Primary Direct Action + Preview Button)
    let card_action_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_start(10)
        .margin_end(10)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let btn_trace_direct = gtk4::Button::builder()
        .css_classes(["suggested-action", "pill"])
        .hexpand(true)
        .build();
    let trace_btn_content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk4::Align::Center)
        .build();
    trace_btn_content.append(&crate::ui::icons::make_symbolic_image("object-to-path-symbolic", 14));
    trace_btn_content.append(&gtk4::Label::builder().label(&crate::core::gettext("Trace to Paths")).css_classes(["heading"]).build());
    btn_trace_direct.set_child(Some(&trace_btn_content));

    {
        let cv = canvas.clone();
        let m_ref = current_trace_mode.clone();
        let sc_th = thresh_scale.clone();
        let cv_ref = colors_val_ref.clone();
        let sc_de = detail_scale.clone();
        let sc_sm = smooth_scale.clone();
        let nv_ref = noise_val_ref.clone();
        let sw_in = inv_trace_sw.clone();
        let sw_ke = keep_trace_sw.clone();
        btn_trace_direct.connect_clicked(move |_| {
            let Some(img) = cv.get_selected_image_element() else {
                return;
            };
            let mode = match m_ref.get() {
                1 => crate::core::trace::TraceMode::ColorQuantization,
                2 => crate::core::trace::TraceMode::EdgeDetection,
                _ => crate::core::trace::TraceMode::BrightnessCutoff,
            };
            let config = crate::core::trace::TraceConfig {
                mode,
                threshold: (sc_th.value() / 100.0) as f32,
                num_colors: cv_ref.get() as usize,
                detail: sc_de.value() as f32,
                smoothness: (sc_sm.value() / 100.0) as f32,
                despeckle: nv_ref.get() as usize,
                corner_threshold: 1.25,
                invert: sw_in.is_active(),
                keep_original: sw_ke.is_active(),
                fill_color: Some(crate::core::Color::new(0.12, 0.12, 0.14, 1.0)),
            };
            if let Ok(elem) = crate::core::trace::trace_image_element(&img, &config) {
                cv.apply_traced_elements(img.id, elem, config.keep_original);
            }
        });
    }
    card_action_row.append(&btn_trace_direct);

    let btn_trace_dialog = gtk4::Button::builder()
        .css_classes(["pill"])
        .tooltip_text(crate::core::gettext("Open advanced interactive live preview dialog"))
        .build();
    let dialog_btn_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .halign(gtk4::Align::Center)
        .build();
    dialog_btn_box.append(&crate::ui::icons::make_symbolic_image("zoom-fit-selection-symbolic", 14));
    dialog_btn_box.append(&gtk4::Label::new(Some(&crate::core::gettext("Preview"))));
    btn_trace_dialog.set_child(Some(&dialog_btn_box));
    {
        let cv = canvas.clone();
        btn_trace_dialog.connect_clicked(move |btn| {
            crate::ui::dialogs::show_trace_bitmap_dialog(btn, cv.clone());
        });
    }
    card_action_row.append(&btn_trace_dialog);

    trace_card.append(&card_action_row);
    content_box.append(&trace_card);

    // ─────────────────────────────────────────────────────────────
    // Wire Top View Switcher Filter
    // ─────────────────────────────────────────────────────────────
    {
        let a_card = adj_card.clone();
        let t_card = trace_card.clone();
        let b_adj = tab_adj_btn.clone();
        let b_trace = tab_trace_btn.clone();
        let b_all = tab_all_btn.clone();

        // Default: Adjustments active, Trace hidden unless user switches
        t_card.set_visible(false);

        b_adj.connect_toggled({
            let ac = a_card.clone();
            let tc = t_card.clone();
            let bt = b_trace.clone();
            let ba = b_all.clone();
            move |btn| {
                if btn.is_active() {
                    bt.set_active(false);
                    ba.set_active(false);
                    ac.set_visible(true);
                    tc.set_visible(false);
                }
            }
        });

        b_trace.connect_toggled({
            let ac = a_card.clone();
            let tc = t_card.clone();
            let ba = b_adj.clone();
            let ball = b_all.clone();
            move |btn| {
                if btn.is_active() {
                    ba.set_active(false);
                    ball.set_active(false);
                    ac.set_visible(false);
                    tc.set_visible(true);
                }
            }
        });

        b_all.connect_toggled({
            let ac = a_card;
            let tc = t_card;
            let ba = b_adj;
            let bt = b_trace;
            move |btn| {
                if btn.is_active() {
                    ba.set_active(false);
                    bt.set_active(false);
                    ac.set_visible(true);
                    tc.set_visible(true);
                }
            }
        });
    }

    // ─────────────────────────────────────────────────────────────
    // Wire Real-time Adjustment Handlers
    // ─────────────────────────────────────────────────────────────
    let build_current_adj = {
        let b_s = brightness_scale.clone();
        let c_s = contrast_scale.clone();
        let s_s = saturation_scale.clone();
        let h_s = hue_scale.clone();
        let bl_s = blur_scale.clone();
        let inv_s = invert_switch.clone();
        let gray_s = grayscale_switch.clone();
        let sep_s = sepia_switch.clone();

        Rc::new(move || {
            let b_pct = b_s.value();
            let c_pct = c_s.value();
            let s_pct = s_s.value();
            let h_val = h_s.value();
            let blur_val = bl_s.value();

            let brightness = (b_pct as f32) / 100.0;
            let contrast = if c_pct >= 0.0 {
                1.0 + (c_pct as f32 / 100.0) * 1.5
            } else {
                1.0 + (c_pct as f32 / 100.0)
            };
            let saturation = if s_pct >= 0.0 {
                1.0 + (s_pct as f32 / 100.0) * 1.5
            } else {
                1.0 + (s_pct as f32 / 100.0)
            };
            let hue_rotate = h_val as f32;
            let blur = blur_val as f32;

            ImageAdjustments {
                brightness,
                contrast,
                saturation,
                hue_rotate,
                blur,
                invert: inv_s.is_active(),
                grayscale: gray_s.is_active(),
                sepia: sep_s.is_active(),
            }
        })
    };

    // Brightness
    {
        let cv = canvas.clone();
        let get_adj = build_current_adj.clone();
        let upd = is_updating.clone();
        let val_lbl = b_val_lbl;
        brightness_scale.connect_value_changed(move |s| {
            let v = s.value();
            val_lbl.set_text(&format!("{:+0.0}%", v));
            if upd.get() {
                return;
            }
            cv.set_selected_image_adjustments(get_adj(), false);
        });
    }

    // Contrast
    {
        let cv = canvas.clone();
        let get_adj = build_current_adj.clone();
        let upd = is_updating.clone();
        let val_lbl = c_val_lbl;
        contrast_scale.connect_value_changed(move |s| {
            let v = s.value();
            val_lbl.set_text(&format!("{:+0.0}%", v));
            if upd.get() {
                return;
            }
            cv.set_selected_image_adjustments(get_adj(), false);
        });
    }

    // Saturation
    {
        let cv = canvas.clone();
        let get_adj = build_current_adj.clone();
        let upd = is_updating.clone();
        let val_lbl = s_val_lbl;
        saturation_scale.connect_value_changed(move |s| {
            let v = s.value();
            val_lbl.set_text(&format!("{:+0.0}%", v));
            if upd.get() {
                return;
            }
            cv.set_selected_image_adjustments(get_adj(), false);
        });
    }

    // Hue Rotate
    {
        let cv = canvas.clone();
        let get_adj = build_current_adj.clone();
        let upd = is_updating.clone();
        let val_lbl = h_val_lbl;
        hue_scale.connect_value_changed(move |s| {
            let v = s.value();
            val_lbl.set_text(&format!("{:.0}°", v));
            if upd.get() {
                return;
            }
            cv.set_selected_image_adjustments(get_adj(), false);
        });
    }

    // Blur
    {
        let cv = canvas.clone();
        let get_adj = build_current_adj.clone();
        let upd = is_updating.clone();
        let val_lbl = blur_val_lbl;
        blur_scale.connect_value_changed(move |s| {
            let v = s.value();
            val_lbl.set_text(&format!("{:.1} px", v));
            if upd.get() {
                return;
            }
            cv.set_selected_image_adjustments(get_adj(), false);
        });
    }

    // Switches
    {
        let cv = canvas.clone();
        let get_adj = build_current_adj.clone();
        let upd = is_updating.clone();
        invert_switch.connect_active_notify(move |_| {
            if upd.get() {
                return;
            }
            cv.set_selected_image_adjustments(get_adj(), true);
        });
    }
    {
        let cv = canvas.clone();
        let get_adj = build_current_adj.clone();
        let upd = is_updating.clone();
        grayscale_switch.connect_active_notify(move |_| {
            if upd.get() {
                return;
            }
            cv.set_selected_image_adjustments(get_adj(), true);
        });
    }
    {
        let cv = canvas.clone();
        let get_adj = build_current_adj;
        let upd = is_updating.clone();
        sepia_switch.connect_active_notify(move |_| {
            if upd.get() {
                return;
            }
            cv.set_selected_image_adjustments(get_adj(), true);
        });
    }

    container.append(&content_box);

    // ─────────────────────────────────────────────────────────────
    // update_fn
    // ─────────────────────────────────────────────────────────────
    let cv_for_upd = canvas.clone();
    let name_lbl_c = name_label.clone();
    let dim_lbl_c = dim_label.clone();
    let reset_aspect_c = btn_reset_aspect.clone();
    let bright_scale_c = brightness_scale.clone();
    let cont_scale_c = contrast_scale.clone();
    let sat_scale_c = saturation_scale.clone();
    let hue_scale_c = hue_scale.clone();
    let blur_scale_c = blur_scale.clone();
    let inv_sw_c = invert_switch.clone();
    let gray_sw_c = grayscale_switch.clone();
    let sep_sw_c = sepia_switch.clone();
    let content_b_c = content_box.clone();
    let empty_b_c = empty_state_box.clone();
    let is_upd_c = is_updating.clone();

    let update_fn: Rc<dyn Fn(Option<(String, crate::core::Rect, Option<(f32, f32)>, f32)>, Option<ImageAdjustments>)> = Rc::new(move |passed_info, passed_adj| {
        let info = passed_info.or_else(|| cv_for_upd.get_selected_image_info());
        if let Some((name, rect, intrinsic, _opacity)) = info {
            empty_b_c.set_visible(false);
            content_b_c.set_visible(true);
            is_upd_c.set(true);

            name_lbl_c.set_text(&name);

            let dim_str = if let Some((iw, ih)) = intrinsic {
                format!(
                    "{:.0} × {:.0} px ({}: {:.0} × {:.0})",
                    rect.width, rect.height, crate::core::gettext("Orig"), iw, ih
                )
            } else {
                format!("{:.0} × {:.0} px ({})", rect.width, rect.height, crate::core::gettext("Empty Frame"))
            };
            dim_lbl_c.set_text(&dim_str);
            reset_aspect_c.set_sensitive(intrinsic.is_some());

            let adj_opt = passed_adj.or_else(|| cv_for_upd.get_selected_image_adjustments());
            if let Some(adj) = adj_opt {
                let b_pct = (adj.brightness * 100.0) as f64;
                bright_scale_c.set_value(b_pct.clamp(-100.0, 100.0));

                let c_pct = if adj.contrast >= 1.0 {
                    ((adj.contrast - 1.0) / 1.5 * 100.0) as f64
                } else {
                    ((adj.contrast - 1.0) * 100.0) as f64
                };
                cont_scale_c.set_value(c_pct.clamp(-100.0, 100.0));

                let s_pct = if adj.saturation >= 1.0 {
                    ((adj.saturation - 1.0) / 1.5 * 100.0) as f64
                } else {
                    ((adj.saturation - 1.0) * 100.0) as f64
                };
                sat_scale_c.set_value(s_pct.clamp(-100.0, 100.0));

                hue_scale_c.set_value(adj.hue_rotate as f64);
                blur_scale_c.set_value(adj.blur as f64);

                inv_sw_c.set_active(adj.invert);
                gray_sw_c.set_active(adj.grayscale);
                sep_sw_c.set_active(adj.sepia);
            }

            is_upd_c.set(false);
        } else {
            content_b_c.set_visible(false);
            empty_b_c.set_visible(true);
            btn_rasterize_c.set_sensitive(!cv_for_upd.selected_element_ids().is_empty());
        }
    });

    ImageSection {
        container,
        content_box,
        empty_state_box,
        name_label,
        dim_label,
        brightness_scale,
        contrast_scale,
        saturation_scale,
        hue_scale,
        blur_scale,
        invert_switch,
        grayscale_switch,
        sepia_switch,
        btn_reset_adjustments,
        btn_reset_aspect,
        is_updating,
        update_fn,
    }
}
