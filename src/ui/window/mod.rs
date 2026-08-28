pub mod file_ops;
pub mod header;
pub mod menu;

use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use super::canvas::CanvasWidget;
use super::color_bar::ColorControlBar;
use super::inspector::InspectorSidebar;
use super::layers::LayersSidebar;
use super::tool_options::ToolOptionsBar;
use super::toolbar::FloatingToolbar;
use file_ops::WindowFileOps;

pub struct DesignWindow {
    window: adw::ApplicationWindow,
}

impl DesignWindow {
    pub fn new(app: &adw::Application) -> Self {
        Self::new_internal(app, None, None)
    }

    pub fn new_with_preset(app: &adw::Application, width: f32, height: f32) -> Self {
        Self::new_internal(app, Some((width, height)), None)
    }

    pub fn new_with_file(app: &adw::Application, path: &std::path::Path) -> Self {
        Self::new_internal(app, None, Some(path))
    }

    fn new_internal(
        app: &adw::Application,
        preset_size: Option<(f32, f32)>,
        open_path: Option<&std::path::Path>,
    ) -> Self {
        let canvas = CanvasWidget::new();

        if let Some((w, h)) = preset_size {
            canvas.new_document();
            canvas.set_active_page_size(w, h);
        } else if let Some(path) = open_path {
            if let Err(e) = canvas.open_from_path(path) {
                eprintln!("Error opening file: {}", e);
            } else {
                crate::core::AppSettings::add_recent_file(&path.to_string_lossy());
            }
        }

        // Restore Display & Canvas Preferences from AppSettings
        let show_grid = crate::core::AppSettings::show_grid();
        canvas.set_grid_visible(show_grid);

        let grid_style_str = crate::core::AppSettings::grid_style();
        let grid_style = match grid_style_str.as_str() {
            "lines" => crate::core::GridStyle::Lines,
            "none" => crate::core::GridStyle::None,
            _ => crate::core::GridStyle::Dots,
        };
        canvas.set_grid_style(grid_style);
        canvas.set_grid_cell_size(crate::core::AppSettings::grid_cell_size() as f32);
        canvas.set_grid_subdivisions(crate::core::AppSettings::grid_subdivisions());

        canvas.set_show_workspace_dots(crate::core::AppSettings::workspace_dots());
        canvas.set_page_shadow(crate::core::AppSettings::page_shadow());
        canvas.set_page_border(crate::core::AppSettings::page_border());

        let canvas_bg = crate::core::AppSettings::canvas_bg_color();
        let canvas_bg_col = if canvas_bg == "system" {
            None
        } else {
            crate::core::Color::from_hex(&canvas_bg)
        };
        canvas.set_canvas_bg_color(canvas_bg_col);

        let page_bg = crate::core::AppSettings::page_bg_color();
        let page_bg_col = if page_bg == "transparent" {
            Some(crate::core::Color::new(0.0, 0.0, 0.0, 0.0))
        } else {
            crate::core::Color::from_hex(&page_bg)
        };
        canvas.set_page_bg_color(page_bg_col);

        canvas.set_hardware_accelerated(crate::core::AppSettings::hardware_acceleration());
        canvas.set_high_precision_aa(crate::core::AppSettings::high_precision_aa());

        canvas.set_rulers_visible(crate::core::AppSettings::show_rulers());
        canvas.set_guides_visible(crate::core::AppSettings::show_guides());

        canvas.set_snap_enabled(crate::core::AppSettings::snap_enabled());
        canvas.set_snap_to_grid(crate::core::AppSettings::snap_to_grid());
        canvas.set_snap_to_objects(crate::core::AppSettings::snap_to_objects());
        canvas.set_snap_to_artboard(crate::core::AppSettings::snap_to_artboard());
        canvas.set_snap_to_guides(crate::core::AppSettings::snap_to_guides());

        let unit_str = crate::core::AppSettings::unit();
        let unit = crate::core::Unit::from_suffix(&unit_str).unwrap_or(crate::core::Unit::Px);
        canvas.set_unit(unit);

        let mut pe_cfg = canvas.path_editor_config();
        pe_cfg.node_size = crate::core::AppSettings::node_size() as f32;
        pe_cfg.handle_size = crate::core::AppSettings::handle_size() as f32;
        pe_cfg.handle_display_mode = match crate::core::AppSettings::handle_display_mode().as_str() {
            "path" => crate::core::HandleDisplayMode::AllInSelectedPath,
            "always" => crate::core::HandleDisplayMode::Always,
            _ => crate::core::HandleDisplayMode::SelectedOnly,
        };
        canvas.set_path_editor_config(pe_cfg);

        let sc_preset = match crate::core::AppSettings::shortcut_preset().as_str() {
            "figma" => crate::core::ShortcutPreset::Figma,
            "illustrator" => crate::core::ShortcutPreset::Illustrator,
            "inkscape" => crate::core::ShortcutPreset::Inkscape,
            _ => crate::core::ShortcutPreset::Default,
        };
        canvas.set_shortcut_preset(sc_preset);

        let saved_zoom = crate::core::AppSettings::active_zoom() as f32;
        canvas.set_zoom(saved_zoom);

        let main_win_holder: Rc<RefCell<Option<adw::ApplicationWindow>>> =
            Rc::new(RefCell::new(None));
        let toast_overlay = adw::ToastOverlay::new();
        let file_ops = WindowFileOps::new(canvas.clone(), main_win_holder.clone(), toast_overlay.clone());

        Self::build_content(&canvas, &main_win_holder, &toast_overlay, &file_ops);

        let (win_w, win_h) = crate::core::AppSettings::window_size();
        let is_maximized = crate::core::AppSettings::is_maximized();
        let is_fullscreen = crate::core::AppSettings::is_fullscreen();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("gnome-paths")
            .default_width(win_w)
            .default_height(win_h)
            .content(&toast_overlay)
            .build();

        *main_win_holder.borrow_mut() = Some(window.clone());

        // Instant Live Language Switcher
        let canvas_rebuild = canvas.clone();
        let win_holder_rebuild = main_win_holder.clone();
        let toast_rebuild = toast_overlay.clone();
        let file_ops_rebuild = file_ops.clone();
        crate::core::on_language_change_local(move |_| {
            let canvas_c = canvas_rebuild.clone();
            let win_holder_c = win_holder_rebuild.clone();
            let toast_c = toast_rebuild.clone();
            let file_ops_c = file_ops_rebuild.clone();
            glib::idle_add_local_once(move || {
                Self::build_content(&canvas_c, &win_holder_c, &toast_c, &file_ops_c);
            });
        });

        if is_maximized {
            window.maximize();
        }
        if is_fullscreen {
            window.fullscreen();
        }

        let win_close = window.clone();
        let canvas_close = canvas.clone();
        let is_closing = Rc::new(std::cell::Cell::new(false));
        let is_closing_c = is_closing.clone();
        window.connect_close_request(move |win| {
            if is_closing_c.get() {
                let max = win_close.is_maximized();
                let full = win_close.is_fullscreen();
                crate::core::AppSettings::set_is_maximized(max);
                crate::core::AppSettings::set_is_fullscreen(full);
                if !max && !full {
                    crate::core::AppSettings::set_window_size(
                        win_close.default_width(),
                        win_close.default_height(),
                    );
                }
                crate::core::AppSettings::set_active_zoom(canvas_close.zoom() as f64);
                return glib::Propagation::Proceed;
            }

            if canvas_close.has_unsaved_changes() {
                let win_ref = win.clone();
                let is_cl = is_closing_c.clone();
                crate::ui::window::file_ops::prompt_save_changes(
                    Some(win),
                    &canvas_close,
                    move || {
                        is_cl.set(true);
                        win_ref.close();
                    },
                );
                glib::Propagation::Stop
            } else {
                let max = win_close.is_maximized();
                let full = win_close.is_fullscreen();
                crate::core::AppSettings::set_is_maximized(max);
                crate::core::AppSettings::set_is_fullscreen(full);
                if !max && !full {
                    crate::core::AppSettings::set_window_size(
                        win_close.default_width(),
                        win_close.default_height(),
                    );
                }
                crate::core::AppSettings::set_active_zoom(canvas_close.zoom() as f64);
                glib::Propagation::Proceed
            }
        });

        window.connect_notify(Some("maximized"), move |w, _| {
            crate::core::AppSettings::set_is_maximized(w.is_maximized());
        });

        window.connect_notify(Some("fullscreened"), move |w, _| {
            crate::core::AppSettings::set_is_fullscreen(w.is_fullscreen());
        });

        window.connect_notify(Some("default-width"), move |w, _| {
            if !w.is_maximized() && !w.is_fullscreen() {
                crate::core::AppSettings::set_window_size(w.default_width(), w.default_height());
            }
        });

        window.connect_notify(Some("default-height"), move |w, _| {
            if !w.is_maximized() && !w.is_fullscreen() {
                crate::core::AppSettings::set_window_size(w.default_width(), w.default_height());
            }
        });

        *main_win_holder.borrow_mut() = Some(window.clone());

        // Connect Canvas file dialog requests (e.g. from canvas shortcuts)
        {
            let save_c = file_ops.perform_save.clone();
            let save_as_c = file_ops.perform_save_as.clone();
            let open_c = file_ops.perform_open.clone();
            let new_c = file_ops.perform_new_doc.clone();
            let exp_c = file_ops.perform_quick_export.clone();
            canvas.set_on_file_dialog_request(move |action| match action {
                crate::core::ShortcutAction::Save => save_c(),
                crate::core::ShortcutAction::SaveAs => save_as_c(),
                crate::core::ShortcutAction::Open => open_c(),
                crate::core::ShortcutAction::NewDocument => new_c(),
                crate::core::ShortcutAction::Export => exp_c(crate::core::ExportFormat::Png),
                _ => {}
            });
        }

        // Global shortcuts: Ctrl+S, Ctrl+Shift+S, Ctrl+O, Ctrl+N, Ctrl+,, Ctrl+?
        {
            let key_ctrl = gtk4::EventControllerKey::new();
            let win_weak = window.downgrade();
            let canvas_k = canvas.clone();
            let save_k = file_ops.perform_save.clone();
            let save_as_k = file_ops.perform_save_as.clone();
            let open_k = file_ops.perform_open.clone();
            let new_k = file_ops.perform_new_doc.clone();
            key_ctrl.connect_key_pressed(move |_, keyval, _, state| {
                let ctrl = state.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
                let shift = state.contains(gtk4::gdk::ModifierType::SHIFT_MASK);

                if ctrl && (keyval == gtk4::gdk::Key::s || keyval == gtk4::gdk::Key::S) {
                    if shift {
                        save_as_k();
                    } else {
                        save_k();
                    }
                    return glib::Propagation::Stop;
                }

                if ctrl && !shift && (keyval == gtk4::gdk::Key::o || keyval == gtk4::gdk::Key::O) {
                    open_k();
                    return glib::Propagation::Stop;
                }

                if ctrl && !shift && (keyval == gtk4::gdk::Key::n || keyval == gtk4::gdk::Key::N) {
                    new_k();
                    return glib::Propagation::Stop;
                }

                if ctrl && keyval == gtk4::gdk::Key::comma {
                    if let Some(win) = win_weak.upgrade() {
                        super::preferences::show_preferences_window(&win, canvas_k.clone());
                        return glib::Propagation::Stop;
                    }
                }

                if ctrl && (keyval == gtk4::gdk::Key::question || keyval == gtk4::gdk::Key::slash) {
                    if let Some(win) = win_weak.upgrade() {
                        super::preferences::show_preferences_window(&win, canvas_k.clone());
                        return glib::Propagation::Stop;
                    }
                }

                glib::Propagation::Proceed
            });
            window.add_controller(key_ctrl);
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.set_visible(true);
        self.window.present();
    }

    fn build_content(
        canvas: &CanvasWidget,
        main_win_holder: &Rc<RefCell<Option<adw::ApplicationWindow>>>,
        toast_overlay: &adw::ToastOverlay,
        file_ops: &WindowFileOps,
    ) {
        let saved_zoom = canvas.zoom();
        let color_bar = ColorControlBar::new(canvas.clone());
        let color_bar_ref = Rc::new(color_bar);

        let palette_bar = crate::ui::palette_bar::ColorPaletteBar::new(canvas.clone());
        let palette_bar_ref = Rc::new(palette_bar);
        palette_bar_ref
            .widget()
            .set_visible(canvas.is_plugin_enabled("color_palette_toolbar"));

        let tool_options = ToolOptionsBar::new(canvas.clone());

        // Floating Dock Container (holds separate Color Bar and Toolbar capsules)
        let bottom_dock = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::End)
            .margin_bottom(24)
            .build();
        bottom_dock.append(color_bar_ref.widget());

        let dock_manager = crate::ui::dock_manager::DockLayoutManager::new();

        let initial_tb_pos_str = crate::core::AppSettings::toolbar_position();
        let initial_tb_pos = match initial_tb_pos_str.as_str() {
            "top" => crate::plugins::manifest::BarPosition::Top,
            "left" => crate::plugins::manifest::BarPosition::Left,
            "right" => crate::plugins::manifest::BarPosition::Right,
            _ => crate::plugins::manifest::BarPosition::Bottom,
        };

        dock_manager.register("main_toolbar", &bottom_dock, initial_tb_pos, 0, 64, 24);
        dock_manager.register("tool_options", tool_options.widget(), crate::plugins::manifest::BarPosition::Top, 1, 54, 32);
        dock_manager.register("color_palette", palette_bar_ref.widget(), crate::plugins::manifest::BarPosition::Left, 2, 50, 24);

        let _toolbar = FloatingToolbar::new(
            canvas.clone(),
            bottom_dock.clone(),
            (*color_bar_ref).clone(),
            (*palette_bar_ref).clone(),
            tool_options.clone(),
            dock_manager.clone(),
        );

        let inspector = InspectorSidebar::new(canvas.clone(), main_win_holder.clone());
        let inspector_ref = Rc::new(inspector);
        let layers = LayersSidebar::new(canvas.clone());
        let layers_ref = Rc::new(layers);

        let menu_btn = menu::build_main_menu(file_ops, canvas, main_win_holder);

        let header_comps = header::build_header_bar(canvas, file_ops, main_win_holder, &menu_btn);

        // Floating Zoom Controls (Positioned at bottom-right)
        let zoom_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .css_classes(["toolbar", "card"])
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::End)
            .margin_end(16)
            .margin_bottom(24)
            .spacing(2)
            .build();

        let zoom_out_btn = gtk4::Button::builder()
            .icon_name("zoom-out-symbolic")
            .tooltip_text(&crate::core::gettext("Zoom Out (-)"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .build();
        let canvas_zout = canvas.clone();
        zoom_out_btn.connect_clicked(move |_| {
            canvas_zout.zoom_by(1.0 / 1.2);
        });

        let zoom_pct = (saved_zoom * 100.0).round() as i32;
        let zoom_label = gtk4::Label::builder()
            .label(&format!("{}%", zoom_pct))
            .width_chars(5)
            .css_classes(["numeric"])
            .build();

        let zoom_in_btn = gtk4::Button::builder()
            .icon_name("zoom-in-symbolic")
            .tooltip_text(&crate::core::gettext("Zoom In (+)"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .build();
        let canvas_zin = canvas.clone();
        zoom_in_btn.connect_clicked(move |_| {
            canvas_zin.zoom_by(1.2);
        });

        let zoom_reset_btn = gtk4::Button::builder()
            .icon_name("zoom-original-symbolic")
            .tooltip_text(&crate::core::gettext("Reset Zoom (1:1)"))
            .css_classes(["flat"])
            .focus_on_click(false)
            .build();
        let canvas_zreset = canvas.clone();
        zoom_reset_btn.connect_clicked(move |_| {
            canvas_zreset.reset_zoom();
        });

        zoom_box.append(&zoom_out_btn);
        zoom_box.append(&zoom_label);
        zoom_box.append(&zoom_in_btn);
        zoom_box.append(&zoom_reset_btn);

        // Center Overlay: Canvas at bottom + Tool Options + Bottom Dock + Zoom HUD + Colors Palette Bar (topmost)
        let overlay = gtk4::Overlay::builder()
            .hexpand(true)
            .vexpand(true)
            .child(canvas.widget())
            .build();
        overlay.add_overlay(tool_options.widget());
        overlay.add_overlay(&bottom_dock);
        overlay.add_overlay(&zoom_box);
        overlay.add_overlay(palette_bar_ref.widget());

        // Center Content: ToolbarView with top HeaderBar and Overlay content
        let center_toolbar = adw::ToolbarView::new();
        center_toolbar.add_top_bar(&header_comps.header_bar);
        center_toolbar.set_content(Some(&overlay));

        // Restore sidebar visibility from AppSettings
        let show_layers = crate::core::AppSettings::show_layers_sidebar();
        let show_inspector = crate::core::AppSettings::show_inspector_sidebar();

        // Right OverlaySplitView for Inspector (PackType::End)
        let right_split_view = adw::OverlaySplitView::builder()
            .content(&center_toolbar)
            .sidebar(inspector_ref.widget())
            .sidebar_position(gtk4::PackType::End)
            .show_sidebar(show_inspector)
            .min_sidebar_width(250.0)
            .max_sidebar_width(300.0)
            .sidebar_width_fraction(0.18)
            .hexpand(true)
            .vexpand(true)
            .build();

        // Left OverlaySplitView for Layers (PackType::Start)
        let left_split_view = adw::OverlaySplitView::builder()
            .content(&right_split_view)
            .sidebar(layers_ref.widget())
            .sidebar_position(gtk4::PackType::Start)
            .show_sidebar(show_layers)
            .min_sidebar_width(240.0)
            .max_sidebar_width(300.0)
            .sidebar_width_fraction(0.18)
            .hexpand(true)
            .vexpand(true)
            .build();

        header_comps.toggle_layers_btn.set_active(show_layers);
        header_comps.toggle_sidebar_btn.set_active(show_inspector);

        let left_split_tgl = left_split_view.clone();
        header_comps.toggle_layers_btn.connect_toggled(move |btn| {
            let active = btn.is_active();
            left_split_tgl.set_show_sidebar(active);
            crate::core::AppSettings::set_show_layers_sidebar(active);
        });

        let right_split_tgl = right_split_view.clone();
        header_comps.toggle_sidebar_btn.connect_toggled(move |btn| {
            let active = btn.is_active();
            right_split_tgl.set_show_sidebar(active);
            crate::core::AppSettings::set_show_inspector_sidebar(active);
        });

        let dm_pal = dock_manager.clone();
        palette_bar_ref.set_on_reposition(move |pos| {
            dm_pal.update_position("color_palette", pos.into());
        });

        let dm_opt = dock_manager.clone();
        tool_options.set_on_reposition(move |is_top| {
            let pos = if is_top {
                crate::plugins::manifest::BarPosition::Top
            } else {
                crate::plugins::manifest::BarPosition::Bottom
            };
            let base_margin = if is_top { 32 } else { 24 };
            dm_opt.update_position_and_base_margin("tool_options", pos, base_margin);
        });

        // Wire status & selection updates
        let zoom_lbl_clone = zoom_label.clone();
        let tool_options_ref = Rc::new(tool_options);
        let tool_options_clone = tool_options_ref.clone();
        let inspector_clone = inspector_ref.clone();
        let layers_clone = layers_ref.clone();
        let color_bar_clone = color_bar_ref.clone();
        let palette_bar_clone = palette_bar_ref.clone();
        let dock_manager_clone = dock_manager.clone();
        let grid_btn_clone = header_comps.grid_btn.clone();
        let ruler_btn_clone = header_comps.ruler_btn.clone();
        let snap_btn_clone = header_comps.snap_btn.clone();
        let undo_btn_clone = header_comps.undo_btn.clone();
        let redo_btn_clone = header_comps.redo_btn.clone();
        let is_syncing_cb = header_comps.is_syncing_header.clone();
        let canvas_unit_sync = canvas.clone();

        let state_rc = canvas.state();
        let mut state = state_rc.borrow_mut();
        state.on_status_change = Some(Box::new(
            move |snap| {
                let pct = (snap.zoom * 100.0).round() as i32;
                let zoom_str = format!("{}%", pct);
                if zoom_lbl_clone.text().as_str() != zoom_str.as_str() {
                    zoom_lbl_clone.set_label(&zoom_str);
                }

                crate::core::AppSettings::set_active_zoom(snap.zoom as f64);

                if undo_btn_clone.is_sensitive() != snap.can_undo {
                    undo_btn_clone.set_sensitive(snap.can_undo);
                }
                if redo_btn_clone.is_sensitive() != snap.can_redo {
                    redo_btn_clone.set_sensitive(snap.can_redo);
                }

                is_syncing_cb.set(true);
                if grid_btn_clone.is_active() != snap.grid_visible {
                    grid_btn_clone.set_active(snap.grid_visible);
                }
                if ruler_btn_clone.is_active() != snap.ruler_visible {
                    ruler_btn_clone.set_active(snap.ruler_visible);
                }
                if snap_btn_clone.is_active() != snap.snap_enabled {
                    snap_btn_clone.set_active(snap.snap_enabled);
                }
                is_syncing_cb.set(false);

                tool_options_clone.update_state(
                    snap.tool_id,
                    snap.selected_count,
                    snap.bounds,
                    snap.is_editing_text,
                    snap.text_info.clone(),
                    snap.can_convert_to_path,
                    snap.shape_origin.clone(),
                    snap.page_info.clone(),
                    snap.node_coord,
                    snap.image_info.clone(),
                    snap.gradient_info.clone(),
                    snap.pattern_info.clone(),
                    snap.mesh_info,
                );
                inspector_clone.update_context(
                    snap.tool_id,
                    snap.selected_count,
                    snap.bounds,
                    snap.style,
                    snap.can_convert_to_path,
                    snap.fills_and_strokes.clone(),
                    snap.blend_info,
                    snap.image_info.clone(),
                    snap.image_adjustments,
                );
                color_bar_clone.update_state(snap.selected_count, snap.style);
                let is_palette_enabled = canvas_unit_sync.is_plugin_enabled("color_palette_toolbar");
                palette_bar_clone.widget().set_visible(is_palette_enabled);
                dock_manager_clone.update_visibility("color_palette", is_palette_enabled);
                if is_palette_enabled {
                    palette_bar_clone.update_state(snap.selected_count, snap.style, &snap.doc_colors);
                    palette_bar_clone.widget().queue_draw();
                }
                layers_clone.update_state(&snap.layers_info);
            },
        ));

        let init_style = state.document.get_selected_style().unwrap_or((
            Some(state.active_fill_color),
            state.active_stroke_color,
            state.active_stroke_width,
        ));
        let init_tool = state.plugin_manager.active_id();
        let init_count = state.document.selected_ids.len();
        let init_can_convert = state.document.has_non_path_selected();
        let init_page_info = state.document.active_page().map(|p| {
            let r = p.rect.normalize();
            (
                p.name.clone(),
                r.x,
                r.y,
                r.width,
                r.height,
                state.document.pages.len(),
            )
        });
        let init_layers = state.document.get_layers_info();
        let init_can_undo = state.document.can_undo();
        let init_can_redo = state.document.can_redo();
        let init_fills_and_strokes = state.document.get_selected_fills_and_strokes();
        let init_blend_info = state.document.get_selection_blend_info();
        drop(state);

        header_comps.undo_btn.set_sensitive(init_can_undo);
        header_comps.redo_btn.set_sensitive(init_can_redo);

        inspector_ref.update_context(
            init_tool,
            init_count,
            None,
            init_style,
            init_can_convert,
            init_fills_and_strokes,
            init_blend_info,
            None,
            None,
        );
        tool_options_ref.update_state(
            init_tool,
            init_count,
            None,
            false,
            None,
            init_can_convert,
            None,
            init_page_info,
            None,
            None,
            None,
            None,
            None,
        );
        color_bar_ref.update_state(init_count, init_style);
        layers_ref.update_state(&init_layers);

        toast_overlay.set_child(Some(&left_split_view));
    }
}
