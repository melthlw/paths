use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::TabLocation;
use crate::ui::canvas::CanvasWidget;

pub fn build_catalog_popover(
    tab_info: &[(String, Option<&'static str>, &'static str); 9],
    tab_locations: &Rc<RefCell<[TabLocation; 9]>>,
    active_section_tabs: &Rc<RefCell<[usize; 9]>>,
    _canvas: &CanvasWidget,
    refresh_fn: Rc<dyn Fn()>,
) -> gtk4::Popover {
    let add_pop = gtk4::Popover::builder()
        .has_arrow(true)
        .autohide(true)
        .position(gtk4::PositionType::Bottom)
        .css_classes(["menu", "panel-catalog-popover"])
        .build();

    let pop_wrapper = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .width_request(260)
        .build();

    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text(crate::core::gettext("Search panels..."))
        .build();
    pop_wrapper.append(&search_entry);

    let list_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(2)
        .build();

    let mut filterable_items: Vec<(String, gtk4::Widget)> = Vec::new();

    let core_title = gtk4::Label::builder()
        .label(crate::core::gettext("Core Panels"))
        .css_classes(["caption", "dim-label"])
        .xalign(0.0)
        .margin_start(6)
        .margin_top(4)
        .build();
    list_box.append(&core_title);
    filterable_items.push((
        core_title.text().to_string().to_lowercase(),
        core_title.upcast(),
    ));

    let locs = *tab_locations.borrow();

    for (i, (title, icon_res, icon_name)) in tab_info.iter().enumerate() {
        let is_open = locs[i] != TabLocation::Closed;
        let item_btn = gtk4::Button::builder()
            .css_classes(["flat", "menu-item-btn"])
            .build();
        let row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .margin_start(6)
            .margin_end(6)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        let icon_img = if let Some(res) = icon_res {
            crate::ui::icons::make_symbolic_image(res, 16)
        } else {
            crate::ui::icons::make_symbolic_image(icon_name, 16)
        };
        let lbl = gtk4::Label::builder()
            .label(title.as_str())
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();
        row.append(&icon_img);
        row.append(&lbl);

        if is_open {
            let check = gtk4::Image::from_icon_name("check-symbolic");
            check.set_pixel_size(14);
            check.add_css_class("accent");
            row.append(&check);
        } else {
            let plus = gtk4::Image::from_icon_name("list-add-symbolic");
            plus.set_pixel_size(14);
            plus.add_css_class("dim-label");
            row.append(&plus);
        }

        item_btn.set_child(Some(&row));

        let locs_c = tab_locations.clone();
        let act_sec_c = active_section_tabs.clone();
        let re_c = refresh_fn.clone();
        let pop_c = add_pop.clone();
        item_btn.connect_clicked(move |_| {
            pop_c.popdown();
            if locs_c.borrow()[i] == TabLocation::Closed {
                locs_c.borrow_mut()[i] = TabLocation::Docked(0);
                act_sec_c.borrow_mut()[0] = i;
            } else {
                locs_c.borrow_mut()[i] = TabLocation::Closed;
            }
            re_c();
        });

        list_box.append(&item_btn);
        filterable_items.push((title.to_lowercase(), item_btn.upcast()));
    }

    // Placeholder/Hook for plugin & tool panels (panels only, not general plugin toggles)
    let plugin_panels: Vec<(&str, &str, &str)> = Vec::new();
    if !plugin_panels.is_empty() {
        let plug_title = gtk4::Label::builder()
            .label(crate::core::gettext("Plugins & Tools"))
            .css_classes(["caption", "dim-label"])
            .xalign(0.0)
            .margin_start(6)
            .margin_top(8)
            .build();
        list_box.append(&plug_title);
        filterable_items.push((
            plug_title.text().to_string().to_lowercase(),
            plug_title.upcast(),
        ));

        for (_id, name, icon) in plugin_panels {
            let p_btn = gtk4::Button::builder()
                .css_classes(["flat", "menu-item-btn"])
                .build();
            let p_row = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(10)
                .margin_start(6)
                .margin_end(6)
                .margin_top(4)
                .margin_bottom(4)
                .build();
            let p_icon = gtk4::Image::from_icon_name(icon);
            p_icon.set_pixel_size(16);
            let p_lbl = gtk4::Label::builder()
                .label(name)
                .hexpand(true)
                .halign(gtk4::Align::Start)
                .build();
            p_row.append(&p_icon);
            p_row.append(&p_lbl);
            p_btn.set_child(Some(&p_row));
            list_box.append(&p_btn);
            filterable_items.push((name.to_lowercase(), p_btn.upcast()));
        }
    }

    let filterable_rc = Rc::new(RefCell::new(filterable_items));
    {
        let filterable_c = filterable_rc.clone();
        search_entry.connect_search_changed(move |entry| {
            let q = entry.text().trim().to_lowercase();
            let items = filterable_c.borrow();
            for (search_text, widget) in items.iter() {
                if q.is_empty() || search_text.contains(&q) {
                    widget.set_visible(true);
                } else {
                    widget.set_visible(false);
                }
            }
        });
    }

    let scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .max_content_height(380)
        .propagate_natural_height(true)
        .child(&list_box)
        .build();
    pop_wrapper.append(&scroll);

    add_pop.set_child(Some(&pop_wrapper));
    add_pop
}
