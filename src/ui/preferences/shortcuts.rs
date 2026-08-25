use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::widgets::{make_page, make_segmented_capsule};
use crate::core::{KeyCombo, ShortcutAction, ShortcutCategory, ShortcutPreset};
use crate::ui::canvas::CanvasWidget;

pub fn build_shortcuts_page(window: &adw::Window, canvas: &CanvasWidget) -> gtk4::ScrolledWindow {
    let key_theme_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Shortcut Scheme"))
        .description(crate::core::gettext(
            "Keyboard compatibility presets — click any shortcut to customize it",
        ))
        .build();

    let cur_preset = canvas.shortcuts().preset;
    let cur_preset_code = match cur_preset {
        ShortcutPreset::Figma => "figma",
        ShortcutPreset::Illustrator => "illustrator",
        ShortcutPreset::Inkscape => "inkscape",
        ShortcutPreset::Default => "default",
    };

    let filterable_rows: Rc<RefCell<Vec<(String, adw::ActionRow)>>> =
        Rc::new(RefCell::new(Vec::new()));

    let registered_labels: Rc<RefCell<Vec<(ShortcutAction, gtk4::Label)>>> =
        Rc::new(RefCell::new(Vec::new()));

    let key_preset_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Keyboard scheme"))
        .subtitle(crate::core::gettext(
            "Adapts shortcut keys to your preferred software",
        ))
        .build();

    {
        let canvas_c = canvas.clone();
        let labels_c = registered_labels.clone();
        let preset_capsule = make_segmented_capsule(
            vec![
                ("Default", "default"),
                ("Inkscape", "inkscape"),
                ("Figma", "figma"),
                ("Illustrator", "illustrator"),
            ],
            cur_preset_code,
            move |code| {
                let preset = match code {
                    "figma" => ShortcutPreset::Figma,
                    "illustrator" => ShortcutPreset::Illustrator,
                    "inkscape" => ShortcutPreset::Inkscape,
                    _ => ShortcutPreset::Default,
                };
                canvas_c.set_shortcut_preset(preset);
                crate::core::AppSettings::set_shortcut_preset(code);
                let sm = canvas_c.shortcuts();
                for (act, lbl) in &*labels_c.borrow() {
                    if let Some(combo) = sm.get_shortcut(*act) {
                        lbl.set_label(&combo.to_display_string());
                    }
                }
            },
        );
        key_preset_row.add_suffix(&preset_capsule);
    }
    key_theme_group.add(&key_preset_row);

    // Shortcut search filter
    let shortcut_search_entry = gtk4::SearchEntry::builder()
        .placeholder_text(crate::core::gettext("Search shortcuts..."))
        .margin_top(4)
        .margin_bottom(8)
        .build();

    let search_group = adw::PreferencesGroup::builder().build();
    search_group.add(&shortcut_search_entry);

    // Tools Group
    let tools_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Creation and Selection Tools"))
        .description(crate::core::gettext("Click row to customize shortcut"))
        .build();

    let tool_items = canvas.shortcuts().list_by_category(ShortcutCategory::Tools);
    for (action, desc, combo) in tool_items {
        let row = adw::ActionRow::builder()
            .title(&desc)
            .activatable(true)
            .build();

        let key_btn = gtk4::Button::builder()
            .css_classes(["flat"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Edit shortcut"))
            .build();

        let key_badge = gtk4::Label::builder()
            .label(&combo.to_display_string())
            .css_classes(["dim-label", "numeric"])
            .build();

        key_btn.set_child(Some(&key_badge));
        row.add_suffix(&key_btn);
        tools_group.add(&row);

        filterable_rows
            .borrow_mut()
            .push((desc.to_lowercase(), row.clone()));
        registered_labels
            .borrow_mut()
            .push((action, key_badge.clone()));

        let canvas_edit = canvas.clone();
        let badge_clone = key_badge.clone();
        let win_clone = window.clone();
        let desc_clone = desc.clone();
        let trigger_edit = move || {
            show_shortcut_editor_dialog(
                &win_clone,
                canvas_edit.clone(),
                action,
                &desc_clone,
                badge_clone.clone(),
            );
        };

        let tr_btn = trigger_edit.clone();
        key_btn.connect_clicked(move |_| {
            tr_btn();
        });

        let tr_row = trigger_edit.clone();
        row.connect_activated(move |_| {
            tr_row();
        });
    }

    // Edit Group
    let edit_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Editing, Organization and History"))
        .description(crate::core::gettext("Click row to customize shortcut"))
        .build();

    let edit_items = canvas.shortcuts().list_by_category(ShortcutCategory::Edit);
    for (action, desc, combo) in edit_items {
        let row = adw::ActionRow::builder()
            .title(&desc)
            .activatable(true)
            .build();

        let key_btn = gtk4::Button::builder()
            .css_classes(["flat"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Edit shortcut"))
            .build();

        let key_badge = gtk4::Label::builder()
            .label(&combo.to_display_string())
            .css_classes(["dim-label", "numeric"])
            .build();

        key_btn.set_child(Some(&key_badge));
        row.add_suffix(&key_btn);
        edit_group.add(&row);

        filterable_rows
            .borrow_mut()
            .push((desc.to_lowercase(), row.clone()));
        registered_labels
            .borrow_mut()
            .push((action, key_badge.clone()));

        let canvas_edit = canvas.clone();
        let badge_clone = key_badge.clone();
        let win_clone = window.clone();
        let desc_clone = desc.clone();
        let trigger_edit = move || {
            show_shortcut_editor_dialog(
                &win_clone,
                canvas_edit.clone(),
                action,
                &desc_clone,
                badge_clone.clone(),
            );
        };

        let tr_btn = trigger_edit.clone();
        key_btn.connect_clicked(move |_| {
            tr_btn();
        });

        let tr_row = trigger_edit.clone();
        row.connect_activated(move |_| {
            tr_row();
        });
    }

    // View Group
    let view_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("View and Zoom"))
        .description(crate::core::gettext("Click row to customize shortcut"))
        .build();

    let view_items = canvas.shortcuts().list_by_category(ShortcutCategory::View);
    for (action, desc, combo) in view_items {
        let row = adw::ActionRow::builder()
            .title(&desc)
            .activatable(true)
            .build();

        let key_btn = gtk4::Button::builder()
            .css_classes(["flat"])
            .valign(gtk4::Align::Center)
            .tooltip_text(crate::core::gettext("Edit shortcut"))
            .build();

        let key_badge = gtk4::Label::builder()
            .label(&combo.to_display_string())
            .css_classes(["dim-label", "numeric"])
            .build();

        key_btn.set_child(Some(&key_badge));
        row.add_suffix(&key_btn);
        view_group.add(&row);

        filterable_rows
            .borrow_mut()
            .push((desc.to_lowercase(), row.clone()));
        registered_labels
            .borrow_mut()
            .push((action, key_badge.clone()));

        let canvas_edit = canvas.clone();
        let badge_clone = key_badge.clone();
        let win_clone = window.clone();
        let desc_clone = desc.clone();
        let trigger_edit = move || {
            show_shortcut_editor_dialog(
                &win_clone,
                canvas_edit.clone(),
                action,
                &desc_clone,
                badge_clone.clone(),
            );
        };

        let tr_btn = trigger_edit.clone();
        key_btn.connect_clicked(move |_| {
            tr_btn();
        });

        let tr_row = trigger_edit.clone();
        row.connect_activated(move |_| {
            tr_row();
        });
    }

    // Connect Search filtering
    {
        let filterable = filterable_rows.clone();
        shortcut_search_entry.connect_search_changed(move |entry| {
            let query = entry.text().trim().to_lowercase();
            let rows = filterable.borrow();
            for (title, row) in &*rows {
                if query.is_empty() || title.contains(&query) {
                    row.set_visible(true);
                } else {
                    row.set_visible(false);
                }
            }
        });
    }

    // Reset Group
    let reset_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Restore Default"))
        .build();

    let reset_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Restore all shortcuts"))
        .subtitle(crate::core::gettext(
            "Reverts all keys to the default Paths scheme",
        ))
        .activatable(true)
        .build();

    let reset_btn = gtk4::Button::builder()
        .label(crate::core::gettext("Restore"))
        .valign(gtk4::Align::Center)
        .css_classes(["destructive-action"])
        .build();

    {
        let canvas_reset = canvas.clone();
        let labels_reset = registered_labels.clone();
        let do_reset = move || {
            canvas_reset.set_shortcut_preset(ShortcutPreset::Default);
            let sm = canvas_reset.shortcuts();
            for (act, lbl) in &*labels_reset.borrow() {
                if let Some(combo) = sm.get_shortcut(*act) {
                    lbl.set_label(&combo.to_display_string());
                }
            }
        };

        let do_r_btn = do_reset.clone();
        reset_btn.connect_clicked(move |_| {
            do_r_btn();
        });

        let do_r_row = do_reset.clone();
        reset_row.connect_activated(move |_| {
            do_r_row();
        });
    }

    reset_row.add_suffix(&reset_btn);
    reset_group.add(&reset_row);

    make_page(vec![
        key_theme_group,
        search_group,
        tools_group,
        edit_group,
        view_group,
        reset_group,
    ])
}

pub fn show_shortcut_editor_dialog(
    parent: &impl IsA<gtk4::Widget>,
    canvas: CanvasWidget,
    action: ShortcutAction,
    action_name: &str,
    badge_label: gtk4::Label,
) {
    let current_combo = canvas
        .shortcuts()
        .get_shortcut(action)
        .unwrap_or(KeyCombo::new(
            gtk4::gdk::Key::VoidSymbol,
            false,
            false,
            false,
        ));

    let window = adw::Window::builder()
        .title(format!(
            "{}: {}",
            crate::core::gettext("Edit Shortcut"),
            action_name
        ))
        .modal(true)
        .default_width(440)
        .default_height(290)
        .build();

    if let Some(root) = parent.root() {
        if let Some(win) = root.downcast_ref::<gtk4::Window>() {
            window.set_transient_for(Some(win));
        }
    }

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::builder()
        .title_widget(&adw::WindowTitle::new(
            &format!("{}: {}", crate::core::gettext("Shortcut"), action_name),
            "",
        ))
        .show_start_title_buttons(true)
        .show_end_title_buttons(true)
        .build();
    toolbar_view.add_top_bar(&header);

    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(16)
        .margin_start(28)
        .margin_end(28)
        .margin_top(20)
        .margin_bottom(24)
        .halign(gtk4::Align::Fill)
        .valign(gtk4::Align::Center)
        .build();

    let instruction_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Press the new key or combination on your keyboard (e.g. <b>Ctrl</b>, <b>Shift</b>, <b>Alt</b> + Key):"))
        .use_markup(true)
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .build();
    content_box.append(&instruction_lbl);

    let key_cell = Rc::new(RefCell::new(current_combo.clone()));
    let preview_badge = gtk4::Label::builder()
        .label(&current_combo.to_display_string())
        .css_classes(["title-1", "accent"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .margin_top(12)
        .margin_bottom(12)
        .build();
    content_box.append(&preview_badge);

    let status_hint = gtk4::Label::builder()
        .label(crate::core::gettext("Waiting for keys..."))
        .css_classes(["dim-label", "caption"])
        .halign(gtk4::Align::Center)
        .build();
    content_box.append(&status_hint);

    let key_controller = gtk4::EventControllerKey::new();
    let key_cell_clone = key_cell.clone();
    let preview_clone = preview_badge.clone();
    let hint_clone = status_hint.clone();
    key_controller.connect_key_pressed(move |_ctrl, keyval, _keycode, state| {
        if matches!(
            keyval,
            gtk4::gdk::Key::Control_L
                | gtk4::gdk::Key::Control_R
                | gtk4::gdk::Key::Shift_L
                | gtk4::gdk::Key::Shift_R
                | gtk4::gdk::Key::Alt_L
                | gtk4::gdk::Key::Alt_R
                | gtk4::gdk::Key::Super_L
                | gtk4::gdk::Key::Super_R
                | gtk4::gdk::Key::Meta_L
                | gtk4::gdk::Key::Meta_R
        ) {
            return glib::Propagation::Proceed;
        }

        let ctrl = state.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
        let shift = state.contains(gtk4::gdk::ModifierType::SHIFT_MASK);
        let alt = state.contains(gtk4::gdk::ModifierType::ALT_MASK);

        let new_combo = KeyCombo::new(keyval, ctrl, shift, alt);
        preview_clone.set_label(&new_combo.to_display_string());
        hint_clone.set_label(&crate::core::gettext("New key captured successfully!"));
        *key_cell_clone.borrow_mut() = new_combo;

        glib::Propagation::Stop
    });
    window.add_controller(key_controller);

    let btn_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::Center)
        .margin_top(12)
        .build();

    let cancel_btn = gtk4::Button::builder()
        .label(crate::core::gettext("Cancel"))
        .build();
    let w_close = window.clone();
    cancel_btn.connect_clicked(move |_| {
        w_close.close();
    });

    let apply_btn = gtk4::Button::builder()
        .label(crate::core::gettext("Save Shortcut"))
        .css_classes(["suggested-action"])
        .build();
    let canvas_apply = canvas.clone();
    let w_apply = window.clone();
    let key_cell_apply = key_cell.clone();
    let badge_label_clone = badge_label.clone();
    apply_btn.connect_clicked(move |_| {
        let final_combo = key_cell_apply.borrow().clone();
        canvas_apply.set_custom_shortcut(action, final_combo.clone());
        badge_label_clone.set_label(&final_combo.to_display_string());
        w_apply.close();
    });

    btn_box.append(&cancel_btn);
    btn_box.append(&apply_btn);
    content_box.append(&btn_box);

    toolbar_view.set_content(Some(&content_box));
    window.set_content(Some(&toolbar_view));
    window.present();
}
