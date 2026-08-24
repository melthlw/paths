use libadwaita as adw;
use libadwaita::prelude::*;

use super::widgets::{make_page, make_segmented_capsule};
use crate::ui::canvas::CanvasWidget;

pub fn build_node_editor_page(canvas: &CanvasWidget) -> gtk4::ScrolledWindow {
    let node_visual_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Node and Handle Appearance"))
        .description(crate::core::gettext(
            "Configure the visual sizing, shapes, and handles for vector editing",
        ))
        .build();

    let cur_cfg = canvas.path_editor_config();

    // 1. Node Display Size
    let node_size_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Anchor Node Size"))
        .subtitle(crate::core::gettext("Visual size of anchor points on canvas"))
        .build();
    {
        let canvas_c = canvas.clone();
        let cur_size_int = if cur_cfg.node_size <= 6.5 {
            6
        } else if cur_cfg.node_size <= 9.5 {
            9
        } else if cur_cfg.node_size <= 13.5 {
            12
        } else {
            16
        };
        let size_capsule = make_segmented_capsule(
            vec![
                ("6 px", 6),
                ("9 px", 9),
                ("12 px", 12),
                ("16 px", 16),
            ],
            cur_size_int,
            move |sz| {
                let mut cfg = canvas_c.path_editor_config();
                cfg.node_size = sz as f32;
                canvas_c.set_path_editor_config(cfg);
            },
        );
        node_size_row.add_suffix(&size_capsule);
    }
    node_visual_group.add(&node_size_row);

    // 2. Handle Display Mode
    let handle_mode_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Handle Visibility"))
        .subtitle(crate::core::gettext("When to show Bézier control arms and handles"))
        .build();
    {
        let canvas_c = canvas.clone();
        let cur_mode_code = match cur_cfg.handle_display_mode {
            crate::core::HandleDisplayMode::SelectedOnly => "selected",
            crate::core::HandleDisplayMode::AllInSelectedPath => "path",
            crate::core::HandleDisplayMode::Always => "always",
        };
        let handle_capsule = make_segmented_capsule(
            vec![
                ("Selected Nodes", "selected"),
                ("Active Path", "path"),
                ("Always Visible", "always"),
            ],
            cur_mode_code,
            move |code| {
                let mut cfg = canvas_c.path_editor_config();
                cfg.handle_display_mode = match code {
                    "path" => crate::core::HandleDisplayMode::AllInSelectedPath,
                    "always" => crate::core::HandleDisplayMode::Always,
                    _ => crate::core::HandleDisplayMode::SelectedOnly,
                };
                canvas_c.set_path_editor_config(cfg);
            },
        );
        handle_mode_row.add_suffix(&handle_capsule);
    }
    node_visual_group.add(&handle_mode_row);

    // 3. Handle Endpoint Size
    let handle_size_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Control Handle Size"))
        .subtitle(crate::core::gettext("Size of Bézier handle endpoints"))
        .build();
    {
        let canvas_c = canvas.clone();
        let cur_hsize_int = if cur_cfg.handle_size <= 4.5 {
            4
        } else if cur_cfg.handle_size <= 6.5 {
            6
        } else {
            8
        };
        let hsize_capsule = make_segmented_capsule(
            vec![
                ("4 px", 4),
                ("6 px", 6),
                ("8 px", 8),
            ],
            cur_hsize_int,
            move |sz| {
                let mut cfg = canvas_c.path_editor_config();
                cfg.handle_size = sz as f32;
                canvas_c.set_path_editor_config(cfg);
            },
        );
        handle_size_row.add_suffix(&hsize_capsule);
    }
    node_visual_group.add(&handle_size_row);

    // 4. Distinct Node Shapes by Type
    let distinct_shapes_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Differentiate Node Shapes"))
        .subtitle(crate::core::gettext(
            "Show Squares for Corners, Diamonds for Smooth, and Circles for Symmetric/Auto",
        ))
        .active(cur_cfg.show_distinct_node_shapes)
        .build();
    {
        let canvas_c = canvas.clone();
        distinct_shapes_row.connect_active_notify(move |sw| {
            let mut cfg = canvas_c.path_editor_config();
            cfg.show_distinct_node_shapes = sw.is_active();
            canvas_c.set_path_editor_config(cfg);
        });
    }
    node_visual_group.add(&distinct_shapes_row);

    // 5. Show Path Direction Arrows
    let path_dir_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Show Path Direction Arrows"))
        .subtitle(crate::core::gettext(
            "Display tangent direction arrows along vector segments",
        ))
        .active(cur_cfg.show_path_direction)
        .build();
    {
        let canvas_c = canvas.clone();
        path_dir_row.connect_active_notify(move |sw| {
            let mut cfg = canvas_c.path_editor_config();
            cfg.show_path_direction = sw.is_active();
            canvas_c.set_path_editor_config(cfg);
        });
    }
    node_visual_group.add(&path_dir_row);

    // 6. Highlight Hovered Segment
    let hover_seg_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Highlight Hovered Segments"))
        .subtitle(crate::core::gettext(
            "Show dynamic add-point cross and segment highlight under cursor",
        ))
        .active(cur_cfg.highlight_hovered_segment)
        .build();
    {
        let canvas_c = canvas.clone();
        hover_seg_row.connect_active_notify(move |sw| {
            let mut cfg = canvas_c.path_editor_config();
            cfg.highlight_hovered_segment = sw.is_active();
            canvas_c.set_path_editor_config(cfg);
        });
    }
    node_visual_group.add(&hover_seg_row);

    // Interaction & Manipulation Group
    let node_interact_group = adw::PreferencesGroup::builder()
        .title(crate::core::gettext("Interaction and Manipulation"))
        .description(crate::core::gettext(
            "Configure behavior and sensitivities for vector path editing",
        ))
        .build();

    // 7. Direct Segment Dragging
    let direct_drag_row = adw::SwitchRow::builder()
        .title(crate::core::gettext("Direct Curve Dragging"))
        .subtitle(crate::core::gettext(
            "Click and drag anywhere on a curve to reshape Bézier curvature directly",
        ))
        .active(cur_cfg.enable_direct_segment_drag)
        .build();
    {
        let canvas_c = canvas.clone();
        direct_drag_row.connect_active_notify(move |sw| {
            let mut cfg = canvas_c.path_editor_config();
            cfg.enable_direct_segment_drag = sw.is_active();
            canvas_c.set_path_editor_config(cfg);
        });
    }
    node_interact_group.add(&direct_drag_row);

    // 8. Hit Tolerance Radius
    let hit_tol_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Click Grab Sensitivity"))
        .subtitle(crate::core::gettext("Tolerance distance for selecting nodes and handles"))
        .build();
    {
        let canvas_c = canvas.clone();
        let cur_tol_int = if cur_cfg.hit_tolerance <= 9.0 {
            8
        } else if cur_cfg.hit_tolerance <= 14.0 {
            12
        } else if cur_cfg.hit_tolerance <= 20.0 {
            18
        } else {
            24
        };
        let tol_capsule = make_segmented_capsule(
            vec![
                ("8 px", 8),
                ("12 px", 12),
                ("18 px", 18),
                ("24 px", 24),
            ],
            cur_tol_int,
            move |tol| {
                let mut cfg = canvas_c.path_editor_config();
                cfg.hit_tolerance = tol as f32;
                canvas_c.set_path_editor_config(cfg);
            },
        );
        hit_tol_row.add_suffix(&tol_capsule);
    }
    node_interact_group.add(&hit_tol_row);

    // 9. Handle Angle Snapping Step
    let angle_step_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Handle Angle Snapping (Shift)"))
        .subtitle(crate::core::gettext(
            "Constrain handle rotation angle when holding Shift",
        ))
        .build();
    {
        let canvas_c = canvas.clone();
        let cur_angle_int = if cur_cfg.angle_snapping_step <= 0.1 {
            0
        } else if cur_cfg.angle_snapping_step <= 18.0 {
            15
        } else if cur_cfg.angle_snapping_step <= 32.0 {
            30
        } else if cur_cfg.angle_snapping_step <= 50.0 {
            45
        } else {
            90
        };
        let angle_capsule = make_segmented_capsule(
            vec![
                ("Free", 0),
                ("15°", 15),
                ("30°", 30),
                ("45°", 45),
                ("90°", 90),
            ],
            cur_angle_int,
            move |deg| {
                let mut cfg = canvas_c.path_editor_config();
                cfg.angle_snapping_step = deg as f32;
                canvas_c.set_path_editor_config(cfg);
            },
        );
        angle_step_row.add_suffix(&angle_capsule);
    }
    node_interact_group.add(&angle_step_row);

    // 10. Default Node Type
    let default_type_row = adw::ActionRow::builder()
        .title(crate::core::gettext("Default Type for New Nodes"))
        .subtitle(crate::core::gettext(
            "Node type assigned when creating or inserting new anchor points",
        ))
        .build();
    {
        let canvas_c = canvas.clone();
        let cur_def_code = match cur_cfg.default_node_type {
            crate::core::NodeType::Corner => "corner",
            crate::core::NodeType::Smooth => "smooth",
            crate::core::NodeType::Symmetric => "symmetric",
            crate::core::NodeType::Auto => "auto",
        };
        let type_capsule = make_segmented_capsule(
            vec![
                ("Corner", "corner"),
                ("Smooth", "smooth"),
                ("Symmetric", "symmetric"),
                ("Auto", "auto"),
            ],
            cur_def_code,
            move |code| {
                let mut cfg = canvas_c.path_editor_config();
                cfg.default_node_type = match code {
                    "corner" => crate::core::NodeType::Corner,
                    "smooth" => crate::core::NodeType::Smooth,
                    "symmetric" => crate::core::NodeType::Symmetric,
                    "auto" => crate::core::NodeType::Auto,
                    _ => crate::core::NodeType::Smooth,
                };
                canvas_c.set_path_editor_config(cfg);
            },
        );
        default_type_row.add_suffix(&type_capsule);
    }
    node_interact_group.add(&default_type_row);

    make_page(vec![node_visual_group, node_interact_group])
}
