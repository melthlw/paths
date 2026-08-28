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
    #[allow(dead_code)]
    pub btn_reset_adjustments: gtk4::Button,
    #[allow(dead_code)]
    pub btn_reset_aspect: gtk4::Button,
    pub is_updating: Rc<Cell<bool>>,
    pub update_fn: Rc<dyn Fn(Option<(String, crate::core::Rect, Option<(f32, f32)>, f32)>, Option<ImageAdjustments>)>,
}

fn create_slider_row(
    icon_name: &str,
    label_text: &str,
    min: f64,
    max: f64,
    step: f64,
    default_val: f64,
) -> (gtk4::Box, gtk4::Scale, gtk4::Label) {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .margin_start(8)
        .margin_end(8)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    let header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();

    let icon = crate::ui::icons::make_symbolic_image(icon_name, 14);
    header.append(&icon);

    let lbl = gtk4::Label::builder()
        .label(label_text)
        .css_classes(["caption", "dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    header.append(&lbl);

    let spacer = gtk4::Box::builder().hexpand(true).build();
    header.append(&spacer);

    let val_lbl = gtk4::Label::builder()
        .label(&format!("{:.0}", default_val))
        .css_classes(["caption", "numeric", "dim-label"])
        .halign(gtk4::Align::End)
        .build();
    header.append(&val_lbl);
    row.append(&header);

    let scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&gtk4::Adjustment::new(default_val, min, max, step, step * 5.0, 0.0))
        .draw_value(false)
        .hexpand(true)
        .css_classes(["fine-tune"])
        .build();
    row.append(&scale);

    (row, scale, val_lbl)
}

fn create_switch_row(
    label_text: &str,
    tooltip: &str,
) -> (gtk4::Box, gtk4::Switch) {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(4)
        .margin_bottom(4)
        .tooltip_text(tooltip)
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
        .margin_top(48)
        .margin_bottom(48)
        .margin_start(24)
        .margin_end(24)
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
            "Select an image on the canvas or create a new Image Frame (Shift+I) to adjust non-destructive color properties.",
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
    container.append(&empty_state_box);

    // ─────────────────────────────────────────────────────────────
    // Active Content Box (Adwaita Cards)
    // ─────────────────────────────────────────────────────────────
    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_top(8)
        .margin_bottom(16)
        .visible(false)
        .build();

    // ─────────────────────────────────────────────────────────────
    // 1. Info & Quick Replace Card
    // ─────────────────────────────────────────────────────────────
    let info_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .css_classes(["card"])
        .build();

    let info_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();

    let img_icon = crate::ui::icons::make_symbolic_image("tool-image-symbolic", 16);
    info_header.append(&img_icon);

    let name_label = gtk4::Label::builder()
        .label(&crate::core::gettext("Image"))
        .css_classes(["heading", "caption"])
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .max_width_chars(16)
        .build();
    info_header.append(&name_label);

    let info_h_spacer = gtk4::Box::builder().hexpand(true).build();
    info_header.append(&info_h_spacer);

    // Replace Image button in header
    let btn_replace = gtk4::Button::builder()
        .icon_name("document-open-symbolic")
        .css_classes(["flat", "circular"])
        .tooltip_text(crate::core::gettext("Replace image file..."))
        .valign(gtk4::Align::Center)
        .build();
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
    info_header.append(&btn_replace);
    info_card.append(&info_header);

    let info_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    info_card.append(&info_sep);

    // Dimension & DPI readout label
    let dim_label = gtk4::Label::builder()
        .label("—")
        .css_classes(["dim-label", "caption"])
        .halign(gtk4::Align::Start)
        .margin_start(10)
        .margin_end(10)
        .margin_top(6)
        .margin_bottom(2)
        .build();
    info_card.append(&dim_label);

    // Restore aspect ratio action button
    let btn_reset_aspect = gtk4::Button::builder()
        .css_classes(["flat", "card-action-btn"])
        .hexpand(true)
        .halign(gtk4::Align::Fill)
        .margin_bottom(4)
        .build();
    let aspect_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk4::Align::Center)
        .build();
    aspect_box.append(&crate::ui::icons::make_symbolic_image("lock-aspect-ratio-symbolic", 14));
    aspect_box.append(&gtk4::Label::new(Some(&crate::core::gettext("Restore Aspect Ratio"))));
    btn_reset_aspect.set_child(Some(&aspect_box));
    {
        let cv = canvas.clone();
        btn_reset_aspect.connect_clicked(move |_| {
            cv.reset_selected_image_aspect_ratio();
        });
    }
    info_card.append(&btn_reset_aspect);
    content_box.append(&info_card);

    // ─────────────────────────────────────────────────────────────
    // 2. Non-Destructive Adjustments Card
    // ─────────────────────────────────────────────────────────────
    let adj_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(4)
        .margin_start(12)
        .margin_end(12)
        .css_classes(["card"])
        .build();

    let adj_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();

    let adj_img = crate::ui::icons::make_symbolic_image("applications-graphics-symbolic", 14);
    adj_header.append(&adj_img);
    adj_header.append(
        &gtk4::Label::builder()
            .label(crate::core::gettext("Adjustments"))
            .css_classes(["heading", "caption"])
            .build(),
    );

    let adj_h_spacer = gtk4::Box::builder().hexpand(true).build();
    adj_header.append(&adj_h_spacer);

    // Reset adjustments button
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

    let adj_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    adj_card.append(&adj_sep);

    // ── Sliders ──
    let (b_row, brightness_scale, b_val_lbl) = create_slider_row(
        "weather-clear-symbolic",
        &crate::core::gettext("Brightness"),
        -100.0,
        100.0,
        1.0,
        0.0,
    );
    adj_card.append(&b_row);

    let (c_row, contrast_scale, c_val_lbl) = create_slider_row(
        "contrast-symbolic",
        &crate::core::gettext("Contrast"),
        -100.0,
        100.0,
        1.0,
        0.0,
    );
    adj_card.append(&c_row);

    let (s_row, saturation_scale, s_val_lbl) = create_slider_row(
        "color-select-symbolic",
        &crate::core::gettext("Saturation"),
        -100.0,
        100.0,
        1.0,
        0.0,
    );
    adj_card.append(&s_row);

    let (h_row, hue_scale, h_val_lbl) = create_slider_row(
        "rotate-right-symbolic",
        &crate::core::gettext("Hue Rotate"),
        -180.0,
        180.0,
        1.0,
        0.0,
    );
    adj_card.append(&h_row);

    let (blur_row, blur_scale, blur_val_lbl) = create_slider_row(
        "edit-find-symbolic",
        &crate::core::gettext("Blur"),
        0.0,
        50.0,
        0.5,
        0.0,
    );
    adj_card.append(&blur_row);

    let sw_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .margin_top(4)
        .margin_bottom(4)
        .build();
    adj_card.append(&sw_sep);

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
        &crate::core::gettext("Applies a classic warm sepia photographic tint"),
    );
    adj_card.append(&sepia_row);
    content_box.append(&adj_card);

    // ─────────────────────────────────────────────────────────────
    // 3. Quick Presets Card
    // ─────────────────────────────────────────────────────────────
    let preset_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_start(12)
        .margin_end(12)
        .css_classes(["card"])
        .build();

    let preset_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes(["card-header-bar"])
        .valign(gtk4::Align::Center)
        .build();
    preset_header.append(&crate::ui::icons::make_symbolic_image("starred-symbolic", 14));
    preset_header.append(
        &gtk4::Label::builder()
            .label(crate::core::gettext("Presets"))
            .css_classes(["heading", "caption"])
            .build(),
    );
    preset_card.append(&preset_header);

    let preset_sep = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    preset_card.append(&preset_sep);

    let preset_grid = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(4)
        .margin_start(8)
        .margin_end(8)
        .margin_top(6)
        .margin_bottom(8)
        .build();

    let row1 = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .homogeneous(true)
        .build();

    let btn_normal = gtk4::Button::builder()
        .label(&crate::core::gettext("Normal"))
        .css_classes(["flat"])
        .build();
    {
        let cv = canvas.clone();
        btn_normal.connect_clicked(move |_| {
            cv.reset_selected_image_adjustments();
        });
    }
    row1.append(&btn_normal);

    let btn_vibrant = gtk4::Button::builder()
        .label(&crate::core::gettext("Vibrant"))
        .css_classes(["flat"])
        .build();
    {
        let cv = canvas.clone();
        btn_vibrant.connect_clicked(move |_| {
            cv.set_selected_image_adjustments(
                ImageAdjustments {
                    brightness: 0.04,
                    contrast: 1.18,
                    saturation: 1.45,
                    hue_rotate: 0.0,
                    blur: 0.0,
                    invert: false,
                    grayscale: false,
                    sepia: false,
                },
                true,
            );
        });
    }
    row1.append(&btn_vibrant);

    let btn_bw = gtk4::Button::builder()
        .label(&crate::core::gettext("B&W High"))
        .css_classes(["flat"])
        .build();
    {
        let cv = canvas.clone();
        btn_bw.connect_clicked(move |_| {
            cv.set_selected_image_adjustments(
                ImageAdjustments {
                    brightness: -0.02,
                    contrast: 1.35,
                    saturation: 0.0,
                    hue_rotate: 0.0,
                    blur: 0.0,
                    invert: false,
                    grayscale: true,
                    sepia: false,
                },
                true,
            );
        });
    }
    row1.append(&btn_bw);
    preset_grid.append(&row1);

    let row2 = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .homogeneous(true)
        .build();

    let btn_vintage = gtk4::Button::builder()
        .label(&crate::core::gettext("Vintage"))
        .css_classes(["flat"])
        .build();
    {
        let cv = canvas.clone();
        btn_vintage.connect_clicked(move |_| {
            cv.set_selected_image_adjustments(
                ImageAdjustments {
                    brightness: -0.04,
                    contrast: 1.12,
                    saturation: 0.85,
                    hue_rotate: 0.0,
                    blur: 0.0,
                    invert: false,
                    grayscale: false,
                    sepia: true,
                },
                true,
            );
        });
    }
    row2.append(&btn_vintage);

    let btn_warm = gtk4::Button::builder()
        .label(&crate::core::gettext("Warm"))
        .css_classes(["flat"])
        .build();
    {
        let cv = canvas.clone();
        btn_warm.connect_clicked(move |_| {
            cv.set_selected_image_adjustments(
                ImageAdjustments {
                    brightness: 0.03,
                    contrast: 1.05,
                    saturation: 1.15,
                    hue_rotate: 15.0,
                    blur: 0.0,
                    invert: false,
                    grayscale: false,
                    sepia: false,
                },
                true,
            );
        });
    }
    row2.append(&btn_warm);

    let btn_cool = gtk4::Button::builder()
        .label(&crate::core::gettext("Cool"))
        .css_classes(["flat"])
        .build();
    {
        let cv = canvas.clone();
        btn_cool.connect_clicked(move |_| {
            cv.set_selected_image_adjustments(
                ImageAdjustments {
                    brightness: 0.02,
                    contrast: 1.08,
                    saturation: 1.10,
                    hue_rotate: -15.0,
                    blur: 0.0,
                    invert: false,
                    grayscale: false,
                    sepia: false,
                },
                true,
            );
        });
    }
    row2.append(&btn_cool);
    preset_grid.append(&row2);

    preset_card.append(&preset_grid);
    content_box.append(&preset_card);

    container.append(&content_box);

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

    // Brightness change
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

    // Contrast change
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

    // Saturation change
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

    // Hue change
    {
        let cv = canvas.clone();
        let get_adj = build_current_adj.clone();
        let upd = is_updating.clone();
        let val_lbl = h_val_lbl;
        hue_scale.connect_value_changed(move |s| {
            let v = s.value();
            val_lbl.set_text(&format!("{:0.0}°", v));
            if upd.get() {
                return;
            }
            cv.set_selected_image_adjustments(get_adj(), false);
        });
    }

    // Blur change
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

    // Invert switch change
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

    // Grayscale switch change
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

    // Sepia switch change
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
                    "{:.0} × {:.0} px (Native: {:.0} × {:.0} px)",
                    rect.width, rect.height, iw, ih
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
