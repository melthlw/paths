use libadwaita as adw;
use libadwaita::prelude::*;

use super::widgets::{make_color_swatches, make_page, make_segmented_capsule};
use crate::ui::canvas::CanvasWidget;

pub fn build_canvas_page(canvas: &CanvasWidget) -> gtk4::ScrolledWindow {
    let canvas_bg_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Workspace Background"))
        .description(crate::core::gettext(
            "Customize workspace canvas background color and pattern",
        ))
        .build();

    let cur_bg_hex = canvas
        .canvas_bg_color()
        .map(|c| c.to_hex().to_lowercase())
        .unwrap_or_else(|| "system".to_string());

    let canvas_bg_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Workspace Color"))
        .subtitle(crate::core::gettext("Background color behind pages and artboards"))
        .build();

    {
        let c_c = canvas.clone();
        let bg_swatches = make_color_swatches(
            vec![
                ("System Default", "chip-system-gradient", "system".to_string()),
                ("Dark Studio (#242424)", "chip-dark", "#242424".to_string()),
                ("Neutral Charcoal (#383838)", "chip-charcoal", "#383838".to_string()),
                ("Light Gray (#E8E8E8)", "chip-gray", "#e8e8e8".to_string()),
                ("Pure Black (#000000)", "chip-black", "#000000".to_string()),
                ("Pure White (#FFFFFF)", "chip-white", "#ffffff".to_string()),
            ],
            cur_bg_hex,
            move |code| {
                let color = if code == "system" {
                    None
                } else {
                    crate::core::Color::from_hex(&code)
                };
                c_c.set_canvas_bg_color(color);
            },
        );
        canvas_bg_row.add_suffix(&bg_swatches);
    }
    canvas_bg_group.add(&canvas_bg_row);

    let workspace_dots_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Infinite Dot Pattern"))
        .subtitle(crate::core::gettext("Show subtle ambient dot pattern across workspace"))
        .active(canvas.show_workspace_dots())
        .build();
    {
        let c_c = canvas.clone();
        workspace_dots_row.connect_active_notify(move |sw| {
            c_c.set_show_workspace_dots(sw.is_active());
        });
    }
    canvas_bg_group.add(&workspace_dots_row);

    // Page / Artboard Customization Group (Visual Paper Cards)
    let page_style_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Page and Artboard"))
        .description(crate::core::gettext(
            "Visual styling for artboard paper color, drop shadow, and borders",
        ))
        .build();

    let cur_paper_hex = match canvas.page_bg_color() {
        Some(c) if c.a < 0.01 => "transparent".to_string(),
        Some(c) => c.to_hex().to_lowercase(),
        None => "#ffffff".to_string(),
    };

    let page_bg_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Page Background Color"))
        .subtitle(crate::core::gettext("Default paper color for active document artboards"))
        .build();

    {
        let c_c = canvas.clone();
        let paper_swatches = make_color_swatches(
            vec![
                ("White Paper (#FFFFFF)", "chip-white", "#ffffff".to_string()),
                ("Warm Cream (#FCF9F2)", "chip-cream", "#fcf9f2".to_string()),
                ("Light Gray (#E8E8E8)", "chip-gray", "#e8e8e8".to_string()),
                ("Dark Paper (#1E1E1E)", "chip-paper-dark", "#1e1e1e".to_string()),
                ("Pure Black (#000000)", "chip-black", "#000000".to_string()),
                ("Transparent", "pref-checkerboard", "transparent".to_string()),
            ],
            cur_paper_hex,
            move |code| {
                let color = if code == "transparent" {
                    Some(crate::core::Color::new(0.0, 0.0, 0.0, 0.0))
                } else {
                    crate::core::Color::from_hex(&code)
                };
                c_c.set_page_bg_color(color);
            },
        );
        page_bg_row.add_suffix(&paper_swatches);
    }
    page_style_group.add(&page_bg_row);

    let page_shadow_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Page Drop Shadow"))
        .subtitle(crate::core::gettext("Realistic elevation shadow around page boundaries"))
        .active(canvas.page_shadow())
        .build();
    {
        let c_c = canvas.clone();
        page_shadow_row.connect_active_notify(move |sw| {
            c_c.set_page_shadow(sw.is_active());
        });
    }
    page_style_group.add(&page_shadow_row);

    let page_border_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Page Outline"))
        .subtitle(crate::core::gettext("Fine boundary outline highlighting page dimensions"))
        .active(canvas.page_border())
        .build();
    {
        let c_c = canvas.clone();
        page_border_row.connect_active_notify(move |sw| {
            c_c.set_page_border(sw.is_active());
        });
    }
    page_style_group.add(&page_border_row);

    // Alignment Grid Group (Visual Capsule & Chips)
    let grid_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Grid and Alignment"))
        .description(crate::core::gettext(
            "Visual configuration of precision grid and points",
        ))
        .build();

    let grid_cfg = canvas.grid_config();

    let grid_vis_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Show Grid"))
        .subtitle(crate::core::gettext("Display precision grid across the canvas"))
        .active(grid_cfg.visible)
        .build();
    {
        let c_c = canvas.clone();
        grid_vis_row.connect_active_notify(move |sw| {
            c_c.set_grid_visible(sw.is_active());
        });
    }
    grid_group.add(&grid_vis_row);

    let grid_style_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Grid Style"))
        .subtitle(crate::core::gettext("Graphic pattern displayed for grid subdivisions"))
        .build();
    {
        let c_c = canvas.clone();
        let cur_style_code = match grid_cfg.style {
            crate::core::GridStyle::Dots => "dots",
            crate::core::GridStyle::None => "none",
            crate::core::GridStyle::Lines => "lines",
        };
        let style_capsule = make_segmented_capsule(
            vec![
                ("Lines", "lines"),
                ("Dots", "dots"),
                ("Hidden", "none"),
            ],
            cur_style_code,
            move |code| {
                let style = match code {
                    "dots" => crate::core::GridStyle::Dots,
                    "none" => crate::core::GridStyle::None,
                    _ => crate::core::GridStyle::Lines,
                };
                c_c.set_grid_style(style);
            },
        );
        grid_style_row.add_suffix(&style_capsule);
    }
    grid_group.add(&grid_style_row);

    let cell_adj = gtk4::Adjustment::new(grid_cfg.cell_size as f64, 4.0, 200.0, 2.0, 10.0, 0.0);
    let cell_row = adw::SpinRow::builder()
        .title(crate::core::gettext("Grid Spacing (px)"))
        .subtitle(crate::core::gettext("Pixel distance between primary grid lines"))
        .adjustment(&cell_adj)
        .build();
    {
        let c_c = canvas.clone();
        cell_row.connect_value_notify(move |spin| {
            c_c.set_grid_cell_size(spin.value() as f32);
        });
    }
    grid_group.add(&cell_row);

    let sub_adj = gtk4::Adjustment::new(grid_cfg.subdivisions as f64, 1.0, 16.0, 1.0, 2.0, 0.0);
    let sub_row = adw::SpinRow::builder()
        .title(crate::core::gettext("Grid Subdivisions"))
        .subtitle(crate::core::gettext("Number of fine divisions in each cell"))
        .adjustment(&sub_adj)
        .build();
    {
        let c_c = canvas.clone();
        sub_row.connect_value_notify(move |spin| {
            c_c.set_grid_subdivisions(spin.value() as u32);
        });
    }
    grid_group.add(&sub_row);

    let grid_op_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Grid Opacity"))
        .subtitle(crate::core::gettext("Controls how visible the grid overlay appears"))
        .build();
    {
        let c_c = canvas.clone();
        let cur_op_code = if grid_cfg.opacity < 0.18 {
            10
        } else if grid_cfg.opacity < 0.38 {
            25
        } else if grid_cfg.opacity < 0.63 {
            50
        } else if grid_cfg.opacity < 0.88 {
            75
        } else {
            100
        };
        let op_capsule = make_segmented_capsule(
            vec![
                ("10%", 10),
                ("25%", 25),
                ("50%", 50),
                ("75%", 75),
                ("100%", 100),
            ],
            cur_op_code,
            move |op_int| {
                let op = (op_int as f32) / 100.0;
                c_c.set_grid_opacity(op);
            },
        );
        grid_op_row.add_suffix(&op_capsule);
    }
    grid_group.add(&grid_op_row);

    let grid_col_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Grid Color"))
        .subtitle(crate::core::gettext("Hue tint used for major and minor grid lines"))
        .build();
    {
        let c_c = canvas.clone();
        let cur_grid_hex = grid_cfg
            .color
            .map(|c| c.to_hex().to_lowercase())
            .unwrap_or_else(|| "#3584e4".to_string());
        let col_swatches = make_color_swatches(
            vec![
                ("GNOME Blue", "chip-blue", "#3584e4".to_string()),
                ("Neutral Gray", "chip-gray", "#888888".to_string()),
                ("Bright Cyan", "chip-cyan", "#00d2ff".to_string()),
                ("Vibrant Magenta", "chip-magenta", "#ff007a".to_string()),
                ("High Contrast White", "chip-white", "#ffffff".to_string()),
            ],
            cur_grid_hex,
            move |hex| {
                let col = crate::core::Color::from_hex(&hex);
                c_c.set_grid_color(col);
            },
        );
        grid_col_row.add_suffix(&col_swatches);
    }
    grid_group.add(&grid_col_row);

    let units_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Units and Measurements"))
        .description(crate::core::gettext(
            "Define coordinate system and default studio unit",
        ))
        .build();

    let unit_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Default measurement unit"))
        .subtitle(crate::core::gettext(
            "Used in dimension inspector, rulers, and transformations",
        ))
        .build();
    {
        let c_c = canvas.clone();
        let cur_unit_suffix = canvas.unit().suffix();
        let unit_capsule = make_segmented_capsule(
            vec![
                ("px", "px"),
                ("mm", "mm"),
                ("cm", "cm"),
                ("m", "m"),
                ("in", "in"),
                ("pt", "pt"),
                ("pc", "pc"),
            ],
            cur_unit_suffix,
            move |u_str| {
                if let Some(unit) = crate::core::Unit::from_suffix(&u_str) {
                    c_c.set_unit(unit);
                }
            },
        );
        unit_row.add_suffix(&unit_capsule);
    }
    units_group.add(&unit_row);

    let snap_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Magnetic Snapping and Guides"))
        .description(crate::core::gettext(
            "Snapping rules and precision visual guides",
        ))
        .build();

    let snap_cfg = canvas.snap_config();
    let ruler_cfg = canvas.ruler_config();

    let snap_grid_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Snap to grid"))
        .subtitle(crate::core::gettext(
            "Snaps points and objects to nearest grid",
        ))
        .active(snap_cfg.snap_to_grid)
        .build();
    {
        let canvas_c = canvas.clone();
        snap_grid_row.connect_active_notify(move |sw| {
            canvas_c.set_snap_to_grid(sw.is_active());
        });
    }
    snap_group.add(&snap_grid_row);

    let snap_obj_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Snap to other objects"))
        .subtitle(crate::core::gettext(
            "Displays smart alignment guides between elements",
        ))
        .active(snap_cfg.snap_to_objects)
        .build();
    {
        let canvas_c = canvas.clone();
        snap_obj_row.connect_active_notify(move |sw| {
            canvas_c.set_snap_to_objects(sw.is_active());
        });
    }
    snap_group.add(&snap_obj_row);

    let snap_artboard_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Snap to page"))
        .subtitle(crate::core::gettext(
            "Snaps objects to artboard margins and center",
        ))
        .active(snap_cfg.snap_to_artboard)
        .build();
    {
        let canvas_c = canvas.clone();
        snap_artboard_row.connect_active_notify(move |sw| {
            canvas_c.set_snap_to_artboard(sw.is_active());
        });
    }
    snap_group.add(&snap_artboard_row);

    let guides_vis_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Show rulers and guides"))
        .subtitle(crate::core::gettext(
            "Shows alignment guides and measurement rulers",
        ))
        .active(ruler_cfg.visible)
        .build();
    {
        let canvas_c = canvas.clone();
        guides_vis_row.connect_active_notify(move |sw| {
            canvas_c.set_guides_visible(sw.is_active());
        });
    }
    snap_group.add(&guides_vis_row);

    make_page(vec![
        canvas_bg_group,
        page_style_group,
        grid_group,
        units_group,
        snap_group,
    ])
}
