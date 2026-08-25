pub mod alignment;
pub mod appearance;
pub mod catalog;
pub mod clones;
pub mod dock;
pub mod export;
pub mod floating;
pub mod libraries;
pub mod swatch;
pub mod transform;

use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use self::alignment::build_alignment_section;
use self::appearance::{rebuild_fill_list, rebuild_stroke_list};
use self::clones::build_clones_section;
use self::export::build_export_tab;
use self::libraries::build_libraries_section;
use self::swatch::PillSlider;
use self::transform::build_transform_section;
use crate::core::{Color, FillLayer, StrokeLayer};
use crate::ui::canvas::CanvasWidget;

// ─────────────────────────────────────────────────────────────
// Studio Modular Tab Bar & Split Paned Docking System
// ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabLocation {
    Docked(usize),
    Floating,
    Closed,
}

// ─────────────────────────────────────────────────────────────
// Inspector Sidebar (public API)
// ─────────────────────────────────────────────────────────────

pub struct InspectorSidebar {
    toolbar_view: adw::ToolbarView,
    align_box: gtk4::Box,
    fill_list_box: gtk4::Box,
    fill_sep: gtk4::Separator,
    stroke_list_box: gtk4::Box,
    stroke_sep: gtk4::Separator,
    x_entry: gtk4::Entry,
    y_entry: gtk4::Entry,
    w_entry: gtk4::Entry,
    h_entry: gtk4::Entry,
    unit_dd: gtk4::DropDown,
    blend_dd: gtk4::DropDown,
    blur_slider: PillSlider,
    opacity_slider: PillSlider,
    fills: Rc<RefCell<Vec<FillLayer>>>,
    strokes: Rc<RefCell<Vec<StrokeLayer>>>,
    canvas: CanvasWidget,
    is_updating: Rc<Cell<bool>>,
    convert_path_row: adw::ActionRow,

    update_clones_section: Rc<dyn Fn()>,
    update_export_pages: Rc<dyn Fn()>,
}

impl InspectorSidebar {
    pub fn new(
        canvas: CanvasWidget,
        main_win_holder: Rc<RefCell<Option<adw::ApplicationWindow>>>,
    ) -> Self {
        let is_updating = Rc::new(Cell::new(false));

        let toolbar_view = adw::ToolbarView::builder()
            .width_request(260)
            .css_classes(["sidebar"])
            .build();

        let header_bar = adw::HeaderBar::builder()
            .show_start_title_buttons(false)
            .show_end_title_buttons(false)
            .build();

        let title_lbl = gtk4::Label::builder()
            .label(crate::core::gettext("Panels"))
            .css_classes(["heading"])
            .build();
        header_bar.set_title_widget(Some(&title_lbl));

        let header_add_btn = gtk4::MenuButton::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text(crate::core::gettext("Add Panel"))
            .css_classes(["flat"])
            .valign(gtk4::Align::Center)
            .focus_on_click(false)
            .visible(false)
            .build();
        header_bar.pack_end(&header_add_btn);

        toolbar_view.add_top_bar(&header_bar);

        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .vexpand(true)
            .hexpand(true)
            .build();

        // 1. ALIGNMENT SECTION
        let align_sec = build_alignment_section(&canvas);
        let align_box = align_sec.align_box;
        let align_container = align_sec.container;

        // 2. MAIN STUDIO TAB BAR & DYNAMIC DOCK SECTIONS LAYOUT
        let sections_container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .hexpand(true)
            .build();

        let sections_scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .hexpand(true)
            .child(&sections_container)
            .build();

        let drop_preview_top_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Center)
            .css_classes(["dock-preview-overlay"])
            .build();
        drop_preview_top_box.append(&gtk4::Image::from_icon_name("sidebar-inspector-symbolic"));
        drop_preview_top_box.append(&gtk4::Label::new(Some(&crate::core::gettext(
            "Drop to Dock at Top",
        ))));

        let drop_preview_top = gtk4::Revealer::builder()
            .transition_type(gtk4::RevealerTransitionType::SlideDown)
            .reveal_child(false)
            .child(&drop_preview_top_box)
            .build();

        let drop_preview_bottom_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Center)
            .css_classes(["dock-preview-overlay"])
            .build();
        drop_preview_bottom_box.append(&gtk4::Image::from_icon_name("layer-move-down-symbolic"));
        drop_preview_bottom_box.append(&gtk4::Label::new(Some(&crate::core::gettext(
            "Drop to Create New Section Below",
        ))));

        let drop_preview_bottom = gtk4::Revealer::builder()
            .transition_type(gtk4::RevealerTransitionType::SlideUp)
            .reveal_child(false)
            .child(&drop_preview_bottom_box)
            .build();

        container.append(&drop_preview_top);
        container.append(&sections_scrolled);
        container.append(&drop_preview_bottom);

        // 3. APPEARANCE (Fills, Strokes, Blend Mode & Sliders)
        let fill_stroke_body = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(8)
            .margin_top(4)
            .margin_bottom(8)
            .build();

        // ── PREENCHIMENTO (Fill Card) ──
        let fill_card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .margin_start(12)
            .margin_end(12)
            .css_classes(["card"])
            .build();

        let fill_header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .css_classes(["card-header-bar"])
            .valign(gtk4::Align::Center)
            .build();
        let fill_img = crate::ui::icons::make_symbolic_image("tool-square", 14);
        fill_header.append(&fill_img);
        fill_header.append(
            &gtk4::Label::builder()
                .label(crate::core::gettext("Fill"))
                .css_classes(["heading", "caption"])
                .build(),
        );

        let fill_h_spacer = gtk4::Box::builder().hexpand(true).build();
        fill_header.append(&fill_h_spacer);

        let fill_eyedropper_btn = gtk4::Button::builder()
            .icon_name("color-picker-symbolic")
            .css_classes(["flat", "circular"])
            .tooltip_text(crate::core::gettext("Eyedropper (I)"))
            .valign(gtk4::Align::Center)
            .build();
        {
            let canvas_eye = canvas.clone();
            fill_eyedropper_btn.connect_clicked(move |_| {
                canvas_eye.set_active_tool("eyedropper");
            });
        }
        fill_header.append(&fill_eyedropper_btn);

        fill_card.append(&fill_header);

        let fill_header_sep = gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .build();
        fill_card.append(&fill_header_sep);

        let fill_list_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .build();
        fill_card.append(&fill_list_box);

        let fill_sep = gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .build();
        fill_card.append(&fill_sep);

        let add_fill_btn = gtk4::Button::builder()
            .css_classes(["flat", "card-action-btn"])
            .hexpand(true)
            .halign(gtk4::Align::Fill)
            .build();
        let add_fill_content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();
        add_fill_content.append(&gtk4::Image::from_icon_name("list-add-symbolic"));
        add_fill_content.append(&gtk4::Label::new(Some(&crate::core::gettext("Add Fill"))));
        add_fill_btn.set_child(Some(&add_fill_content));
        fill_card.append(&add_fill_btn);

        fill_stroke_body.append(&fill_card);

        // ── CONTORNO (Stroke Card) ──
        let stroke_card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .margin_start(12)
            .margin_end(12)
            .css_classes(["card"])
            .build();

        let stroke_header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .css_classes(["card-header-bar"])
            .valign(gtk4::Align::Center)
            .build();
        let stroke_img = crate::ui::icons::make_symbolic_image("format-stroke", 14);
        stroke_header.append(&stroke_img);
        stroke_header.append(
            &gtk4::Label::builder()
                .label(crate::core::gettext("Stroke"))
                .css_classes(["heading", "caption"])
                .build(),
        );
        let stroke_h_spacer = gtk4::Box::builder().hexpand(true).build();
        stroke_header.append(&stroke_h_spacer);

        let stroke_eyedropper_btn = gtk4::Button::builder()
            .icon_name("color-picker-symbolic")
            .css_classes(["flat", "circular"])
            .tooltip_text(crate::core::gettext("Eyedropper (I)"))
            .valign(gtk4::Align::Center)
            .build();
        {
            let canvas_eye_s = canvas.clone();
            stroke_eyedropper_btn.connect_clicked(move |_| {
                canvas_eye_s.set_active_tool("eyedropper");
            });
        }
        stroke_header.append(&stroke_eyedropper_btn);

        stroke_card.append(&stroke_header);

        let stroke_header_sep = gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .build();
        stroke_card.append(&stroke_header_sep);

        let stroke_list_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .build();
        stroke_card.append(&stroke_list_box);

        let stroke_sep = gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .build();
        stroke_card.append(&stroke_sep);

        let add_stroke_btn = gtk4::Button::builder()
            .css_classes(["flat", "card-action-btn"])
            .hexpand(true)
            .halign(gtk4::Align::Fill)
            .build();
        let add_stroke_content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();
        add_stroke_content.append(&gtk4::Image::from_icon_name("list-add-symbolic"));
        add_stroke_content.append(&gtk4::Label::new(Some(&crate::core::gettext("Add Stroke"))));
        add_stroke_btn.set_child(Some(&add_stroke_content));
        stroke_card.append(&add_stroke_btn);

        fill_stroke_body.append(&stroke_card);

        // ── MODO DE MESCLAGEM & EFEITOS (Blend Mode Card) ──
        let blend_card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_bottom(8)
            .css_classes(["card"])
            .build();

        // Row 1: Header [ Modo de Mesclagem ] ──────── [ Normal ▾ ]
        let blend_header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .margin_top(8)
            .margin_bottom(4)
            .margin_start(10)
            .margin_end(10)
            .build();
        let blend_icon = crate::ui::icons::make_symbolic_image("blend-mode", 16);
        blend_icon.set_valign(gtk4::Align::Center);
        blend_icon.set_opacity(0.8);
        let blend_lbl = gtk4::Label::builder()
            .label(crate::core::gettext("Blend Mode"))
            .css_classes(["heading", "caption"])
            .build();
        let blend_spacer = gtk4::Box::builder().hexpand(true).build();
        let blend_model = gtk4::StringList::new(&[
            &crate::core::gettext("Normal"),
            &crate::core::gettext("Multiply"),
            &crate::core::gettext("Screen"),
            &crate::core::gettext("Overlay"),
            &crate::core::gettext("Darken"),
            &crate::core::gettext("Lighten"),
            &crate::core::gettext("Color Dodge"),
            &crate::core::gettext("Color Burn"),
            &crate::core::gettext("Hard Light"),
            &crate::core::gettext("Soft Light"),
            &crate::core::gettext("Difference"),
            &crate::core::gettext("Exclusion"),
            &crate::core::gettext("Hue"),
            &crate::core::gettext("Saturation"),
            &crate::core::gettext("Color"),
            &crate::core::gettext("Luminosity"),
        ]);
        let blend_dd = gtk4::DropDown::builder()
            .model(&blend_model)
            .selected(0)
            .css_classes(["flat"])
            .valign(gtk4::Align::Center)
            .build();
        blend_header.append(&blend_lbl);
        blend_header.append(&blend_spacer);
        blend_header.append(&blend_dd);

        blend_card.append(&blend_header);

        let blend_header_sep = gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .margin_top(2)
            .margin_bottom(4)
            .build();
        blend_card.append(&blend_header_sep);

        // Row 2: Full-width Blur (%) Interactive Pill Slider
        let blur_slider = PillSlider::new(&crate::core::gettext("Blur"), 0.0);
        blend_card.append(blur_slider.widget());

        // Row 3: Full-width Opacity (%) Interactive Pill Slider
        let opacity_slider = PillSlider::new(&crate::core::gettext("Opacity"), 1.0);
        opacity_slider.widget().set_margin_bottom(10);
        blend_card.append(opacity_slider.widget());

        // Wire Blend Mode changes
        let canvas_blend = canvas.clone();
        let upd_blend = is_updating.clone();
        blend_dd.connect_selected_notify(move |dd| {
            if upd_blend.get() {
                return;
            }
            let sel = dd.selected();
            let mode = crate::core::BlendMode::from_index(sel);
            canvas_blend.set_selected_blend_mode(mode);
        });

        // Wire Blur changes
        let canvas_blur = canvas.clone();
        let upd_blur = is_updating.clone();
        blur_slider.set_on_change(move |val| {
            if upd_blur.get() {
                return;
            }
            canvas_blur.set_selected_blur(val as f32);
        });

        // Wire Opacity changes
        let canvas_op = canvas.clone();
        let upd_op = is_updating.clone();
        opacity_slider.set_on_change(move |val| {
            if upd_op.get() {
                return;
            }
            canvas_op.set_selected_opacity(val as f32);
        });

        fill_stroke_body.append(&blend_card);

        // 4. TRANSFORM SECTION
        let trans_sec = build_transform_section(&canvas);
        let x_entry = trans_sec.x_entry;
        let y_entry = trans_sec.y_entry;
        let w_entry = trans_sec.w_entry;
        let h_entry = trans_sec.h_entry;
        let unit_dd = trans_sec.unit_dd;
        let convert_path_row = trans_sec.convert_path_row;
        let transform_body = trans_sec.container;

        // 4. CLONES SECTION
        let (clones_body, update_clones_section) = build_clones_section(&canvas);

        // 5. EXPORT SECTION
        let (export_body, update_export_pages) = build_export_tab(&canvas, &main_win_holder);

        // 6. LIBRARIES SECTION
        let libraries_body = build_libraries_section(&canvas);

        // Setup Dynamic Multi-Section Tab Bar & Split System
        let tab_locations = Rc::new(RefCell::new([
            TabLocation::Docked(0), // 0: Appearance
            TabLocation::Docked(0), // 1: Alignment
            TabLocation::Closed,    // 2: Transform
            TabLocation::Closed,    // 3: Clones
            TabLocation::Closed,    // 4: Export
            TabLocation::Closed,    // 5: Libraries (Closed by default)
        ]));
        let tab_order = Rc::new(RefCell::new(vec![0usize, 1usize, 2usize, 3usize, 4usize, 5usize]));
        let active_section_tabs = Rc::new(RefCell::new([0usize, 1usize, 2usize, 3usize, 4usize, 5usize]));
        let floating_wins: Rc<RefCell<[Option<adw::Window>; 6]>> =
            Rc::new(RefCell::new([None, None, None, None, None, None]));

        let tab_widgets: [gtk4::Widget; 6] = [
            fill_stroke_body.upcast(),
            align_container.upcast(),
            transform_body.upcast(),
            clones_body,
            export_body,
            libraries_body,
        ];

        let refresh_tabs: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        // Connect GtkDropTarget to the Inspector container for dynamic cross-window tab docking
        {
            let locs_drop = tab_locations.clone();
            let act_sec_tabs_d = active_section_tabs.clone();
            let pv_top_c = drop_preview_top.clone();
            let pv_bot_c = drop_preview_bottom.clone();
            let re_drop = refresh_tabs.clone();
            let container_weak = container.downgrade();

            let drop_target =
                gtk4::DropTarget::new(glib::types::Type::STRING, gdk::DragAction::MOVE);

            drop_target.connect_motion(move |_, _, y| {
                if let Some(cont) = container_weak.upgrade() {
                    let h = cont.height() as f64;
                    if y < 80.0 {
                        pv_top_c.set_reveal_child(true);
                        pv_bot_c.set_reveal_child(false);
                    } else if y > h - 100.0 {
                        pv_top_c.set_reveal_child(false);
                        pv_bot_c.set_reveal_child(true);
                    } else {
                        pv_top_c.set_reveal_child(false);
                        pv_bot_c.set_reveal_child(false);
                    }
                }
                gdk::DragAction::MOVE
            });

            let pv_top_l = drop_preview_top.clone();
            let pv_bot_l = drop_preview_bottom.clone();
            drop_target.connect_leave(move |_| {
                pv_top_l.set_reveal_child(false);
                pv_bot_l.set_reveal_child(false);
            });

            let pv_top_d = drop_preview_top.clone();
            let pv_bot_d = drop_preview_bottom.clone();
            let container_weak_d = container.downgrade();
            drop_target.connect_drop(move |_, value, _, y| {
                pv_top_d.set_reveal_child(false);
                pv_bot_d.set_reveal_child(false);
                if let Ok(s) = value.get::<String>() {
                    if let Some(idx_str) = s.strip_prefix("tab:") {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            if idx < 6 {
                                let cur_locs = *locs_drop.borrow();
                                let max_sec = (0..6)
                                    .filter_map(|k| {
                                        if let TabLocation::Docked(s) = cur_locs[k] {
                                            Some(s)
                                        } else {
                                            None
                                        }
                                    })
                                    .max()
                                    .unwrap_or(0);

                                if let Some(cont) = container_weak_d.upgrade() {
                                    let h = cont.height() as f64;
                                    if y < 80.0 {
                                        // Insert section at top
                                        let mut new_locs = cur_locs;
                                        for k in 0..6 {
                                            if let TabLocation::Docked(s) = new_locs[k] {
                                                new_locs[k] = TabLocation::Docked(s + 1);
                                            }
                                        }
                                        new_locs[idx] = TabLocation::Docked(0);
                                        *locs_drop.borrow_mut() = new_locs;
                                        act_sec_tabs_d.borrow_mut()[0] = idx;
                                    } else if y > h - 100.0 {
                                        // Create new section at bottom
                                        locs_drop.borrow_mut()[idx] =
                                            TabLocation::Docked(max_sec + 1);
                                        if max_sec + 1 < act_sec_tabs_d.borrow().len() {
                                            act_sec_tabs_d.borrow_mut()[max_sec + 1] = idx;
                                        }
                                    } else {
                                        // Dock into top section
                                        locs_drop.borrow_mut()[idx] = TabLocation::Docked(0);
                                        act_sec_tabs_d.borrow_mut()[0] = idx;
                                    }
                                } else {
                                    locs_drop.borrow_mut()[idx] = TabLocation::Docked(0);
                                    act_sec_tabs_d.borrow_mut()[0] = idx;
                                }

                                if let Some(ref re) = *re_drop.borrow() {
                                    re();
                                }
                                return true;
                            }
                        }
                    }
                }
                false
            });

            container.add_controller(drop_target);
        }

        let refresh_tabs_impl = {
            let tab_locations = tab_locations.clone();
            let tab_order = tab_order.clone();
            let active_section_tabs = active_section_tabs.clone();
            let floating_wins = floating_wins.clone();
            let tab_widgets = tab_widgets.clone();
            let sections_container = sections_container.clone();
            let refresh_tabs_cell = refresh_tabs.clone();
            let main_win_holder = main_win_holder.clone();
            let canvas_c = canvas.clone();

            Rc::new(move || {
                let tab_info: [(String, Option<&'static str>, &'static str); 6] = [
                    (
                        crate::core::gettext("Appearance"),
                        Some("/io/github/lewis/GnomePaths/icons/panel-appearance.svg"),
                        "panel-appearance-symbolic",
                    ),
                    (
                        crate::core::gettext("Alignment"),
                        None,
                        "text-align-left-symbolic",
                    ),
                    (
                        crate::core::gettext("Transform"),
                        Some("/io/github/lewis/GnomePaths/icons/transform.svg"),
                        "transform-symbolic",
                    ),
                    (
                        crate::core::gettext("Clones"),
                        Some("/io/github/lewis/GnomePaths/icons/clone.svg"),
                        "check-symbolic",
                    ),
                    (
                        crate::core::gettext("Export"),
                        None,
                        "document-save-symbolic",
                    ),
                    (
                        crate::core::gettext("Libraries"),
                        None,
                        "clone-master-symbolic",
                    ),
                ];

                let locs = *tab_locations.borrow();

                // 1. Clean up floating windows that are no longer floating before building docked sections
                for i in 0..6 {
                    if locs[i] != TabLocation::Floating {
                        let maybe_w = floating_wins.borrow_mut()[i].take();
                        if let Some(w) = maybe_w {
                            tab_widgets[i].unparent();
                            w.set_content(Option::<&gtk4::Widget>::None);
                            w.destroy();
                        }
                    }
                }

                // 2. Render multi-section dock
                dock::render_dock_sections(
                    &tab_info,
                    &tab_locations,
                    &active_section_tabs,
                    &tab_order,
                    &tab_widgets,
                    &sections_container,
                    &header_add_btn,
                    &canvas_c,
                    refresh_tabs_cell.borrow().as_ref().unwrap().clone(),
                );

                // 5. Floating Windows Management
                for i in 0..6 {
                    if locs[i] == TabLocation::Floating {
                        if floating_wins.borrow()[i].is_none() {
                            let (title, icon_res, icon_name) = &tab_info[i];
                            let float_win = floating::create_floating_window(
                                i,
                                title,
                                *icon_res,
                                *icon_name,
                                &tab_widgets[i],
                                &tab_locations,
                                &active_section_tabs,
                                refresh_tabs_cell.borrow().as_ref().unwrap().clone(),
                                &main_win_holder,
                            );
                            floating_wins.borrow_mut()[i] = Some(float_win.clone());
                            float_win.present();
                        }
                    }
                }
            })
        };

        *refresh_tabs.borrow_mut() = Some(refresh_tabs_impl.clone());

        // Initial render of tabs
        refresh_tabs_impl();

        // Default data
        let fills: Rc<RefCell<Vec<FillLayer>>> = Rc::new(RefCell::new(vec![]));
        let strokes: Rc<RefCell<Vec<StrokeLayer>>> = Rc::new(RefCell::new(vec![]));

        let main_stack = adw::ViewStack::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        main_stack.add_titled_with_icon(
            &container,
            Some("properties"),
            &crate::core::gettext("Properties"),
            "prefs-toolbars-symbolic",
        );

        let tab_switcher = adw::ViewSwitcher::builder()
            .stack(&main_stack)
            .policy(adw::ViewSwitcherPolicy::Narrow)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(8)
            .margin_end(8)
            .halign(gtk4::Align::Fill)
            .visible(false)
            .build();

        let root_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .vexpand(true)
            .hexpand(true)
            .build();
        root_box.append(&tab_switcher);
        root_box.append(&main_stack);

        toolbar_view.set_content(Some(&root_box));

        let sidebar = Self {
            toolbar_view,
            align_box,
            fill_list_box,
            fill_sep,
            stroke_list_box,
            stroke_sep,
            x_entry,
            y_entry,
            w_entry,
            h_entry,
            unit_dd,
            blend_dd,
            blur_slider,
            opacity_slider,
            fills,
            strokes,
            canvas: canvas.clone(),
            is_updating,
            convert_path_row,
            update_clones_section,
            update_export_pages,
        };

        sidebar.rebuild_fill_rows();
        sidebar.rebuild_stroke_rows();

        // Wire "Add" buttons
        {
            let fills = sidebar.fills.clone();
            let fill_list_box = sidebar.fill_list_box.clone();
            let fill_sep = sidebar.fill_sep.clone();
            let updating = sidebar.is_updating.clone();
            let strokes = sidebar.strokes.clone();
            let stroke_list_box = sidebar.stroke_list_box.clone();
            let stroke_sep = sidebar.stroke_sep.clone();

            let fills_for_rebuild = fills.clone();
            let fl_box = fill_list_box.clone();
            let fl_sep = fill_sep.clone();
            let canvas_for_fill_rebuild = canvas.clone();
            let upd_for_fill = updating.clone();

            let strokes_for_rebuild = strokes.clone();
            let sl_box = stroke_list_box.clone();
            let sl_sep = stroke_sep.clone();
            let canvas_for_stroke_rebuild = canvas.clone();
            let upd_for_stroke = updating.clone();

            let rebuild_fills: Rc<dyn Fn()> = Rc::new(move || {
                rebuild_fill_list(
                    &fl_box,
                    &fills_for_rebuild,
                    &canvas_for_fill_rebuild,
                    &upd_for_fill,
                    &fl_sep,
                );
            });

            let rebuild_strokes: Rc<dyn Fn()> = Rc::new(move || {
                rebuild_stroke_list(
                    &sl_box,
                    &strokes_for_rebuild,
                    &canvas_for_stroke_rebuild,
                    &upd_for_stroke,
                    &sl_sep,
                );
            });

            // Wire fill list initial render
            rebuild_fills();
            rebuild_strokes();

            let fills_add = fills.clone();
            let rebuild_f = rebuild_fills.clone();
            let canvas_f = canvas.clone();
            add_fill_btn.connect_clicked(move |_| {
                let col = canvas_f.active_fill_color();
                let mut l = fills_add.borrow_mut();
                l.push(FillLayer::new(col));
                let cloned = l.clone();
                drop(l);
                canvas_f.set_selected_fills(cloned);
                rebuild_f();
            });

            let strokes_add = strokes.clone();
            let rebuild_s = rebuild_strokes.clone();
            let canvas_s = canvas.clone();
            add_stroke_btn.connect_clicked(move |_| {
                let col = canvas_s.active_stroke_color().unwrap_or(Color::BLACK);
                let w = canvas_s.active_stroke_width();
                let mut l = strokes_add.borrow_mut();
                l.push(StrokeLayer::new(col, w));
                let cloned = l.clone();
                drop(l);
                canvas_s.set_selected_strokes(cloned);
                rebuild_s();
            });
        }

        sidebar
    }

    fn rebuild_fill_rows(&self) {
        rebuild_fill_list(
            &self.fill_list_box,
            &self.fills,
            &self.canvas,
            &self.is_updating,
            &self.fill_sep,
        );
    }

    fn rebuild_stroke_rows(&self) {
        rebuild_stroke_list(
            &self.stroke_list_box,
            &self.strokes,
            &self.canvas,
            &self.is_updating,
            &self.stroke_sep,
        );
    }

    pub fn widget(&self) -> &adw::ToolbarView {
        &self.toolbar_view
    }

    pub fn refresh_export_pages(&self) {
        (self.update_export_pages)();
    }

    pub fn refresh_clones_section(&self) {
        (self.update_clones_section)();
    }

    pub fn update_context(
        &self,
        _tool_id: &str,
        selected_count: usize,
        bounds: Option<crate::core::Rect>,
        _style: (Option<Color>, Option<Color>, f32),
        can_convert_to_path: bool,
        fills_and_strokes: Option<(Vec<crate::core::FillLayer>, Vec<crate::core::StrokeLayer>)>,
        blend_info: Option<(crate::core::BlendMode, f32, f32)>,
    ) {
        let has_selection = selected_count >= 1;
        self.refresh_export_pages();
        self.refresh_clones_section();

        // Keep inspector functional and interactive
        self.align_box.set_sensitive(has_selection);
        self.convert_path_row.set_visible(can_convert_to_path);

        let cur_unit = self.canvas.unit();
        let cur_unit_idx = cur_unit.to_index();
        if self.unit_dd.selected() != cur_unit_idx {
            self.unit_dd.set_selected(cur_unit_idx);
        }

        // Sync Geometry entries in Transform tab only when values actually change
        if let Some(r) = bounds {
            if !self.x_entry.has_focus() {
                let s = cur_unit.format(r.x);
                if self.x_entry.text().as_str() != s.as_str() {
                    self.x_entry.set_text(&s);
                }
            }
            if !self.y_entry.has_focus() {
                let s = cur_unit.format(r.y);
                if self.y_entry.text().as_str() != s.as_str() {
                    self.y_entry.set_text(&s);
                }
            }
            if !self.w_entry.has_focus() {
                let s = cur_unit.format(r.width);
                if self.w_entry.text().as_str() != s.as_str() {
                    self.w_entry.set_text(&s);
                }
            }
            if !self.h_entry.has_focus() {
                let s = cur_unit.format(r.height);
                if self.h_entry.text().as_str() != s.as_str() {
                    self.h_entry.set_text(&s);
                }
            }
        } else {
            if !self.x_entry.has_focus() && !self.x_entry.text().is_empty() {
                self.x_entry.set_text("");
            }
            if !self.y_entry.has_focus() && !self.y_entry.text().is_empty() {
                self.y_entry.set_text("");
            }
            if !self.w_entry.has_focus() && !self.w_entry.text().is_empty() {
                self.w_entry.set_text("");
            }
            if !self.h_entry.has_focus() && !self.h_entry.text().is_empty() {
                self.h_entry.set_text("");
            }
        }

        // Sync fill/stroke from selection
        if self.is_updating.get() {
            return;
        }
        self.is_updating.set(true);

        // Sync blend mode, blur, and opacity from selection
        if let Some((blend_mode, blur, opacity)) = blend_info {
            let idx = blend_mode.to_index();
            if self.blend_dd.selected() != idx {
                self.blend_dd.set_selected(idx);
            }
            if (self.blur_slider.value() - blur as f64).abs() > 0.001 {
                self.blur_slider.set_value(blur as f64);
            }
            if (self.opacity_slider.value() - opacity as f64).abs() > 0.001 {
                self.opacity_slider.set_value(opacity as f64);
            }
        } else {
            if self.blend_dd.selected() != 0 {
                self.blend_dd.set_selected(0);
            }
            if self.blur_slider.value() > 0.001 {
                self.blur_slider.set_value(0.0);
            }
            if (self.opacity_slider.value() - 1.0).abs() > 0.001 {
                self.opacity_slider.set_value(1.0);
            }
        }

        // Sync all fills and strokes without unnecessarily destroying rows when contents are identical
        if has_selection {
            if let Some((sel_fills, sel_strokes)) = fills_and_strokes {
                let fills_changed = *self.fills.borrow() != sel_fills;
                let strokes_changed = *self.strokes.borrow() != sel_strokes;

                if fills_changed {
                    *self.fills.borrow_mut() = sel_fills;
                    if !appearance::is_any_popover_visible(self.fill_list_box.upcast_ref()) {
                        self.rebuild_fill_rows();
                    }
                }
                if strokes_changed {
                    *self.strokes.borrow_mut() = sel_strokes;
                    if !appearance::is_any_popover_visible(self.stroke_list_box.upcast_ref()) {
                        self.rebuild_stroke_rows();
                    }
                }
            } else {
                let was_not_empty =
                    !self.fills.borrow().is_empty() || !self.strokes.borrow().is_empty();
                self.fills.borrow_mut().clear();
                self.strokes.borrow_mut().clear();
                if was_not_empty {
                    self.rebuild_fill_rows();
                    self.rebuild_stroke_rows();
                }
            }
        } else {
            let was_not_empty =
                !self.fills.borrow().is_empty() || !self.strokes.borrow().is_empty();
            self.fills.borrow_mut().clear();
            self.strokes.borrow_mut().clear();
            if was_not_empty {
                self.rebuild_fill_rows();
                self.rebuild_stroke_rows();
            }
        }

        self.is_updating.set(false);
    }
}
