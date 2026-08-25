use gtk4::prelude::*;
use libadwaita as adw;

use crate::ui::canvas::CanvasWidget;

pub struct AlignmentSection {
    pub container: gtk4::Box,
    pub align_box: gtk4::Box,
}

pub fn build_alignment_section(canvas: &CanvasWidget) -> AlignmentSection {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .build();

    let align_stack = adw::ViewStack::builder()
        .vhomogeneous(false)
        .hhomogeneous(false)
        .valign(gtk4::Align::Start)
        .build();

    let align_switcher = adw::ViewSwitcher::builder()
        .stack(&align_stack)
        .policy(adw::ViewSwitcherPolicy::Narrow)
        .margin_top(8)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
        .halign(gtk4::Align::Fill)
        .build();

    container.append(&align_switcher);

    // Sub-page 1: Align (Card Organizar)
    let align_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(6)
        .css_classes(["card"])
        .valign(gtk4::Align::Start)
        .sensitive(false)
        .build();

    // Arrange Header: [Icon Organizar] ────── [Último Selecionado ▾] [⧉]
    let arrange_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_top(8)
        .margin_start(10)
        .margin_end(10)
        .build();

    let arrange_img = crate::ui::icons::make_symbolic_image("align-left", 16);
    let arrange_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Arrange"))
        .css_classes(["heading", "caption"])
        .build();

    arrange_header.append(&arrange_img);
    arrange_header.append(&arrange_lbl);

    let arrange_spacer = gtk4::Box::builder().hexpand(true).build();
    arrange_header.append(&arrange_spacer);

    let align_relative_model = gtk4::StringList::new(&[
        &crate::core::gettext("Last Selected"),
        &crate::core::gettext("First Selected"),
        &crate::core::gettext("Biggest Object"),
        &crate::core::gettext("Smallest Object"),
        &crate::core::gettext("Artboard"),
    ]);
    let align_relative_dd = gtk4::DropDown::builder()
        .model(&align_relative_model)
        .selected(0)
        .css_classes(["flat"])
        .valign(gtk4::Align::Center)
        .build();
    arrange_header.append(&align_relative_dd);

    align_box.append(&arrange_header);

    // Arrange Buttons: [Align 6 buttons capsule]  [Distribute 2 buttons capsule]
    let arrange_btns_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(10)
        .build();

    // 1. Align Capsule (6 buttons)
    let align_capsule = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(1)
        .css_classes(["align-capsule", "card"])
        .build();

    let align_btns: Vec<(&str, String, Box<dyn Fn(&CanvasWidget)>)> = vec![
        (
            "/io/gitlab/lewisHeart/GnomePaths/icons/align-left.svg",
            crate::core::gettext("Align left"),
            Box::new(|c| c.align_left()),
        ),
        (
            "/io/gitlab/lewisHeart/GnomePaths/icons/align-center-h.svg",
            crate::core::gettext("Center horizontally"),
            Box::new(|c| c.align_center_h()),
        ),
        (
            "/io/gitlab/lewisHeart/GnomePaths/icons/align-right.svg",
            crate::core::gettext("Align right"),
            Box::new(|c| c.align_right()),
        ),
        (
            "/io/gitlab/lewisHeart/GnomePaths/icons/align-top.svg",
            crate::core::gettext("Align top"),
            Box::new(|c| c.align_top()),
        ),
        (
            "/io/gitlab/lewisHeart/GnomePaths/icons/align-center-v.svg",
            crate::core::gettext("Center vertically"),
            Box::new(|c| c.align_center_v()),
        ),
        (
            "/io/gitlab/lewisHeart/GnomePaths/icons/align-bottom.svg",
            crate::core::gettext("Align bottom"),
            Box::new(|c| c.align_bottom()),
        ),
    ];
    for (res, tip, action) in align_btns {
        let img = crate::ui::icons::make_symbolic_image(res, 15);
        let btn = gtk4::Button::builder()
            .child(&img)
            .tooltip_text(tip)
            .css_classes(["flat"])
            .focus_on_click(false)
            .build();
        let c = canvas.clone();
        btn.connect_clicked(move |_| action(&c));
        align_capsule.append(&btn);
    }
    arrange_btns_row.append(&align_capsule);

    // 2. Distribute Capsule (2 buttons)
    let dist_capsule = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(1)
        .css_classes(["align-capsule", "card"])
        .build();

    let dist_btns: Vec<(&str, String, Box<dyn Fn(&CanvasWidget)>)> = vec![
        (
            "/io/gitlab/lewisHeart/GnomePaths/icons/distribute-horizontal.svg",
            crate::core::gettext("Distribute horizontally"),
            Box::new(|c| c.distribute_h()),
        ),
        (
            "/io/gitlab/lewisHeart/GnomePaths/icons/distribute-vertical.svg",
            crate::core::gettext("Distribute vertically"),
            Box::new(|c| c.distribute_v()),
        ),
    ];
    for (res, tip, action) in dist_btns {
        let img = crate::ui::icons::make_symbolic_image(res, 15);
        let btn = gtk4::Button::builder()
            .child(&img)
            .tooltip_text(tip)
            .css_classes(["flat"])
            .focus_on_click(false)
            .build();
        let c = canvas.clone();
        btn.connect_clicked(move |_| action(&c));
        dist_capsule.append(&btn);
    }
    arrange_btns_row.append(&dist_capsule);

    align_box.append(&arrange_btns_row);
    align_stack.add_titled_with_icon(
        &align_box,
        Some("align"),
        &crate::core::gettext("Alignment"),
        "text-align-left-symbolic",
    );

    // Sub-page 2: Multi-Object Grid Arrangement
    let grid_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(6)
        .css_classes(["card"])
        .valign(gtk4::Align::Start)
        .build();

    let grid_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_top(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let grid_icon = gtk4::Image::from_icon_name("view-grid-symbolic");
    grid_icon.set_pixel_size(16);
    grid_header.append(&grid_icon);
    grid_header.append(
        &gtk4::Label::builder()
            .label(crate::core::gettext("Align in Grid"))
            .css_classes(["heading", "caption"])
            .build(),
    );
    grid_box.append(&grid_header);

    let grid_grid = gtk4::Grid::builder()
        .column_spacing(12)
        .row_spacing(6)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(4)
        .build();

    // 1. Linhas (Rows)
    let rows_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Rows:"))
        .css_classes(["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    let rows_spin = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
    rows_spin.set_value(2.0);
    rows_spin.set_hexpand(true);
    grid_grid.attach(&rows_lbl, 0, 0, 1, 1);
    grid_grid.attach(&rows_spin, 1, 0, 1, 1);

    // 2. Colunas (Cols)
    let cols_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Columns:"))
        .css_classes(["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    let cols_spin = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
    cols_spin.set_value(2.0);
    cols_spin.set_hexpand(true);
    grid_grid.attach(&cols_lbl, 0, 1, 1, 1);
    grid_grid.attach(&cols_spin, 1, 1, 1, 1);

    // 3. Espaçamento X (Horizontal Gap)
    let gap_x_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("X Spacing (px):"))
        .css_classes(["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    let gap_x_spin = gtk4::SpinButton::with_range(0.0, 500.0, 4.0);
    gap_x_spin.set_value(16.0);
    gap_x_spin.set_hexpand(true);
    grid_grid.attach(&gap_x_lbl, 0, 2, 1, 1);
    grid_grid.attach(&gap_x_spin, 1, 2, 1, 1);

    // 4. Espaçamento Y (Vertical Gap)
    let gap_y_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Y Spacing (px):"))
        .css_classes(["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    let gap_y_spin = gtk4::SpinButton::with_range(0.0, 500.0, 4.0);
    gap_y_spin.set_value(16.0);
    gap_y_spin.set_hexpand(true);
    grid_grid.attach(&gap_y_lbl, 0, 3, 1, 1);
    grid_grid.attach(&gap_y_spin, 1, 3, 1, 1);

    grid_box.append(&grid_grid);

    let apply_grid_btn = gtk4::Button::builder()
        .label(crate::core::gettext("Arrange in Grid"))
        .icon_name("view-grid-symbolic")
        .css_classes(["suggested-action"])
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(10)
        .build();
    let canvas_grid_apply = canvas.clone();
    let rows_spin_clone = rows_spin.clone();
    let cols_spin_clone = cols_spin.clone();
    let gap_x_clone = gap_x_spin.clone();
    let gap_y_clone = gap_y_spin.clone();
    apply_grid_btn.connect_clicked(move |_| {
        let r = rows_spin_clone.value() as usize;
        let c = cols_spin_clone.value() as usize;
        let gx = gap_x_clone.value() as f32;
        let gy = gap_y_clone.value() as f32;
        canvas_grid_apply.arrange_selected_in_grid(r, c, gx, gy);
    });
    grid_box.append(&apply_grid_btn);

    align_stack.add_titled_with_icon(
        &grid_box,
        Some("grid"),
        &crate::core::gettext("Grid"),
        "view-grid-symbolic",
    );

    // Sub-page 3: Circular
    let circ_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(6)
        .css_classes(["card"])
        .valign(gtk4::Align::Start)
        .build();
    let circ_header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .margin_top(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let circ_icon = gtk4::Image::from_icon_name("align-artboard-symbolic");
    circ_icon.set_pixel_size(16);
    circ_header.append(&circ_icon);
    circ_header.append(
        &gtk4::Label::builder()
            .label(crate::core::gettext("Align in Circle"))
            .css_classes(["heading", "caption"])
            .build(),
    );
    circ_box.append(&circ_header);

    let circ_grid = gtk4::Grid::builder()
        .column_spacing(12)
        .row_spacing(6)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(4)
        .build();

    let circ_rad_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Radius (px):"))
        .css_classes(["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    let circ_rad_spin = gtk4::SpinButton::with_range(10.0, 2000.0, 10.0);
    circ_rad_spin.set_value(120.0);
    circ_rad_spin.set_hexpand(true);

    let circ_ang_lbl = gtk4::Label::builder()
        .label(crate::core::gettext("Start Angle:"))
        .css_classes(["dim-label"])
        .halign(gtk4::Align::Start)
        .build();
    let circ_ang_spin = gtk4::SpinButton::with_range(0.0, 360.0, 15.0);
    circ_ang_spin.set_value(0.0);
    circ_ang_spin.set_hexpand(true);

    circ_grid.attach(&circ_rad_lbl, 0, 0, 1, 1);
    circ_grid.attach(&circ_rad_spin, 1, 0, 1, 1);
    circ_grid.attach(&circ_ang_lbl, 0, 1, 1, 1);
    circ_grid.attach(&circ_ang_spin, 1, 1, 1, 1);

    circ_box.append(&circ_grid);

    let apply_circ_btn = gtk4::Button::builder()
        .label(crate::core::gettext("Arrange in Circle"))
        .icon_name("align-artboard-symbolic")
        .css_classes(["suggested-action"])
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(10)
        .build();
    let canvas_circ_apply = canvas.clone();
    let rad_clone = circ_rad_spin.clone();
    let ang_clone = circ_ang_spin.clone();
    apply_circ_btn.connect_clicked(move |_| {
        let rad = rad_clone.value() as f32;
        let ang = ang_clone.value() as f32;
        canvas_circ_apply.arrange_selected_circular(rad, ang);
    });
    circ_box.append(&apply_circ_btn);

    align_stack.add_titled_with_icon(
        &circ_box,
        Some("circular"),
        &crate::core::gettext("Circular"),
        "align-artboard-symbolic",
    );

    container.append(&align_stack);

    AlignmentSection {
        container,
        align_box,
    }
}
