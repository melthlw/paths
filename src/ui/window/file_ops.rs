use gtk4::gio;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::ExportFormat;
use crate::ui::canvas::CanvasWidget;

#[derive(Clone)]
pub struct WindowFileOps {
    pub perform_save: Rc<dyn Fn()>,
    pub perform_save_as: Rc<dyn Fn()>,
    pub perform_open: Rc<dyn Fn()>,
    pub perform_new_doc: Rc<dyn Fn()>,
    pub perform_import: Rc<dyn Fn()>,
    pub perform_quick_export: Rc<dyn Fn(ExportFormat)>,
}

impl WindowFileOps {
    pub fn new(
        canvas: CanvasWidget,
        main_win_holder: Rc<RefCell<Option<adw::ApplicationWindow>>>,
        toast_overlay: adw::ToastOverlay,
    ) -> Self {
        let perform_save_as = {
            let canvas = canvas.clone();
            let main_win_holder = main_win_holder.clone();
            let toast_ov = toast_overlay.clone();
            Rc::new(move || {
                let file_dialog = gtk4::FileDialog::builder()
                    .title(&crate::core::gettext("Save As..."))
                    .modal(true)
                    .initial_name(
                        canvas
                            .current_file_path()
                            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                            .unwrap_or_else(|| "design.svg".to_string()),
                    )
                    .build();

                let filter_svg = gtk4::FileFilter::new();
                filter_svg.set_name(Some(&crate::core::gettext("SVG Vector Graphics (*.svg)")));
                filter_svg.add_pattern("*.svg");
                filter_svg.add_pattern("*.SVG");

                let filters = gio::ListStore::new::<gtk4::FileFilter>();
                filters.append(&filter_svg);
                file_dialog.set_filters(Some(&filters));

                let canvas_c = canvas.clone();
                let toast_ov_c = toast_ov.clone();
                let parent_win = main_win_holder.borrow().clone();
                file_dialog.save(parent_win.as_ref(), gio::Cancellable::NONE, move |res| {
                    if let Ok(file) = res {
                        if let Some(path) = file.path() {
                            let path_buf = if path.extension().is_none() {
                                path.with_extension("svg")
                            } else {
                                path
                            };
                            if let Err(e) = canvas_c.save_to_path(&path_buf) {
                                eprintln!("{}: {}", crate::core::gettext("Error saving: {}"), e);
                            } else {
                                let fname =
                                    path_buf.file_name().unwrap_or_default().to_string_lossy();
                                let toast = adw::Toast::new(&crate::i18n!(
                                    "Document saved to \"{}\"",
                                    fname
                                ));
                                toast.set_timeout(3);
                                toast_ov_c.add_toast(toast);
                            }
                        }
                    }
                });
            })
        };

        let perform_save = {
            let canvas = canvas.clone();
            let perform_save_as = perform_save_as.clone();
            let toast_ov = toast_overlay.clone();
            Rc::new(move || match canvas.save() {
                Ok(true) => {
                    let fname = canvas
                        .current_file_path()
                        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                        .unwrap_or_else(|| "design.svg".to_string());
                    let toast = adw::Toast::new(&crate::i18n!("Document saved to \"{}\"", fname));
                    toast.set_timeout(3);
                    toast_ov.add_toast(toast);
                }
                Ok(false) => {
                    perform_save_as();
                }
                Err(e) => {
                    eprintln!("{}", crate::i18n!("Error saving: {}", e));
                }
            })
        };

        let perform_open = {
            let canvas = canvas.clone();
            let main_win_holder = main_win_holder.clone();
            let toast_ov = toast_overlay.clone();
            Rc::new(move || {
                let cv = canvas.clone();
                let cv_for_cb = canvas.clone();
                let main_win = main_win_holder.clone();
                let tov = toast_ov.clone();
                let parent_win = main_win_holder.borrow().clone();
                prompt_save_changes(parent_win.as_ref(), &cv, move || {
                    let file_dialog = gtk4::FileDialog::builder()
                        .title(&crate::core::gettext("Open Document"))
                        .modal(true)
                        .build();

                    let filter_all = gtk4::FileFilter::new();
                    filter_all.set_name(Some(&crate::core::gettext(
                        "Supported Formats (*.svg, *.png, *.jpg, *.jpeg, *.webp)",
                    )));
                    filter_all.add_pattern("*.svg");
                    filter_all.add_pattern("*.SVG");
                    filter_all.add_pattern("*.png");
                    filter_all.add_pattern("*.PNG");
                    filter_all.add_pattern("*.jpg");
                    filter_all.add_pattern("*.JPG");
                    filter_all.add_pattern("*.jpeg");
                    filter_all.add_pattern("*.JPEG");
                    filter_all.add_pattern("*.webp");
                    filter_all.add_pattern("*.WEBP");

                    let filters = gio::ListStore::new::<gtk4::FileFilter>();
                    filters.append(&filter_all);
                    file_dialog.set_filters(Some(&filters));

                    let cv_open = cv_for_cb.clone();
                    let tov_open = tov.clone();
                    let pwin = main_win.borrow().clone();
                    file_dialog.open(pwin.as_ref(), gio::Cancellable::NONE, move |res| {
                        if let Ok(file) = res {
                            if let Some(path) = file.path() {
                                if let Err(e) = cv_open.open_from_path(&path) {
                                    eprintln!("{}", crate::i18n!("Error opening file: {}", e));
                                } else {
                                    let fname =
                                        path.file_name().unwrap_or_default().to_string_lossy();
                                    let toast = adw::Toast::new(&crate::i18n!(
                                        "Document \"{}\" opened",
                                        fname
                                    ));
                                    toast.set_timeout(3);
                                    tov_open.add_toast(toast);
                                }
                            }
                        }
                    });
                });
            })
        };

        let perform_new_doc = {
            let canvas = canvas.clone();
            let main_win_holder = main_win_holder.clone();
            let toast_ov = toast_overlay.clone();
            Rc::new(move || {
                let cv = canvas.clone();
                let cv_for_cb = canvas.clone();
                let tov = toast_ov.clone();
                let parent_win = main_win_holder.borrow().clone();
                prompt_save_changes(parent_win.as_ref(), &cv, move || {
                    cv_for_cb.new_document();
                    let toast = adw::Toast::new(&crate::core::gettext("New document created"));
                    toast.set_timeout(2);
                    tov.add_toast(toast);
                });
            })
        };

        let perform_import = {
            let canvas = canvas.clone();
            let main_win_holder = main_win_holder.clone();
            Rc::new(move || {
                let file_dialog = gtk4::FileDialog::builder()
                    .title(&crate::core::gettext("Import Image or Vector"))
                    .modal(true)
                    .build();

                let filter_all = gtk4::FileFilter::new();
                filter_all.set_name(Some(&crate::core::gettext(
                    "Image Formats (*.png, *.jpg, *.jpeg, *.webp, *.svg, *.pdf)",
                )));
                filter_all.add_pattern("*.png");
                filter_all.add_pattern("*.PNG");
                filter_all.add_pattern("*.jpg");
                filter_all.add_pattern("*.JPG");
                filter_all.add_pattern("*.jpeg");
                filter_all.add_pattern("*.JPEG");
                filter_all.add_pattern("*.webp");
                filter_all.add_pattern("*.WEBP");
                filter_all.add_pattern("*.svg");
                filter_all.add_pattern("*.SVG");
                filter_all.add_pattern("*.pdf");
                filter_all.add_pattern("*.PDF");

                let filters = gio::ListStore::new::<gtk4::FileFilter>();
                filters.append(&filter_all);
                file_dialog.set_filters(Some(&filters));

                let canvas_c = canvas.clone();
                let parent_win = main_win_holder.borrow().clone();
                file_dialog.open(parent_win.as_ref(), gio::Cancellable::NONE, move |res| {
                    if let Ok(file) = res {
                        if let Some(path) = file.path() {
                            if let Some(path_str) = path.to_str() {
                                let _ = canvas_c.import_file(path_str);
                            }
                        }
                    }
                });
            })
        };

        let perform_quick_export = {
            let canvas = canvas.clone();
            let main_win_holder = main_win_holder.clone();
            Rc::new(move |fmt: ExportFormat| {
                let mut cfg = crate::core::ExportConfig::default();
                cfg.format = fmt;
                let state_doc = canvas
                    .state()
                    .try_borrow()
                    .map(|s| s.document.clone())
                    .unwrap_or_else(|_| crate::core::Document::new());
                let file_dialog = gtk4::FileDialog::builder()
                    .title(&crate::core::gettext("Quick Export"))
                    .modal(true)
                    .initial_name(format!("design.{}", fmt.extension()))
                    .build();
                let canvas_c = canvas.clone();
                let parent_win = main_win_holder.borrow().clone();
                file_dialog.save(parent_win.as_ref(), gio::Cancellable::NONE, move |res| {
                    if let Ok(file) = res {
                        if let Some(file_path) = file.path() {
                            let exported = crate::core::export_document(&state_doc, &cfg);
                            if let Some((_, data)) = exported.into_iter().next() {
                                let _ = std::fs::write(file_path, data);
                                canvas_c.notify_status();
                            }
                        }
                    }
                });
            })
        };

        Self {
            perform_save,
            perform_save_as,
            perform_open,
            perform_new_doc,
            perform_import,
            perform_quick_export,
        }
    }
}

pub fn prompt_save_changes(
    parent_window: Option<&adw::ApplicationWindow>,
    canvas: &CanvasWidget,
    on_proceed: impl Fn() + 'static,
) {
    if !canvas.has_unsaved_changes() {
        on_proceed();
        return;
    }

    let dialog = adw::AlertDialog::builder()
        .heading(&crate::core::gettext("Save Changes?"))
        .body(&crate::core::gettext(
            "Open document contains unsaved changes. If you don't save, changes will be permanently lost.",
        ))
        .build();

    dialog.add_response("cancel", &crate::core::gettext("Cancel"));
    dialog.add_response("discard", &crate::core::gettext("Discard Changes"));
    dialog.add_response("save", &crate::core::gettext("Save"));

    dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("save"));
    dialog.set_close_response("cancel");

    let on_proc = Rc::new(on_proceed);
    let canvas_c = canvas.clone();
    let on_proc_c = on_proc.clone();
    let parent_win_c = parent_window.cloned();

    dialog.choose(parent_window, gio::Cancellable::NONE, move |response| {
        match response.as_str() {
            "save" => {
                if let Ok(true) = canvas_c.save() {
                    on_proc_c();
                } else {
                    let file_dialog = gtk4::FileDialog::builder()
                        .title(&crate::core::gettext("Save As..."))
                        .modal(true)
                        .initial_name(
                            canvas_c
                                .current_file_path()
                                .and_then(|p| {
                                    p.file_name().map(|n| n.to_string_lossy().into_owned())
                                })
                                .unwrap_or_else(|| "design.svg".to_string()),
                        )
                        .build();

                    let filter_svg = gtk4::FileFilter::new();
                    filter_svg.set_name(Some(&crate::core::gettext(
                        "SVG Vector Graphics (*.svg)",
                    )));
                    filter_svg.add_pattern("*.svg");
                    filter_svg.add_pattern("*.SVG");

                    let filters = gio::ListStore::new::<gtk4::FileFilter>();
                    filters.append(&filter_svg);
                    file_dialog.set_filters(Some(&filters));

                    let cv = canvas_c.clone();
                    let on_p = on_proc_c.clone();
                    file_dialog.save(
                        parent_win_c.as_ref(),
                        gio::Cancellable::NONE,
                        move |res| {
                            if let Ok(file) = res {
                                if let Some(path) = file.path() {
                                    let path_buf = if path.extension().is_none() {
                                        path.with_extension("svg")
                                    } else {
                                        path
                                    };
                                    if cv.save_to_path(&path_buf).is_ok() {
                                        on_p();
                                    }
                                }
                            }
                        },
                    );
                }
            }
            "discard" => {
                on_proc_c();
            }
            _ => {}
        }
    });
}
