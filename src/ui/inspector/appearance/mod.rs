pub mod fills;
pub mod strokes;

pub use fills::FillRow;
pub use strokes::StrokeRow;

use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::core::{FillLayer, StrokeLayer};
use crate::ui::canvas::CanvasWidget;

pub fn unparent_all_popovers(w: &gtk4::Widget) {
    let mut child_opt = w.first_child();
    while let Some(child) = child_opt {
        if let Some(pop) = child.downcast_ref::<gtk4::Popover>() {
            pop.unparent();
        } else {
            unparent_all_popovers(&child);
        }
        child_opt = child.next_sibling();
    }
}

pub fn clear_box(bx: &gtk4::Box) {
    while let Some(child) = bx.first_child() {
        unparent_all_popovers(&child);
        bx.remove(&child);
    }
}

pub fn rebuild_fill_list(
    list_box: &gtk4::Box,
    fills: &Rc<RefCell<Vec<FillLayer>>>,
    canvas: &CanvasWidget,
    is_updating: &Rc<Cell<bool>>,
    fill_sep: &gtk4::Separator,
) {
    clear_box(list_box);
    let entries = fills.borrow();
    fill_sep.set_visible(!entries.is_empty());

    let fills_rc = fills.clone();
    let canvas_c = canvas.clone();
    let upd = is_updating.clone();

    let lb = list_box.clone();
    let sep_c = fill_sep.clone();
    let fills_for_rebuild = fills.clone();
    let canvas_for_rebuild = canvas.clone();
    let upd_for_rebuild = is_updating.clone();
    let rebuild_cb: Rc<dyn Fn()> = Rc::new(move || {
        rebuild_fill_list(
            &lb,
            &fills_for_rebuild,
            &canvas_for_rebuild,
            &upd_for_rebuild,
            &sep_c,
        );
    });

    for (idx, entry) in entries.iter().enumerate() {
        if idx > 0 {
            let sep = gtk4::Separator::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .margin_top(4)
                .margin_bottom(4)
                .build();
            list_box.append(&sep);
        }
        let row = FillRow::new(
            entry,
            canvas_c.clone(),
            upd.clone(),
            fills_rc.clone(),
            idx,
            rebuild_cb.clone(),
        );
        list_box.append(&row.container);
    }
}

pub fn rebuild_stroke_list(
    list_box: &gtk4::Box,
    strokes: &Rc<RefCell<Vec<StrokeLayer>>>,
    canvas: &CanvasWidget,
    is_updating: &Rc<Cell<bool>>,
    stroke_sep: &gtk4::Separator,
) {
    clear_box(list_box);
    let entries = strokes.borrow();
    stroke_sep.set_visible(!entries.is_empty());

    let strokes_rc = strokes.clone();
    let canvas_c = canvas.clone();
    let upd = is_updating.clone();

    let lb = list_box.clone();
    let sep_c = stroke_sep.clone();
    let strokes_for_rebuild = strokes.clone();
    let canvas_for_rebuild = canvas.clone();
    let upd_for_rebuild = is_updating.clone();
    let rebuild_cb: Rc<dyn Fn()> = Rc::new(move || {
        rebuild_stroke_list(
            &lb,
            &strokes_for_rebuild,
            &canvas_for_rebuild,
            &upd_for_rebuild,
            &sep_c,
        );
    });

    for (idx, entry) in entries.iter().enumerate() {
        if idx > 0 {
            let sep = gtk4::Separator::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .margin_top(6)
                .margin_bottom(6)
                .build();
            list_box.append(&sep);
        }
        let row = StrokeRow::new(
            entry,
            canvas_c.clone(),
            upd.clone(),
            strokes_rc.clone(),
            idx,
            rebuild_cb.clone(),
        );
        list_box.append(&row.container);
    }
}
