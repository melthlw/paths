use gtk4::gio;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::ui::canvas::ops_document::ImageAdjustments;
use crate::ui::canvas::CanvasWidget;

#[allow(dead_code)]
pub struct ImageSection {
    pub container: gtk4::Box,
    pub content_box: gtk4::Box,
    pub empty_state_box: gtk4::Widget,
    pub name_label: gtk4::Label,
    pub dim_label: gtk4::Label,
    pub brightness_scale: gtk4::Scale,
    pub contrast_scale: gtk4::Scale,
    pub saturation_scale: gtk4::Scale,
    pub hue_scale: gtk4::Scale,
    pub blur_scale: gtk4::Scale,
    pub invert_switch_row: adw::SwitchRow,
    pub grayscale_switch_row: adw::SwitchRow,
    pub sepia_switch_row: adw::SwitchRow,
    pub btn_reset_adjustments: gtk4::Button,
    pub btn_reset_aspect: gtk4::Button,
    pub is_updating: Rc<Cell<bool>>,
    pub update_fn: Rc<dyn Fn(Option<(String, crate::core::Rect, Option<(f32, f32)>, f32)>, Option<ImageAdjustments>)>,
}

/// Create a sleek Libadwaita ActionRow with an expansive slider and a numeric readout badge
fn create_adw_slider_row(
    icon_name: &str,
    title: &str,
    min: f64,
    max: f64,
    step: f64,
    default_val: f64,
    suffix: &str,
) -> (adw::ActionRow, gtk4::Scale, gtk4::Label) {
    let row = adw::ActionRow::builder()
        .title(title)
        .build();

    let icon = crate::ui::icons::make_symbolic_image(icon_name, 14);
    row.add_prefix(&icon);

    let suffix_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .build();

    let scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&gtk4::Adjustment::new(default_val, min, max, step, step * 5.0, 0.0))
        .draw_value(false)
        .hexpand(true)
        .width_request(110)
        .valign(gtk4::Align::Center)
        .build();
    suffix_box.append(&scale);

    let val_lbl = gtk4::Label::builder()
        .label(&format!("{:.0}{}", default_val, suffix))
        .css_classes(["numeric", "dim-label"])
        .width_request(42)
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .build();
    suffix_box.append(&val_lbl);

    row.add_suffix(&suffix_box);

    (row, scale, val_lbl)
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
    let empty_status = adw::StatusPage::builder()
        .icon_name("tool-image-symbolic")
        .title(&crate::core::gettext("No Image Selected"))
        .description(&crate::core::gettext(
            "Select a bitmap image to adjust colors or vectorize to paths, or rasterize selected vectors into an image.",
        ))
        .hexpand(true)
        .vexpand(true)
        .build();

    let empty_actions_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .margin_top(4)
        .build();

    let btn_activate_tool = gtk4::Button::builder()
        .label(&crate::core::gettext("Create Image Frame"))
        .css_classes(["suggested-action"])
        .halign(gtk4::Align::Center)
        .build();
    {
        let cv = canvas.clone();
        btn_activate_tool.connect_clicked(move |_| {
            cv.set_active_tool("image");
        });
    }
    empty_actions_box.append(&btn_activate_tool);

    let btn_rasterize = gtk4::Button::builder()
        .label(&crate::core::gettext("Rasterize Selection to Bitmap"))
        .halign(gtk4::Align::Center)
        .build();
    let btn_rasterize_c = btn_rasterize.clone();
    {
        let cv = canvas.clone();
        btn_rasterize.connect_clicked(move |_| {
            cv.rasterize_selected_to_image();
        });
    }
    empty_actions_box.append(&btn_rasterize);

    let btn_import = gtk4::Button::builder()
        .label(&crate::core::gettext("Import Image File..."))
        .css_classes(["flat"])
        .halign(gtk4::Align::Center)
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
    empty_actions_box.append(&btn_import);
    empty_status.set_child(Some(&empty_actions_box));

    let empty_state_box: gtk4::Widget = empty_status.upcast();
    container.append(&empty_state_box);

    // ─────────────────────────────────────────────────────────────
    // Active Content Box (with standard 12px inspector margins)
    // ─────────────────────────────────────────────────────────────
    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(16)
        .visible(false)
        .build();

    // ─────────────────────────────────────────────────────────────
    // TOP SEGMENTED VIEW SWITCHER: [ Ajustes ] [ Vetorização ] [ Ambos ]
    // ─────────────────────────────────────────────────────────────
    let view_switch_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(0)
        .homogeneous(true)
        .css_classes(["linked"])
        .margin_bottom(2)
        .build();

    let tab_adj_btn = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Adjustments"))
        .active(true)
        .build();
    let tab_trace_btn = gtk4::ToggleButton::builder()
        .label(&crate::core::gettext("Vectorize"))
        .build();

    view_switch_box.append(&tab_adj_btn);
    view_switch_box.append(&tab_trace_btn);
    content_box.append(&view_switch_box);

    // ─────────────────────────────────────────────────────────────
    // 1. Image Information & Actions (Libadwaita PreferencesGroup)
    // ─────────────────────────────────────────────────────────────
    let img_group = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Image"))
        .build();

    // Row: File Name + Replace Button
    let file_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("File"))
        .subtitle("—")
        .build();
    file_row.add_prefix(&gtk4::Image::from_icon_name("tool-image-symbolic"));

    let btn_replace = gtk4::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text(&crate::core::gettext("Replace Image File..."))
        .css_classes(["flat"])
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
    file_row.add_suffix(&btn_replace);
    img_group.add(&file_row);

    // Row: Dimensions + Reset Aspect Ratio Button
    let dim_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("Dimensions"))
        .subtitle("—")
        .build();
    dim_row.add_prefix(&gtk4::Image::from_icon_name("view-grid-symbolic"));

    let btn_reset_aspect = gtk4::Button::builder()
        .icon_name("lock-aspect-ratio-symbolic")
        .tooltip_text(&crate::core::gettext("Restore original aspect ratio"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();
    {
        let cv = canvas.clone();
        btn_reset_aspect.connect_clicked(move |_| {
            cv.reset_selected_image_aspect_ratio();
        });
    }
    dim_row.add_suffix(&btn_reset_aspect);
    img_group.add(&dim_row);

    // Row: Rotation & Flip Quick Actions
    let trans_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("Transform"))
        .subtitle(&crate::core::gettext("Rotation & Flip"))
        .build();
    trans_row.add_prefix(&gtk4::Image::from_icon_name("object-rotate-right-symbolic"));

    let trans_actions = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(0)
        .css_classes(["linked"])
        .valign(gtk4::Align::Center)
        .build();

    let btn_rot_l = gtk4::Button::builder()
        .icon_name("object-rotate-left-symbolic")
        .css_classes(["flat"])
        .tooltip_text(&crate::core::gettext("Rotate -90° counter-clockwise"))
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
        .tooltip_text(&crate::core::gettext("Rotate +90° clockwise"))
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
        .tooltip_text(&crate::core::gettext("Flip horizontally"))
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
        .tooltip_text(&crate::core::gettext("Flip vertically"))
        .build();
    {
        let cv = canvas.clone();
        btn_flip_v.connect_clicked(move |_| {
            cv.flip_vertical();
        });
    }
    trans_actions.append(&btn_flip_v);

    trans_row.add_suffix(&trans_actions);
    img_group.add(&trans_row);

    content_box.append(&img_group);

    // Dummy labels for ImageSection struct compatibility if needed
    let name_label = gtk4::Label::new(None);
    let dim_label = gtk4::Label::new(None);

    // ─────────────────────────────────────────────────────────────
    // 2. Color Adjustments Group (Libadwaita PreferencesGroup)
    // ─────────────────────────────────────────────────────────────
    let adj_group = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Color Adjustments"))
        .build();

    let btn_reset_adjustments = gtk4::Button::builder()
        .icon_name("view-refresh-symbolic")
        .css_classes(["flat", "circular"])
        .tooltip_text(&crate::core::gettext("Reset all adjustments"))
        .valign(gtk4::Align::Center)
        .build();
    {
        let cv = canvas.clone();
        btn_reset_adjustments.connect_clicked(move |_| {
            cv.reset_selected_image_adjustments();
        });
    }
    adj_group.set_header_suffix(Some(&btn_reset_adjustments));

    // Preset ComboRow (Native Libadwaita)
    let preset_names = [
        crate::core::gettext("Normal"),
        crate::core::gettext("Vibrant"),
        crate::core::gettext("B&W"),
        crate::core::gettext("Vintage"),
        crate::core::gettext("Warm"),
        crate::core::gettext("Cool"),
    ];
    let preset_model = gtk4::StringList::new(&preset_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let preset_row = adw::ComboRow::builder()
        .title(&crate::core::gettext("Preset"))
        .model(&preset_model)
        .build();
    preset_row.add_prefix(&gtk4::Image::from_icon_name("applications-graphics-symbolic"));
    adj_group.add(&preset_row);

    // High Density Sliders (using expansive tracks that fill the card)
    let (b_row, brightness_scale, b_val_lbl) = create_adw_slider_row(
        "weather-clear-symbolic",
        &crate::core::gettext("Brightness"),
        -100.0,
        100.0,
        1.0,
        0.0,
        "%",
    );
    adj_group.add(&b_row);

    let (c_row, contrast_scale, c_val_lbl) = create_adw_slider_row(
        "contrast-symbolic",
        &crate::core::gettext("Contrast"),
        -100.0,
        100.0,
        1.0,
        0.0,
        "%",
    );
    adj_group.add(&c_row);

    let (s_row, saturation_scale, s_val_lbl) = create_adw_slider_row(
        "color-select-symbolic",
        &crate::core::gettext("Saturation"),
        -100.0,
        100.0,
        1.0,
        0.0,
        "%",
    );
    adj_group.add(&s_row);

    let (h_row, hue_scale, h_val_lbl) = create_adw_slider_row(
        "rotate-right-symbolic",
        &crate::core::gettext("Hue Rotate"),
        -180.0,
        180.0,
        1.0,
        0.0,
        "°",
    );
    adj_group.add(&h_row);

    let (blur_row, blur_scale, blur_val_lbl) = create_adw_slider_row(
        "edit-find-symbolic",
        &crate::core::gettext("Blur"),
        0.0,
        50.0,
        0.5,
        0.0,
        " px",
    );
    adj_group.add(&blur_row);

    // Native Libadwaita SwitchRows
    let invert_switch_row = adw::SwitchRow::builder()
        .title(&crate::core::gettext("Invert Colors"))
        .subtitle(&crate::core::gettext("Inverts the RGB color spectrum"))
        .build();
    adj_group.add(&invert_switch_row);

    let grayscale_switch_row = adw::SwitchRow::builder()
        .title(&crate::core::gettext("Black & White (Grayscale)"))
        .subtitle(&crate::core::gettext("Converts image to monochrome grayscale"))
        .build();
    adj_group.add(&grayscale_switch_row);

    let sepia_switch_row = adw::SwitchRow::builder()
        .title(&crate::core::gettext("Sepia Tone"))
        .subtitle(&crate::core::gettext("Applies classic warm photographic tint"))
        .build();
    adj_group.add(&sepia_switch_row);

    content_box.append(&adj_group);

    // Preset selection logic
    {
        let cv = canvas.clone();
        preset_row.connect_selected_notify(move |row| {
            let (br, ct, st, hr, bl, inv, gs, sp) = match row.selected() {
                1 => (0.04, 1.18, 1.45, 0.0, 0.0, false, false, false), // Vibrant
                2 => (-0.02, 1.35, 0.0, 0.0, 0.0, false, true, false),  // B&W
                3 => (-0.04, 1.12, 0.85, 0.0, 0.0, false, false, true), // Vintage
                4 => (0.03, 1.05, 1.15, 15.0, 0.0, false, false, false),// Warm
                5 => (0.02, 1.08, 1.10, -15.0, 0.0, false, false, false),// Cool
                _ => (0.0, 1.0, 1.0, 0.0, 0.0, false, false, false),    // Normal
            };
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
    }

    // ─────────────────────────────────────────────────────────────
    // 3. Bitmap Vectorization Group (Libadwaita PreferencesGroup)
    // ─────────────────────────────────────────────────────────────
    let trace_group = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Bitmap Vectorization"))
        .build();

    // Mode ComboRow (Native Libadwaita)
    let trace_mode_names = [
        crate::core::gettext("Monochrome"),
        crate::core::gettext("Colors"),
        crate::core::gettext("Outlines"),
    ];
    let trace_mode_model = gtk4::StringList::new(&trace_mode_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let mode_row = adw::ComboRow::builder()
        .title(&crate::core::gettext("Mode"))
        .subtitle(&crate::core::gettext("Vectorization algorithm"))
        .model(&trace_mode_model)
        .build();
    mode_row.add_prefix(&gtk4::Image::from_icon_name("object-to-path-symbolic"));
    trace_group.add(&mode_row);

    // Parameter: Threshold Slider (Monochrome / Outlines)
    let (thresh_row, thresh_scale, thresh_lbl) = create_adw_slider_row(
        "weather-clear-night-symbolic",
        &crate::core::gettext("Threshold"),
        1.0,
        99.0,
        1.0,
        50.0,
        "%",
    );
    trace_group.add(&thresh_row);

    // Parameter: Color Layers (SpinRow)
    let colors_adj = gtk4::Adjustment::new(4.0, 2.0, 16.0, 1.0, 2.0, 0.0);
    let colors_row = adw::SpinRow::builder()
        .title(&crate::core::gettext("Color Layers"))
        .subtitle(&crate::core::gettext("Number of quantized color paths"))
        .adjustment(&colors_adj)
        .build();
    colors_row.set_visible(false);
    trace_group.add(&colors_row);

    // Parameter: Noise Filter (SpinRow)
    let noise_adj = gtk4::Adjustment::new(8.0, 0.0, 50.0, 1.0, 5.0, 0.0);
    let noise_row = adw::SpinRow::builder()
        .title(&crate::core::gettext("Noise Filter (px)"))
        .subtitle(&crate::core::gettext("Filter out small speckled artifacts"))
        .adjustment(&noise_adj)
        .build();
    trace_group.add(&noise_row);

    // Parameter: Detail / RDP Simplification
    let (detail_row, detail_scale, detail_lbl) = create_adw_slider_row(
        "pen-simplify-symbolic",
        &crate::core::gettext("Detail (RDP)"),
        0.2,
        4.0,
        0.1,
        1.0,
        "",
    );
    trace_group.add(&detail_row);

    // Parameter: Curve Smoothness
    let (smooth_row, smooth_scale, smooth_lbl) = create_adw_slider_row(
        "node-smooth-symbolic",
        &crate::core::gettext("Smoothness"),
        0.0,
        100.0,
        5.0,
        65.0,
        "%",
    );
    trace_group.add(&smooth_row);

    // Options: SwitchRows
    let inv_trace_sw = adw::SwitchRow::builder()
        .title(&crate::core::gettext("Invert Selection"))
        .subtitle(&crate::core::gettext("Invert vector cutout logic"))
        .build();
    trace_group.add(&inv_trace_sw);

    let keep_trace_sw = adw::SwitchRow::builder()
        .title(&crate::core::gettext("Keep Original Image"))
        .subtitle(&crate::core::gettext("Keep bitmap image and place vector paths above"))
        .build();
    trace_group.add(&keep_trace_sw);

    // Mode ComboRow listener to toggle threshold vs colors row
    {
        let r_thresh = thresh_row.clone();
        let r_colors = colors_row.clone();
        mode_row.connect_selected_notify(move |row| {
            match row.selected() {
                1 => {
                    // Color Quantization
                    r_thresh.set_visible(false);
                    r_colors.set_visible(true);
                }
                _ => {
                    // Monochrome / Outlines
                    r_thresh.set_visible(true);
                    r_colors.set_visible(false);
                }
            }
        });
    }

    // Connect slider readout labels
    {
        let lbl = thresh_lbl;
        thresh_scale.adjustment().connect_value_changed(move |adj| {
            lbl.set_text(&format!("{:.0}%", adj.value()));
        });
    }
    {
        let lbl = detail_lbl;
        detail_scale.adjustment().connect_value_changed(move |adj| {
            lbl.set_text(&format!("{:.1}", adj.value()));
        });
    }
    {
        let lbl = smooth_lbl;
        smooth_scale.adjustment().connect_value_changed(move |adj| {
            lbl.set_text(&format!("{:.0}%", adj.value()));
        });
    }

    // ─────────────────────────────────────────────────────────────
    // Action Buttons: Clear, prominent, compact native GNOME actions
    // ─────────────────────────────────────────────────────────────
    let action_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .margin_bottom(6)
        .margin_start(4)
        .margin_end(4)
        .build();

    let btn_trace_direct = gtk4::Button::builder()
        .tooltip_text(&crate::core::gettext("Vectorize raster image into editable paths"))
        .css_classes(["suggested-action"])
        .halign(gtk4::Align::Fill)
        .build();
    let direct_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .build();
    direct_box.append(&gtk4::Image::from_icon_name("object-to-path-symbolic"));
    direct_box.append(&gtk4::Label::builder().label(&crate::core::gettext("Trace to Paths")).build());
    btn_trace_direct.set_child(Some(&direct_box));

    {
        let cv = canvas.clone();
        let m_row = mode_row.clone();
        let sc_th = thresh_scale.clone();
        let row_col = colors_row.clone();
        let sc_de = detail_scale.clone();
        let sc_sm = smooth_scale.clone();
        let row_noi = noise_row.clone();
        let sw_in = inv_trace_sw.clone();
        let sw_ke = keep_trace_sw.clone();
        btn_trace_direct.connect_clicked(move |_| {
            let Some(img) = cv.get_selected_image_element() else {
                return;
            };
            let mode = match m_row.selected() {
                1 => crate::core::trace::TraceMode::ColorQuantization,
                2 => crate::core::trace::TraceMode::EdgeDetection,
                _ => crate::core::trace::TraceMode::BrightnessCutoff,
            };
            let config = crate::core::trace::TraceConfig {
                mode,
                threshold: (sc_th.value() / 100.0) as f32,
                num_colors: row_col.value() as usize,
                detail: sc_de.value() as f32,
                smoothness: (sc_sm.value() / 100.0) as f32,
                despeckle: row_noi.value() as usize,
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

    let btn_trace_dialog = gtk4::Button::builder()
        .tooltip_text(&crate::core::gettext("Open advanced interactive live preview dialog"))
        .css_classes(["flat"])
        .halign(gtk4::Align::Fill)
        .build();
    let dialog_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .build();
    dialog_box.append(&gtk4::Image::from_icon_name("zoom-fit-selection-symbolic"));
    dialog_box.append(&gtk4::Label::builder().label(&crate::core::gettext("Advanced Preview...")).build());
    btn_trace_dialog.set_child(Some(&dialog_box));
    {
        let cv = canvas.clone();
        btn_trace_dialog.connect_clicked(move |btn| {
            crate::ui::dialogs::show_trace_bitmap_dialog(btn, cv.clone());
        });
    }

    action_box.append(&btn_trace_direct);
    action_box.append(&btn_trace_dialog);

    trace_group.add(&action_box);
    content_box.append(&trace_group);

    // ─────────────────────────────────────────────────────────────
    // Wire Top View Switcher Filter
    // ─────────────────────────────────────────────────────────────
    {
        let a_grp = adj_group.clone();
        let t_grp = trace_group.clone();
        let b_adj = tab_adj_btn.clone();
        let b_trace = tab_trace_btn.clone();

        // Default: Adjustments active, Trace hidden unless user switches
        t_grp.set_visible(false);

        b_adj.connect_toggled({
            let ag = a_grp.clone();
            let tg = t_grp.clone();
            let bt = b_trace.clone();
            move |btn| {
                if btn.is_active() {
                    bt.set_active(false);
                    ag.set_visible(true);
                    tg.set_visible(false);
                } else if !bt.is_active() {
                    btn.set_active(true);
                }
            }
        });

        b_trace.connect_toggled({
            let ag = a_grp;
            let tg = t_grp;
            let ba = b_adj;
            move |btn| {
                if btn.is_active() {
                    ba.set_active(false);
                    ag.set_visible(false);
                    tg.set_visible(true);
                } else if !ba.is_active() {
                    btn.set_active(true);
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
        let inv_s = invert_switch_row.clone();
        let gray_s = grayscale_switch_row.clone();
        let sep_s = sepia_switch_row.clone();

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
        brightness_scale.adjustment().connect_value_changed(move |adj| {
            let v = adj.value();
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
        contrast_scale.adjustment().connect_value_changed(move |adj| {
            let v = adj.value();
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
        saturation_scale.adjustment().connect_value_changed(move |adj| {
            let v = adj.value();
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
        hue_scale.adjustment().connect_value_changed(move |adj| {
            let v = adj.value();
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
        blur_scale.adjustment().connect_value_changed(move |adj| {
            let v = adj.value();
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
        invert_switch_row.connect_active_notify(move |_| {
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
        grayscale_switch_row.connect_active_notify(move |_| {
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
        sepia_switch_row.connect_active_notify(move |_| {
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
    let file_row_c = file_row.clone();
    let dim_row_c = dim_row.clone();
    let reset_aspect_c = btn_reset_aspect.clone();
    let bright_scale_c = brightness_scale.clone();
    let cont_scale_c = contrast_scale.clone();
    let sat_scale_c = saturation_scale.clone();
    let hue_scale_c = hue_scale.clone();
    let blur_scale_c = blur_scale.clone();
    let inv_sw_c = invert_switch_row.clone();
    let gray_sw_c = grayscale_switch_row.clone();
    let sep_sw_c = sepia_switch_row.clone();
    let content_b_c = content_box.clone();
    let empty_b_c = empty_state_box.clone();
    let is_upd_c = is_updating.clone();

    let update_fn: Rc<dyn Fn(Option<(String, crate::core::Rect, Option<(f32, f32)>, f32)>, Option<ImageAdjustments>)> = Rc::new(move |passed_info, passed_adj| {
        let info = passed_info.or_else(|| cv_for_upd.get_selected_image_info());
        if let Some((name, rect, intrinsic, _opacity)) = info {
            empty_b_c.set_visible(false);
            content_b_c.set_visible(true);
            is_upd_c.set(true);

            file_row_c.set_subtitle(&name);

            let dim_str = if let Some((iw, ih)) = intrinsic {
                format!(
                    "{:.0} × {:.0} px ({}: {:.0} × {:.0})",
                    rect.width, rect.height, crate::core::gettext("Orig"), iw, ih
                )
            } else {
                format!("{:.0} × {:.0} px ({})", rect.width, rect.height, crate::core::gettext("Empty Frame"))
            };
            dim_row_c.set_subtitle(&dim_str);
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
        invert_switch_row,
        grayscale_switch_row,
        sepia_switch_row,
        btn_reset_adjustments,
        btn_reset_aspect,
        is_updating,
        update_fn,
    }
}
