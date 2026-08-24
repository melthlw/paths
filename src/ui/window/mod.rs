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
        let canvas = CanvasWidget::new();
        let main_win_holder: Rc<RefCell<Option<adw::ApplicationWindow>>> =
            Rc::new(RefCell::new(None));
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

        let toolbar = FloatingToolbar::new(
            canvas.clone(),
            bottom_dock.clone(),
            (*color_bar_ref).clone(),
            tool_options.clone(),
        );
        bottom_dock.append(toolbar.widget());

        let inspector = InspectorSidebar::new(canvas.clone(), main_win_holder.clone());
        let inspector_ref = Rc::new(inspector);
        let layers = LayersSidebar::new(canvas.clone());
        let layers_ref = Rc::new(layers);

        let toast_overlay = adw::ToastOverlay::new();

        let file_ops = WindowFileOps::new(canvas.clone(), main_win_holder.clone(), toast_overlay.clone());

        let menu_btn = menu::build_main_menu(&file_ops, &canvas, &main_win_holder);

        let header_comps = header::build_header_bar(&canvas, &file_ops, &menu_btn);

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

        let zoom_label = gtk4::Label::builder()
            .label("100%")
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

        // Center Overlay: Canvas at bottom + Tool Options + Bottom Dock + Lateral Palette Bar + Zoom HUD
        let overlay = gtk4::Overlay::builder()
            .hexpand(true)
            .vexpand(true)
            .child(canvas.widget())
            .build();
        overlay.add_overlay(tool_options.widget());
        overlay.add_overlay(&bottom_dock);
        overlay.add_overlay(palette_bar_ref.widget());
        overlay.add_overlay(&zoom_box);

        // Center Content: ToolbarView with top HeaderBar and Overlay content
        let center_toolbar = adw::ToolbarView::new();
        center_toolbar.add_top_bar(&header_comps.header_bar);
        center_toolbar.set_content(Some(&overlay));

        // Right OverlaySplitView for Inspector (PackType::End)
        let right_split_view = adw::OverlaySplitView::builder()
            .content(&center_toolbar)
            .sidebar(inspector_ref.widget())
            .sidebar_position(gtk4::PackType::End)
            .show_sidebar(true)
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
            .show_sidebar(true)
            .min_sidebar_width(240.0)
            .max_sidebar_width(300.0)
            .sidebar_width_fraction(0.18)
            .hexpand(true)
            .vexpand(true)
            .build();

        let left_split_tgl = left_split_view.clone();
        header_comps.toggle_layers_btn.connect_toggled(move |btn| {
            left_split_tgl.set_show_sidebar(btn.is_active());
        });

        let right_split_tgl = right_split_view.clone();
        header_comps.toggle_sidebar_btn.connect_toggled(move |btn| {
            right_split_tgl.set_show_sidebar(btn.is_active());
        });

        // Wire status & selection updates
        let zoom_lbl_clone = zoom_label.clone();
        let tool_options_ref = Rc::new(tool_options);
        let tool_options_clone = tool_options_ref.clone();
        let inspector_clone = inspector_ref.clone();
        let layers_clone = layers_ref.clone();
        let color_bar_clone = color_bar_ref.clone();
        let palette_bar_clone = palette_bar_ref.clone();
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
            move |tool_id,
                  zoom,
                  selected_count,
                  sel_bounds,
                  style,
                  is_editing_text,
                  text_info,
                  grid_vis,
                  ruler_vis,
                  snap_en,
                  can_convert,
                  shape_origin,
                  page_info,
                  layers_info,
                  can_undo,
                  can_redo,
                  fills_and_strokes,
                  blend_info,
                  node_coord,
                  doc_colors| {
                let pct = (zoom * 100.0).round() as i32;
                let zoom_str = format!("{}%", pct);
                if zoom_lbl_clone.text().as_str() != zoom_str.as_str() {
                    zoom_lbl_clone.set_label(&zoom_str);
                }

                if undo_btn_clone.is_sensitive() != can_undo {
                    undo_btn_clone.set_sensitive(can_undo);
                }
                if redo_btn_clone.is_sensitive() != can_redo {
                    redo_btn_clone.set_sensitive(can_redo);
                }

                is_syncing_cb.set(true);
                if grid_btn_clone.is_active() != grid_vis {
                    grid_btn_clone.set_active(grid_vis);
                }
                if ruler_btn_clone.is_active() != ruler_vis {
                    ruler_btn_clone.set_active(ruler_vis);
                }
                if snap_btn_clone.is_active() != snap_en {
                    snap_btn_clone.set_active(snap_en);
                }
                is_syncing_cb.set(false);

                tool_options_clone.update_state(
                    tool_id,
                    selected_count,
                    sel_bounds,
                    is_editing_text,
                    text_info,
                    can_convert,
                    shape_origin,
                    page_info,
                    node_coord,
                );
                inspector_clone.update_context(
                    tool_id,
                    selected_count,
                    sel_bounds,
                    style,
                    can_convert,
                    fills_and_strokes,
                    blend_info,
                );
                color_bar_clone.update_state(selected_count, style);
                let is_palette_enabled = canvas_unit_sync.is_plugin_enabled("color_palette_toolbar");
                palette_bar_clone.widget().set_visible(is_palette_enabled);
                if is_palette_enabled {
                    palette_bar_clone.update_state(selected_count, style, &doc_colors);
                    palette_bar_clone.widget().queue_draw();
                }
                layers_clone.update_state(&layers_info);
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
        );
        color_bar_ref.update_state(init_count, init_style);
        layers_ref.update_state(&init_layers);

        toast_overlay.set_child(Some(&left_split_view));

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("gnome-paths")
            .default_width(1280)
            .default_height(780)
            .content(&toast_overlay)
            .build();

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
        self.window.present();
    }
}
