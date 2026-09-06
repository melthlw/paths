use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::trace::{trace_image_element, TraceConfig, TraceMode};
use crate::ui::canvas::CanvasWidget;

pub fn show_trace_bitmap_dialog(parent: &impl IsA<gtk4::Widget>, canvas: CanvasWidget) {
    let Some(image_elem) = canvas.get_selected_image_element() else {
        return;
    };

    let window = adw::Window::builder()
        .title(crate::core::gettext("Trace Bitmap"))
        .modal(true)
        .default_width(860)
        .default_height(580)
        .build();

    if let Some(root) = parent.root() {
        if let Some(win) = root.downcast_ref::<gtk4::Window>() {
            window.set_transient_for(Some(win));
        }
    }

    let toolbar_view = adw::ToolbarView::new();

    // ─────────────────────────────────────────────────────────────
    // HEADER BAR
    // ─────────────────────────────────────────────────────────────
    let header = adw::HeaderBar::builder()
        .show_title(true)
        .title_widget(&adw::WindowTitle::new(
            &crate::core::gettext("Trace Bitmap"),
            &crate::core::gettext("Vectorize raster image into editable paths"),
        ))
        .show_start_title_buttons(false)
        .show_end_title_buttons(false)
        .build();

    let btn_cancel = gtk4::Button::builder()
        .label(&crate::core::gettext("Cancel"))
        .build();
    let win_cancel = window.clone();
    btn_cancel.connect_clicked(move |_| {
        win_cancel.close();
    });
    header.pack_start(&btn_cancel);

    let btn_apply = gtk4::Button::builder()
        .label(&crate::core::gettext("Vectorize"))
        .css_classes(["suggested-action"])
        .build();
    header.pack_end(&btn_apply);
    toolbar_view.add_top_bar(&header);

    // ─────────────────────────────────────────────────────────────
    // MAIN CONTENT SPLIT (Left: Preview, Right: Controls)
    // ─────────────────────────────────────────────────────────────
    let main_paned = gtk4::Paned::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .position(460)
        .wide_handle(true)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Config state
    let config_rc = Rc::new(RefCell::new(TraceConfig::default()));
    let target_img_rc = Rc::new(image_elem);
    let latest_traced: Rc<RefCell<Option<crate::core::Element>>> = Rc::new(RefCell::new(None));

    // ─────────────────────────────────────────────────────────────
    // LEFT: PREVIEW AREA
    // ─────────────────────────────────────────────────────────────
    let preview_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(16)
        .margin_end(16)
        .margin_top(16)
        .margin_bottom(16)
        .hexpand(true)
        .vexpand(true)
        .build();

    let preview_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .css_classes(["card"])
        .hexpand(true)
        .vexpand(true)
        .overflow(gtk4::Overflow::Hidden)
        .build();

    let drawing_area = gtk4::DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    preview_card.append(&drawing_area);
    preview_box.append(&preview_card);

    let stats_label = gtk4::Label::builder()
        .css_classes(["caption", "dim-label"])
        .halign(gtk4::Align::Center)
        .label(&crate::core::gettext("Generating preview..."))
        .build();
    preview_box.append(&stats_label);

    main_paned.set_start_child(Some(&preview_box));

    // ─────────────────────────────────────────────────────────────
    // RIGHT: SETTINGS PANEL
    // ─────────────────────────────────────────────────────────────
    let settings_scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .width_request(340)
        .hexpand(false)
        .build();

    let settings_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_start(16)
        .margin_end(16)
        .margin_top(16)
        .margin_bottom(24)
        .build();

    // Group 1: Tracing Mode
    let mode_group = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Tracing Mode"))
        .build();

    let modes = [
        crate::core::gettext("Monochrome"),
        crate::core::gettext("Colors"),
        crate::core::gettext("Outlines"),
    ];
    let mode_model = gtk4::StringList::new(&modes.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let mode_row = adw::ComboRow::builder()
        .title(&crate::core::gettext("Mode"))
        .subtitle(&crate::core::gettext("Vectorization algorithm"))
        .model(&mode_model)
        .build();
    mode_group.add(&mode_row);
    settings_box.append(&mode_group);

    // Group 2: Parameters
    let param_group = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Parameters"))
        .build();

    // Threshold Slider
    let threshold_adj = gtk4::Adjustment::new(50.0, 1.0, 99.0, 1.0, 5.0, 0.0);
    let threshold_scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&threshold_adj)
        .draw_value(false)
        .width_request(140)
        .css_classes(["fine-tune"])
        .build();
    let threshold_val_lbl = gtk4::Label::builder()
        .label("50%")
        .css_classes(["numeric", "caption", "dim-label"])
        .width_request(36)
        .build();
    let thresh_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    thresh_box.append(&threshold_scale);
    thresh_box.append(&threshold_val_lbl);
    let thresh_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("Threshold"))
        .subtitle(&crate::core::gettext("Luminance cutoff level"))
        .build();
    thresh_row.add_suffix(&thresh_box);
    param_group.add(&thresh_row);

    // Number of Colors Slider (for Colors mode)
    let colors_adj = gtk4::Adjustment::new(4.0, 2.0, 16.0, 1.0, 2.0, 0.0);
    let colors_scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&colors_adj)
        .draw_value(false)
        .width_request(140)
        .css_classes(["fine-tune"])
        .build();
    let colors_val_lbl = gtk4::Label::builder()
        .label("4")
        .css_classes(["numeric", "caption", "dim-label"])
        .width_request(36)
        .build();
    let colors_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    colors_box.append(&colors_scale);
    colors_box.append(&colors_val_lbl);
    let colors_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("Color Layers"))
        .subtitle(&crate::core::gettext("Number of quantized colors"))
        .visible(false)
        .build();
    colors_row.add_suffix(&colors_box);
    param_group.add(&colors_row);

    // Detail / Tolerance Slider
    let detail_adj = gtk4::Adjustment::new(1.0, 0.2, 4.0, 0.1, 0.5, 0.0);
    let detail_scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&detail_adj)
        .draw_value(false)
        .width_request(140)
        .css_classes(["fine-tune"])
        .build();
    let detail_val_lbl = gtk4::Label::builder()
        .label("1.0")
        .css_classes(["numeric", "caption", "dim-label"])
        .width_request(36)
        .build();
    let detail_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    detail_box.append(&detail_scale);
    detail_box.append(&detail_val_lbl);
    let detail_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("Simplification"))
        .subtitle(&crate::core::gettext("Polygon node reduction (RDP)"))
        .build();
    detail_row.add_suffix(&detail_box);
    param_group.add(&detail_row);

    // Smoothness Slider
    let smooth_adj = gtk4::Adjustment::new(65.0, 0.0, 100.0, 5.0, 10.0, 0.0);
    let smooth_scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&smooth_adj)
        .draw_value(false)
        .width_request(140)
        .css_classes(["fine-tune"])
        .build();
    let smooth_val_lbl = gtk4::Label::builder()
        .label("65%")
        .css_classes(["numeric", "caption", "dim-label"])
        .width_request(36)
        .build();
    let smooth_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    smooth_box.append(&smooth_scale);
    smooth_box.append(&smooth_val_lbl);
    let smooth_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("Curve Smoothness"))
        .subtitle(&crate::core::gettext("Bézier handle curve tension"))
        .build();
    smooth_row.add_suffix(&smooth_box);
    param_group.add(&smooth_row);

    // Noise Reduction / Despeckle Slider
    let noise_adj = gtk4::Adjustment::new(8.0, 1.0, 60.0, 1.0, 5.0, 0.0);
    let noise_scale = gtk4::Scale::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .adjustment(&noise_adj)
        .draw_value(false)
        .width_request(140)
        .css_classes(["fine-tune"])
        .build();
    let noise_val_lbl = gtk4::Label::builder()
        .label("8 px")
        .css_classes(["numeric", "caption", "dim-label"])
        .width_request(36)
        .build();
    let noise_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    noise_box.append(&noise_scale);
    noise_box.append(&noise_val_lbl);
    let noise_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("Noise Filter"))
        .subtitle(&crate::core::gettext("Minimum speck area to ignore"))
        .build();
    noise_row.add_suffix(&noise_box);
    param_group.add(&noise_row);

    settings_box.append(&param_group);

    // Group 3: Options
    let opt_group = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Options"))
        .build();

    let invert_switch = gtk4::Switch::builder().valign(gtk4::Align::Center).build();
    let invert_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("Invert"))
        .subtitle(&crate::core::gettext("Invert foreground/background selection"))
        .build();
    invert_row.add_suffix(&invert_switch);
    opt_group.add(&invert_row);

    let keep_switch = gtk4::Switch::builder().valign(gtk4::Align::Center).build();
    let keep_row = adw::ActionRow::builder()
        .title(&crate::core::gettext("Keep Original"))
        .subtitle(&crate::core::gettext("Place vector paths on top of the image"))
        .build();
    keep_row.add_suffix(&keep_switch);
    opt_group.add(&keep_row);

    settings_box.append(&opt_group);

    settings_scrolled.set_child(Some(&settings_box));
    main_paned.set_end_child(Some(&settings_scrolled));
    toolbar_view.set_content(Some(&main_paned));
    window.set_content(Some(&toolbar_view));

    // ─────────────────────────────────────────────────────────────
    // LIVE PREVIEW RENDERING & RECALCULATION
    // ─────────────────────────────────────────────────────────────
    let drawing_c = drawing_area.clone();
    let stats_c = stats_label.clone();
    let latest_traced_calc = latest_traced.clone();
    let config_calc = config_rc.clone();
    let target_img_calc = target_img_rc.clone();
    let pending_source_id: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

    let trigger_update: Rc<dyn Fn()> = {
        let pending = pending_source_id.clone();
        Rc::new(move || {
            if let Ok(mut p) = pending.try_borrow_mut() {
                if let Some(id) = p.take() {
                    id.remove();
                }
            }

            let drawing_inner = drawing_c.clone();
            let stats_inner = stats_c.clone();
            let latest_inner = latest_traced_calc.clone();
            let cfg_inner = config_calc.clone();
            let img_inner = target_img_calc.clone();
            let pending_inner = pending.clone();

            let source_id = glib::timeout_add_local_once(
                std::time::Duration::from_millis(50),
                move || {
                    if let Ok(mut p) = pending_inner.try_borrow_mut() {
                        *p = None;
                    }
                    let cfg = match cfg_inner.try_borrow() {
                        Ok(b) => b.clone(),
                        Err(_) => return,
                    };
                    match trace_image_element(&img_inner, &cfg) {
                        Ok(elem) => {
                            let (paths_count, nodes_count) = match &elem {
                                crate::core::Element::Path(p) => {
                                    let subpaths = if p.subpath_lengths.is_empty() {
                                        1
                                    } else {
                                        p.subpath_lengths.len()
                                    };
                                    (subpaths, p.nodes.len())
                                }
                                crate::core::Element::Group(g) => {
                                    let total_nodes: usize = g
                                        .children
                                        .iter()
                                        .filter_map(|c| match c {
                                            crate::core::Element::Path(p) => Some(p.nodes.len()),
                                            _ => None,
                                        })
                                        .sum();
                                    (g.children.len(), total_nodes)
                                }
                                _ => (0, 0),
                            };
                            stats_inner.set_text(&format!(
                                "{}: {}  •  {}: {}",
                                crate::core::gettext("Paths"),
                                paths_count,
                                crate::core::gettext("Nodes"),
                                nodes_count
                            ));
                            if let Ok(mut l) = latest_inner.try_borrow_mut() {
                                *l = Some(elem);
                            }
                        }
                        Err(e) => {
                            stats_inner.set_text(&format!("{}: {}", crate::core::gettext("Notice"), e));
                            if let Ok(mut l) = latest_inner.try_borrow_mut() {
                                *l = None;
                            }
                        }
                    }
                    drawing_inner.queue_draw();
                },
            );
            if let Ok(mut p) = pending.try_borrow_mut() {
                *p = Some(source_id);
            }
        })
    };

    // Wire DrawingArea draw function with Cairo
    {
        let latest_draw = latest_traced.clone();
        let target_img_draw = target_img_rc.clone();
        drawing_area.set_draw_func(move |_, cr, width, height| {
            let w = width as f64;
            let h = height as f64;

            // Draw clean dark background
            cr.set_source_rgb(0.12, 0.12, 0.14);
            let _ = cr.paint();

            // Draw subtle checkerboard for transparency
            cr.set_source_rgb(0.15, 0.15, 0.18);
            let check_size = 16.0;
            let mut y = 0.0;
            while y < h {
                let mut x = if ((y / check_size) as i32) % 2 == 0 { 0.0 } else { check_size };
                while x < w {
                    cr.rectangle(x, y, check_size, check_size);
                    x += check_size * 2.0;
                }
                y += check_size;
            }
            let _ = cr.fill();

            // Compute fitted rect for preview
            let bounds = target_img_draw.rect.normalize();
            if bounds.width <= 0.0 || bounds.height <= 0.0 {
                return;
            }

            let margin = 24.0;
            let avail_w = (w - margin * 2.0).max(10.0);
            let avail_h = (h - margin * 2.0).max(10.0);

            let scale = (avail_w / bounds.width as f64).min(avail_h / bounds.height as f64);
            let draw_w = bounds.width as f64 * scale;
            let draw_h = bounds.height as f64 * scale;
            let offset_x = (w - draw_w) * 0.5;
            let offset_y = (h - draw_h) * 0.5;

            let _ = cr.save();
            cr.translate(offset_x, offset_y);
            cr.scale(scale, scale);
            cr.translate(-bounds.x as f64, -bounds.y as f64);

            // Render traced elements
            if let Ok(l) = latest_draw.try_borrow() {
                if let Some(ref elem) = *l {
                    render_element_cairo(cr, elem);
                }
            }

            let _ = cr.restore();
        });
    }

    // Connect slider and option signals
    {
        let cfg = config_rc.clone();
        let trig = trigger_update.clone();
        let colors_r = colors_row.clone();
        let thresh_r = thresh_row.clone();
        mode_row.connect_selected_notify(move |row| {
            let idx = row.selected();
            if let Ok(mut c) = cfg.try_borrow_mut() {
                match idx {
                    0 => c.mode = TraceMode::BrightnessCutoff,
                    1 => c.mode = TraceMode::ColorQuantization,
                    2 => c.mode = TraceMode::EdgeDetection,
                    _ => {}
                }
            }
            match idx {
                0 => {
                    colors_r.set_visible(false);
                    thresh_r.set_visible(true);
                }
                1 => {
                    colors_r.set_visible(true);
                    thresh_r.set_visible(false);
                }
                2 => {
                    colors_r.set_visible(false);
                    thresh_r.set_visible(true);
                }
                _ => {}
            }
            trig();
        });
    }

    {
        let cfg = config_rc.clone();
        let trig = trigger_update.clone();
        let lbl = threshold_val_lbl.clone();
        threshold_adj.connect_value_changed(move |adj| {
            let val = adj.value();
            lbl.set_text(&format!("{:.0}%", val));
            if let Ok(mut c) = cfg.try_borrow_mut() {
                c.threshold = (val / 100.0) as f32;
            }
            trig();
        });
    }

    {
        let cfg = config_rc.clone();
        let trig = trigger_update.clone();
        let lbl = colors_val_lbl.clone();
        colors_adj.connect_value_changed(move |adj| {
            let val = adj.value().round() as usize;
            lbl.set_text(&format!("{}", val));
            if let Ok(mut c) = cfg.try_borrow_mut() {
                c.num_colors = val;
            }
            trig();
        });
    }

    {
        let cfg = config_rc.clone();
        let trig = trigger_update.clone();
        let lbl = detail_val_lbl.clone();
        detail_adj.connect_value_changed(move |adj| {
            let val = adj.value();
            lbl.set_text(&format!("{:.1}", val));
            if let Ok(mut c) = cfg.try_borrow_mut() {
                c.detail = val as f32;
            }
            trig();
        });
    }

    {
        let cfg = config_rc.clone();
        let trig = trigger_update.clone();
        let lbl = smooth_val_lbl.clone();
        smooth_adj.connect_value_changed(move |adj| {
            let val = adj.value();
            lbl.set_text(&format!("{:.0}%", val));
            if let Ok(mut c) = cfg.try_borrow_mut() {
                c.smoothness = (val / 100.0) as f32;
            }
            trig();
        });
    }

    {
        let cfg = config_rc.clone();
        let trig = trigger_update.clone();
        let lbl = noise_val_lbl.clone();
        noise_adj.connect_value_changed(move |adj| {
            let val = adj.value().round() as usize;
            lbl.set_text(&format!("{} px", val));
            if let Ok(mut c) = cfg.try_borrow_mut() {
                c.despeckle = val;
            }
            trig();
        });
    }

    {
        let cfg = config_rc.clone();
        let trig = trigger_update.clone();
        invert_switch.connect_active_notify(move |sw| {
            if let Ok(mut c) = cfg.try_borrow_mut() {
                c.invert = sw.is_active();
            }
            trig();
        });
    }

    {
        let cfg = config_rc.clone();
        keep_switch.connect_active_notify(move |sw| {
            if let Ok(mut c) = cfg.try_borrow_mut() {
                c.keep_original = sw.is_active();
            }
        });
    }

    // Cancel any pending debounce timers if dialog closes
    {
        let pending = pending_source_id.clone();
        window.connect_close_request(move |_| {
            if let Ok(mut p) = pending.try_borrow_mut() {
                if let Some(id) = p.take() {
                    id.remove();
                }
            }
            glib::Propagation::Proceed
        });
    }

    // Apply Button
    {
        let canvas_app = canvas.clone();
        let target_id = target_img_rc.id;
        let latest_app = latest_traced.clone();
        let cfg_app = config_rc.clone();
        let win_app = window.clone();
        let img_app = target_img_rc.clone();
        btn_apply.connect_clicked(move |_| {
            let keep_orig = cfg_app.try_borrow().map(|c| c.keep_original).unwrap_or(false);
            let elem_opt = latest_app
                .try_borrow_mut()
                .ok()
                .and_then(|mut l| l.take())
                .or_else(|| {
                    cfg_app
                        .try_borrow()
                        .ok()
                        .and_then(|c| trace_image_element(&img_app, &c).ok())
                });
            if let Some(traced) = elem_opt {
                canvas_app.apply_traced_elements(target_id, traced, keep_orig);
                win_app.close();
            }
        });
    }

    window.present();

    // Initial update trigger after window presentation
    trigger_update();
}

/// Helper function to draw vector `Element` to Cairo context for the preview area
fn render_element_cairo(cr: &cairo::Context, elem: &crate::core::Element) {
    match elem {
        crate::core::Element::Path(p) => {
            render_path_cairo(cr, p);
        }
        crate::core::Element::Group(g) => {
            for child in &g.children {
                render_element_cairo(cr, child);
            }
        }
        _ => {}
    }
}

fn render_path_cairo(cr: &cairo::Context, p: &crate::core::PathElement) {
    if p.nodes.is_empty() {
        return;
    }

    let color = p
        .fill_color
        .as_ref()
        .or_else(|| p.fills.first().map(|f| &f.color))
        .cloned()
        .unwrap_or_else(|| crate::core::Color::new(0.9, 0.9, 0.9, 1.0));

    cr.set_fill_rule(cairo::FillRule::EvenOdd);
    cr.set_source_rgba(
        color.r as f64,
        color.g as f64,
        color.b as f64,
        color.a as f64,
    );

    let subpaths = if p.subpath_lengths.is_empty() {
        vec![p.nodes.len()]
    } else {
        p.subpath_lengths.clone()
    };

    let mut offset = 0;
    for len in subpaths {
        if len == 0 || offset >= p.nodes.len() {
            continue;
        }
        let end = (offset + len).min(p.nodes.len());
        let sub = &p.nodes[offset..end];
        offset = end;

        cr.move_to(sub[0].point.x as f64, sub[0].point.y as f64);
        for i in 0..sub.len() {
            let n1 = &sub[i];
            let n2 = &sub[(i + 1) % sub.len()];

            match (n1.handle_out, n2.handle_in) {
                (Some(h1), Some(h2)) => {
                    cr.curve_to(
                        h1.x as f64,
                        h1.y as f64,
                        h2.x as f64,
                        h2.y as f64,
                        n2.point.x as f64,
                        n2.point.y as f64,
                    );
                }
                (Some(h1), None) => {
                    cr.curve_to(
                        h1.x as f64,
                        h1.y as f64,
                        n2.point.x as f64,
                        n2.point.y as f64,
                        n2.point.x as f64,
                        n2.point.y as f64,
                    );
                }
                (None, Some(h2)) => {
                    cr.curve_to(
                        n1.point.x as f64,
                        n1.point.y as f64,
                        h2.x as f64,
                        h2.y as f64,
                        n2.point.x as f64,
                        n2.point.y as f64,
                    );
                }
                (None, None) => {
                    cr.line_to(n2.point.x as f64, n2.point.y as f64);
                }
            }
        }
        cr.close_path();
    }

    let _ = cr.fill();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_config_defaults() {
        let config = TraceConfig::default();
        assert_eq!(config.threshold, 0.5);
        assert_eq!(config.num_colors, 4);
        assert_eq!(config.despeckle, 8);
        assert_eq!(config.smoothness, 0.65);
        assert_eq!(config.detail, 1.0);
        assert!(!config.invert);
        assert!(!config.keep_original);
    }

    #[test]
    fn test_trace_image_element_empty() {
        let empty_img = crate::core::ImageElement::new(
            crate::core::Rect::new(0.0, 0.0, 100.0, 100.0),
            Vec::new(),
            Some("test.png".to_string()),
        );
        let cfg = TraceConfig::default();
        assert!(trace_image_element(&empty_img, &cfg).is_err());
    }
}
