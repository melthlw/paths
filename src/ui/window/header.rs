use gtk4::prelude::*;
use libadwaita as adw;
use std::cell::Cell;
use std::rc::Rc;

use super::file_ops::WindowFileOps;
use crate::core::ExportFormat;
use crate::ui::canvas::CanvasWidget;

pub struct HeaderBarComponents {
    pub header_bar: adw::HeaderBar,
    pub toggle_layers_btn: gtk4::ToggleButton,
    pub toggle_sidebar_btn: gtk4::ToggleButton,
    pub undo_btn: gtk4::Button,
    pub redo_btn: gtk4::Button,
    pub grid_btn: gtk4::ToggleButton,
    pub ruler_btn: gtk4::ToggleButton,
    pub snap_btn: gtk4::ToggleButton,
    pub is_syncing_header: Rc<Cell<bool>>,
}

pub fn build_header_bar(
    canvas: &CanvasWidget,
    file_ops: &WindowFileOps,
    menu_btn: &gtk4::MenuButton,
) -> HeaderBarComponents {
    let header_bar = adw::HeaderBar::builder()
        .show_end_title_buttons(true)
        .show_start_title_buttons(true)
        .build();

    let is_syncing_header = Rc::new(Cell::new(false));

    // Left Section: Document Actions & Canvas View Toggles
    let start_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .build();

    let toggle_layers_btn = gtk4::ToggleButton::builder()
        .icon_name("sidebar-layers-symbolic")
        .tooltip_text(&crate::core::gettext("Layers Panel"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .active(true)
        .build();
    start_box.append(&toggle_layers_btn);

    let sep_layers = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    start_box.append(&sep_layers);

    let new_doc_btn = gtk4::Button::builder()
        .icon_name("document-new-symbolic")
        .tooltip_text(&crate::core::gettext("New Document (Ctrl+N)"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let new_doc_action = file_ops.perform_new_doc.clone();
    new_doc_btn.connect_clicked(move |_| {
        new_doc_action();
    });
    start_box.append(&new_doc_btn);

    let open_doc_btn = gtk4::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text(&crate::core::gettext("Open Document (Ctrl+O)"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let open_doc_action = file_ops.perform_open.clone();
    open_doc_btn.connect_clicked(move |_| {
        open_doc_action();
    });
    start_box.append(&open_doc_btn);

    let save_doc_btn = gtk4::Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text(&crate::core::gettext("Save (Ctrl+S)"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let save_doc_action = file_ops.perform_save.clone();
    save_doc_btn.connect_clicked(move |_| {
        save_doc_action();
    });
    start_box.append(&save_doc_btn);

    let sep_undo = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    start_box.append(&sep_undo);

    let undo_btn = gtk4::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text(&crate::core::gettext("Undo (Ctrl+Z)"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let canvas_undo = canvas.clone();
    undo_btn.connect_clicked(move |_| {
        canvas_undo.undo();
    });
    start_box.append(&undo_btn);

    let redo_btn = gtk4::Button::builder()
        .icon_name("edit-redo-symbolic")
        .tooltip_text(&crate::core::gettext("Redo (Ctrl+Y)"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let canvas_redo = canvas.clone();
    redo_btn.connect_clicked(move |_| {
        canvas_redo.redo();
    });
    start_box.append(&redo_btn);

    let sep_view = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    start_box.append(&sep_view);

    let grid_btn = gtk4::ToggleButton::builder()
        .icon_name("grid-symbolic")
        .tooltip_text(&crate::core::gettext("Grid (Ctrl+G)"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .active(canvas.grid_config().visible)
        .build();
    let canvas_grid = canvas.clone();
    let is_syncing_g = is_syncing_header.clone();
    grid_btn.connect_toggled(move |btn| {
        if is_syncing_g.get() {
            return;
        }
        let active = btn.is_active();
        canvas_grid.set_grid_visible(active);
        crate::core::AppSettings::set_show_grid(active);
    });
    start_box.append(&grid_btn);

    let ruler_btn = gtk4::ToggleButton::builder()
        .icon_name("ruler-symbolic")
        .tooltip_text(&crate::core::gettext("Rulers (Ctrl+R)"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .active(canvas.ruler_config().visible)
        .build();
    let canvas_ruler = canvas.clone();
    let is_syncing_r = is_syncing_header.clone();
    ruler_btn.connect_toggled(move |btn| {
        if is_syncing_r.get() {
            return;
        }
        let active = btn.is_active();
        canvas_ruler.set_rulers_visible(active);
        crate::core::AppSettings::set_show_rulers(active);
    });
    start_box.append(&ruler_btn);

    let snap_btn = gtk4::ToggleButton::builder()
        .icon_name("snap-symbolic")
        .tooltip_text(&crate::core::gettext("Magnetic Snapping (Ctrl+Shift+')"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .active(canvas.snap_config().enabled)
        .build();
    let canvas_snap = canvas.clone();
    let is_syncing_s = is_syncing_header.clone();
    snap_btn.connect_toggled(move |btn| {
        if is_syncing_s.get() {
            return;
        }
        let active = btn.is_active();
        canvas_snap.set_snap_enabled(active);
        crate::core::AppSettings::set_snap_enabled(active);
    });
    start_box.append(&snap_btn);

    header_bar.pack_start(&start_box);

    // Window Title
    let title_widget = adw::WindowTitle::builder()
        .title(&crate::core::gettext("Untitled"))
        .subtitle(&crate::core::gettext("GNOME Paths"))
        .build();
    header_bar.set_title_widget(Some(&title_widget));

    let title_widget_file = title_widget.clone();
    canvas.set_on_file_state_changed(move |path_opt, is_dirty| {
        let default_name = crate::core::gettext("Untitled");
        let name = path_opt
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(&default_name);
        if is_dirty {
            title_widget_file.set_title(&format!("● {}", name));
            title_widget_file.set_subtitle(&crate::core::gettext("Unsaved changes — GNOME Paths"));
        } else {
            title_widget_file.set_title(name);
            title_widget_file.set_subtitle(&crate::core::gettext("GNOME Paths"));
        }
    });

    // Quick Export Split Button
    let current_quick_fmt = Rc::new(Cell::new(ExportFormat::Png));

    let quick_exp_content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_start(4)
        .margin_end(2)
        .build();
    let q_icon = gtk4::Image::from_icon_name("document-save-symbolic");
    let q_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Export PNG"))
        .build();
    quick_exp_content.append(&q_icon);
    quick_exp_content.append(&q_lbl);

    let q_popover = gtk4::Popover::builder()
        .has_arrow(true)
        .css_classes(["app-menu-popover"])
        .build();
    let q_pop_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .width_request(190)
        .build();

    let quick_exp_btn = adw::SplitButton::builder()
        .tooltip_text(&format!("{} (PNG)", crate::core::gettext("Quick Export")))
        .valign(gtk4::Align::Center)
        .build();
    quick_exp_btn.set_child(Some(&quick_exp_content));
    quick_exp_btn.set_popover(Some(&q_popover));

    let formats = [
        (
            crate::core::gettext("PNG (Raster Image)"),
            ExportFormat::Png,
            crate::core::gettext("Export PNG"),
        ),
        (
            crate::core::gettext("JPG / JPEG (Photography)"),
            ExportFormat::Jpg,
            crate::core::gettext("Export JPG"),
        ),
        (
            crate::core::gettext("SVG (Scalable Vector)"),
            ExportFormat::Svg,
            crate::core::gettext("Export SVG"),
        ),
        (
            crate::core::gettext("PDF (Portable Document)"),
            ExportFormat::Pdf,
            crate::core::gettext("Export PDF"),
        ),
        (
            crate::core::gettext("WebP (Compact Web)"),
            ExportFormat::WebP,
            crate::core::gettext("Export WebP"),
        ),
    ];

    for (label_text, fmt_val, btn_title) in formats {
        let item_btn = gtk4::Button::builder()
            .css_classes(["flat", "menu-item-btn"])
            .halign(gtk4::Align::Fill)
            .build();
        let row_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        let row_lbl = gtk4::Label::builder()
            .label(label_text)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();
        row_box.append(&row_lbl);
        item_btn.set_child(Some(&row_box));

        let pop_c = q_popover.clone();
        let q_lbl_c = q_lbl.clone();
        let q_btn_c = quick_exp_btn.clone();
        let cur_fmt_c = current_quick_fmt.clone();
        let do_exp = file_ops.perform_quick_export.clone();
        item_btn.connect_clicked(move |_| {
            pop_c.popdown();
            cur_fmt_c.set(fmt_val);
            q_lbl_c.set_label(&btn_title);
            let tt = crate::i18n!("Quick Export ({})", fmt_val.extension().to_uppercase());
            q_btn_c.set_tooltip_text(Some(&tt));
            do_exp(fmt_val);
        });
        q_pop_box.append(&item_btn);
    }

    q_popover.set_child(Some(&q_pop_box));

    let cur_fmt_main = current_quick_fmt.clone();
    let do_exp_main = file_ops.perform_quick_export.clone();
    quick_exp_btn.connect_clicked(move |_| {
        do_exp_main(cur_fmt_main.get());
    });

    let toggle_sidebar_btn = gtk4::ToggleButton::builder()
        .icon_name("sidebar-inspector-symbolic")
        .tooltip_text(&crate::core::gettext("Panels"))
        .active(true)
        .build();

    // Pack end (from right edge inward):
    header_bar.pack_end(menu_btn);
    header_bar.pack_end(&toggle_sidebar_btn);
    header_bar.pack_end(&quick_exp_btn);

    HeaderBarComponents {
        header_bar,
        toggle_layers_btn,
        toggle_sidebar_btn,
        undo_btn,
        redo_btn,
        grid_btn,
        ruler_btn,
        snap_btn,
        is_syncing_header,
    }
}
