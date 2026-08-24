use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// ─────────────────────────────────────────────────────────────
// VISUAL COMPONENT BUILDERS (COLOR & PAPER CHIPS, CAPSULES)
// ─────────────────────────────────────────────────────────────

pub fn make_color_swatches<T: Clone + PartialEq + 'static>(
    items: Vec<(&'static str, &'static str, T)>,
    current: T,
    on_selected: impl Fn(T) + Clone + 'static,
) -> gtk4::Box {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(12)
        .margin_end(12)
        .build();

    let buttons: Rc<RefCell<Vec<(gtk4::Button, gtk4::Image)>>> = Rc::new(RefCell::new(Vec::new()));

    for (name, chip_class, val) in items {
        let check_img = gtk4::Image::from_icon_name("check-symbolic");
        check_img.set_pixel_size(14);
        check_img.set_valign(gtk4::Align::Center);
        check_img.set_halign(gtk4::Align::Center);

        let is_active = val == current;
        check_img.set_visible(is_active);

        let btn = gtk4::Button::builder()
            .tooltip_text(crate::core::gettext(name))
            .valign(gtk4::Align::Center)
            .css_classes(["pref-color-chip", chip_class])
            .child(&check_img)
            .build();

        if is_active {
            btn.add_css_class("active");
        }

        let on_sel = on_selected.clone();
        let val_c = val.clone();
        let btn_list = buttons.clone();
        let btn_self = btn.clone();
        let check_self = check_img.clone();

        btn.connect_clicked(move |_| {
            for (b, chk) in btn_list.borrow().iter() {
                b.remove_css_class("active");
                chk.set_visible(false);
            }
            btn_self.add_css_class("active");
            check_self.set_visible(true);
            on_sel(val_c.clone());
        });

        buttons.borrow_mut().push((btn.clone(), check_img));
        container.append(&btn);
    }

    container
}

pub fn make_segmented_capsule<T: Clone + PartialEq + 'static>(
    items: Vec<(&'static str, T)>,
    current: T,
    on_selected: impl Fn(T) + Clone + 'static,
) -> gtk4::Box {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .css_classes(["segmented-capsule"])
        .valign(gtk4::Align::Center)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    let buttons: Rc<RefCell<Vec<gtk4::Button>>> = Rc::new(RefCell::new(Vec::new()));

    for (label, val) in items {
        let btn = gtk4::Button::builder()
            .label(crate::core::gettext(label))
            .css_classes(["flat"])
            .valign(gtk4::Align::Center)
            .build();

        if val == current {
            btn.add_css_class("active");
        }

        let on_sel = on_selected.clone();
        let val_c = val.clone();
        let btn_list = buttons.clone();
        let btn_self = btn.clone();

        btn.connect_clicked(move |_| {
            for b in btn_list.borrow().iter() {
                b.remove_css_class("active");
            }
            btn_self.add_css_class("active");
            on_sel(val_c.clone());
        });

        buttons.borrow_mut().push(btn.clone());
        container.append(&btn);
    }

    container
}

pub fn show_language_chooser_dialog(
    parent: &impl IsA<gtk4::Widget>,
    current_lang_label: gtk4::Label,
    current_lang_badge: gtk4::Label,
) {
    let window = adw::Window::builder()
        .title(crate::core::gettext("Select Language"))
        .modal(true)
        .default_width(520)
        .default_height(580)
        .build();

    if let Some(root) = parent.root() {
        if let Some(win) = root.downcast_ref::<gtk4::Window>() {
            window.set_transient_for(Some(win));
        }
    }

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::builder()
        .title_widget(&adw::WindowTitle::new(
            &crate::core::gettext("Select Language"),
            &crate::core::gettext("Choose application interface language"),
        ))
        .show_start_title_buttons(true)
        .show_end_title_buttons(true)
        .build();
    toolbar_view.add_top_bar(&header);

    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_start(18)
        .margin_end(18)
        .margin_top(14)
        .margin_bottom(18)
        .build();

    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text(crate::core::gettext("Search language or country..."))
        .build();
    content_box.append(&search_entry);

    let list_box = gtk4::ListBox::builder()
        .css_classes(["boxed-list"])
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    let cur_lang = crate::core::get_language();
    let mut search_items: Vec<(String, adw::ActionRow, gtk4::CheckButton)> = Vec::new();
    let mut first_radio: Option<gtk4::CheckButton> = None;

    let all_languages = crate::core::Language::all_info();
    for info in all_languages {
        let is_current = info.lang == cur_lang;

        let title_str = if info.lang == crate::core::Language::System {
            crate::core::gettext("System Default")
        } else {
            info.native_name.to_string()
        };

        let row = adw::ActionRow::builder()
            .title(&title_str)
            .subtitle(format!(
                "{} • {}",
                crate::core::gettext(info.localized_name),
                crate::core::gettext(info.region)
            ))
            .activatable(true)
            .build();

        let radio = gtk4::CheckButton::builder()
            .valign(gtk4::Align::Center)
            .active(is_current)
            .can_focus(false)
            .build();

        if let Some(ref first) = first_radio {
            radio.set_group(Some(first));
        } else {
            first_radio = Some(radio.clone());
        }

        row.add_prefix(&radio);

        let icon = gtk4::Image::from_icon_name("prefs-language-symbolic");
        icon.set_pixel_size(18);
        row.add_prefix(&icon);

        let badge = gtk4::Label::builder()
            .label(info.code)
            .css_classes(["dim-label", "numeric"])
            .valign(gtk4::Align::Center)
            .build();
        row.add_suffix(&badge);

        if is_current {
            let check_icon = gtk4::Image::from_icon_name("check-symbolic");
            check_icon.set_pixel_size(16);
            check_icon.add_css_class("accent");
            row.add_suffix(&check_icon);
            row.add_css_class("accent");
        }

        list_box.append(&row);

        let search_text = format!(
            "{} {} {} {}",
            info.native_name.to_lowercase(),
            info.localized_name.to_lowercase(),
            info.region.to_lowercase(),
            info.code.to_lowercase()
        );
        search_items.push((search_text, row.clone(), radio.clone()));

        let target_lang = info.lang;
        let lbl_update = current_lang_label.clone();
        let bdg_update = current_lang_badge.clone();
        let win_close = window.clone();
        let radio_toggle = radio.clone();

        row.connect_activated(move |_| {
            radio_toggle.set_active(true);
            crate::core::set_language(target_lang);
            crate::core::AppSettings::set_language(target_lang.code());
            let updated_info = target_lang.info();
            let display_name = if target_lang == crate::core::Language::System {
                crate::core::gettext("System Default")
            } else {
                updated_info.native_name.to_string()
            };
            lbl_update.set_label(&display_name);
            bdg_update.set_label(updated_info.code);
            win_close.close();
        });
    }

    let search_items_rc = Rc::new(RefCell::new(search_items));
    {
        let items_c = search_items_rc.clone();
        search_entry.connect_search_changed(move |entry| {
            let q = entry.text().trim().to_lowercase();
            let items = items_c.borrow();
            for (text, row, _) in items.iter() {
                if q.is_empty() || text.contains(&q) {
                    row.set_visible(true);
                } else {
                    row.set_visible(false);
                }
            }
        });
    }

    let scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .child(&list_box)
        .build();

    content_box.append(&scroll);

    let clamp = adw::Clamp::builder()
        .maximum_size(480)
        .vexpand(true)
        .child(&content_box)
        .build();

    toolbar_view.set_content(Some(&clamp));
    window.set_content(Some(&toolbar_view));
    window.present();
}

pub fn make_page(groups: Vec<adw::PreferencesGroup>) -> gtk4::ScrolledWindow {
    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(24)
        .build();

    for group in groups {
        content_box.append(&group);
    }

    let clamp = adw::Clamp::builder()
        .maximum_size(740)
        .tightening_threshold(520)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(32)
        .child(&content_box)
        .build();

    gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .child(&clamp)
        .build()
}
