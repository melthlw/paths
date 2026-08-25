//! Dock Layout Manager — Manages dynamic stacking and margin layout for floating UI bars.

use crate::plugins::manifest::BarPosition;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct DockItem {
    pub _id: String,
    pub widget: gtk4::Widget,
    pub position: BarPosition,
    pub priority: u32,
    pub thickness: i32,
    pub base_margin: i32,
    pub visible: bool,
}

#[derive(Clone, Default)]
pub struct DockLayoutManager {
    items: Rc<RefCell<HashMap<String, DockItem>>>,
}

impl DockLayoutManager {
    pub fn new() -> Self {
        Self {
            items: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn register(
        &self,
        id: impl Into<String>,
        widget: &impl IsA<gtk4::Widget>,
        position: BarPosition,
        priority: u32,
        thickness: i32,
        base_margin: i32,
    ) {
        let id_str = id.into();
        let item = DockItem {
            _id: id_str.clone(),
            widget: widget.clone().upcast(),
            position,
            priority,
            thickness,
            base_margin,
            visible: widget.is_visible(),
        };
        self.items.borrow_mut().insert(id_str, item);
        self.update_layout();
    }

    pub fn update_position(&self, id: &str, position: BarPosition) {
        if let Some(item) = self.items.borrow_mut().get_mut(id) {
            item.position = position;
        }
        self.update_layout();
    }

    pub fn update_position_and_base_margin(&self, id: &str, position: BarPosition, base_margin: i32) {
        if let Some(item) = self.items.borrow_mut().get_mut(id) {
            item.position = position;
            item.base_margin = base_margin;
        }
        self.update_layout();
    }

    pub fn update_visibility(&self, id: &str, visible: bool) {
        if let Some(item) = self.items.borrow_mut().get_mut(id) {
            item.visible = visible;
        }
        self.update_layout();
    }

    pub fn update_layout(&self) {
        let items = self.items.borrow();

        for &pos in &[
            BarPosition::Top,
            BarPosition::Bottom,
            BarPosition::Left,
            BarPosition::Right,
        ] {
            let mut edge_items: Vec<_> = items
                .values()
                .filter(|item| item.position == pos && item.visible && item.widget.is_visible())
                .collect();

            // Sort by priority (lower priority = closer to edge)
            edge_items.sort_by_key(|item| item.priority);

            let gap = 12;
            let mut current_offset = 0;

            for (idx, item) in edge_items.iter().enumerate() {
                let margin = if idx == 0 {
                    item.base_margin
                } else {
                    current_offset + gap
                };

                match pos {
                    BarPosition::Top => {
                        item.widget.set_halign(gtk4::Align::Center);
                        item.widget.set_valign(gtk4::Align::Start);
                        item.widget.set_margin_top(margin);
                        item.widget.set_margin_bottom(0);
                    }
                    BarPosition::Bottom => {
                        item.widget.set_halign(gtk4::Align::Center);
                        item.widget.set_valign(gtk4::Align::End);
                        item.widget.set_margin_bottom(margin);
                        item.widget.set_margin_top(0);
                    }
                    BarPosition::Left => {
                        item.widget.set_halign(gtk4::Align::Start);
                        item.widget.set_valign(gtk4::Align::Center);
                        item.widget.set_margin_start(margin);
                        item.widget.set_margin_end(0);
                    }
                    BarPosition::Right => {
                        item.widget.set_halign(gtk4::Align::End);
                        item.widget.set_valign(gtk4::Align::Center);
                        item.widget.set_margin_end(margin);
                        item.widget.set_margin_start(0);
                    }
                }

                let measured_thickness = match pos {
                    BarPosition::Top | BarPosition::Bottom => {
                        let h = item.widget.height();
                        if h > 10 { h } else { item.thickness }
                    }
                    BarPosition::Left | BarPosition::Right => {
                        let w = item.widget.width();
                        if w > 10 { w } else { item.thickness }
                    }
                };

                current_offset = margin + measured_thickness;
            }
        }
    }
}
