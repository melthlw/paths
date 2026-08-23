use gtk4::prelude::*;
use std::cell::Cell;

use crate::core::Point;
use crate::ui::canvas::CanvasWidget;
use super::helpers::{create_item, create_toggle_item};

pub struct CanvasMenuWidgets {
    pub paste_here_btn: gtk4::Button,
    pub paste_canvas_btn: gtk4::Button,
    pub paste_in_place_btn: gtk4::Button,
    pub sep_canvas_clip: gtk4::Separator,

    pub select_all_btn: gtk4::Button,
    pub sep_canvas_sel: gtk4::Separator,

    pub insert_submenu_btn: gtk4::Button,
    pub guides_grid_submenu_btn: gtk4::Button,
    pub sep_canvas_tools: gtk4::Separator,

    pub zoom_fit_btn: gtk4::Button,
    pub zoom_100_btn: gtk4::Button,
    pub zoom_in_btn: gtk4::Button,
    pub zoom_out_btn: gtk4::Button,
    pub sep_canvas_zoom: gtk4::Separator,

    pub add_page_btn: gtk4::Button,

    pub grid_check_img: gtk4::Image,
    pub snap_grid_check_img: gtk4::Image,
    pub guides_check_img: gtk4::Image,
    pub snap_guides_check_img: gtk4::Image,
    pub clear_guides_btn: gtk4::Button,

    pub insert_box: gtk4::Box,
    pub guides_grid_box: gtk4::Box,
}

pub fn build_canvas_menu(
    canvas: &CanvasWidget,
    popover: &gtk4::Popover,
    stack: &gtk4::Stack,
    last_click_pos: &Cell<Point>,
) -> CanvasMenuWidgets {
    // 1. Canvas level actions in main menu
    let (paste_here_btn, _) = create_item(
        "edit-paste-symbolic",
        &crate::core::gettext("Paste Here"),
        None,
    );
    let (paste_canvas_btn, _) = create_item(
        "edit-paste-symbolic",
        &crate::core::gettext("Paste"),
        Some("Ctrl + V"),
    );
    let (paste_in_place_btn, _) = create_item(
        "edit-paste-symbolic",
        &crate::core::gettext("Paste in Place"),
        Some("Ctrl + Shift + V"),
    );

    let sep_canvas_clip = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let (select_all_btn, _) = create_item(
        "edit-select-all-symbolic",
        &crate::core::gettext("Select All"),
        Some("Ctrl + A"),
    );

    let sep_canvas_sel = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let (insert_submenu_btn, _) = create_item(
        "list-add-symbolic",
        &crate::core::gettext("Insert Object"),
        Some("›"),
    );
    let (guides_grid_submenu_btn, _) = create_item(
        "view-grid-symbolic",
        &crate::core::gettext("Guides & Grid"),
        Some("›"),
    );

    let sep_canvas_tools = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let (zoom_fit_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-zoom-fit-all.svg",
        &crate::core::gettext("Fit to Window"),
        Some("Ctrl + 0"),
    );
    let (zoom_100_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-zoom-100.svg",
        &crate::core::gettext("Actual Size (100%)"),
        Some("Ctrl + 1"),
    );
    let (zoom_in_btn, _) = create_item(
        "zoom-in-symbolic",
        &crate::core::gettext("Zoom In"),
        Some("Ctrl + +"),
    );
    let (zoom_out_btn, _) = create_item(
        "zoom-out-symbolic",
        &crate::core::gettext("Zoom Out"),
        Some("Ctrl + -"),
    );

    let sep_canvas_zoom = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let (add_page_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-page.svg",
        &crate::core::gettext("Add New Page"),
        Some("Ctrl + Shift + N"),
    );

    // 2. Page 3: Insert Submenu
    let insert_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(1)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .width_request(240)
        .build();

    let back_insert_btn = gtk4::Button::builder()
        .css_classes(["flat", "menu-button"])
        .focus_on_click(false)
        .build();
    let back_insert_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    let back_insert_icon = gtk4::Image::from_icon_name("go-previous-symbolic");
    back_insert_icon.set_pixel_size(16);
    let back_insert_label = gtk4::Label::builder()
        .label(&crate::core::gettext("Insert Object"))
        .css_classes(["heading"])
        .xalign(0.0)
        .build();
    back_insert_row.append(&back_insert_icon);
    back_insert_row.append(&back_insert_label);
    back_insert_btn.set_child(Some(&back_insert_row));

    insert_box.append(&back_insert_btn);
    insert_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    let (tool_rect_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-square.svg",
        &crate::core::gettext("Rectangle"),
        Some("R"),
    );
    let (tool_circle_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-circle.svg",
        &crate::core::gettext("Circle / Ellipse"),
        Some("O"),
    );
    let (tool_star_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-star.svg",
        &crate::core::gettext("Star"),
        Some("S"),
    );
    let (tool_triangle_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-triangle.svg",
        &crate::core::gettext("Triangle / Polygon"),
        Some("P"),
    );
    let (tool_spiral_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-spiral.svg",
        &crate::core::gettext("Spiral"),
        Some("E"),
    );
    let (tool_text_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-text.svg",
        &crate::core::gettext("Text"),
        Some("T"),
    );
    let (tool_pen_btn, _) = create_item(
        "/io/github/lewis/GnomePaths/icons/tool-pen.svg",
        &crate::core::gettext("Bézier Pen"),
        Some("B"),
    );

    insert_box.append(&tool_rect_btn);
    insert_box.append(&tool_circle_btn);
    insert_box.append(&tool_star_btn);
    insert_box.append(&tool_triangle_btn);
    insert_box.append(&tool_spiral_btn);
    insert_box.append(&tool_text_btn);
    insert_box.append(&tool_pen_btn);

    // 3. Page 4: Guides & Grid Submenu
    let guides_grid_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(1)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .width_request(240)
        .build();

    let back_guides_btn = gtk4::Button::builder()
        .css_classes(["flat", "menu-button"])
        .focus_on_click(false)
        .build();
    let back_guides_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    let back_guides_icon = gtk4::Image::from_icon_name("go-previous-symbolic");
    back_guides_icon.set_pixel_size(16);
    let back_guides_label = gtk4::Label::builder()
        .label(&crate::core::gettext("Guides & Grid"))
        .css_classes(["heading"])
        .xalign(0.0)
        .build();
    back_guides_row.append(&back_guides_icon);
    back_guides_row.append(&back_guides_label);
    back_guides_btn.set_child(Some(&back_guides_row));

    guides_grid_box.append(&back_guides_btn);
    guides_grid_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    let (toggle_grid_btn, grid_check_img) = create_toggle_item(
        "view-grid-symbolic",
        &crate::core::gettext("Show Grid"),
        Some("Ctrl + '"),
    );
    let (toggle_snap_grid_btn, snap_grid_check_img) = create_toggle_item(
        "view-grid-symbolic",
        &crate::core::gettext("Snap to Grid"),
        None,
    );

    guides_grid_box.append(&toggle_grid_btn);
    guides_grid_box.append(&toggle_snap_grid_btn);
    guides_grid_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    let (toggle_guides_btn, guides_check_img) = create_toggle_item(
        "ruler-symbolic",
        &crate::core::gettext("Show Guides"),
        Some("Ctrl + ;"),
    );
    let (toggle_snap_guides_btn, snap_guides_check_img) = create_toggle_item(
        "ruler-symbolic",
        &crate::core::gettext("Snap to Guides"),
        None,
    );

    guides_grid_box.append(&toggle_guides_btn);
    guides_grid_box.append(&toggle_snap_guides_btn);
    guides_grid_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    let (clear_guides_btn, _) = create_item(
        "user-trash-symbolic",
        &crate::core::gettext("Clear All Guides"),
        None,
    );
    guides_grid_box.append(&clear_guides_btn);

    // Submenu navigations
    {
        let stack = stack.clone();
        insert_submenu_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("insert");
        });
    }
    {
        let stack = stack.clone();
        back_insert_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("main");
        });
    }
    {
        let stack = stack.clone();
        guides_grid_submenu_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("guides_grid");
        });
    }
    {
        let stack = stack.clone();
        back_guides_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("main");
        });
    }

    // Connect handlers for canvas actions
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        let last_pos = last_click_pos.clone();
        paste_here_btn.connect_clicked(move |_| {
            canvas.paste_at_screen_pos(last_pos.get());
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        paste_canvas_btn.connect_clicked(move |_| {
            canvas.paste(None);
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        paste_in_place_btn.connect_clicked(move |_| {
            canvas.paste_in_place();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        select_all_btn.connect_clicked(move |_| {
            canvas.select_all();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        zoom_fit_btn.connect_clicked(move |_| {
            canvas.zoom_to_fit_all();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        zoom_100_btn.connect_clicked(move |_| {
            canvas.reset_zoom();
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        zoom_in_btn.connect_clicked(move |_| {
            canvas.zoom_by(1.25);
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        zoom_out_btn.connect_clicked(move |_| {
            canvas.zoom_by(0.8);
            popover.popdown();
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        add_page_btn.connect_clicked(move |_| {
            canvas.add_new_page_auto();
            popover.popdown();
        });
    }

    // Connect tool insert buttons
    let connect_tool_btn = |btn: &gtk4::Button, tool_name: &'static str| {
        let canvas = canvas.clone();
        let popover = popover.clone();
        btn.connect_clicked(move |_| {
            canvas.set_active_tool(tool_name);
            popover.popdown();
        });
    };
    connect_tool_btn(&tool_rect_btn, "rectangle");
    connect_tool_btn(&tool_circle_btn, "circle");
    connect_tool_btn(&tool_star_btn, "star");
    connect_tool_btn(&tool_triangle_btn, "triangle");
    connect_tool_btn(&tool_spiral_btn, "spiral");
    connect_tool_btn(&tool_text_btn, "text");
    connect_tool_btn(&tool_pen_btn, "pen");

    // Guides and grid handlers
    {
        let canvas = canvas.clone();
        let img = grid_check_img.clone();
        toggle_grid_btn.connect_clicked(move |_| {
            let next = !canvas.is_grid_visible();
            canvas.set_grid_visible(next);
            img.set_visible(next);
        });
    }
    {
        let canvas = canvas.clone();
        let img = snap_grid_check_img.clone();
        toggle_snap_grid_btn.connect_clicked(move |_| {
            let next = !canvas.is_snap_to_grid();
            canvas.set_snap_to_grid(next);
            img.set_visible(next);
        });
    }
    {
        let canvas = canvas.clone();
        let img = guides_check_img.clone();
        toggle_guides_btn.connect_clicked(move |_| {
            let next = !canvas.is_guides_visible();
            canvas.set_guides_visible(next);
            img.set_visible(next);
        });
    }
    {
        let canvas = canvas.clone();
        let img = snap_guides_check_img.clone();
        toggle_snap_guides_btn.connect_clicked(move |_| {
            let next = !canvas.is_snap_to_guides();
            canvas.set_snap_to_guides(next);
            img.set_visible(next);
        });
    }
    {
        let canvas = canvas.clone();
        let popover = popover.clone();
        clear_guides_btn.connect_clicked(move |_| {
            canvas.clear_user_guides();
            popover.popdown();
        });
    }

    CanvasMenuWidgets {
        paste_here_btn,
        paste_canvas_btn,
        paste_in_place_btn,
        sep_canvas_clip,
        select_all_btn,
        sep_canvas_sel,
        insert_submenu_btn,
        guides_grid_submenu_btn,
        sep_canvas_tools,
        zoom_fit_btn,
        zoom_100_btn,
        zoom_in_btn,
        zoom_out_btn,
        sep_canvas_zoom,
        add_page_btn,
        grid_check_img,
        snap_grid_check_img,
        guides_check_img,
        snap_guides_check_img,
        clear_guides_btn,
        insert_box,
        guides_grid_box,
    }
}
