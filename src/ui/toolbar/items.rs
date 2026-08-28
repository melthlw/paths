use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::plugins::manifest::BarPosition;
use crate::plugins::registry::ToolbarGroup;
use crate::ui::canvas::CanvasWidget;

#[derive(Clone)]
pub struct SubToolItem {
    pub id: &'static str,
    pub name: String,
    pub shortcut: Option<String>,
    pub icon_resource: Option<&'static str>,
    pub icon_name: &'static str,
    pub tooltip: &'static str,
    pub visible: bool,
}

#[derive(Clone)]
pub struct ToolItem {
    pub id: &'static str,
    pub name: String,
    pub shortcut: Option<String>,
    pub icon_resource: Option<&'static str>,
    pub icon_name: &'static str,
    pub tooltip: &'static str,
    pub visible: bool,
    pub sub_tools: Vec<SubToolItem>,
}

pub fn get_tool_meta(
    tool_id: &'static str,
    fallback_tooltip: &'static str,
) -> (String, Option<&'static str>) {
    match tool_id {
        "select" => (crate::core::gettext("Select"), Some("V")),
        "path_editor" => (crate::core::gettext("Path Editor"), Some("A")),
        "rectangle" => (crate::core::gettext("Rectangle"), Some("R")),
        "circle" => (crate::core::gettext("Circle"), Some("C")),
        "star" => (crate::core::gettext("Star"), Some("S")),
        "triangle" => (crate::core::gettext("Triangle"), Some("Y")),
        "spiral" => (crate::core::gettext("Spiral"), Some("W")),
        "pen" => (crate::core::gettext("Pen"), Some("P")),
        "brush" => (crate::core::gettext("Brush"), Some("B")),
        "text" => (crate::core::gettext("Text"), Some("T")),
        "image" => (crate::core::gettext("Image Frame"), Some("Shift+I")),
        "boolean-union" => (crate::core::gettext("Arrange & Order"), Some("Ctrl++")),
        "paint_bucket" => (crate::core::gettext("Paint Bucket"), Some("K")),
        "eyedropper" => (crate::core::gettext("Eyedropper"), Some("I")),
        "measure" => (crate::core::gettext("Measure"), Some("M")),
        "zoom" => (crate::core::gettext("Zoom"), Some("Z")),
        "page" => (crate::core::gettext("Artboard / Page"), Some("F")),
        "gradient" => (crate::core::gettext("Gradient"), Some("G")),
        "mesh_gradient" | "mesh" => (crate::core::gettext("Mesh Gradient"), Some("M")),
        "pattern" => (crate::core::gettext("Pattern Tool"), Some("Shift+P")),
        _ => (crate::core::gettext(fallback_tooltip), None),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SerializedSubTool {
    pub id: String,
    pub visible: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SerializedToolItem {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub sub_tools: Vec<SerializedSubTool>,
}

pub fn serialize_toolbar_items(items: &[ToolItem]) -> String {
    let serialized: Vec<SerializedToolItem> = items
        .iter()
        .map(|it| SerializedToolItem {
            id: it.id.to_string(),
            name: it.name.clone(),
            visible: it.visible,
            sub_tools: it
                .sub_tools
                .iter()
                .map(|st| SerializedSubTool {
                    id: st.id.to_string(),
                    visible: st.visible,
                })
                .collect(),
        })
        .collect();
    serde_json::to_string(&serialized).unwrap_or_default()
}

pub fn restore_toolbar_items(json_str: &str, groups: &[ToolbarGroup]) -> Vec<ToolItem> {
    if let Ok(serialized) = serde_json::from_str::<Vec<SerializedToolItem>>(json_str) {
        if !serialized.is_empty() {
            let mut all_tools: HashMap<&str, (Option<&'static str>, &'static str, &'static str)> =
                HashMap::new();
            for g in groups {
                for t in &g.tools {
                    all_tools.insert(t.tool_id, (t.icon_resource, t.icon_name, t.tooltip));
                }
            }

            let mut items = Vec::new();
            for sit in serialized {
                if let Some(&(ic_res, ic_name, tt)) = all_tools.get(sit.id.as_str()) {
                    let (p_name, p_sc) = get_tool_meta(leak_str(&sit.id), tt);

                    let mut sub_items = Vec::new();
                    for sst in sit.sub_tools {
                        if let Some(&(s_ic_res, s_ic_name, s_tt)) = all_tools.get(sst.id.as_str()) {
                            let (s_name, s_sc) = get_tool_meta(leak_str(&sst.id), s_tt);
                            sub_items.push(SubToolItem {
                                id: leak_str(&sst.id),
                                name: s_name,
                                shortcut: s_sc.map(|s| s.to_string()),
                                icon_resource: s_ic_res,
                                icon_name: s_ic_name,
                                tooltip: s_tt,
                                visible: sst.visible,
                            });
                        }
                    }

                    items.push(ToolItem {
                        id: leak_str(&sit.id),
                        name: if sit.name.trim().is_empty() {
                            p_name
                        } else {
                            sit.name
                        },
                        shortcut: p_sc.map(|s| s.to_string()),
                        icon_resource: ic_res,
                        icon_name: ic_name,
                        tooltip: tt,
                        visible: sit.visible,
                        sub_tools: sub_items,
                    });
                }
            }
            if !items.is_empty() {
                return items;
            }
        }
    }
    generate_default_items(groups)
}

fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

pub fn generate_default_items(groups: &[ToolbarGroup]) -> Vec<ToolItem> {
    let mut items = Vec::new();
    for group in groups {
        let primary = &group.tools[0];
        let is_grouped = group.tools.len() > 1;
        let (p_name, p_sc) = get_tool_meta(primary.tool_id, primary.tooltip);

        let mut sub_items = Vec::new();
        if is_grouped {
            for tool in &group.tools {
                let (s_name, s_sc) = get_tool_meta(tool.tool_id, tool.tooltip);
                sub_items.push(SubToolItem {
                    id: tool.tool_id,
                    name: s_name.to_string(),
                    shortcut: s_sc.map(|s| s.to_string()),
                    icon_resource: tool.icon_resource,
                    icon_name: tool.icon_name,
                    tooltip: tool.tooltip,
                    visible: true,
                });
            }
        }

        items.push(ToolItem {
            id: primary.tool_id,
            name: p_name.to_string(),
            shortcut: p_sc.map(|s| s.to_string()),
            icon_resource: primary.icon_resource,
            icon_name: primary.icon_name,
            tooltip: primary.tooltip,
            visible: true,
            sub_tools: sub_items,
        });
    }
    items
}

pub fn rebuild_toolbar_items(
    items_box: &gtk4::Box,
    items: &[ToolItem],
    canvas: &CanvasWidget,
    is_updating: &Rc<Cell<bool>>,
    buttons: &Rc<RefCell<HashMap<&'static str, gtk4::ToggleButton>>>,
    tool_icon_setters: &Rc<RefCell<HashMap<&'static str, Box<dyn Fn()>>>>,
    popovers: &Rc<RefCell<Vec<gtk4::Popover>>>,
    position: BarPosition,
) {
    while let Some(child) = items_box.first_child() {
        items_box.remove(&child);
    }
    buttons.borrow_mut().clear();
    tool_icon_setters.borrow_mut().clear();
    popovers.borrow_mut().clear();

    let active_tool = canvas
        .state()
        .try_borrow()
        .map(|s| s.plugin_manager.active_id())
        .unwrap_or("select");
    let mut first_toggle: Option<gtk4::ToggleButton> = None;

    let popover_pos = match position {
        BarPosition::Top => gtk4::PositionType::Bottom,
        BarPosition::Bottom => gtk4::PositionType::Top,
        BarPosition::Left => gtk4::PositionType::Right,
        BarPosition::Right => gtk4::PositionType::Left,
    };

    for item in items {
        if !item.visible {
            continue;
        }

        let is_grouped = !item.sub_tools.is_empty();

        let icon_img = if let Some(res) = item.icon_resource {
            crate::ui::icons::make_symbolic_image(res, 22)
        } else {
            crate::ui::icons::make_symbolic_image(item.icon_name, 22)
        };

        let button_child: gtk4::Widget = if is_grouped {
            let overlay = gtk4::Overlay::new();
            overlay.set_child(Some(&icon_img));

            let indicator = gtk4::DrawingArea::new();
            indicator.set_content_width(6);
            indicator.set_content_height(6);
            indicator.set_halign(gtk4::Align::End);
            indicator.set_valign(gtk4::Align::End);
            indicator.set_margin_end(1);
            indicator.set_margin_bottom(1);
            indicator.set_draw_func(move |da, cr, w, h| {
                let w = w as f64;
                let h = h as f64;
                let r = 1.5_f64; // corner radius
                // Rounded triangle: top-right -> bottom-right -> bottom-left
                cr.new_path();
                // Top-right corner
                cr.arc(w - r, r, r, -std::f64::consts::FRAC_PI_2, 0.0);
                // Bottom-right corner
                cr.arc(w - r, h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
                // Bottom-left corner
                cr.arc(r, h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
                cr.close_path();
                let c = da.color();
                cr.set_source_rgba(
                    c.red() as f64,
                    c.green() as f64,
                    c.blue() as f64,
                    0.5,
                );
                let _ = cr.fill();
            });
            overlay.add_overlay(&indicator);

            overlay.upcast()
        } else {
            icon_img.clone().upcast()
        };

        let is_active = if is_grouped {
            item.sub_tools.iter().any(|s| s.id == active_tool)
        } else {
            item.id == active_tool
        };

        let btn = gtk4::ToggleButton::builder()
            .child(&button_child)
            .tooltip_text(&crate::core::gettext(item.tooltip))
            .active(is_active)
            .css_classes(["flat"])
            .focus_on_click(false)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .build();

        if let Some(ref first) = first_toggle {
            btn.set_group(Some(first));
        } else {
            first_toggle = Some(btn.clone());
        }

        let current_tool_id = Rc::new(Cell::new(item.id));

        if is_grouped {
            let popover = gtk4::Popover::builder()
                .has_arrow(true)
                .position(popover_pos)
                .build();
            popover.set_parent(&btn);
            {
                let pop_c = popover.clone();
                btn.connect_destroy(move |_| {
                    if pop_c.parent().is_some() {
                        pop_c.unparent();
                    }
                });
            }
            popovers.borrow_mut().push(popover.clone());

            let grid = gtk4::Grid::builder()
                .column_spacing(6)
                .row_spacing(6)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(6)
                .margin_end(6)
                .build();

            for (idx, sub) in item.sub_tools.iter().enumerate() {
                if !sub.visible {
                    continue;
                }
                buttons.borrow_mut().insert(sub.id, btn.clone());

                let main_icon_img = icon_img.clone();
                let cur_tool_sub = current_tool_id.clone();
                let sub_icon_res = sub.icon_resource;
                let sub_icon_name = sub.icon_name;
                let sub_tool_id = sub.id;
                let setter = Box::new(move || {
                    let sym_name = if let Some(r) = sub_icon_res {
                        crate::ui::icons::symbolic_icon_name(r)
                    } else {
                        crate::ui::icons::symbolic_icon_name(sub_icon_name)
                    };
                    main_icon_img.set_icon_name(Some(&sym_name));
                    cur_tool_sub.set(sub_tool_id);
                });
                tool_icon_setters.borrow_mut().insert(sub.id, setter);

                let sub_img = if let Some(res) = sub.icon_resource {
                    crate::ui::icons::make_symbolic_image(res, 20)
                } else {
                    crate::ui::icons::make_symbolic_image(sub.icon_name, 20)
                };

                let sub_btn = gtk4::Button::builder()
                    .child(&sub_img)
                    .tooltip_text(&crate::core::gettext(sub.tooltip))
                    .css_classes(["flat"])
                    .focus_on_click(false)
                    .build();

                let canvas_sub = canvas.clone();
                let main_icon_img_c = icon_img.clone();
                let main_btn = btn.clone();
                let popover_close = popover.clone();
                let is_updating_sub = is_updating.clone();
                let cur_tool_sub_c = current_tool_id.clone();

                sub_btn.connect_clicked(move |_| {
                    let sym_name = if let Some(r) = sub_icon_res {
                        crate::ui::icons::symbolic_icon_name(r)
                    } else {
                        crate::ui::icons::symbolic_icon_name(sub_icon_name)
                    };
                    main_icon_img_c.set_icon_name(Some(&sym_name));
                    cur_tool_sub_c.set(sub_tool_id);
                    is_updating_sub.set(true);
                    main_btn.set_active(true);
                    canvas_sub.set_active_tool(sub_tool_id);
                    is_updating_sub.set(false);
                    popover_close.popdown();
                });

                let col = (idx % 2) as i32;
                let row = (idx / 2) as i32;
                grid.attach(&sub_btn, col, row, 1, 1);
            }

            popover.set_child(Some(&grid));

            let gesture_right = gtk4::GestureClick::builder().button(3).build();
            let popover_right = popover.clone();
            gesture_right.connect_released(move |_, _, _, _| {
                popover_right.popup();
            });
            btn.add_controller(gesture_right);

            let gesture_long = gtk4::GestureLongPress::new();
            let popover_long = popover.clone();
            gesture_long.connect_pressed(move |_, _, _| {
                popover_long.popup();
            });
            btn.add_controller(gesture_long);
        } else {
            buttons.borrow_mut().insert(item.id, btn.clone());
        }

        let canvas_tool = canvas.clone();
        let cur_tool_toggled = current_tool_id.clone();
        let is_updating_btn = is_updating.clone();

        btn.connect_toggled(move |b| {
            if is_updating_btn.get() {
                return;
            }
            if b.is_active() {
                is_updating_btn.set(true);
                canvas_tool.set_active_tool(cur_tool_toggled.get());
                is_updating_btn.set(false);
            }
        });

        items_box.append(&btn);
    }
}
