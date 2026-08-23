pub mod about;
pub mod appearance;
pub mod canvas;
pub mod general;
pub mod node_editor;
pub mod plugins;
pub mod shortcuts;
pub mod toolbars;
pub mod widgets;

use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use self::about::build_about_page;
use self::appearance::build_appearance_page;
use self::canvas::build_canvas_page;
use self::general::build_general_page;
use self::node_editor::build_node_editor_page;
use self::plugins::build_plugins_page;
use self::shortcuts::build_shortcuts_page;
use self::toolbars::build_toolbars_page;
use crate::ui::canvas::CanvasWidget;

pub fn show_preferences_window(parent: &impl IsA<gtk4::Widget>, canvas: CanvasWidget) {
    let window = adw::Window::builder()
        .title(crate::core::gettext("Preferences"))
        .modal(true)
        .default_width(960)
        .default_height(680)
        .build();

    if let Some(root) = parent.root() {
        if let Some(win) = root.downcast_ref::<gtk4::Window>() {
            window.set_transient_for(Some(win));
        }
    }

    // ─────────────────────────────────────────────────────────────
    // LEFT SIDEBAR (Standard Libadwaita Navigation Sidebar)
    // ─────────────────────────────────────────────────────────────
    let sidebar_toolbar = adw::ToolbarView::new();

    let sidebar_header = adw::HeaderBar::builder()
        .show_title(true)
        .title_widget(&adw::WindowTitle::new(
            &crate::core::gettext("Preferences"),
            "",
        ))
        .show_start_title_buttons(true)
        .show_end_title_buttons(false)
        .build();
    sidebar_toolbar.add_top_bar(&sidebar_header);

    let sidebar_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(6)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let nav_list = gtk4::ListBox::builder()
        .css_classes(["navigation-sidebar"])
        .selection_mode(gtk4::SelectionMode::Single)
        .build();

    struct NavSection {
        id: &'static str,
        title: String,
        icon_resource: Option<&'static str>,
        icon_name: &'static str,
    }

    let nav_sections = [
        NavSection {
            id: "general",
            title: crate::core::gettext("General and System"),
            icon_resource: None,
            icon_name: "preferences-system-symbolic",
        },
        NavSection {
            id: "appearance",
            title: crate::core::gettext("Appearance"),
            icon_resource: Some("/io/github/lewis/GnomePaths/icons/panel-appearance.svg"),
            icon_name: "panel-appearance-symbolic",
        },
        NavSection {
            id: "canvas",
            title: crate::core::gettext("Canvas and Grid"),
            icon_resource: None,
            icon_name: "preferences-desktop-display-symbolic",
        },
        NavSection {
            id: "node_editor",
            title: crate::core::gettext("Path & Node Editor"),
            icon_resource: Some("/io/github/lewis/GnomePaths/icons/tool-path-editor.svg"),
            icon_name: "edit-symbolic",
        },
        NavSection {
            id: "toolbars",
            title: crate::core::gettext("Toolbars"),
            icon_resource: None,
            icon_name: "view-grid-symbolic",
        },
        NavSection {
            id: "shortcuts",
            title: crate::core::gettext("Keyboard Shortcuts"),
            icon_resource: None,
            icon_name: "input-keyboard-symbolic",
        },
        NavSection {
            id: "plugins",
            title: crate::core::gettext("Plugins"),
            icon_resource: None,
            icon_name: "system-software-install-symbolic",
        },
        NavSection {
            id: "about",
            title: crate::core::gettext("About"),
            icon_resource: None,
            icon_name: "help-about-symbolic",
        },
    ];

    for section in &nav_sections {
        let row = gtk4::ListBoxRow::builder()
            .activatable(true)
            .selectable(true)
            .build();

        let r_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(10)
            .margin_bottom(10)
            .valign(gtk4::Align::Center)
            .build();

        let icon = if let Some(res) = section.icon_resource {
            crate::ui::icons::make_symbolic_image(res, 18)
        } else {
            crate::ui::icons::make_symbolic_image(section.icon_name, 18)
        };

        let lbl = gtk4::Label::builder()
            .label(&section.title)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        r_box.append(&icon);
        r_box.append(&lbl);
        row.set_child(Some(&r_box));
        nav_list.append(&row);
    }

    sidebar_box.append(&nav_list);

    let nav_scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .child(&sidebar_box)
        .build();
    sidebar_toolbar.set_content(Some(&nav_scroll));

    // ─────────────────────────────────────────────────────────────
    // RIGHT CONTENT PANE
    // ─────────────────────────────────────────────────────────────
    let content_toolbar = adw::ToolbarView::new();

    let content_title_widget =
        adw::WindowTitle::new(&crate::core::gettext("General and System"), "");
    let content_header = adw::HeaderBar::builder()
        .show_title(true)
        .title_widget(&content_title_widget)
        .show_start_title_buttons(false)
        .show_end_title_buttons(true)
        .build();

    let esc_controller = gtk4::EventControllerKey::new();
    let w_esc = window.clone();
    esc_controller.connect_key_pressed(move |_ctrl, keyval, _code, _state| {
        if keyval == gtk4::gdk::Key::Escape {
            w_esc.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(esc_controller);

    content_toolbar.add_top_bar(&content_header);

    let stack = gtk4::Stack::builder()
        .transition_type(gtk4::StackTransitionType::Crossfade)
        .transition_duration(150)
        .hexpand(true)
        .vexpand(true)
        .build();

    let page_general = build_general_page(&window, &canvas);
    stack.add_named(&page_general, Some("general"));

    let page_appearance = build_appearance_page();
    stack.add_named(&page_appearance, Some("appearance"));

    let page_canvas = build_canvas_page(&canvas);
    stack.add_named(&page_canvas, Some("canvas"));

    let page_node_editor = build_node_editor_page(&canvas);
    stack.add_named(&page_node_editor, Some("node_editor"));

    let page_toolbars = build_toolbars_page(&window, &canvas);
    stack.add_named(&page_toolbars, Some("toolbars"));

    let page_shortcuts = build_shortcuts_page(&window, &canvas);
    stack.add_named(&page_shortcuts, Some("shortcuts"));

    let page_plugins = build_plugins_page(&window, &canvas);
    stack.add_named(&page_plugins, Some("plugins"));

    let page_about = build_about_page();
    stack.add_named(&page_about, Some("about"));

    content_toolbar.set_content(Some(&stack));

    // ─────────────────────────────────────────────────────────────
    // ASSEMBLE WITH ADW NAVIGATION SPLIT VIEW (SIDEBAR ALWAYS VISIBLE)
    // ─────────────────────────────────────────────────────────────
    let split_view = adw::NavigationSplitView::builder()
        .min_sidebar_width(260.0)
        .max_sidebar_width(320.0)
        .sidebar_width_fraction(0.28)
        .collapsed(false)
        .sidebar(
            &adw::NavigationPage::builder()
                .title(crate::core::gettext("Preferences"))
                .child(&sidebar_toolbar)
                .build(),
        )
        .content(
            &adw::NavigationPage::builder()
                .title(crate::core::gettext("Content"))
                .child(&content_toolbar)
                .build(),
        )
        .build();

    // ─────────────────────────────────────────────────────────────
    // WIRE SIDEBAR SELECTION -> STACK & HEADER TITLE
    // ─────────────────────────────────────────────────────────────
    let stack_clone = stack.clone();
    let title_widget_clone = content_title_widget.clone();
    let nav_sections_clone = nav_sections;
    nav_list.connect_row_selected(move |_, row_opt| {
        if let Some(row) = row_opt {
            let idx = row.index() as usize;
            if let Some(sec) = nav_sections_clone.get(idx) {
                stack_clone.set_visible_child_name(sec.id);
                title_widget_clone.set_title(&sec.title);
            }
        }
    });

    if let Some(first_row) = nav_list.row_at_index(0) {
        nav_list.select_row(Some(&first_row));
    }

    window.set_content(Some(&split_view));
    window.present();
}
