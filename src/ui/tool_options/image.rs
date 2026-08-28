use gtk4::gio;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use super::helpers::create_resource_btn;
use crate::ui::canvas::CanvasWidget;

#[derive(Clone)]
pub struct ImageControls {
    pub image_box: gtk4::Box,
    #[allow(dead_code)]
    pub btn_choose_image: gtk4::Button,
    pub btn_reset_aspect: gtk4::Button,
    pub opacity_spin: gtk4::SpinButton,
    pub name_label: gtk4::Label,
    #[allow(dead_code)]
    pub is_syncing: Rc<Cell<bool>>,
}

pub fn build_image_controls(canvas: &CanvasWidget, is_syncing: &Rc<Cell<bool>>) -> ImageControls {
    let image_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    // 1. Choose/Replace Image Button
    let choose_content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_start(4)
        .margin_end(4)
        .build();
    let choose_icon = crate::ui::icons::make_symbolic_image("document-open-symbolic", 16);
    let choose_label = gtk4::Label::builder()
        .label(&crate::core::gettext("Choose Image..."))
        .build();
    choose_content.append(&choose_icon);
    choose_content.append(&choose_label);

    let btn_choose_image = gtk4::Button::builder()
        .child(&choose_content)
        .tooltip_text(&crate::core::gettext("Open and place image into frame (PNG, JPEG, WebP, SVG, GIF)"))
        .css_classes(["flat"])
        .focus_on_click(false)
        .build();

    let canvas_choose = canvas.clone();
    btn_choose_image.connect_clicked(move |btn| {
        let file_dialog = gtk4::FileDialog::builder()
            .title(&crate::core::gettext("Choose Image File"))
            .modal(true)
            .build();

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(&crate::core::gettext("Image Files (*.png, *.jpg, *.jpeg, *.webp, *.svg, *.gif, *.bmp)")));
        filter.add_pattern("*.png");
        filter.add_pattern("*.PNG");
        filter.add_pattern("*.jpg");
        filter.add_pattern("*.JPG");
        filter.add_pattern("*.jpeg");
        filter.add_pattern("*.JPEG");
        filter.add_pattern("*.webp");
        filter.add_pattern("*.WEBP");
        filter.add_pattern("*.svg");
        filter.add_pattern("*.SVG");
        filter.add_pattern("*.gif");
        filter.add_pattern("*.GIF");
        filter.add_pattern("*.bmp");
        filter.add_pattern("*.BMP");

        let filters = gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);
        file_dialog.set_filters(Some(&filters));

        let root_win = btn.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
        let cv = canvas_choose.clone();
        file_dialog.open(root_win.as_ref(), gio::Cancellable::NONE, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    if let Some(path_str) = path.to_str() {
                        let _ = cv.replace_selected_image(path_str);
                    }
                }
            }
        });
    });
    image_box.append(&btn_choose_image);

    // Separator
    let sep1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    image_box.append(&sep1);

    // 2. Reset Aspect Ratio Button
    let btn_reset_aspect = create_resource_btn(
        "lock-aspect-ratio-symbolic",
        &crate::core::gettext("Restore Original Image Aspect Ratio"),
    );
    let canvas_aspect = canvas.clone();
    btn_reset_aspect.connect_clicked(move |_| {
        canvas_aspect.reset_selected_image_aspect_ratio();
    });
    image_box.append(&btn_reset_aspect);

    // Separator
    let sep2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    image_box.append(&sep2);

    // 3. Opacity Control
    let op_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .valign(gtk4::Align::Center)
        .build();

    let op_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Opacity"))
        .css_classes(["dim-label", "caption"])
        .build();
    op_box.append(&op_lbl);

    let opacity_spin = gtk4::SpinButton::builder()
        .adjustment(&gtk4::Adjustment::new(100.0, 0.0, 100.0, 1.0, 10.0, 0.0))
        .climb_rate(1.0)
        .digits(0)
        .width_chars(4)
        .css_classes(["numeric"])
        .valign(gtk4::Align::Center)
        .tooltip_text(&crate::core::gettext("Image Opacity (%)"))
        .build();

    let canvas_op = canvas.clone();
    let is_syncing_op = is_syncing.clone();
    opacity_spin.connect_value_changed(move |spin| {
        if is_syncing_op.get() {
            return;
        }
        let val = spin.value() as f32 / 100.0;
        let mut modified = false;
        {
            let state_rc = canvas_op.state();
            let mut state = state_rc.borrow_mut();
            let sel = state.document.selected_ids.clone();
            if !sel.is_empty() {
                state.document.snapshot();
                for &id in &sel {
                    if let Some(crate::core::Element::Image(img)) = state.document.elements.iter_mut().find(|e| e.id() == id) {
                        img.opacity = val.clamp(0.0, 1.0);
                        modified = true;
                    }
                }
                if modified {
                    state.mark_dirty();
                }
            }
        }
        if modified {
            canvas_op.notify_status();
            canvas_op.queue_draw();
        }
    });
    op_box.append(&opacity_spin);

    let op_pct = gtk4::Label::builder()
        .label("%")
        .css_classes(["dim-label", "caption"])
        .build();
    op_box.append(&op_pct);
    image_box.append(&op_box);

    // 4. File name / info label
    let name_label = gtk4::Label::builder()
        .css_classes(["dim-label", "caption"])
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .max_width_chars(18)
        .margin_start(4)
        .margin_end(4)
        .build();
    image_box.append(&name_label);

    ImageControls {
        image_box,
        btn_choose_image,
        btn_reset_aspect,
        opacity_spin,
        name_label,
        is_syncing: is_syncing.clone(),
    }
}
