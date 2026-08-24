use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use super::TabLocation;

pub fn create_floating_window(
    i: usize,
    title: &str,
    icon_res: Option<&'static str>,
    icon_name: &'static str,
    tab_widget: &gtk4::Widget,
    tab_locations: &Rc<RefCell<[TabLocation; 6]>>,
    active_section_tabs: &Rc<RefCell<[usize; 6]>>,
    refresh_fn: Rc<dyn Fn()>,
    main_win_holder: &Rc<RefCell<Option<adw::ApplicationWindow>>>,
) -> adw::Window {
    let tb_view = adw::ToolbarView::new();
    let float_header = adw::HeaderBar::builder()
        .show_start_title_buttons(false)
        .show_end_title_buttons(false)
        .build();

    // Dock Button in Floating Header
    let dock_btn = gtk4::Button::builder()
        .icon_name("sidebar-show-right-symbolic")
        .tooltip_text(crate::core::gettext("Dock to sidebar"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let locs_dock = tab_locations.clone();
    let act_sec_dock = active_section_tabs.clone();
    let re_dock = refresh_fn.clone();
    dock_btn.connect_clicked(move |_| {
        locs_dock.borrow_mut()[i] = TabLocation::Docked(0);
        act_sec_dock.borrow_mut()[0] = i;
        re_dock();
    });
    float_header.pack_start(&dock_btn);

    // Close Button in Floating Header
    let close_btn = gtk4::Button::builder()
        .icon_name("window-close-symbolic")
        .tooltip_text(crate::core::gettext("Close Panel"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();
    let locs_close_f = tab_locations.clone();
    let re_close_f = refresh_fn.clone();
    close_btn.connect_clicked(move |_| {
        locs_close_f.borrow_mut()[i] = TabLocation::Closed;
        re_close_f();
    });
    float_header.pack_end(&close_btn);

    // Draggable Tab Handle / Badge in Floating Window Header
    let tab_pill = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .css_classes([
            "flat",
            "studio-tab-btn",
            "active",
            "floating-tab-pill",
        ])
        .tooltip_text(crate::core::gettext(
            "Drag to sidebar or right click to dock/close",
        ))
        .build();
    tab_pill.set_cursor_from_name(Some("grab"));

    let drag_icon = gtk4::Image::from_icon_name("list-drag-handle-symbolic");
    drag_icon.set_opacity(0.6);
    let icon_img = if let Some(res) = icon_res {
        crate::ui::icons::make_symbolic_image(res, 16)
    } else {
        crate::ui::icons::make_symbolic_image(icon_name, 16)
    };
    let lbl = gtk4::Label::builder()
        .label(&crate::i18n!("{} — GNOME Paths", title))
        .css_classes(["heading"])
        .build();

    tab_pill.append(&drag_icon);
    tab_pill.append(&icon_img);
    tab_pill.append(&lbl);

    let drag_src = gtk4::DragSource::new();
    drag_src.set_actions(gdk::DragAction::MOVE);
    let tab_idx = i;
    drag_src.connect_prepare(move |_, _, _| {
        Some(gdk::ContentProvider::for_value(
            &format!("tab:{}", tab_idx).to_value(),
        ))
    });
    let pill_paintable = gtk4::WidgetPaintable::new(Some(&tab_pill));
    drag_src.set_icon(Some(&pill_paintable), 24, 14);
    tab_pill.add_controller(drag_src);

    let float_popover = gtk4::Popover::builder()
        .has_arrow(false)
        .autohide(true)
        .position(gtk4::PositionType::Bottom)
        .css_classes(["menu", "tab-context-popover"])
        .build();
    float_popover.set_parent(&tab_pill);
    {
        let fpop_c = float_popover.clone();
        tab_pill.connect_destroy(move |_| {
            if fpop_c.parent().is_some() {
                fpop_c.unparent();
            }
        });
    }

    let float_menu_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(1)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .width_request(230)
        .build();

    let create_float_item = |icon_name: &str, label_text: &str| -> (gtk4::Button, gtk4::Box) {
        let b = gtk4::Button::builder()
            .css_classes(["flat", "menu-button"])
            .halign(gtk4::Align::Fill)
            .focus_on_click(false)
            .build();
        let row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .margin_start(4)
            .margin_end(6)
            .margin_top(2)
            .margin_bottom(2)
            .build();
        let img = gtk4::Image::from_icon_name(icon_name);
        img.set_pixel_size(16);
        img.add_css_class("dim-label");
        let lbl = gtk4::Label::builder()
            .label(label_text)
            .xalign(0.0)
            .hexpand(true)
            .build();
        row.append(&img);
        row.append(&lbl);
        b.set_child(Some(&row));
        (b, row)
    };

    let (btn_dock_top, _) = create_float_item(
        "sidebar-show-right-symbolic",
        &crate::core::gettext("Dock into Top Bar"),
    );
    {
        let locs_c = tab_locations.clone();
        let act_sec_tabs_c = active_section_tabs.clone();
        let re_c = refresh_fn.clone();
        let pop_c = float_popover.clone();
        btn_dock_top.connect_clicked(move |_| {
            pop_c.popdown();
            locs_c.borrow_mut()[i] = TabLocation::Docked(0);
            act_sec_tabs_c.borrow_mut()[0] = i;
            re_c();
        });
    }
    float_menu_box.append(&btn_dock_top);

    let (btn_dock_bottom, _) = create_float_item(
        "go-down-symbolic",
        &crate::core::gettext("Dock into New Section Below"),
    );
    {
        let locs_c = tab_locations.clone();
        let act_sec_tabs_c = active_section_tabs.clone();
        let re_c = refresh_fn.clone();
        let pop_c = float_popover.clone();
        btn_dock_bottom.connect_clicked(move |_| {
            pop_c.popdown();
            let cur_locs = *locs_c.borrow();
            let max_sec = (0..4)
                .filter_map(|k| {
                    if let TabLocation::Docked(s) = cur_locs[k] {
                        Some(s)
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0);
            locs_c.borrow_mut()[i] = TabLocation::Docked(max_sec + 1);
            if max_sec + 1 < act_sec_tabs_c.borrow().len() {
                act_sec_tabs_c.borrow_mut()[max_sec + 1] = i;
            }
            re_c();
        });
    }
    float_menu_box.append(&btn_dock_bottom);

    let (btn_float_close, _) = create_float_item(
        "window-close-symbolic",
        &crate::core::gettext("Close Panel"),
    );
    {
        let locs_c = tab_locations.clone();
        let re_c = refresh_fn.clone();
        let pop_c = float_popover.clone();
        btn_float_close.connect_clicked(move |_| {
            pop_c.popdown();
            locs_c.borrow_mut()[i] = TabLocation::Closed;
            re_c();
        });
    }
    float_menu_box.append(&btn_float_close);

    float_popover.set_child(Some(&float_menu_box));

    let float_rclick = gtk4::GestureClick::builder()
        .button(gdk::BUTTON_SECONDARY)
        .build();
    let pop_fc = float_popover.clone();
    float_rclick.connect_pressed(move |_, _, _, _| {
        pop_fc.popup();
    });
    tab_pill.add_controller(float_rclick);

    float_header.set_title_widget(Some(&tab_pill));

    let float_scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .child(tab_widget)
        .build();

    tb_view.add_top_bar(&float_header);
    tb_view.set_content(Some(&float_scroll));

    let float_win = adw::Window::builder()
        .title(&format!("{} — GNOME Paths", title))
        .default_width(320)
        .default_height(560)
        .content(&tb_view)
        .resizable(true)
        .build();

    if let Some(ref p) = *main_win_holder.borrow() {
        float_win.set_transient_for(Some(p));
    }
    float_win.set_destroy_with_parent(true);

    let locs_close = tab_locations.clone();
    let re_close = refresh_fn.clone();
    float_win.connect_close_request(move |_| {
        if locs_close.borrow()[i] == TabLocation::Floating {
            locs_close.borrow_mut()[i] = TabLocation::Closed;
            re_close();
        }
        glib::Propagation::Proceed
    });

    float_win
}
