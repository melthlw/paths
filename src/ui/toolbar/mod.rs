pub mod customizer;
pub mod items;

pub use customizer::{show_customize_toolbar_dialog, show_customize_toolbar_dialog_standalone};

pub use items::{generate_default_items, rebuild_toolbar_items, restore_toolbar_items};

use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::plugins::manifest::BarPosition;
use crate::ui::canvas::CanvasWidget;

pub struct FloatingToolbar {}

impl FloatingToolbar {
    pub fn new(
        canvas: CanvasWidget,
        dock_box: gtk4::Box,
        color_bar: super::color_bar::ColorControlBar,
        options_bar: super::tool_options::ToolOptionsBar,
    ) -> Self {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .css_classes(["toolbar", "card"])
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        let items_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .build();
        container.append(&items_box);
        dock_box.append(&container);

        let buttons = Rc::new(RefCell::new(HashMap::new()));
        let is_updating = Rc::new(Cell::new(false));
        let saved_pos_str = crate::core::AppSettings::toolbar_position();
        let initial_pos = match saved_pos_str.as_str() {
            "top" => BarPosition::Top,
            "left" => BarPosition::Left,
            "right" => BarPosition::Right,
            _ => BarPosition::Bottom,
        };

        let position = Rc::new(Cell::new(initial_pos));
        let popovers = Rc::new(RefCell::new(Vec::new()));
        let tool_icon_setters: Rc<RefCell<HashMap<&'static str, Box<dyn Fn()>>>> =
            Rc::new(RefCell::new(HashMap::new()));

        let state_rc = canvas.state();
        let default_groups = state_rc.borrow().plugin_manager.toolbar_groups();
        let initial_items = if let Some(saved_json) = crate::core::AppSettings::toolbar_customization() {
            restore_toolbar_items(&saved_json, &default_groups)
        } else {
            generate_default_items(&default_groups)
        };
        let items_model = Rc::new(RefCell::new(initial_items));

        rebuild_toolbar_items(
            &items_box,
            &*items_model.borrow(),
            &canvas,
            &is_updating,
            &buttons,
            &tool_icon_setters,
            &popovers,
            position.get(),
        );

        // Drag Grip Handle
        let grip = crate::ui::icons::make_symbolic_image("tool-drag", 18);
        grip.set_opacity(0.7);
        grip.set_margin_start(4);
        grip.set_margin_end(4);
        grip.set_tooltip_text(Some(&crate::core::gettext(
            "Toolbar Options and Customization",
        )));
        container.append(&grip);

        // Menu Popover attached to Grip
        let popover = gtk4::Popover::builder()
            .has_arrow(true)
            .position(gtk4::PositionType::Top)
            .build();
        popovers.borrow_mut().push(popover.clone());

        let menu_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let title_lbl = gtk4::Label::builder()
            .label(&crate::core::gettext("Toolbar Position"))
            .css_classes(["heading"])
            .halign(gtk4::Align::Start)
            .build();
        menu_box.append(&title_lbl);

        let pos_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .build();

        let positions = [
            (
                crate::core::gettext("Bottom"),
                BarPosition::Bottom,
                "layer-move-down-symbolic",
            ),
            (
                crate::core::gettext("Top"),
                BarPosition::Top,
                "layer-move-up-symbolic",
            ),
            (
                crate::core::gettext("Left"),
                BarPosition::Left,
                "tab-move-left-symbolic",
            ),
            (
                crate::core::gettext("Right"),
                BarPosition::Right,
                "tab-move-right-symbolic",
            ),
        ];

        let apply_pos = {
            let dock_box_c = dock_box.clone();
            let color_bar_c = color_bar.clone();
            let options_bar_c = options_bar.clone();
            let container_c = container.clone();
            let items_box_c = items_box.clone();
            let popovers_c = popovers.clone();
            let pos_c = position.clone();

            Rc::new(move |pos: BarPosition| {
                pos_c.set(pos);
                let pos_str = match pos {
                    BarPosition::Top => "top",
                    BarPosition::Left => "left",
                    BarPosition::Right => "right",
                    BarPosition::Bottom => "bottom",
                };
                crate::core::AppSettings::set_toolbar_position(pos_str);

                let reorder_dock = |first: &gtk4::Box, second: &gtk4::Box| {
                    if first.parent().as_ref() == Some(dock_box_c.upcast_ref())
                        && second.parent().as_ref() == Some(dock_box_c.upcast_ref())
                    {
                        dock_box_c.reorder_child_after(first, None::<&gtk4::Widget>);
                        dock_box_c.reorder_child_after(second, Some(first));
                    }
                };

                match pos {
                    BarPosition::Bottom => {
                        dock_box_c.set_orientation(gtk4::Orientation::Horizontal);
                        dock_box_c.set_halign(gtk4::Align::Center);
                        dock_box_c.set_valign(gtk4::Align::End);
                        dock_box_c.set_margin_bottom(24);
                        dock_box_c.set_margin_top(0);
                        dock_box_c.set_margin_start(0);
                        dock_box_c.set_margin_end(0);

                        reorder_dock(color_bar_c.widget(), &container_c);

                        color_bar_c.set_orientation(gtk4::Orientation::Horizontal);
                        container_c.set_orientation(gtk4::Orientation::Horizontal);
                        items_box_c.set_orientation(gtk4::Orientation::Horizontal);

                        for p in &*popovers_c.borrow() {
                            p.set_position(gtk4::PositionType::Top);
                        }

                        options_bar_c.update_margin_for_toolbar(BarPosition::Bottom);
                    }
                    BarPosition::Top => {
                        dock_box_c.set_orientation(gtk4::Orientation::Horizontal);
                        dock_box_c.set_halign(gtk4::Align::Center);
                        dock_box_c.set_valign(gtk4::Align::Start);
                        dock_box_c.set_margin_top(24);
                        dock_box_c.set_margin_bottom(0);
                        dock_box_c.set_margin_start(0);
                        dock_box_c.set_margin_end(0);

                        reorder_dock(&container_c, color_bar_c.widget());

                        color_bar_c.set_orientation(gtk4::Orientation::Horizontal);
                        container_c.set_orientation(gtk4::Orientation::Horizontal);
                        items_box_c.set_orientation(gtk4::Orientation::Horizontal);

                        for p in &*popovers_c.borrow() {
                            p.set_position(gtk4::PositionType::Bottom);
                        }

                        options_bar_c.update_margin_for_toolbar(BarPosition::Top);
                    }
                    BarPosition::Left => {
                        dock_box_c.set_orientation(gtk4::Orientation::Vertical);
                        dock_box_c.set_halign(gtk4::Align::Start);
                        dock_box_c.set_valign(gtk4::Align::Center);
                        dock_box_c.set_margin_start(24);
                        dock_box_c.set_margin_end(0);
                        dock_box_c.set_margin_top(0);
                        dock_box_c.set_margin_bottom(0);

                        reorder_dock(color_bar_c.widget(), &container_c);

                        color_bar_c.set_orientation(gtk4::Orientation::Vertical);
                        container_c.set_orientation(gtk4::Orientation::Vertical);
                        items_box_c.set_orientation(gtk4::Orientation::Vertical);

                        for p in &*popovers_c.borrow() {
                            p.set_position(gtk4::PositionType::Right);
                        }

                        options_bar_c.update_margin_for_toolbar(BarPosition::Left);
                    }
                    BarPosition::Right => {
                        dock_box_c.set_orientation(gtk4::Orientation::Vertical);
                        dock_box_c.set_halign(gtk4::Align::End);
                        dock_box_c.set_valign(gtk4::Align::Center);
                        dock_box_c.set_margin_end(24);
                        dock_box_c.set_margin_start(0);
                        dock_box_c.set_margin_top(0);
                        dock_box_c.set_margin_bottom(0);

                        reorder_dock(&container_c, color_bar_c.widget());

                        color_bar_c.set_orientation(gtk4::Orientation::Vertical);
                        container_c.set_orientation(gtk4::Orientation::Vertical);
                        items_box_c.set_orientation(gtk4::Orientation::Vertical);

                        for p in &*popovers_c.borrow() {
                            p.set_position(gtk4::PositionType::Left);
                        }

                        options_bar_c.update_margin_for_toolbar(BarPosition::Right);
                    }
                }
            })
        };

        for (name, pos, icon) in positions {
            let btn = gtk4::Button::builder()
                .tooltip_text(&name)
                .icon_name(icon)
                .css_classes(["flat"])
                .build();

            let apply_pos_btn = apply_pos.clone();
            let pop_close = popover.clone();

            btn.connect_clicked(move |_| {
                apply_pos_btn(pos);
                pop_close.popdown();
            });

            pos_box.append(&btn);
        }
        apply_pos(initial_pos);
        menu_box.append(&pos_box);

        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        menu_box.append(&sep);

        // Open Dedicated Modal Dialog
        let cust_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Start)
            .margin_start(4)
            .margin_end(4)
            .build();
        let cust_icon = gtk4::Image::from_icon_name("prefs-toolbars-symbolic");
        cust_icon.set_pixel_size(16);
        let cust_label = gtk4::Label::new(Some("Personalizar Ferramentas..."));
        cust_box.append(&cust_icon);
        cust_box.append(&cust_label);

        let customize_btn = gtk4::Button::builder()
            .child(&cust_box)
            .css_classes(["flat"])
            .halign(gtk4::Align::Fill)
            .build();

        let canvas_cust = canvas.clone();
        let items_box_cust = items_box.clone();
        let items_model_cust = items_model.clone();
        let grip_ref = grip.clone();
        let pop_close_cust = popover.clone();
        let is_updating_cust = is_updating.clone();
        let buttons_cust = buttons.clone();
        let tool_icon_setters_cust = tool_icon_setters.clone();
        let popovers_cust = popovers.clone();
        let position_cust = position.clone();

        customize_btn.connect_clicked(move |_| {
            pop_close_cust.popdown();
            show_customize_toolbar_dialog(
                &grip_ref,
                &items_box_cust,
                &items_model_cust,
                &canvas_cust,
                &is_updating_cust,
                &buttons_cust,
                &tool_icon_setters_cust,
                &popovers_cust,
                &position_cust,
            );
        });
        menu_box.append(&customize_btn);

        popover.set_child(Some(&menu_box));
        popover.set_parent(&grip);
        {
            let pop_c = popover.clone();
            grip.connect_destroy(move |_| {
                if pop_c.parent().is_some() {
                    pop_c.unparent();
                }
            });
        }

        let grip_click = gtk4::GestureClick::builder().button(3).build();
        let pop_gc = popover.clone();
        grip_click.connect_released(move |_, _, _, _| {
            pop_gc.popup();
        });
        grip.add_controller(grip_click);

        let grip_long = gtk4::GestureLongPress::new();
        let pop_gl = popover.clone();
        grip_long.connect_pressed(move |_, _, _| {
            pop_gl.popup();
        });
        grip.add_controller(grip_long);

        let toolbar = Self {};

        // Listen for canvas tool changes (e.g. keyboard shortcuts V, A, P, B, R, T)
        let buttons_clone = buttons.clone();
        let is_updating_sync = is_updating.clone();
        let setters_clone = tool_icon_setters.clone();
        let mut state = state_rc.borrow_mut();
        state.on_tool_change = Some(Box::new(move |tool_id| {
            if let Some(setter) = setters_clone.borrow().get(tool_id) {
                setter();
            }
            if let Some(btn) = buttons_clone.borrow().get(tool_id) {
                if !btn.is_active() {
                    is_updating_sync.set(true);
                    btn.set_active(true);
                    is_updating_sync.set(false);
                }
            }
        }));

        toolbar
    }
}
