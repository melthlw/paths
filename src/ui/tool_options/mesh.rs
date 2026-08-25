use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use super::helpers::{create_resource_btn, create_resource_toggle_btn};
use crate::core::Color;
use crate::ui::canvas::CanvasWidget;
use crate::ui::color_picker::ColorPickerPopover;

#[derive(Clone)]
pub struct MeshControls {
    pub mesh_box: gtk4::Box,
    pub btn_smooth_curves: gtk4::ToggleButton,
    pub btn_delete_node: gtk4::Button,
    pub color_btn: gtk4::Button,
    pub color_da: gtk4::DrawingArea,
    pub node_color_cell: Rc<Cell<Color>>,
}

pub fn build_mesh_controls(canvas: &CanvasWidget, is_syncing: &Rc<Cell<bool>>) -> MeshControls {
    let mesh_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .valign(gtk4::Align::Center)
        .visible(false)
        .build();

    // 1. Split Row (+)
    let btn_split_row = create_resource_btn(
        "list-add-symbolic",
        &crate::core::gettext("Split Row (Add horizontal line across mesh)"),
    );
    let cv_sr = canvas.clone();
    btn_split_row.connect_clicked(move |_| {
        cv_sr.split_selected_mesh_row();
    });
    mesh_box.append(&btn_split_row);

    // 2. Split Col (+)
    let btn_split_col = create_resource_btn(
        "view-more-symbolic",
        &crate::core::gettext("Split Column (Add vertical line across mesh)"),
    );
    let cv_sc = canvas.clone();
    btn_split_col.connect_clicked(move |_| {
        cv_sc.split_selected_mesh_col();
    });
    mesh_box.append(&btn_split_col);

    let sep1 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    mesh_box.append(&sep1);

    // 3. Smooth Curved Mesh vs Linear Grid
    let btn_smooth_curves = create_resource_toggle_btn(
        "node-smooth-symbolic",
        &crate::core::gettext("Smooth Organic Curves (Toggle curved mesh interpolation)"),
        true,
    );
    let cv_sm = canvas.clone();
    let sync_sm = is_syncing.clone();
    btn_smooth_curves.connect_toggled(move |_| {
        if sync_sm.get() {
            return;
        }
        cv_sm.toggle_selected_mesh_smooth_curves();
    });
    mesh_box.append(&btn_smooth_curves);

    // 4. Even Spacing (Laplacian relaxation)
    let btn_even_spacing = create_resource_btn(
        "align-horizontal-center-symbolic",
        &crate::core::gettext("Relax & Distribute (Evenly space mesh lines)"),
    );
    let cv_es = canvas.clone();
    btn_even_spacing.connect_clicked(move |_| {
        cv_es.smooth_selected_mesh_spacing();
    });
    mesh_box.append(&btn_even_spacing);

    // 5. Reset Grid
    let btn_reset_grid = create_resource_btn(
        "view-refresh-symbolic",
        &crate::core::gettext("Reset Mesh Grid (Restore clean rectangular grid to shape bounds)"),
    );
    let cv_rg = canvas.clone();
    btn_reset_grid.connect_clicked(move |_| {
        cv_rg.reset_selected_mesh_to_bounds();
    });
    mesh_box.append(&btn_reset_grid);

    let sep2 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    mesh_box.append(&sep2);

    // 6. Delete Node / Row / Column
    let btn_delete_node = create_resource_btn(
        "user-trash-symbolic",
        &crate::core::gettext("Delete Selected Node / Row / Column"),
    );
    let cv_del = canvas.clone();
    btn_delete_node.connect_clicked(move |_| {
        cv_del.delete_selected_mesh_node();
    });
    mesh_box.append(&btn_delete_node);

    let sep3 = gtk4::Separator::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_start(2)
        .margin_end(2)
        .build();
    mesh_box.append(&sep3);

    // 7. Node Color Swatch Button with direct Color Picker Popover!
    let color_btn = gtk4::Button::builder()
        .css_classes(["flat", "circular"])
        .tooltip_text(&crate::core::gettext("Selected Mesh Node Color"))
        .valign(gtk4::Align::Center)
        .build();

    let node_color_cell = Rc::new(Cell::new(Color::BLACK));
    let node_col_c = node_color_cell.clone();

    let color_da = gtk4::DrawingArea::builder()
        .content_width(18)
        .content_height(18)
        .build();
    {
        let n_c = node_col_c.clone();
        color_da.set_draw_func(move |_, cr, w, h| {
            let col = n_c.get();
            cr.arc(
                w as f64 * 0.5,
                h as f64 * 0.5,
                (w.min(h) as f64 * 0.5) - 1.0,
                0.0,
                std::f64::consts::TAU,
            );
            cr.set_source_rgba(col.r as f64, col.g as f64, col.b as f64, col.a as f64);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.3);
            cr.set_line_width(1.0);
            let _ = cr.stroke();
        });
    }
    color_btn.set_child(Some(&color_da));

    {
        let cv_col = canvas.clone();
        let col_b = color_btn.clone();
        let col_cell_c = node_color_cell.clone();
        let da_c = color_da.clone();
        color_btn.connect_clicked(move |_| {
            let info = cv_col.get_active_mesh_info();
            let cur_col = info.map(|(_, _, _, c, _)| c).unwrap_or(Color::BLACK);
            let active_node = info.map(|(n, _, _, _, _)| n).unwrap_or(0);
            let cv_apply = cv_col.clone();
            let cell_upd = col_cell_c.clone();
            let da_upd = da_c.clone();
            let pop = ColorPickerPopover::with_mode_switcher(
                cv_col.clone(),
                cur_col,
                2, // mesh mode
                true,
            );
            pop.on_color_changed(move |new_c| {
                cell_upd.set(new_c);
                da_upd.queue_draw();
                cv_apply.set_mesh_node_color(active_node, new_c);
            });
            pop.attach_to(&col_b);
            pop.popup();
        });
    }

    mesh_box.append(&color_btn);

    MeshControls {
        mesh_box,
        btn_smooth_curves,
        btn_delete_node,
        color_btn,
        color_da,
        node_color_cell,
    }
}
