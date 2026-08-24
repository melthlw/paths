use gtk4::gio;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::canvas::CanvasWidget;

pub fn build_export_tab(
    canvas: &CanvasWidget,
    main_win_holder: &Rc<RefCell<Option<adw::ApplicationWindow>>>,
) -> (gtk4::Widget, Rc<dyn Fn()>) {
    let pref_page = adw::PreferencesPage::builder().build();

    let export_cfg = Rc::new(RefCell::new(crate::core::ExportConfig::default()));

    // 1. Arquivo e Formato
    let file_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("File"))
        .build();

    let name_row = adw::EntryRow::builder()
        .title(crate::core::gettext("Base Name"))
        .text("design")
        .build();

    let ext_label = gtk4::Label::builder()
        .label(".png")
        .css_classes(["dim-label"])
        .valign(gtk4::Align::Center)
        .margin_end(8)
        .build();
    name_row.add_suffix(&ext_label);
    file_group.add(&name_row);

    let format_model = gtk4::StringList::new(&["PNG", "JPG", "SVG", "PDF", "WebP"]);
    let format_dd = gtk4::DropDown::builder()
        .model(&format_model)
        .selected(0)
        .valign(gtk4::Align::Center)
        .build();

    let format_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Format"))
        .build();
    format_row.add_suffix(&format_dd);
    file_group.add(&format_row);

    pref_page.add(&file_group);

    // 2. Escopo
    let scope_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Scope"))
        .build();

    let scope_model = gtk4::StringList::new(&[
        &crate::core::gettext("Current Page"),
        &crate::core::gettext("All Pages"),
        &crate::core::gettext("Selected Pages"),
        &crate::core::gettext("Complete Document"),
        &crate::core::gettext("Selection Only"),
    ]);
    let scope_dd = gtk4::DropDown::builder()
        .model(&scope_model)
        .selected(0)
        .valign(gtk4::Align::Center)
        .build();

    let scope_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Area to Export"))
        .build();
    scope_row.add_suffix(&scope_dd);
    scope_group.add(&scope_row);

    let sep_files_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Export Separate Files"))
        .subtitle(crate::core::gettext("Create a file for each page"))
        .active(false)
        .build();
    let export_cfg_sep = export_cfg.clone();
    sep_files_row.connect_active_notify(move |s| {
        export_cfg_sep.borrow_mut().separate_files = s.is_active();
    });
    scope_group.add(&sep_files_row);

    pref_page.add(&scope_group);

    // Multi-page checklist dedicated group
    let pages_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Pages to Export"))
        .description(crate::core::gettext(
            "Select which pages to include in final file",
        ))
        .build();

    // Quick selection header buttons in linked pill capsule
    let header_btns_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["linked", "export-header-actions"])
        .valign(gtk4::Align::Center)
        .build();

    let btn_all = gtk4::Button::builder()
        .label(crate::core::gettext("All"))
        .tooltip_text(crate::core::gettext("Select all pages"))
        .css_classes(["flat", "caption"])
        .valign(gtk4::Align::Center)
        .build();

    let btn_none = gtk4::Button::builder()
        .label(crate::core::gettext("None"))
        .tooltip_text(crate::core::gettext("Deselect all pages"))
        .css_classes(["flat", "caption"])
        .valign(gtk4::Align::Center)
        .build();

    header_btns_box.append(&btn_all);
    header_btns_box.append(&btn_none);
    pages_group.set_header_suffix(Some(&header_btns_box));

    let pages_list = gtk4::ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();

    let selected_page_indices = Rc::new(RefCell::new(Vec::<usize>::new()));

    let update_pages_box: Rc<dyn Fn()> = {
        let canvas = canvas.clone();
        let pages_list = pages_list.clone();
        let selected_pages = selected_page_indices.clone();
        Rc::new(move || {
            let pages = if let Ok(state) = canvas.state().try_borrow() {
                state.document.pages.clone()
            } else {
                return;
            };

            while let Some(child) = pages_list.first_child() {
                pages_list.remove(&child);
            }
            let mut sel = selected_pages.borrow_mut();
            sel.retain(|&i| i < pages.len());
            if sel.is_empty() && !pages.is_empty() {
                sel.push(0);
            }
            for (idx, p) in pages.iter().enumerate() {
                let name = if p.name.is_empty() {
                    crate::i18n!("Page {}", idx + 1)
                } else {
                    p.name.clone()
                };
                let row = adw::ActionRow::builder()
                    .title(glib::markup_escape_text(&name))
                    .subtitle(format!("{:.0} × {:.0} px", p.rect.width, p.rect.height))
                    .build();

                let chk = gtk4::CheckButton::builder()
                    .active(sel.contains(&idx))
                    .valign(gtk4::Align::Center)
                    .build();

                let icon = gtk4::Image::from_icon_name("page-export-symbolic");
                icon.set_opacity(0.65);
                icon.set_valign(gtk4::Align::Center);

                let num_lbl = gtk4::Label::builder()
                    .label(format!("#{}", idx + 1))
                    .css_classes(["dim-label", "caption"])
                    .valign(gtk4::Align::Center)
                    .margin_end(6)
                    .build();

                row.add_prefix(&chk);
                row.add_prefix(&icon);
                row.add_suffix(&num_lbl);
                row.set_activatable_widget(Some(&chk));

                let sel_c = selected_pages.clone();
                chk.connect_toggled(move |btn| {
                    let mut s = sel_c.borrow_mut();
                    if btn.is_active() {
                        if !s.contains(&idx) {
                            s.push(idx);
                        }
                    } else {
                        s.retain(|&i| i != idx);
                    }
                });

                pages_list.append(&row);
            }
        })
    };

    // Wire "Todas" and "Nenhuma"
    {
        let canvas = canvas.clone();
        let selected_pages = selected_page_indices.clone();
        let upd = update_pages_box.clone();
        btn_all.connect_clicked(move |_| {
            let total = if let Ok(state) = canvas.state().try_borrow() {
                state.document.pages.len()
            } else {
                0
            };
            *selected_pages.borrow_mut() = (0..total).collect();
            upd();
        });
    }

    {
        let selected_pages = selected_page_indices.clone();
        let upd = update_pages_box.clone();
        btn_none.connect_clicked(move |_| {
            selected_pages.borrow_mut().clear();
            upd();
        });
    }

    pages_group.add(&pages_list);
    pages_group.set_visible(false);
    pref_page.add(&pages_group);

    // 3. Configurações e Resolução
    let settings_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Settings"))
        .build();

    let scale_model = gtk4::StringList::new(&["1x", "2x", "3x", "4x", "0.5x"]);
    let scale_dd = gtk4::DropDown::builder()
        .model(&scale_model)
        .selected(0)
        .valign(gtk4::Align::Center)
        .build();
    let scale_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Scale"))
        .build();
    scale_row.add_suffix(&scale_dd);
    settings_group.add(&scale_row);

    let dpi_model = gtk4::StringList::new(&[
        &crate::core::gettext("72 DPI (Screen / Web)"),
        &crate::core::gettext("150 DPI (Medium)"),
        &crate::core::gettext("300 DPI (Print)"),
        &crate::core::gettext("600 DPI (High Resolution)"),
    ]);
    let dpi_dd = gtk4::DropDown::builder()
        .model(&dpi_model)
        .selected(0)
        .valign(gtk4::Align::Center)
        .build();
    let dpi_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Resolution (DPI)"))
        .build();
    dpi_row.add_suffix(&dpi_dd);
    settings_group.add(&dpi_row);

    let qual_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Quality"))
        .subtitle("95%")
        .visible(false)
        .build();
    let qual_adj = gtk4::Adjustment::new(95.0, 1.0, 100.0, 1.0, 5.0, 0.0);
    let qual_scale = gtk4::Scale::builder()
        .adjustment(&qual_adj)
        .digits(0)
        .hexpand(true)
        .width_request(120)
        .valign(gtk4::Align::Center)
        .build();
    let export_cfg_q = export_cfg.clone();
    let qual_row_c = qual_row.clone();
    qual_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        export_cfg_q.borrow_mut().quality = val;
        qual_row_c.set_subtitle(&format!("{}%", val));
    });
    qual_row.add_suffix(&qual_scale);
    settings_group.add(&qual_row);

    // Fundo Transparente - Ativo por padrão (true)
    let trans_bg_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Transparent Background"))
        .subtitle(crate::core::gettext("Preserve alpha transparency"))
        .active(true)
        .visible(true)
        .build();
    let export_cfg_bg = export_cfg.clone();
    trans_bg_row.connect_active_notify(move |s| {
        export_cfg_bg.borrow_mut().transparent_background = s.is_active();
    });
    settings_group.add(&trans_bg_row);

    let aa_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Antialiasing"))
        .subtitle(crate::core::gettext("Smooth edge rendering"))
        .active(true)
        .build();
    let export_cfg_aa = export_cfg.clone();
    aa_row.connect_active_notify(move |s| {
        export_cfg_aa.borrow_mut().antialiasing = s.is_active();
    });
    settings_group.add(&aa_row);

    pref_page.add(&settings_group);

    // 4. Botão de Ação no Padrão GNOME
    let action_group = adw::PreferencesGroup::builder().build();
    let btn_export = gtk4::Button::builder()
        .label(crate::core::gettext("Export"))
        .icon_name("document-save-symbolic")
        .css_classes(["suggested-action", "pill"])
        .halign(gtk4::Align::Fill)
        .margin_top(6)
        .margin_bottom(18)
        .build();
    action_group.add(&btn_export);
    pref_page.add(&action_group);

    // Dynamic Connections
    {
        let export_cfg = export_cfg.clone();
        name_row.connect_text_notify(move |entry| {
            let t = entry.text().to_string();
            let base = if t.trim().is_empty() {
                "design".to_string()
            } else {
                t.trim().to_string()
            };
            export_cfg.borrow_mut().base_filename = base;
        });
    }

    {
        let export_cfg = export_cfg.clone();
        let ext_label = ext_label.clone();
        let qual_row_vis = qual_row.clone();
        let trans_bg_row_vis = trans_bg_row.clone();
        format_dd.connect_selected_notify(move |dd| {
            let f = match dd.selected() {
                0 => crate::core::ExportFormat::Png,
                1 => crate::core::ExportFormat::Jpg,
                2 => crate::core::ExportFormat::Svg,
                3 => crate::core::ExportFormat::Pdf,
                _ => crate::core::ExportFormat::WebP,
            };
            export_cfg.borrow_mut().format = f;
            ext_label.set_label(&format!(".{}", f.extension()));

            let is_lossy = matches!(
                f,
                crate::core::ExportFormat::Jpg | crate::core::ExportFormat::WebP
            );
            qual_row_vis.set_visible(is_lossy);

            let supports_alpha = matches!(
                f,
                crate::core::ExportFormat::Png
                    | crate::core::ExportFormat::Svg
                    | crate::core::ExportFormat::WebP
            );
            trans_bg_row_vis.set_visible(supports_alpha);
        });
    }

    {
        let export_cfg = export_cfg.clone();
        let pages_grp_c = pages_group.clone();
        let upd_pages_c = update_pages_box.clone();
        let sel_pages_c = selected_page_indices.clone();
        scope_dd.connect_selected_notify(move |dd| {
            let idx = dd.selected();
            let is_selected_pages = idx == 2;
            pages_grp_c.set_visible(is_selected_pages);
            if is_selected_pages {
                upd_pages_c();
            }
            let mut cfg = export_cfg.borrow_mut();
            cfg.scope = match idx {
                0 => crate::core::ExportScope::CurrentPage,
                1 => crate::core::ExportScope::AllPages,
                2 => crate::core::ExportScope::SelectedPages(sel_pages_c.borrow().clone()),
                3 => crate::core::ExportScope::CompleteDocument,
                _ => crate::core::ExportScope::Selection,
            };
        });
    }

    {
        let export_cfg = export_cfg.clone();
        scale_dd.connect_selected_notify(move |dd| {
            export_cfg.borrow_mut().scale = match dd.selected() {
                0 => 1.0,
                1 => 2.0,
                2 => 3.0,
                3 => 4.0,
                _ => 0.5,
            };
        });
    }

    {
        let export_cfg = export_cfg.clone();
        dpi_dd.connect_selected_notify(move |dd| {
            export_cfg.borrow_mut().dpi = match dd.selected() {
                0 => 72.0,
                1 => 150.0,
                2 => 300.0,
                _ => 600.0,
            };
        });
    }

    // Wire Export Button Click
    {
        let canvas_exp = canvas.clone();
        let export_cfg_btn = export_cfg.clone();
        let sel_pages_btn = selected_page_indices.clone();
        let main_win_btn = main_win_holder.clone();

        btn_export.connect_clicked(move |_| {
            let mut cfg = export_cfg_btn.borrow().clone();
            if let crate::core::ExportScope::SelectedPages(_) = cfg.scope {
                cfg.scope = crate::core::ExportScope::SelectedPages(sel_pages_btn.borrow().clone());
            }

            let is_multi = cfg.separate_files
                && (cfg.scope == crate::core::ExportScope::AllPages
                    || matches!(cfg.scope, crate::core::ExportScope::SelectedPages(_)));

            let state_doc = canvas_exp
                .state()
                .try_borrow()
                .map(|s| s.document.clone())
                .unwrap_or_else(|_| crate::core::Document::new());
            let parent_win = main_win_btn.borrow().clone();

            if is_multi {
                let file_dialog = gtk4::FileDialog::builder()
                    .title(crate::core::gettext("Select Folder to Export Files"))
                    .modal(true)
                    .build();
                let canvas_sub = canvas_exp.clone();
                file_dialog.select_folder(
                    parent_win.as_ref(),
                    gio::Cancellable::NONE,
                    move |res| {
                        if let Ok(folder) = res {
                            if let Some(folder_path) = folder.path() {
                                let exported = crate::core::export_document(&state_doc, &cfg);
                                for (fname, data) in exported {
                                    let out_path = folder_path.join(&fname);
                                    let _ = std::fs::write(out_path, data);
                                }
                                canvas_sub.notify_status();
                            }
                        }
                    },
                );
            } else {
                let file_dialog = gtk4::FileDialog::builder()
                    .title(crate::core::gettext("Save Exported File"))
                    .modal(true)
                    .initial_name(format!("{}.{}", cfg.base_filename, cfg.format.extension()))
                    .build();
                let canvas_sub = canvas_exp.clone();
                file_dialog.save(parent_win.as_ref(), gio::Cancellable::NONE, move |res| {
                    if let Ok(file) = res {
                        if let Some(file_path) = file.path() {
                            let exported = crate::core::export_document(&state_doc, &cfg);
                            if let Some((_, data)) = exported.into_iter().next() {
                                let _ = std::fs::write(file_path, data);
                                canvas_sub.notify_status();
                            }
                        }
                    }
                });
            }
        });
    }

    (pref_page.upcast(), update_pages_box)
}
