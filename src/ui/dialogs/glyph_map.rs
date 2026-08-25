use gtk4::prelude::*;
use crate::core::element::get_curated_font_glyphs;
use crate::ui::canvas::CanvasWidget;

pub fn build_glyph_map_popover(canvas: &CanvasWidget) -> gtk4::Popover {
    let popover = gtk4::Popover::builder()
        .has_arrow(true)
        .autohide(true)
        .position(gtk4::PositionType::Bottom)
        .css_classes(["menu", "glyph-catalog-popover"])
        .build();

    let main_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .width_request(280)
        .build();

    let header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk4::Align::Center)
        .build();
    let icon = gtk4::Image::from_icon_name("preferences-desktop-font-symbolic");
    let title = gtk4::Label::builder()
        .label(crate::core::gettext("Glyph & Symbol Catalog"))
        .css_classes(["heading", "caption"])
        .hexpand(true)
        .xalign(0.0)
        .build();
    header.append(&icon);
    header.append(&title);
    main_box.append(&header);

    main_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    let notebook = gtk4::Notebook::builder()
        .tab_pos(gtk4::PositionType::Top)
        .scrollable(true)
        .show_border(false)
        .build();

    let categories = get_curated_font_glyphs();
    for (cat_name, chars) in categories {
        let page_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .max_content_height(200)
            .propagate_natural_height(true)
            .build();

        let grid = gtk4::FlowBox::builder()
            .valign(gtk4::Align::Start)
            .max_children_per_line(5)
            .min_children_per_line(4)
            .selection_mode(gtk4::SelectionMode::None)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        for ch in chars {
            let ch_str = ch.to_string();
            let btn = gtk4::Button::builder()
                .label(&ch_str)
                .tooltip_text(&format!("U+{:04X}", ch as u32))
                .css_classes(["flat", "glyph-grid-btn"])
                .focus_on_click(false)
                .build();

            let canvas_c = canvas.clone();
            let pop_c = popover.clone();
            let ch_s = ch_str.clone();
            btn.connect_clicked(move |_| {
                canvas_c.insert_symbol_to_selected_text(&ch_s);
                pop_c.popdown();
            });

            grid.insert(&btn, -1);
        }

        page_scroll.set_child(Some(&grid));

        let tab_lbl = gtk4::Label::builder()
            .label(cat_name)
            .css_classes(["caption"])
            .build();

        notebook.append_page(&page_scroll, Some(&tab_lbl));
    }

    main_box.append(&notebook);
    popover.set_child(Some(&main_box));
    popover
}
