use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::rc::Rc;

use crate::core::eval_math_expression;
use crate::ui::canvas::CanvasWidget;

pub struct TransformSection {
    pub container: gtk4::Box,
    pub x_entry: gtk4::Entry,
    pub y_entry: gtk4::Entry,
    pub w_entry: gtk4::Entry,
    pub h_entry: gtk4::Entry,
    pub unit_dd: gtk4::DropDown,
    pub convert_path_row: adw::ActionRow,
}

fn attach_numeric_evaluator<F: Fn(f32) + 'static>(
    entry: &gtk4::Entry,
    canvas: &CanvasWidget,
    get_base_val: impl Fn(&CanvasWidget) -> Option<f32> + 'static,
    apply_val: F,
) {
    let apply_rc = Rc::new(apply_val);
    let get_base_rc = Rc::new(get_base_val);

    // On Enter pressed:
    {
        let c = canvas.clone();
        let app = apply_rc.clone();
        let gb = get_base_rc.clone();
        entry.connect_activate(move |e| {
            let text = e.text();
            let unit = c.unit();
            let base = gb(&c);
            if let Ok(px_val) = eval_math_expression(&text, unit, base) {
                app(px_val);
                e.set_text(&unit.format(px_val));
            }
        });
    }

    // On Focus Lost (user clicked outside):
    {
        let c = canvas.clone();
        let app = apply_rc.clone();
        let gb = get_base_rc.clone();
        let entry_weak = entry.downgrade();
        let focus_ctrl = gtk4::EventControllerFocus::new();
        focus_ctrl.connect_leave(move |_| {
            if let Some(e) = entry_weak.upgrade() {
                let text = e.text();
                let unit = c.unit();
                let base = gb(&c);
                if let Ok(px_val) = eval_math_expression(&text, unit, base) {
                    app(px_val);
                    e.set_text(&unit.format(px_val));
                }
            }
        });
        entry.add_controller(focus_ctrl);
    }
}

pub fn build_transform_section(canvas: &CanvasWidget) -> TransformSection {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(0)
        .build();

    let transform_page = adw::PreferencesPage::builder().build();

    // ── 1. POSIÇÃO E DIMENSÕES ──
    let geom_group = adw::PreferencesGroup::new();
    let geom_title_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Position and Dimensions"))
        .build();
    geom_title_row.add_prefix(&gtk4::Image::from_icon_name("view-grid-symbolic"));

    // Document Unit Switcher DropDown in Transform Header
    let unit_strings: Vec<&str> = crate::core::Unit::ALL
        .iter()
        .map(|u| u.display_name())
        .collect();
    let unit_list = gtk4::StringList::new(&unit_strings);
    let unit_dd = gtk4::DropDown::builder()
        .model(&unit_list)
        .selected(canvas.unit().to_index())
        .tooltip_text(crate::core::gettext("Document Unit"))
        .valign(gtk4::Align::Center)
        .build();

    {
        let c = canvas.clone();
        unit_dd.connect_selected_notify(move |dd| {
            let unit = crate::core::Unit::from_index(dd.selected());
            if c.unit() != unit {
                c.set_unit(unit);
            }
        });
    }
    geom_title_row.add_suffix(&unit_dd);
    geom_group.add(&geom_title_row);

    let geom_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    // Row for X and Y
    let xy_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .build();

    let x_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("X:"))
        .css_classes(["dim-label"])
        .build();
    let x_entry = gtk4::Entry::builder()
        .placeholder_text("—")
        .width_chars(6)
        .css_classes(["numeric"])
        .hexpand(true)
        .build();

    attach_numeric_evaluator(
        &x_entry,
        canvas,
        |c| {
            c.state()
                .try_borrow()
                .ok()
                .and_then(|s| s.document.selection_bounds())
                .map(|b| b.x)
        },
        {
            let c = canvas.clone();
            move |val| c.set_selected_x(val)
        },
    );

    xy_row.append(&x_lbl);
    xy_row.append(&x_entry);

    let y_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Y:"))
        .css_classes(["dim-label"])
        .build();
    let y_entry = gtk4::Entry::builder()
        .placeholder_text("—")
        .width_chars(6)
        .css_classes(["numeric"])
        .hexpand(true)
        .build();

    attach_numeric_evaluator(
        &y_entry,
        canvas,
        |c| {
            c.state()
                .try_borrow()
                .ok()
                .and_then(|s| s.document.selection_bounds())
                .map(|b| b.y)
        },
        {
            let c = canvas.clone();
            move |val| c.set_selected_y(val)
        },
    );

    xy_row.append(&y_lbl);
    xy_row.append(&y_entry);

    geom_box.append(&xy_row);

    // Row for W and H + Lock
    let wh_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .build();

    let w_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("W:"))
        .css_classes(["dim-label"])
        .build();
    let w_entry = gtk4::Entry::builder()
        .placeholder_text("—")
        .width_chars(6)
        .css_classes(["numeric"])
        .hexpand(true)
        .build();

    let lock_btn = gtk4::ToggleButton::builder()
        .icon_name("changes-prevent-symbolic")
        .tooltip_text(crate::core::gettext("Lock aspect ratio"))
        .css_classes(["flat"])
        .build();

    {
        let c_clone = canvas.clone();
        let lock_clone = lock_btn.clone();
        attach_numeric_evaluator(
            &w_entry,
            canvas,
            |c| {
                c.state()
                    .try_borrow()
                    .ok()
                    .and_then(|s| s.document.selection_bounds())
                    .map(|b| b.width)
            },
            move |val| {
                c_clone.set_selected_width(val, lock_clone.is_active());
            },
        );
    }

    wh_row.append(&w_lbl);
    wh_row.append(&w_entry);
    wh_row.append(&lock_btn);

    let h_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("H:"))
        .css_classes(["dim-label"])
        .build();
    let h_entry = gtk4::Entry::builder()
        .placeholder_text("—")
        .width_chars(6)
        .css_classes(["numeric"])
        .hexpand(true)
        .build();

    {
        let c_clone = canvas.clone();
        let lock_clone = lock_btn.clone();
        attach_numeric_evaluator(
            &h_entry,
            canvas,
            |c| {
                c.state()
                    .try_borrow()
                    .ok()
                    .and_then(|s| s.document.selection_bounds())
                    .map(|b| b.height)
            },
            move |val| {
                c_clone.set_selected_height(val, lock_clone.is_active());
            },
        );
    }

    wh_row.append(&h_lbl);
    wh_row.append(&h_entry);

    geom_box.append(&wh_row);

    let geom_row = adw::ActionRow::builder().build();
    geom_row.set_child(Some(&geom_box));
    geom_group.add(&geom_row);
    transform_page.add(&geom_group);

    // ── 2. ESPELHAR E CAMADAS ──
    let trans_ops_group = adw::PreferencesGroup::new();
    let trans_ops_title = adw::ActionRow::builder()
        .title(crate::core::gettext("Mirror and Layers"))
        .build();
    trans_ops_title.add_prefix(&gtk4::Image::from_icon_name(
        "object-flip-horizontal-symbolic",
    ));
    trans_ops_group.add(&trans_ops_title);

    let ops_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(4)
        .halign(gtk4::Align::Center)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    // Flip H & V
    let flip_h_btn = gtk4::Button::builder()
        .icon_name("object-flip-horizontal-symbolic")
        .tooltip_text(crate::core::gettext("Flip Horizontal"))
        .css_classes(["flat"])
        .build();
    {
        let c = canvas.clone();
        flip_h_btn.connect_clicked(move |_| c.flip_horizontal());
    }
    ops_box.append(&flip_h_btn);

    let flip_v_btn = gtk4::Button::builder()
        .icon_name("object-flip-vertical-symbolic")
        .tooltip_text(crate::core::gettext("Flip Vertical"))
        .css_classes(["flat"])
        .build();
    {
        let c = canvas.clone();
        flip_v_btn.connect_clicked(move |_| c.flip_vertical());
    }
    ops_box.append(&flip_v_btn);

    let sep2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(4)
        .margin_end(4)
        .build();
    ops_box.append(&sep2);

    // Layer ordering
    let front_icon = crate::ui::icons::make_symbolic_image("layer-bring-to-front", 16);
    let front_btn = gtk4::Button::builder()
        .child(&front_icon)
        .tooltip_text(crate::core::gettext("Bring to Front"))
        .css_classes(["flat"])
        .build();
    {
        let c = canvas.clone();
        front_btn.connect_clicked(move |_| c.bring_to_front());
    }
    ops_box.append(&front_btn);

    let fwd_icon = crate::ui::icons::make_symbolic_image("layer-bring-forward", 16);
    let fwd_btn = gtk4::Button::builder()
        .child(&fwd_icon)
        .tooltip_text(crate::core::gettext("Bring Forward"))
        .css_classes(["flat"])
        .build();
    {
        let c = canvas.clone();
        fwd_btn.connect_clicked(move |_| c.bring_forward());
    }
    ops_box.append(&fwd_btn);

    let back_icon = crate::ui::icons::make_symbolic_image("layer-send-backward", 16);
    let back_btn = gtk4::Button::builder()
        .child(&back_icon)
        .tooltip_text(crate::core::gettext("Send Backward"))
        .css_classes(["flat"])
        .build();
    {
        let c = canvas.clone();
        back_btn.connect_clicked(move |_| c.send_backward());
    }
    ops_box.append(&back_btn);

    let bottom_icon = crate::ui::icons::make_symbolic_image("layer-send-to-back", 16);
    let bottom_btn = gtk4::Button::builder()
        .child(&bottom_icon)
        .tooltip_text(crate::core::gettext("Send to Back"))
        .css_classes(["flat"])
        .build();
    {
        let c = canvas.clone();
        bottom_btn.connect_clicked(move |_| c.send_to_back());
    }
    ops_box.append(&bottom_btn);

    let ops_row = adw::ActionRow::builder().build();
    ops_row.set_child(Some(&ops_box));
    trans_ops_group.add(&ops_row);

    // Convert to Path Row
    let convert_path_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Convert to Path"))
        .subtitle(crate::core::gettext(
            "Object to nodes and Bézier curves (Ctrl+Shift+C)",
        ))
        .activatable(true)
        .build();
    convert_path_row.add_prefix(&crate::ui::icons::make_symbolic_image(
        "object-to-path",
        16,
    ));
    let convert_btn = gtk4::Button::builder()
        .icon_name("go-next-symbolic")
        .valign(gtk4::Align::Center)
        .css_classes(["flat"])
        .tooltip_text(crate::core::gettext("Convert (Ctrl+Shift+C)"))
        .build();
    {
        let c = canvas.clone();
        convert_btn.connect_clicked(move |_| c.convert_selected_to_path());
    }
    {
        let c = canvas.clone();
        convert_path_row.connect_activated(move |_| c.convert_selected_to_path());
    }
    convert_path_row.add_suffix(&convert_btn);
    trans_ops_group.add(&convert_path_row);

    transform_page.add(&trans_ops_group);
    container.append(&transform_page);

    TransformSection {
        container,
        x_entry,
        y_entry,
        w_entry,
        h_entry,
        unit_dd,
        convert_path_row,
    }
}
