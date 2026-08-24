use gtk4::gdk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::libraries_store::{
    open_libraries_folder, IconDef, LibrariesData, PatternDef, ShapeDef, StrokePresetDef,
    SwatchItem, TypographyPresetDef,
};
use crate::core::Color;
use crate::ui::canvas::events::handle_asset_drop;
use crate::ui::canvas::CanvasWidget;

fn attach_drag<W: IsA<gtk4::Widget>>(widget: &W, payload: String) {
    let drag_source = gtk4::DragSource::builder()
        .actions(gdk::DragAction::COPY)
        .build();
    let paintable = gtk4::WidgetPaintable::new(Some(widget));
    drag_source.set_icon(Some(&paintable), 19, 19);
    let p_clone = payload;
    drag_source.connect_prepare(move |_, _, _| {
        let val = p_clone.to_value();
        Some(gdk::ContentProvider::for_value(&val))
    });
    widget.add_controller(drag_source);
}

fn attach_drag_with_icon<W: IsA<gtk4::Widget>, I: IsA<gtk4::Widget>>(
    widget: &W,
    icon_widget: &I,
    payload: String,
    hot_x: i32,
    hot_y: i32,
) {
    let drag_source = gtk4::DragSource::builder()
        .actions(gdk::DragAction::COPY)
        .build();
    let paintable = gtk4::WidgetPaintable::new(Some(icon_widget));
    drag_source.set_icon(Some(&paintable), hot_x, hot_y);
    let p_clone = payload;
    drag_source.connect_prepare(move |_, _, _| {
        let val = p_clone.to_value();
        Some(gdk::ContentProvider::for_value(&val))
    });
    widget.add_controller(drag_source);
}

/// Helper to render any vector path data directly onto a Cairo DrawingArea
fn create_vector_path_preview(path_d: &str, width: i32, height: i32) -> gtk4::DrawingArea {
    let area = gtk4::DrawingArea::builder()
        .width_request(width)
        .height_request(height)
        .build();

    let subpaths = crate::core::svg_import::parse_svg_path_data_subpaths(path_d);
    area.set_draw_func(move |_, cr, w, h| {
        let wf = w as f64;
        let hf = h as f64;
        if subpaths.is_empty() {
            return;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for sub in &subpaths {
            for n in sub {
                min_x = min_x.min(n.point.x);
                min_y = min_y.min(n.point.y);
                max_x = max_x.max(n.point.x);
                max_y = max_y.max(n.point.y);
            }
        }
        let bw = (max_x - min_x).max(0.1) as f64;
        let bh = (max_y - min_y).max(0.1) as f64;

        let padding = 4.0;
        let avail_w = wf - padding * 2.0;
        let avail_h = hf - padding * 2.0;
        let scale = (avail_w / bw).min(avail_h / bh);

        let ox = padding + (avail_w - bw * scale) / 2.0 - (min_x as f64) * scale;
        let oy = padding + (avail_h - bh * scale) / 2.0 - (min_y as f64) * scale;

        let _ = cr.save();
        cr.translate(ox, oy);
        cr.scale(scale, scale);

        cr.new_path();
        for sub in &subpaths {
            let count = sub.len();
            if count == 0 {
                continue;
            }
            cr.move_to(sub[0].point.x as f64, sub[0].point.y as f64);
            for i in 0..count {
                let n1 = &sub[i];
                let n2 = &sub[(i + 1) % count];
                if let (Some(h_out), Some(h_in)) = (n1.handle_out, n2.handle_in) {
                    cr.curve_to(
                        h_out.x as f64,
                        h_out.y as f64,
                        h_in.x as f64,
                        h_in.y as f64,
                        n2.point.x as f64,
                        n2.point.y as f64,
                    );
                } else {
                    cr.line_to(n2.point.x as f64, n2.point.y as f64);
                }
            }
            cr.close_path();
        }

        // Fill with Adwaita vibrant blue
        cr.set_source_rgba(0.21, 0.52, 0.89, 0.90);
        let _ = cr.fill_preserve();
        cr.set_source_rgba(0.11, 0.42, 0.79, 0.95);
        cr.set_line_width(1.0 / scale);
        let _ = cr.stroke();

        let _ = cr.restore();
    });

    area
}


pub fn build_libraries_section(canvas: &CanvasWidget) -> gtk4::Widget {
    let root_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .vexpand(true)
        .build();

    let data = Rc::new(RefCell::new(LibrariesData::load_or_init()));

    // ─────────────────────────────────────────────────────────────
    // 1. TOP HEADER: LIBADWAITA VIEW SWITCHER & SEARCH
    // ─────────────────────────────────────────────────────────────
    let view_stack = adw::ViewStack::builder()
        .vhomogeneous(false)
        .hhomogeneous(false)
        .build();

    let view_switcher = adw::ViewSwitcher::builder()
        .stack(&view_stack)
        .policy(adw::ViewSwitcherPolicy::Narrow)
        .halign(gtk4::Align::Fill)
        .hexpand(true)
        .build();

    let header_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(4)
        .build();

    header_box.append(&view_switcher);

    let action_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .build();

    let open_folder_btn = gtk4::Button::builder()
        .icon_name("folder-open-symbolic")
        .tooltip_text(&crate::core::gettext("Open Libraries Folder in .config"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();
    open_folder_btn.connect_clicked(|_| {
        open_libraries_folder();
    });

    let reload_btn = gtk4::Button::builder()
        .icon_name("reload-library-symbolic")
        .tooltip_text(&crate::core::gettext("Reload Libraries from disk"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    action_box.append(&open_folder_btn);
    action_box.append(&reload_btn);
    header_box.append(&action_box);

    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text(crate::core::gettext("Search assets and presets..."))
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(6)
        .build();

    root_box.append(&header_box);
    root_box.append(&search_entry);

    let filters_holder: Rc<RefCell<Vec<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(Vec::new()));

    // Build the 6 Libadwaita Pages
    let rebuild_view_stack = {
        let view_stack = view_stack.clone();
        let data = data.clone();
        let canvas = canvas.clone();
        let filters_holder = filters_holder.clone();
        Rc::new(move || {
            while let Some(child) = view_stack.first_child() {
                view_stack.remove(&child);
            }
            filters_holder.borrow_mut().clear();

            let d = data.borrow().clone();
            let (swatches_page, filter_swatches) = build_swatches_page(&canvas, &d, data.clone());
            let (patterns_page, filter_patterns) = build_patterns_page(&canvas, &d.patterns);
            let (icons_page, filter_icons) = build_icons_page(&canvas, &d.icons);
            let (shapes_page, filter_shapes) = build_shapes_page(&canvas, &d.shapes);
            let (strokes_page, filter_strokes) = build_strokes_page(&canvas, &d.strokes);
            let (typography_page, filter_typography) = build_typography_page(&canvas, &d.typography);

            filters_holder.borrow_mut().push(filter_swatches);
            filters_holder.borrow_mut().push(filter_patterns);
            filters_holder.borrow_mut().push(filter_icons);
            filters_holder.borrow_mut().push(filter_shapes);
            filters_holder.borrow_mut().push(filter_strokes);
            filters_holder.borrow_mut().push(filter_typography);

            view_stack.add_titled_with_icon(
                &swatches_page,
                Some("swatches"),
                &crate::core::gettext("Colors"),
                "color-picker-symbolic",
            );
            view_stack.add_titled_with_icon(
                &patterns_page,
                Some("patterns"),
                &crate::core::gettext("Patterns"),
                "view-grid-symbolic",
            );
            view_stack.add_titled_with_icon(
                &icons_page,
                Some("icons"),
                &crate::core::gettext("Icons"),
                "clone-master-symbolic",
            );
            view_stack.add_titled_with_icon(
                &shapes_page,
                Some("shapes"),
                &crate::core::gettext("Shapes"),
                "x-office-drawing-symbolic",
            );
            view_stack.add_titled_with_icon(
                &strokes_page,
                Some("strokes"),
                &crate::core::gettext("Strokes"),
                "tool-pen-symbolic",
            );
            view_stack.add_titled_with_icon(
                &typography_page,
                Some("typography"),
                &crate::core::gettext("Typography"),
                "font-x-generic-symbolic",
            );
        })
    };

    rebuild_view_stack();

    // Wire up Reload
    {
        let data_c = data.clone();
        let rebuild_c = rebuild_view_stack.clone();
        reload_btn.connect_clicked(move |_| {
            *data_c.borrow_mut() = LibrariesData::load_or_init();
            rebuild_c();
        });
    }

    // Connect Search Entry to filter all pages in real-time and auto-switch
    {
        let view_stack_s = view_stack.clone();
        let filters_holder_s = filters_holder.clone();
        search_entry.connect_search_changed(move |entry| {
            let q = entry.text().trim().to_lowercase();
            for f in filters_holder_s.borrow().iter() {
                f(&q);
            }
            if !q.is_empty() {
                let target_key = if q.contains("cor") || q.contains("swatch") || q.contains("color") || q.contains("blue") || q.contains("red") || q.contains("palet") {
                    Some("swatches")
                } else if q.contains("padrao") || q.contains("padrão") || q.contains("pattern") || q.contains("grid") || q.contains("dot") || q.contains("strip") || q.contains("wave") || q.contains("brick") {
                    Some("patterns")
                } else if q.contains("icon") || q.contains("icone") || q.contains("ícone") || q.contains("home") || q.contains("heart") || q.contains("star") || q.contains("user") || q.contains("mail") || q.contains("folder") {
                    Some("icons")
                } else if q.contains("forma") || q.contains("shape") || q.contains("badge") || q.contains("arrow") || q.contains("shield") || q.contains("hex") {
                    Some("shapes")
                } else if q.contains("contorno") || q.contains("stroke") || q.contains("dash") || q.contains("linha") || q.contains("tracej") {
                    Some("strokes")
                } else if q.contains("tipo") || q.contains("texto") || q.contains("font") || q.contains("heading") || q.contains("body") {
                    Some("typography")
                } else {
                    None
                };

                if let Some(key) = target_key {
                    view_stack_s.set_visible_child_name(key);
                }
            }
        });
    }

    let scrolled_content = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .child(&view_stack)
        .build();

    root_box.append(&scrolled_content);
    root_box.upcast()
}

// ─────────────────────────────────────────────────────────────────
// PAGE 1: COLORS (DOCUMENT COLORS, CUSTOM PALETTE & PRESETS)
// ─────────────────────────────────────────────────────────────────
fn create_swatch_button(
    canvas: &CanvasWidget,
    hex_val: &str,
    name: &str,
    can_delete: bool,
    on_delete: Option<Rc<dyn Fn(&str)>>,
) -> gtk4::Widget {
    let col = Color::from_hex(hex_val).unwrap_or(Color::BLACK);
    let btn = gtk4::Button::builder()
        .css_classes(["swatch-chip", "flat"])
        .tooltip_text(&format!(
            "{} ({})\n{}",
            crate::core::gettext(name),
            hex_val,
            crate::core::gettext("Click or drag to apply fill color")
        ))
        .width_request(32)
        .height_request(32)
        .build();

    let swatch_area = gtk4::DrawingArea::builder()
        .width_request(28)
        .height_request(28)
        .build();

    let swatch_c = col;
    swatch_area.set_draw_func(move |_, cr, w, h| {
        let wf = w as f64;
        let hf = h as f64;
        let cx = wf / 2.0;
        let cy = hf / 2.0;
        let radius = (wf.min(hf) / 2.0) - 1.0;

        // Clip to perfect circle
        cr.arc(cx, cy, radius, 0.0, std::f64::consts::TAU);
        let _ = cr.clip();

        // Transparency checkerboard
        let check = 4.0;
        let cols = (wf / check).ceil() as usize;
        let rows = (hf / check).ceil() as usize;
        for rx in 0..cols {
            for ry in 0..rows {
                if (rx + ry) % 2 == 0 {
                    cr.set_source_rgb(0.88, 0.88, 0.88);
                } else {
                    cr.set_source_rgb(0.98, 0.98, 0.98);
                }
                cr.rectangle(rx as f64 * check, ry as f64 * check, check, check);
                let _ = cr.fill();
            }
        }

        // Color Fill
        cr.set_source_rgba(
            swatch_c.r as f64,
            swatch_c.g as f64,
            swatch_c.b as f64,
            swatch_c.a as f64,
        );
        cr.rectangle(0.0, 0.0, wf, hf);
        let _ = cr.fill();

        // Smooth subtle border ring
        cr.reset_clip();
        cr.arc(cx, cy, radius, 0.0, std::f64::consts::TAU);
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.18);
        cr.set_line_width(1.0);
        let _ = cr.stroke();
    });

    btn.set_child(Some(&swatch_area));

    let payload = format!("gnome-paths:swatch:{}", hex_val);
    attach_drag(&btn, payload.clone());

    let canvas_c = canvas.clone();
    let payload_c = payload;
    btn.connect_clicked(move |_| {
        if let Ok(mut state) = canvas_c.state().try_borrow_mut() {
            let center = state.viewport.screen_to_world(
                crate::core::Point::new(state.widget_size.0 / 2.0, state.widget_size.1 / 2.0),
                state.widget_size,
            );
            handle_asset_drop(&mut state, &payload_c, center);
            state.notify_status();
        }
        canvas_c.drawing_area.queue_draw();
    });

    if can_delete {
        if let Some(del_cb) = on_delete {
            let hex_owned = hex_val.to_string();
            let popover = gtk4::Popover::builder()
                .has_arrow(true)
                .build();

            let menu_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(2)
                .margin_top(4)
                .margin_bottom(4)
                .margin_start(4)
                .margin_end(4)
                .build();

            let copy_btn = gtk4::Button::builder()
                .label(&format!("{}: {}", crate::core::gettext("Copy HEX"), hex_val))
                .icon_name("edit-copy-symbolic")
                .css_classes(["flat"])
                .halign(gtk4::Align::Fill)
                .build();

            let hex_for_copy = hex_val.to_string();
            let pop_c1 = popover.clone();
            copy_btn.connect_clicked(move |_| {
                if let Some(display) = gtk4::gdk::Display::default() {
                    display.clipboard().set_text(&hex_for_copy);
                }
                pop_c1.popdown();
            });

            let del_btn = gtk4::Button::builder()
                .label(&crate::core::gettext("Remove from Palette"))
                .icon_name("user-trash-symbolic")
                .css_classes(["flat", "destructive-action"])
                .halign(gtk4::Align::Fill)
                .build();

            let hex_for_del = hex_owned.clone();
            let pop_c2 = popover.clone();
            del_btn.connect_clicked(move |_| {
                del_cb(&hex_for_del);
                pop_c2.popdown();
            });

            menu_box.append(&copy_btn);
            menu_box.append(&del_btn);
            popover.set_child(Some(&menu_box));
            popover.set_parent(&btn);

            let gesture = gtk4::GestureClick::builder()
                .button(3) // Right click
                .build();

            let pop_g = popover.clone();
            gesture.connect_pressed(move |_, _, _, _| {
                pop_g.popup();
            });
            btn.add_controller(gesture);
        }
    }

    btn.upcast()
}

fn build_swatches_page(
    canvas: &CanvasWidget,
    data: &LibrariesData,
    data_ref: Rc<RefCell<LibrariesData>>,
) -> (gtk4::Widget, Box<dyn Fn(&str)>) {
    let page = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_top(4)
        .margin_bottom(12)
        .build();

    // ── Libadwaita ViewSwitcher: [ Project Colors ] | [ Preset Palettes ] ──
    let sub_stack = adw::ViewStack::builder()
        .vhomogeneous(false)
        .hhomogeneous(false)
        .build();

    let sub_switcher = adw::ViewSwitcher::builder()
        .stack(&sub_stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(6)
        .build();

    page.append(&sub_switcher);

    // ─────────────────────────────────────────────────────────────
    // TAB 1: PROJECT COLORS (DOCUMENT COLORS + USER CUSTOM PALETTE)
    // ─────────────────────────────────────────────────────────────
    let project_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .build();

    // 1. GROUP: CORES DO DOCUMENTO
    let doc_colors: Vec<Color> = if let Ok(state) = canvas.state().try_borrow() {
        state
            .document
            .get_document_colors()
            .into_iter()
            .flatten()
            .collect()
    } else {
        Vec::new()
    };

    let doc_group = adw::PreferencesGroup::builder()
        .title(glib::markup_escape_text(&crate::core::gettext("Document Colors")))
        .description(glib::markup_escape_text(&format!("{} {}", doc_colors.len(), crate::core::gettext("colors"))))
        .margin_start(12)
        .margin_end(12)
        .build();

    let doc_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .css_classes(["card"])
        .build();

    if doc_colors.is_empty() {
        let empty_lbl = gtk4::Label::builder()
            .label(&crate::core::gettext("Colors used on the canvas will appear here automatically."))
            .css_classes(["caption", "dim-label"])
            .wrap(true)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        doc_card.append(&empty_lbl);
    } else {
        let doc_grid = gtk4::FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .max_children_per_line(8)
            .min_children_per_line(6)
            .homogeneous(true)
            .row_spacing(6)
            .column_spacing(6)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        for col in doc_colors {
            let hex = col.to_hex();
            let chip = create_swatch_button(canvas, &hex, &hex, false, None);
            doc_grid.append(&chip);
        }
        doc_card.append(&doc_grid);
    }
    doc_group.add(&doc_card);
    project_box.append(&doc_group);

    // 2. GROUP: MINHA PALETA PERSONALIZADA
    let custom_group = adw::PreferencesGroup::builder()
        .title(glib::markup_escape_text(&crate::core::gettext("My Custom Palette")))
        .description(glib::markup_escape_text(&crate::core::gettext("Right-click custom swatch for options.")))
        .margin_start(12)
        .margin_end(12)
        .build();

    let add_active_btn = gtk4::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(&crate::core::gettext("Add Active Color to My Palette"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    let pick_color_btn = gtk4::Button::builder()
        .icon_name("color-picker-symbolic")
        .tooltip_text(&crate::core::gettext("Pick New Color from Dialog"))
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();

    let header_btns = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .build();
    header_btns.append(&add_active_btn);
    header_btns.append(&pick_color_btn);
    custom_group.set_header_suffix(Some(&header_btns));

    let custom_card = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .css_classes(["card"])
        .build();

    let custom_grid = gtk4::FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(8)
        .min_children_per_line(6)
        .homogeneous(true)
        .row_spacing(6)
        .column_spacing(6)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .build();

    let rebuild_custom_grid = {
        let custom_grid = custom_grid.clone();
        let canvas = canvas.clone();
        let data_ref = data_ref.clone();
        Rc::new(move || {
            while let Some(child) = custom_grid.first_child() {
                custom_grid.remove(&child);
            }
            let d = data_ref.borrow();
            for sw in &d.custom_colors {
                let data_for_del = data_ref.clone();
                let grid_c = custom_grid.clone();
                let canvas_c = canvas.clone();
                let del_cb = {
                    let data_for_del = data_for_del.clone();
                    Rc::new(move |hex_to_remove: &str| {
                        data_for_del.borrow_mut().custom_colors.retain(|c| c.hex != hex_to_remove);
                        data_for_del.borrow().save_custom_colors();
                        while let Some(child) = grid_c.first_child() {
                            grid_c.remove(&child);
                        }
                        for sw_rem in &data_for_del.borrow().custom_colors {
                            let chip = create_swatch_button(&canvas_c, &sw_rem.hex, &sw_rem.name, true, None);
                            grid_c.append(&chip);
                        }
                    })
                };
                let chip = create_swatch_button(&canvas, &sw.hex, &sw.name, true, Some(del_cb));
                custom_grid.append(&chip);
            }
        })
    };

    rebuild_custom_grid();

    // Hook up Add Active Color button
    {
        let canvas_c = canvas.clone();
        let data_c = data_ref.clone();
        let rebuild_c = rebuild_custom_grid.clone();
        add_active_btn.connect_clicked(move |_| {
            let active_hex = if let Ok(state) = canvas_c.state().try_borrow() {
                state.active_fill_color.to_hex()
            } else {
                "#3584e4".to_string()
            };

            let mut d = data_c.borrow_mut();
            if !d.custom_colors.iter().any(|c| c.hex == active_hex) {
                d.custom_colors.push(SwatchItem {
                    hex: active_hex.clone(),
                    name: active_hex,
                });
                d.save_custom_colors();
            }
            drop(d);
            rebuild_c();
        });
    }

    // Hook up Color Picker Dialog button
    {
        let canvas_c = canvas.clone();
        let data_c = data_ref.clone();
        let rebuild_c2 = rebuild_custom_grid.clone();
        pick_color_btn.connect_clicked(move |btn| {
            let root = btn.root();
            let parent_window = root.and_then(|r| r.downcast::<gtk4::Window>().ok());
            let dialog = gtk4::ColorDialog::builder()
                .title(&crate::core::gettext("Pick Custom Color"))
                .with_alpha(true)
                .build();

            let canvas_inner = canvas_c.clone();
            let data_inner = data_c.clone();
            let rebuild_inner = rebuild_c2.clone();

            dialog.choose_rgba(
                parent_window.as_ref(),
                None,
                None::<&gio::Cancellable>,
                move |result| {
                    if let Ok(rgba) = result {
                        let c = Color::new(rgba.red(), rgba.green(), rgba.blue(), rgba.alpha());
                        let hex = c.to_hex();
                        if let Ok(mut state) = canvas_inner.state().try_borrow_mut() {
                            state.active_fill_color = c;
                            state.notify_status();
                        }
                        let mut d = data_inner.borrow_mut();
                        if !d.custom_colors.iter().any(|s| s.hex == hex) {
                            d.custom_colors.push(SwatchItem {
                                hex: hex.clone(),
                                name: hex,
                            });
                            d.save_custom_colors();
                        }
                        drop(d);
                        rebuild_inner();
                    }
                },
            );
        });
    }

    custom_card.append(&custom_grid);
    custom_group.add(&custom_card);
    project_box.append(&custom_group);

    sub_stack.add_titled(
        &project_box,
        Some("project"),
        &crate::core::gettext("Project Colors"),
    );

    // ─────────────────────────────────────────────────────────────
    // TAB 2: PRESET PALETTES (LIBADWAITA PREFERENCES GROUPS)
    // ─────────────────────────────────────────────────────────────
    let presets_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .build();

    for pal in &data.palettes {
        let title_escaped = glib::markup_escape_text(&crate::core::gettext(&pal.name));
        let desc_escaped = glib::markup_escape_text(&format!("{} {}", pal.colors.len(), crate::core::gettext("colors")));
        let group = adw::PreferencesGroup::builder()
            .title(title_escaped)
            .description(desc_escaped)
            .margin_start(12)
            .margin_end(12)
            .build();

        let card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .css_classes(["card"])
            .build();

        let grid = gtk4::FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .max_children_per_line(8)
            .min_children_per_line(6)
            .homogeneous(true)
            .row_spacing(6)
            .column_spacing(6)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        for sw in &pal.colors {
            let chip = create_swatch_button(canvas, &sw.hex, &sw.name, false, None);
            grid.append(&chip);
        }
        card.append(&grid);
        group.add(&card);
        presets_box.append(&group);
    }

    sub_stack.add_titled(
        &presets_box,
        Some("presets"),
        &crate::core::gettext("Preset Palettes"),
    );

    page.append(&sub_stack);
    let filter_fn = Box::new(move |_query: &str| {});
    (page.upcast(), filter_fn)
}



// ─────────────────────────────────────────────────────────────────
// PAGE 2: VECTOR PATTERNS & TEXTURES (CAIRO DIRECT VECTOR PREVIEWS)
// ─────────────────────────────────────────────────────────────────
fn build_patterns_page(canvas: &CanvasWidget, patterns: &[PatternDef]) -> (gtk4::Widget, Box<dyn Fn(&str)>) {
    let page = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(4)
        .margin_bottom(12)
        .build();

    let flow = gtk4::FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(2)
        .min_children_per_line(2)
        .homogeneous(true)
        .row_spacing(8)
        .column_spacing(8)
        .build();

    let mut pattern_tiles: Vec<(String, gtk4::Widget)> = Vec::new();

    for pt in patterns {
        let card_btn = gtk4::Button::builder()
            .css_classes(["pattern-preset-btn", "flat"])
            .tooltip_text(&format!(
                "{} ({:.0}px)\n{}",
                crate::core::gettext(&pt.name),
                pt.scale,
                crate::core::gettext("Click or drag to apply vector pattern")
            ))
            .build();

        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        // Exact Uniform Preview Card: 84x48px
        let preview_area = gtk4::DrawingArea::builder()
            .width_request(84)
            .height_request(48)
            .build();

        let p_k = pt.key.clone();
        preview_area.set_draw_func(move |_, cr, w, h| {
            let wf = w as f64;
            let hf = h as f64;
            let r = 6.0;

            cr.new_sub_path();
            cr.arc(wf - r, r, r, -std::f64::consts::FRAC_PI_2, 0.0);
            cr.arc(wf - r, hf - r, r, 0.0, std::f64::consts::FRAC_PI_2);
            cr.arc(r, hf - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
            cr.arc(r, r, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
            cr.close_path();
            let _ = cr.clip();

            cr.set_source_rgb(0.96, 0.97, 0.98);
            cr.rectangle(0.0, 0.0, wf, hf);
            let _ = cr.fill();

            match p_k.as_str() {
                "Grid" => {
                    cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                    cr.set_line_width(1.0);
                    let step = 8.0;
                    let mut x = step;
                    while x < wf {
                        cr.move_to(x, 0.0);
                        cr.line_to(x, hf);
                        x += step;
                    }
                    let mut y = step;
                    while y < hf {
                        cr.move_to(0.0, y);
                        cr.line_to(wf, y);
                        y += step;
                    }
                    let _ = cr.stroke();
                }
                "Dots" => {
                    cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                    let step = 9.0;
                    let mut x = 5.0;
                    while x < wf {
                        let mut y = 5.0;
                        while y < hf {
                            cr.arc(x, y, 2.0, 0.0, std::f64::consts::TAU);
                            cr.close_path();
                            let _ = cr.fill();
                            y += step;
                        }
                        x += step;
                    }
                }
                "Stripes" => {
                    cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                    cr.set_line_width(3.5);
                    let step = 8.0;
                    let mut x = -hf;
                    while x < wf + hf {
                        cr.move_to(x, 0.0);
                        cr.line_to(x + hf, hf);
                        x += step;
                    }
                    let _ = cr.stroke();
                }
                "Checkerboard" => {
                    let step = 9.6;
                    let cols = (wf / step).ceil() as usize;
                    let rows = (hf / step).ceil() as usize;
                    for cx in 0..cols {
                        for ry in 0..rows {
                            if (cx + ry) % 2 == 0 {
                                cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                                cr.rectangle(cx as f64 * step, ry as f64 * step, step, step);
                                let _ = cr.fill();
                            }
                        }
                    }
                }
                "Brick" => {
                    cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                    cr.set_line_width(1.0);
                    let bw = 16.0;
                    let bh = 8.0;
                    let mut row = 0;
                    let mut y = 0.0;
                    while y < hf + bh {
                        let offset = if row % 2 == 1 { bw / 2.0 } else { 0.0 };
                        cr.move_to(0.0, y);
                        cr.line_to(wf, y);
                        let mut x = offset;
                        while x < wf + bw {
                            cr.move_to(x, y);
                            cr.line_to(x, y + bh);
                            x += bw;
                        }
                        y += bh;
                        row += 1;
                    }
                    let _ = cr.stroke();
                }
                "Crosshatch" => {
                    cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                    cr.set_line_width(1.0);
                    let step = 8.0;
                    let mut x = -hf;
                    while x < wf + hf {
                        cr.move_to(x, 0.0);
                        cr.line_to(x + hf, hf);
                        cr.move_to(x + hf, 0.0);
                        cr.line_to(x, hf);
                        x += step;
                    }
                    let _ = cr.stroke();
                }
                "Hexagon" => {
                    cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                    cr.set_line_width(1.0);
                    let r_hex = 6.5;
                    let w_step = r_hex * 3.0;
                    let h_step = r_hex * 1.732;
                    let mut col = 0;
                    let mut x = 0.0;
                    while x < wf + r_hex * 2.0 {
                        let y_offset = if col % 2 == 1 { h_step / 2.0 } else { 0.0 };
                        let mut y = -h_step;
                        while y < hf + h_step {
                            let cy = y + y_offset;
                            for i in 0..6 {
                                let angle = (i as f64) * std::f64::consts::PI / 3.0;
                                let px = x + r_hex * angle.cos();
                                let py = cy + r_hex * angle.sin();
                                if i == 0 {
                                    cr.move_to(px, py);
                                } else {
                                    cr.line_to(px, py);
                                }
                            }
                            cr.close_path();
                            let _ = cr.stroke();
                            y += h_step;
                        }
                        x += w_step * 0.5;
                        col += 1;
                    }
                }
                "Scales" => {
                    cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                    cr.set_line_width(1.0);
                    let step = 14.0;
                    let mut row = 0;
                    let mut y = 0.0;
                    while y < hf + step {
                        let offset = if row % 2 == 1 { step / 2.0 } else { 0.0 };
                        let mut x = offset;
                        while x < wf + step {
                            cr.arc(x, y, 7.0, 0.0, std::f64::consts::PI);
                            let _ = cr.stroke();
                            x += step;
                        }
                        y += step * 0.5;
                        row += 1;
                    }
                }
                "Houndstooth" => {
                    cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                    let step = 14.0;
                    let half = step * 0.5;
                    let mut x = 0.0;
                    while x < wf + step {
                        let mut y = 0.0;
                        while y < hf + step {
                            cr.move_to(x, y);
                            cr.line_to(x + half, y);
                            cr.line_to(x + step, y + half);
                            cr.line_to(x + half, y + half);
                            cr.line_to(x + half, y + step);
                            cr.line_to(x, y + half);
                            cr.close_path();
                            let _ = cr.fill();
                            y += step;
                        }
                        x += step;
                    }
                }
                "Basketweave" => {
                    cr.set_source_rgba(0.21, 0.52, 0.89, 0.85);
                    cr.set_line_width(1.5);
                    let step = 14.0;
                    let half = step * 0.5;
                    let mut x = 0.0;
                    while x < wf + step {
                        let mut y = 0.0;
                        while y < hf + step {
                            cr.move_to(x, y + half * 0.5);
                            cr.line_to(x + half, y + half * 0.5);
                            cr.move_to(x + half * 0.5, y + half);
                            cr.line_to(x + half * 0.5, y + step);
                            cr.move_to(x + half, y + step * 0.75);
                            cr.line_to(x + step, y + step * 0.75);
                            cr.move_to(x + step * 0.75, y);
                            cr.line_to(x + step * 0.75, y + half);
                            let _ = cr.stroke();
                            y += step;
                        }
                        x += step;
                    }
                }
                _ => {}
            }

            cr.reset_clip();
            cr.new_sub_path();
            cr.arc(wf - r - 0.5, r + 0.5, r, -std::f64::consts::FRAC_PI_2, 0.0);
            cr.arc(wf - r - 0.5, hf - r - 0.5, r, 0.0, std::f64::consts::FRAC_PI_2);
            cr.arc(r + 0.5, hf - r - 0.5, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
            cr.arc(r + 0.5, r + 0.5, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
            cr.close_path();
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.14);
            cr.set_line_width(1.0);
            let _ = cr.stroke();
        });

        let lbl = gtk4::Label::builder()
            .label(&crate::core::gettext(&pt.name))
            .css_classes(["caption", "heading"])
            .justify(gtk4::Justification::Center)
            .build();

        vbox.append(&preview_area);
        vbox.append(&lbl);
        card_btn.set_child(Some(&vbox));

        let payload = format!("gnome-paths:pattern:{}:{:.1}:{}", pt.key, pt.scale, pt.name);
        attach_drag_with_icon(&card_btn, &preview_area, payload.clone(), 42, 24);

        let canvas_c = canvas.clone();
        let payload_c = payload;
        card_btn.connect_clicked(move |_| {
            if let Ok(mut state) = canvas_c.state().try_borrow_mut() {
                let center = state.viewport.screen_to_world(
                    crate::core::Point::new(state.widget_size.0 / 2.0, state.widget_size.1 / 2.0),
                    state.widget_size,
                );
                handle_asset_drop(&mut state, &payload_c, center);
                canvas_c.drawing_area.queue_draw();
            }
        });

        let search_key = format!("{} {}", pt.name.to_lowercase(), pt.key.to_lowercase());
        let tile_widget: gtk4::Widget = card_btn.upcast();
        flow.append(&tile_widget);
        pattern_tiles.push((search_key, tile_widget));
    }

    page.append(&flow);

    let filter_fn = Box::new(move |query: &str| {
        let q = query.trim().to_lowercase();
        for (key, widget) in &pattern_tiles {
            let visible = q.is_empty() || key.contains(&q);
            widget.set_visible(visible);
        }
    });

    (page.upcast(), filter_fn)
}

fn resolve_gnome_icon_path_data(icon_name: &str, fallback_d: &str) -> String {
    let clean_name = icon_name.strip_suffix("-symbolic").unwrap_or(icon_name);
    let icon_dirs = [
        "/usr/share/icons/Adwaita/symbolic",
        "/usr/share/icons/hicolor/scalable/actions",
        "/usr/share/icons/hicolor/scalable/apps",
        "/usr/share/icons/hicolor/scalable/status",
    ];

    for base_path_str in &icon_dirs {
        let base_dir = std::path::Path::new(base_path_str);
        if !base_dir.is_dir() {
            continue;
        }
        let mut search_dirs = vec![base_dir.to_path_buf()];
        if let Ok(subs) = std::fs::read_dir(base_dir) {
            for sub in subs.flatten() {
                let p = sub.path();
                if p.is_dir() {
                    search_dirs.push(p);
                }
            }
        }

        for dir in search_dirs {
            let candidates = [
                dir.join(format!("{}.svg", icon_name)),
                dir.join(format!("{}-symbolic.svg", clean_name)),
                dir.join(format!("{}.svg", clean_name)),
            ];
            for cand in candidates {
                if cand.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&cand) {
                        if let Some(d) = crate::core::libraries_store::extract_path_d(&content) {
                            if !d.is_empty() {
                                return d;
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(display) = gtk4::gdk::Display::default() {
        let theme = gtk4::IconTheme::for_display(&display);
        for name_to_try in [icon_name, &format!("{}-symbolic", clean_name), clean_name] {
            let paintable = theme.lookup_icon(
                name_to_try,
                &[],
                24,
                1,
                gtk4::TextDirection::None,
                gtk4::IconLookupFlags::PRELOAD,
            );
            if let Some(file) = paintable.file() {
                if let Some(path) = file.path() {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if let Some(d) = crate::core::libraries_store::extract_path_d(&content) {
                            if !d.is_empty() {
                                return d;
                            }
                        }
                    }
                }
            }
        }
    }

    if !fallback_d.is_empty() {
        fallback_d.to_string()
    } else {
        "M 12 2 L 22 12 L 12 22 L 2 12 Z".to_string()
    }
}

// ─────────────────────────────────────────────────────────────────
// PAGE 3: VECTOR ICONS LIBRARY (OFFICIAL GNOME ADWAITA SYMBOLIC ICONS)
// ─────────────────────────────────────────────────────────────────
fn build_icons_page(canvas: &CanvasWidget, icons: &[IconDef]) -> (gtk4::Widget, Box<dyn Fn(&str)>) {
    let page = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_start(4)
        .margin_end(4)
        .margin_top(4)
        .margin_bottom(12)
        .build();

    let grid = gtk4::FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(4)
        .min_children_per_line(4)
        .homogeneous(true)
        .row_spacing(4)
        .column_spacing(4)
        .build();

    let mut icon_tiles: Vec<(String, gtk4::Widget)> = Vec::new();

    for def in icons {
        let btn = gtk4::Button::builder()
            .css_classes(["card", "icon-library-tile", "flat"])
            .tooltip_text(&format!(
                "{} - {}",
                crate::core::gettext(&def.name),
                crate::core::gettext("Click or drag to insert GNOME icon")
            ))
            .build();

        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(2)
            .margin_end(2)
            .halign(gtk4::Align::Center)
            .build();

        let icon_widget: gtk4::Widget = if !def.icon_name.is_empty() {
            let img = gtk4::Image::from_icon_name(&def.icon_name);
            img.set_pixel_size(20);
            img.upcast()
        } else {
            create_vector_path_preview(&def.path_data, 20, 20).upcast()
        };

        let lbl = gtk4::Label::builder()
            .label(&crate::core::gettext(&def.name))
            .css_classes(["caption", "dim-label"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(7)
            .halign(gtk4::Align::Center)
            .build();

        vbox.append(&icon_widget);
        vbox.append(&lbl);
        btn.set_child(Some(&vbox));

        let path_d = if !def.path_data.is_empty() {
            def.path_data.clone()
        } else {
            resolve_gnome_icon_path_data(&def.icon_name, "")
        };

        let payload = format!("gnome-paths:icon:{}:{}", def.name, path_d);
        attach_drag_with_icon(&btn, &icon_widget, payload.clone(), 10, 10);

        let canvas_c = canvas.clone();
        let payload_c = payload;
        btn.connect_clicked(move |_| {
            if let Ok(mut state) = canvas_c.state().try_borrow_mut() {
                let center = state.viewport.screen_to_world(
                    crate::core::Point::new(state.widget_size.0 / 2.0, state.widget_size.1 / 2.0),
                    state.widget_size,
                );
                handle_asset_drop(&mut state, &payload_c, center);
                state.notify_status();
            }
            canvas_c.drawing_area.queue_draw();
        });

        let search_key = format!("{} {}", def.name.to_lowercase(), def.icon_name.to_lowercase());
        let tile_widget: gtk4::Widget = btn.upcast();
        grid.append(&tile_widget);
        icon_tiles.push((search_key, tile_widget));
    }

    page.append(&grid);

    let filter_fn = Box::new(move |query: &str| {
        let q = query.trim().to_lowercase();
        for (key, widget) in &icon_tiles {
            let visible = q.is_empty() || key.contains(&q);
            widget.set_visible(visible);
        }
    });

    (page.upcast(), filter_fn)
}

// ─────────────────────────────────────────────────────────────────
// PAGE 4: VECTOR SHAPES & BADGES (CAIRO DIRECT VECTOR PREVIEWS)
// ─────────────────────────────────────────────────────────────────
fn build_shapes_page(canvas: &CanvasWidget, shapes: &[ShapeDef]) -> (gtk4::Widget, Box<dyn Fn(&str)>) {
    let page = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(4)
        .margin_bottom(12)
        .build();

    let flow = gtk4::FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(2)
        .min_children_per_line(2)
        .homogeneous(true)
        .row_spacing(8)
        .column_spacing(8)
        .build();

    let mut shape_tiles: Vec<(String, gtk4::Widget)> = Vec::new();

    for def in shapes {
        let card = gtk4::Button::builder()
            .css_classes(["card", "shape-library-tile", "flat"])
            .tooltip_text(&format!(
                "{} ({}) - {}",
                crate::core::gettext(&def.name),
                crate::core::gettext(&def.desc),
                crate::core::gettext("Click or drag to insert vector shape")
            ))
            .build();

        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(6)
            .margin_end(6)
            .halign(gtk4::Align::Center)
            .build();

        let preview_area = create_vector_path_preview(&def.path_data, 36, 36);

        let lbl = gtk4::Label::builder()
            .label(&crate::core::gettext(&def.name))
            .css_classes(["heading", "caption"])
            .build();

        let sub = gtk4::Label::builder()
            .label(&crate::core::gettext(&def.desc))
            .css_classes(["caption", "dim-label"])
            .build();

        vbox.append(&preview_area);
        vbox.append(&lbl);
        vbox.append(&sub);
        card.set_child(Some(&vbox));

        let payload = format!("gnome-paths:shape:{}:{}", def.name, def.path_data);
        attach_drag_with_icon(&card, &preview_area, payload.clone(), 18, 18);

        let canvas_c = canvas.clone();
        let payload_c = payload;
        card.connect_clicked(move |_| {
            if let Ok(mut state) = canvas_c.state().try_borrow_mut() {
                let center = state.viewport.screen_to_world(
                    crate::core::Point::new(state.widget_size.0 / 2.0, state.widget_size.1 / 2.0),
                    state.widget_size,
                );
                handle_asset_drop(&mut state, &payload_c, center);
                state.notify_status();
            }
            canvas_c.drawing_area.queue_draw();
        });

        let search_key = format!("{} {}", def.name.to_lowercase(), def.desc.to_lowercase());
        let tile_widget: gtk4::Widget = card.upcast();
        flow.append(&tile_widget);
        shape_tiles.push((search_key, tile_widget));
    }

    page.append(&flow);

    let filter_fn = Box::new(move |query: &str| {
        let q = query.trim().to_lowercase();
        for (key, widget) in &shape_tiles {
            let visible = q.is_empty() || key.contains(&q);
            widget.set_visible(visible);
        }
    });

    (page.upcast(), filter_fn)
}

// ─────────────────────────────────────────────────────────────────
// PAGE 5: STROKE STYLES & PRESETS (CAIRO DIRECT STROKE PREVIEWS)
// ─────────────────────────────────────────────────────────────────
fn build_strokes_page(canvas: &CanvasWidget, strokes: &[StrokePresetDef]) -> (gtk4::Widget, Box<dyn Fn(&str)>) {
    let page = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_start(10)
        .margin_end(10)
        .margin_top(4)
        .margin_bottom(12)
        .build();

    let mut stroke_tiles: Vec<(String, gtk4::Widget)> = Vec::new();

    for def in strokes {
        let row = adw::ActionRow::builder()
            .title(glib::markup_escape_text(&crate::core::gettext(&def.name)))
            .subtitle(glib::markup_escape_text(&crate::core::gettext(&def.desc)))
            .activatable(true)
            .css_classes(["card", "library-action-row"])
            .tooltip_text(crate::core::gettext("Click or drag to apply stroke to object"))
            .build();

        // Direct Cairo Stroke Preview Line
        let preview_line = gtk4::DrawingArea::builder()
            .width_request(40)
            .height_request(20)
            .valign(gtk4::Align::Center)
            .build();

        let s_k = def.key.clone();
        let w_v = def.width;
        preview_line.set_draw_func(move |_, cr, w, h| {
            let wf = w as f64;
            let hf = h as f64;
            cr.set_source_rgba(0.21, 0.52, 0.89, 0.95);
            cr.set_line_width(w_v as f64);
            match s_k.as_str() {
                "Dashed" => {
                    cr.set_dash(&[5.0, 3.0], 0.0);
                }
                "Dotted" => {
                    cr.set_dash(&[2.0, 3.0], 0.0);
                }
                _ => {
                    cr.set_dash(&[], 0.0);
                }
            }
            cr.move_to(2.0, hf / 2.0);
            cr.line_to(wf - 2.0, hf / 2.0);
            let _ = cr.stroke();
        });

        row.add_prefix(&preview_line);

        let apply_btn = gtk4::Button::builder()
            .icon_name("check-symbolic")
            .css_classes(["flat", "circular"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Apply Stroke Style"))
            .build();

        let payload = format!("gnome-paths:stroke:{}:{:.1}", def.key, def.width);
        attach_drag_with_icon(&row, &preview_line, payload.clone(), 20, 10);

        let canvas_c = canvas.clone();
        let payload_c = payload.clone();
        apply_btn.connect_clicked(move |_| {
            if let Ok(mut state) = canvas_c.state().try_borrow_mut() {
                let center = state.viewport.screen_to_world(
                    crate::core::Point::new(state.widget_size.0 / 2.0, state.widget_size.1 / 2.0),
                    state.widget_size,
                );
                handle_asset_drop(&mut state, &payload_c, center);
                state.notify_status();
            }
            canvas_c.drawing_area.queue_draw();
        });

        row.add_suffix(&apply_btn);

        let canvas_r = canvas.clone();
        let payload_r = payload;
        row.connect_activated(move |_| {
            if let Ok(mut state) = canvas_r.state().try_borrow_mut() {
                let center = state.viewport.screen_to_world(
                    crate::core::Point::new(state.widget_size.0 / 2.0, state.widget_size.1 / 2.0),
                    state.widget_size,
                );
                handle_asset_drop(&mut state, &payload_r, center);
                state.notify_status();
            }
            canvas_r.drawing_area.queue_draw();
        });

        let search_key = format!("{} {}", def.name.to_lowercase(), def.desc.to_lowercase());
        let row_widget: gtk4::Widget = row.upcast();
        page.append(&row_widget);
        stroke_tiles.push((search_key, row_widget));
    }

    let filter_fn = Box::new(move |query: &str| {
        let q = query.trim().to_lowercase();
        for (key, widget) in &stroke_tiles {
            let visible = q.is_empty() || key.contains(&q);
            widget.set_visible(visible);
        }
    });

    (page.upcast(), filter_fn)
}

// ─────────────────────────────────────────────────────────────────
// PAGE 6: TYPOGRAPHY PRESETS (ACTUAL FONT PREVIEW BADGES)
// ─────────────────────────────────────────────────────────────────
fn build_typography_page(canvas: &CanvasWidget, typography: &[TypographyPresetDef]) -> (gtk4::Widget, Box<dyn Fn(&str)>) {
    let page = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_start(10)
        .margin_end(10)
        .margin_top(4)
        .margin_bottom(12)
        .build();

    let mut type_tiles: Vec<(String, gtk4::Widget)> = Vec::new();

    for def in typography {
        let row = adw::ActionRow::builder()
            .title(glib::markup_escape_text(&crate::core::gettext(&def.name)))
            .subtitle(glib::markup_escape_text(&format!("{} • {:.0}pt", def.family, def.size)))
            .activatable(true)
            .css_classes(["card", "library-action-row"])
            .tooltip_text(crate::core::gettext("Click to insert text or drag to position on canvas"))
            .build();

        // Direct typographical sample badge
        let badge_lbl = gtk4::Label::builder()
            .label(&def.badge)
            .css_classes(["heading", "accent"])
            .valign(gtk4::Align::Center)
            .width_request(32)
            .build();
        row.add_prefix(&badge_lbl);

        let size_lbl = gtk4::Label::builder()
            .label(&format!("{:.0} pt", def.size))
            .css_classes(["dim-label", "numeric"])
            .build();
        row.add_suffix(&size_lbl);

        let payload = format!("gnome-paths:text:{}:{:.1}:{}:{}", def.family, def.size, def.weight, def.name);
        attach_drag_with_icon(&row, &badge_lbl, payload.clone(), 16, 12);

        let canvas_c = canvas.clone();
        let payload_c = payload;
        row.connect_activated(move |_| {
            if let Ok(mut state) = canvas_c.state().try_borrow_mut() {
                let center = state.viewport.screen_to_world(
                    crate::core::Point::new(state.widget_size.0 / 2.0, state.widget_size.1 / 2.0),
                    state.widget_size,
                );
                handle_asset_drop(&mut state, &payload_c, center);
                state.notify_status();
            }
            canvas_c.drawing_area.queue_draw();
        });

        let search_key = format!("{} {}", def.name.to_lowercase(), def.family.to_lowercase());
        let row_widget: gtk4::Widget = row.upcast();
        page.append(&row_widget);
        type_tiles.push((search_key, row_widget));
    }

    let filter_fn = Box::new(move |query: &str| {
        let q = query.trim().to_lowercase();
        for (key, widget) in &type_tiles {
            let visible = q.is_empty() || key.contains(&q);
            widget.set_visible(visible);
        }
    });

    (page.upcast(), filter_fn)
}
