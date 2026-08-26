use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::rc::Rc;

use crate::ui::canvas::CanvasWidget;
use crate::ui::window::file_ops::WindowFileOps;
use crate::ui::DesignWindow;

#[derive(Clone, Copy)]
pub struct PresetTemplate {
    pub id: &'static str,
    pub category: &'static str,
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub description: &'static str,
    pub icon_name: &'static str,
    pub aspect_w: i32,
    pub aspect_h: i32,
}

pub const ALL_PRESETS: &[PresetTemplate] = &[
    PresetTemplate {
        id: "social_instagram",
        category: "social",
        name: "Instagram Post",
        width: 1080,
        height: 1080,
        description: "1:1 Square layout (1080 × 1080 px)",
        icon_name: "insert-image-symbolic",
        aspect_w: 70,
        aspect_h: 70,
    },
    PresetTemplate {
        id: "social_twitter",
        category: "social",
        name: "Twitter / X Post",
        width: 1200,
        height: 675,
        description: "16:9 Landscape layout (1200 × 675 px)",
        icon_name: "mail-send-symbolic",
        aspect_w: 100,
        aspect_h: 56,
    },
    PresetTemplate {
        id: "social_facebook",
        category: "social",
        name: "Facebook Banner",
        width: 1200,
        height: 630,
        description: "Standard feed link preview (1200 × 630 px)",
        icon_name: "insert-image-symbolic",
        aspect_w: 100,
        aspect_h: 52,
    },
    PresetTemplate {
        id: "social_linkedin",
        category: "social",
        name: "LinkedIn Update",
        width: 1200,
        height: 627,
        description: "Professional update post (1200 × 627 px)",
        icon_name: "mail-send-symbolic",
        aspect_w: 100,
        aspect_h: 52,
    },
    PresetTemplate {
        id: "wallpaper_fhd",
        category: "wallpaper",
        name: "Desktop Full HD",
        width: 1920,
        height: 1080,
        description: "16:9 Widescreen display (1920 × 1080 px)",
        icon_name: "video-display-symbolic",
        aspect_w: 100,
        aspect_h: 56,
    },
    PresetTemplate {
        id: "wallpaper_4k",
        category: "wallpaper",
        name: "Desktop 4K UHD",
        width: 3840,
        height: 2160,
        description: "4K Ultra High Definition (3840 × 2160 px)",
        icon_name: "video-display-symbolic",
        aspect_w: 100,
        aspect_h: 56,
    },
    PresetTemplate {
        id: "wallpaper_mobile",
        category: "wallpaper",
        name: "Mobile Wallpaper",
        width: 1080,
        height: 1920,
        description: "9:16 Portrait wallpaper (1080 × 1920 px)",
        icon_name: "phone-symbolic",
        aspect_w: 50,
        aspect_h: 90,
    },
    PresetTemplate {
        id: "stories_insta",
        category: "stories",
        name: "Instagram Story",
        width: 1080,
        height: 1920,
        description: "9:16 Vertical Story format (1080 × 1920 px)",
        icon_name: "phone-symbolic",
        aspect_w: 50,
        aspect_h: 90,
    },
    PresetTemplate {
        id: "stories_tiktok",
        category: "stories",
        name: "TikTok Video Frame",
        width: 1080,
        height: 1920,
        description: "Vertical video canvas (1080 × 1920 px)",
        icon_name: "phone-symbolic",
        aspect_w: 50,
        aspect_h: 90,
    },
    PresetTemplate {
        id: "stories_snapchat",
        category: "stories",
        name: "Snapchat Snap",
        width: 1080,
        height: 1920,
        description: "Full portrait canvas (1080 × 1920 px)",
        icon_name: "phone-symbolic",
        aspect_w: 50,
        aspect_h: 90,
    },
];

enum LaunchAction {
    Preset(PresetTemplate),
    OpenPath(std::path::PathBuf),
    NewDoc,
}

/// Standalone Welcome Application Window on launch
pub fn show_welcome_app_window(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Paths")
        .default_width(980)
        .default_height(660)
        .build();

    let app_clone = app.clone();
    let win_clone = window.clone();

    let on_launch = move |action: LaunchAction| {
        let design_win = match action {
            LaunchAction::Preset(preset) => {
                DesignWindow::new_with_preset(&app_clone, preset.width as f32, preset.height as f32)
            }
            LaunchAction::OpenPath(path) => {
                DesignWindow::new_with_file(&app_clone, &path)
            }
            LaunchAction::NewDoc => {
                DesignWindow::new(&app_clone)
            }
        };
        design_win.present();
        win_clone.close();
    };

    let main_layout = build_welcome_ui(Box::new(on_launch), None, None, window.clone());
    window.set_content(Some(&main_layout));
    window.present();
}

/// Secondary Welcome Window opened from inside the editor
pub fn show_welcome_window(
    parent: Option<&adw::ApplicationWindow>,
    canvas: CanvasWidget,
    file_ops: WindowFileOps,
) {
    let window = adw::Window::builder()
        .title(&crate::core::gettext("Welcome to Paths"))
        .default_width(980)
        .default_height(660)
        .modal(true)
        .resizable(true)
        .build();

    if let Some(p) = parent {
        window.set_transient_for(Some(p));
    }

    let win_c = window.clone();
    let cv_c = canvas.clone();
    let _fops_c = file_ops.clone();

    let on_launch = move |action: LaunchAction| {
        match action {
            LaunchAction::Preset(preset) => {
                cv_c.new_document();
                cv_c.set_active_page_size(preset.width as f32, preset.height as f32);
            }
            LaunchAction::OpenPath(path) => {
                if let Err(e) = cv_c.open_from_path(&path) {
                    eprintln!("Error opening file: {}", e);
                } else {
                    crate::core::AppSettings::add_recent_file(&path.to_string_lossy());
                }
            }
            LaunchAction::NewDoc => {
                cv_c.new_document();
            }
        }
        win_c.close();
    };

    let main_layout = build_welcome_ui(
        Box::new(on_launch),
        Some(canvas),
        Some(file_ops),
        window.clone(),
    );
    window.set_content(Some(&main_layout));
    window.present();
}

/// Constructs UI using pure Libadwaita native widget classes
/// Constructs UI using pure Libadwaita native widget classes (AdwNavigationSplitView & AdwHeaderBars)
fn build_welcome_ui(
    on_launch: Box<dyn Fn(LaunchAction) + 'static>,
    existing_canvas: Option<CanvasWidget>,
    existing_file_ops: Option<WindowFileOps>,
    _window_widget: impl IsA<gtk4::Window>,
) -> adw::NavigationSplitView {
    let on_launch_rc = Rc::new(on_launch);

    let split_view = adw::NavigationSplitView::builder()
        .min_sidebar_width(220.0)
        .max_sidebar_width(260.0)
        .sidebar_width_fraction(0.25)
        .build();

    // ─────────────────────────────────────────────────────────────
    // LEFT SIDEBAR (AdwNavigationPage + AdwToolbarView + AdwHeaderBar)
    // ─────────────────────────────────────────────────────────────
    let sidebar_header_bar = adw::HeaderBar::builder()
        .show_start_title_buttons(true)
        .show_end_title_buttons(false)
        .show_title(false)
        .build();

    let open_btn = gtk4::Button::builder()
        .label(&crate::core::gettext("Open ▾"))
        .css_classes(["flat"])
        .build();

    let on_launch_open = on_launch_rc.clone();
    let fops_open = existing_file_ops.clone();
    open_btn.connect_clicked(move |_| {
        if let Some(ref fops) = fops_open {
            (fops.perform_open)();
        } else {
            let file_dialog = gtk4::FileDialog::builder()
                .title(&crate::core::gettext("Open Document"))
                .modal(true)
                .build();
            let filter_all = gtk4::FileFilter::new();
            filter_all.set_name(Some(&crate::core::gettext("Supported Formats (*.svg)")));
            filter_all.add_pattern("*.svg");
            filter_all.add_pattern("*.SVG");
            let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
            filters.append(&filter_all);
            file_dialog.set_filters(Some(&filters));

            let ol = on_launch_open.clone();
            file_dialog.open(gtk4::Window::NONE, gtk4::gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        ol(LaunchAction::OpenPath(path));
                    }
                }
            });
        }
    });

    let new_doc_btn = gtk4::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text(&crate::core::gettext("New Document"))
        .css_classes(["flat"])
        .build();

    let on_launch_new = on_launch_rc.clone();
    new_doc_btn.connect_clicked(move |_| {
        on_launch_new(LaunchAction::NewDoc);
    });

    sidebar_header_bar.pack_start(&open_btn);
    sidebar_header_bar.pack_end(&new_doc_btn);

    let sidebar_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(6)
        .margin_end(6)
        .build();

    // Top Navigation List
    let top_sidebar_list = gtk4::ListBox::builder()
        .css_classes(["navigation-sidebar"])
        .selection_mode(gtk4::SelectionMode::Single)
        .margin_start(4)
        .margin_end(4)
        .build();

    // Option 0: Time to draw
    add_sidebar_row(&top_sidebar_list, "edit-symbolic", &crate::core::gettext("Time to draw"));
    // Option 1: Quick Start
    add_sidebar_row(&top_sidebar_list, "insert-image-symbolic", &crate::core::gettext("Quick Start"));

    sidebar_box.append(&top_sidebar_list);

    // Spacer to push Quick Setup & About to bottom
    let spacer = gtk4::Box::builder().vexpand(true).build();
    sidebar_box.append(&spacer);

    // Bottom Navigation List
    let bottom_sidebar_list = gtk4::ListBox::builder()
        .css_classes(["navigation-sidebar"])
        .selection_mode(gtk4::SelectionMode::Single)
        .margin_start(4)
        .margin_end(4)
        .build();

    // Option 2: Quick Setup
    add_sidebar_row(&bottom_sidebar_list, "preferences-system-symbolic", &crate::core::gettext("Quick Setup"));
    // Option 3: About and Get Involved
    add_sidebar_row(&bottom_sidebar_list, "help-about-symbolic", &crate::core::gettext("About and Get Involved"));

    sidebar_box.append(&bottom_sidebar_list);

    let sidebar_toolbar = adw::ToolbarView::builder()
        .content(&sidebar_box)
        .build();
    sidebar_toolbar.add_top_bar(&sidebar_header_bar);

    let sidebar_page = adw::NavigationPage::builder()
        .title(&crate::core::gettext("Navigation"))
        .child(&sidebar_toolbar)
        .build();

    // ─────────────────────────────────────────────────────────────
    // RIGHT CONTENT AREA (AdwNavigationPage + AdwToolbarView + AdwHeaderBar)
    // ─────────────────────────────────────────────────────────────
    let content_header_bar = adw::HeaderBar::builder()
        .show_start_title_buttons(false)
        .show_end_title_buttons(true)
        .build();

    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text(&crate::core::gettext("Search all designs, files, and presets..."))
        .hexpand(true)
        .max_width_chars(36)
        .build();

    content_header_bar.set_title_widget(Some(&search_entry));

    // Stack for 4 Main Pages
    let main_stack = gtk4::Stack::builder()
        .transition_type(gtk4::StackTransitionType::Crossfade)
        .hexpand(true)
        .vexpand(true)
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(12)
        .build();

    // Page 0: Time to draw
    let page_time_to_draw = build_page_time_to_draw(on_launch_rc.clone(), existing_file_ops, search_entry.clone());
    main_stack.add_named(&page_time_to_draw, Some("time_to_draw"));

    // Page 1: Quick Start
    let page_quick_start = build_page_quick_start(on_launch_rc.clone());
    main_stack.add_named(&page_quick_start, Some("quick_start"));

    // Page 2: Quick Setup
    let page_quick_setup = build_page_quick_setup(existing_canvas);
    main_stack.add_named(&page_quick_setup, Some("quick_setup"));

    // Page 3: About and Get Involved
    let page_about = build_page_about();
    main_stack.add_named(&page_about, Some("about"));

    let content_toolbar = adw::ToolbarView::builder()
        .content(&main_stack)
        .build();
    content_toolbar.add_top_bar(&content_header_bar);

    let content_page = adw::NavigationPage::builder()
        .title("Paths")
        .child(&content_toolbar)
        .build();

    // Synchronize selection between top & bottom ListBoxes
    let stack_c = main_stack.clone();
    let b_list_c = bottom_sidebar_list.clone();
    top_sidebar_list.connect_row_selected(move |_, row| {
        if let Some(r) = row {
            b_list_c.select_row(None::<&gtk4::ListBoxRow>);
            match r.index() {
                0 => stack_c.set_visible_child_name("time_to_draw"),
                1 => stack_c.set_visible_child_name("quick_start"),
                _ => {}
            }
        }
    });

    let stack_b = main_stack.clone();
    let t_list_c = top_sidebar_list.clone();
    bottom_sidebar_list.connect_row_selected(move |_, row| {
        if let Some(r) = row {
            t_list_c.select_row(None::<&gtk4::ListBoxRow>);
            match r.index() {
                0 => stack_b.set_visible_child_name("quick_setup"),
                1 => stack_b.set_visible_child_name("about"),
                _ => {}
            }
        }
    });

    // Select "Time to draw" (top list index 0) by default
    if let Some(row) = top_sidebar_list.row_at_index(0) {
        top_sidebar_list.select_row(Some(&row));
    }

    split_view.set_sidebar(Some(&sidebar_page));
    split_view.set_content(Some(&content_page));

    split_view
}

fn add_sidebar_row(list: &gtk4::ListBox, icon_name: &str, title: &str) {
    let box_row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let icon = gtk4::Image::from_icon_name(icon_name);
    icon.set_pixel_size(18);
    let lbl = gtk4::Label::builder()
        .label(title)
        .halign(gtk4::Align::Start)
        .css_classes(["heading"])
        .build();

    box_row.append(&icon);
    box_row.append(&lbl);

    let row = gtk4::ListBoxRow::builder()
        .child(&box_row)
        .css_classes(["navigation-sidebar-row"])
        .build();
    list.append(&row);
}

// ─────────────────────────────────────────────────────────────
// HERO BANNER WIDGET (PURE LIBADWAITA CARD)
// ─────────────────────────────────────────────────────────────
fn create_hero_banner_widget() -> gtk4::Widget {
    let banner_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .height_request(130)
        .css_classes(["card"])
        .margin_bottom(12)
        .build();

    let inner_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .hexpand(true)
        .vexpand(true)
        .spacing(6)
        .margin_top(16)
        .margin_bottom(16)
        .build();

    let icon = gtk4::Image::from_icon_name("io.gitlab.lewisHeart.Paths");
    icon.set_pixel_size(44);
    inner_box.append(&icon);

    let title_lbl = gtk4::Label::builder()
        .label("Paths — Vector Design Studio")
        .css_classes(["title-2"])
        .build();

    let sub_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Create vector illustrations, social posts, stories and wallpapers"))
        .css_classes(["caption", "dim-label"])
        .build();

    inner_box.append(&title_lbl);
    inner_box.append(&sub_lbl);
    banner_box.append(&inner_box);

    banner_box.upcast()
}

// ─────────────────────────────────────────────────────────────
// PAGE 0: TIME TO DRAW (Hero Banner, Recent Files, Favorite Quick Start)
// ─────────────────────────────────────────────────────────────

fn build_page_time_to_draw(
    on_launch: Rc<Box<dyn Fn(LaunchAction)>>,
    _file_ops: Option<WindowFileOps>,
    search_entry: gtk4::SearchEntry,
) -> gtk4::Widget {
    let page_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(16)
        .build();

    // Top Hero Banner
    let hero_banner = create_hero_banner_widget();
    page_box.append(&hero_banner);

    // ── SECTION 1: RECENT FILES (HORIZONTAL CARDS ROW) ──
    let group_recent = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Recent Files"))
        .build();

    let recent_flow_box = gtk4::FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(4)
        .min_children_per_line(1)
        .homogeneous(true)
        .margin_top(8)
        .build();

    // 1. First card: New Document Card (+)
    let new_doc_card = create_new_doc_card(on_launch.clone());
    recent_flow_box.append(&new_doc_card);

    let recent_files = crate::core::AppSettings::recent_files();
    for file_path in &recent_files {
        let card = create_recent_file_card(file_path, on_launch.clone());
        recent_flow_box.append(&card);
    }

    group_recent.add(&recent_flow_box);
    page_box.append(&group_recent);

    // ── SECTION 2: FAVORITE QUICK START ──
    let group_fav = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Favorite quick start"))
        .build();

    let fav_flow_box = gtk4::FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(3)
        .min_children_per_line(1)
        .homogeneous(true)
        .margin_top(8)
        .build();

    let favorites = crate::core::AppSettings::favorite_presets();
    let display_presets = if favorites.is_empty() {
        vec!["social_instagram", "stories_insta", "wallpaper_fhd"]
    } else {
        favorites.iter().map(|s| s.as_str()).collect()
    };

    for preset_id in display_presets {
        if let Some(preset) = ALL_PRESETS.iter().find(|p| p.id == preset_id) {
            let card = create_visual_thumbnail_card(preset, on_launch.clone());
            fav_flow_box.append(&card);
        }
    }

    group_fav.add(&fav_flow_box);
    page_box.append(&group_fav);

    // Global Search Filter Handler
    let flow_recent_c = recent_flow_box.clone();
    let flow_fav_c = fav_flow_box.clone();
    let recent_files_c = recent_files.clone();
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_lowercase();
        
        let mut r_idx = 1;
        for file_path in &recent_files_c {
            let matches = query.is_empty() || file_path.to_lowercase().contains(&query);
            if let Some(child) = flow_recent_c.child_at_index(r_idx) {
                child.set_visible(matches);
            }
            r_idx += 1;
        }

        let mut f_idx = 0;
        for _ in 0..flow_fav_c.observe_children().n_items() {
            if let Some(child) = flow_fav_c.child_at_index(f_idx) {
                child.set_visible(query.is_empty());
            }
            f_idx += 1;
        }
    });

    let scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .child(&page_box)
        .build();

    scroll.upcast()
}

/// New Document (+) Card using Libadwaita card class
fn create_new_doc_card(on_launch: Rc<Box<dyn Fn(LaunchAction)>>) -> gtk4::Widget {
    let card_btn = gtk4::Button::builder()
        .css_classes(["flat"])
        .margin_start(4)
        .margin_end(4)
        .margin_top(4)
        .margin_bottom(4)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let card_vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let preview_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .width_request(160)
        .height_request(110)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .css_classes(["card"])
        .build();

    let inner_center = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .hexpand(true)
        .vexpand(true)
        .build();

    let add_icon = gtk4::Image::from_icon_name("list-add-symbolic");
    add_icon.set_pixel_size(36);
    inner_center.append(&add_icon);
    preview_box.append(&inner_center);

    let name_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("New Document"))
        .css_classes(["caption", "heading"])
        .halign(gtk4::Align::Center)
        .build();

    card_vbox.append(&preview_box);
    card_vbox.append(&name_lbl);
    card_btn.set_child(Some(&card_vbox));

    let ol = on_launch;
    card_btn.connect_clicked(move |_| {
        ol(LaunchAction::NewDoc);
    });

    card_btn.upcast()
}

/// Recent File Card using Libadwaita card class
fn create_recent_file_card(
    file_path: &str,
    on_launch: Rc<Box<dyn Fn(LaunchAction)>>,
) -> gtk4::Widget {
    let card_btn = gtk4::Button::builder()
        .css_classes(["flat"])
        .margin_start(4)
        .margin_end(4)
        .margin_top(4)
        .margin_bottom(4)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let card_vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let preview_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .width_request(160)
        .height_request(110)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .css_classes(["card"])
        .build();

    let inner_center = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .hexpand(true)
        .vexpand(true)
        .build();

    let file_icon = gtk4::Image::from_icon_name("document-open-recent-symbolic");
    file_icon.set_pixel_size(32);
    inner_center.append(&file_icon);
    preview_box.append(&inner_center);

    let path_obj = std::path::Path::new(file_path);
    let fname = path_obj
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let name_lbl = gtk4::Label::builder()
        .label(&fname)
        .css_classes(["caption", "heading"])
        .halign(gtk4::Align::Center)
        .ellipsize(gtk4::pango::EllipsizeMode::Middle)
        .build();

    card_vbox.append(&preview_box);
    card_vbox.append(&name_lbl);
    card_btn.set_child(Some(&card_vbox));

    let fp = file_path.to_string();
    let ol = on_launch;
    card_btn.connect_clicked(move |_| {
        ol(LaunchAction::OpenPath(std::path::PathBuf::from(&fp)));
    });

    card_btn.upcast()
}

/// Visual Format Aspect Ratio Card using Libadwaita card class
fn create_visual_thumbnail_card(
    preset: &PresetTemplate,
    on_launch: Rc<Box<dyn Fn(LaunchAction)>>,
) -> gtk4::Widget {
    let card_btn = gtk4::Button::builder()
        .css_classes(["flat"])
        .margin_start(4)
        .margin_end(4)
        .margin_top(4)
        .margin_bottom(4)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .tooltip_text(preset.description)
        .build();

    let card_vbox = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let preview_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .width_request(160)
        .height_request(110)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .css_classes(["card"])
        .build();

    let aspect_inner = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .vexpand(true)
        .hexpand(true)
        .width_request(preset.aspect_w)
        .height_request(preset.aspect_h)
        .css_classes(["card"])
        .build();

    let aspect_icon = gtk4::Image::from_icon_name(preset.icon_name);
    aspect_icon.set_pixel_size(18);
    aspect_icon.set_opacity(0.45);
    aspect_inner.append(&aspect_icon);

    preview_box.append(&aspect_inner);

    let is_fav = crate::core::AppSettings::is_favorite_preset(preset.id);
    let star_icon = if is_fav { "starred-symbolic" } else { "non-starred-symbolic" };
    let fav_btn = gtk4::Button::builder()
        .icon_name(star_icon)
        .tooltip_text(&crate::core::gettext("Toggle Favorite"))
        .css_classes(["flat", "circular"])
        .valign(gtk4::Align::Start)
        .halign(gtk4::Align::End)
        .margin_top(4)
        .margin_end(4)
        .build();

    let pid = preset.id.to_string();
    fav_btn.connect_clicked(move |btn| {
        let new_fav = crate::core::AppSettings::toggle_favorite_preset(&pid);
        if new_fav {
            btn.set_icon_name("starred-symbolic");
        } else {
            btn.set_icon_name("non-starred-symbolic");
        }
    });

    let overlay = gtk4::Overlay::new();
    overlay.set_child(Some(&preview_box));
    overlay.add_overlay(&fav_btn);

    let name_lbl = gtk4::Label::builder()
        .label(preset.name)
        .css_classes(["caption", "heading"])
        .halign(gtk4::Align::Center)
        .build();

    card_vbox.append(&overlay);
    card_vbox.append(&name_lbl);
    card_btn.set_child(Some(&card_vbox));

    let preset_copy = *preset;
    let ol = on_launch;
    card_btn.connect_clicked(move |_| {
        ol(LaunchAction::Preset(preset_copy));
    });

    card_btn.upcast()
}

// ─────────────────────────────────────────────────────────────
// PAGE 1: QUICK START (SECTIONS: SOCIAL POST, WALLPAPER, STORIES)
// ─────────────────────────────────────────────────────────────

fn build_page_quick_start(on_launch: Rc<Box<dyn Fn(LaunchAction)>>) -> gtk4::Widget {
    let page_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(24)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(24)
        .build();

    let title_lbl = gtk4::Label::builder()
        .label(&crate::core::gettext("Quick Start"))
        .halign(gtk4::Align::Start)
        .css_classes(["title-1"])
        .build();
    page_box.append(&title_lbl);

    // Section 1: Social Post
    let group_social = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Social Post"))
        .description(&crate::core::gettext("Standard dimensions for social media feeds &amp; graphics"))
        .build();

    let flow_social = create_preset_grid_flowbox("social", on_launch.clone());
    group_social.add(&flow_social);
    page_box.append(&group_social);

    // Section 2: Wallpaper
    let group_wallpaper = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Wallpaper"))
        .description(&crate::core::gettext("High-resolution desktop and mobile display wallpapers"))
        .build();

    let flow_wallpaper = create_preset_grid_flowbox("wallpaper", on_launch.clone());
    group_wallpaper.add(&flow_wallpaper);
    page_box.append(&group_wallpaper);

    // Section 3: Stories
    let group_stories = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Stories"))
        .description(&crate::core::gettext("9:16 Vertical Portrait layouts for Instagram, TikTok &amp; Snapchat"))
        .build();

    let flow_stories = create_preset_grid_flowbox("stories", on_launch.clone());
    group_stories.add(&flow_stories);
    page_box.append(&group_stories);

    let scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .child(&page_box)
        .build();

    scroll.upcast()
}

fn create_preset_grid_flowbox(category: &str, on_launch: Rc<Box<dyn Fn(LaunchAction)>>) -> gtk4::FlowBox {
    let flow_box = gtk4::FlowBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .max_children_per_line(3)
        .min_children_per_line(1)
        .homogeneous(true)
        .margin_top(8)
        .build();

    for preset in ALL_PRESETS {
        if preset.category == category {
            let card = create_visual_thumbnail_card(preset, on_launch.clone());
            flow_box.append(&card);
        }
    }

    flow_box
}

// ─────────────────────────────────────────────────────────────
// PAGE 2: QUICK SETUP (adw::PreferencesPage)
// ─────────────────────────────────────────────────────────────

fn build_page_quick_setup(canvas: Option<CanvasWidget>) -> gtk4::Widget {
    let pref_page = adw::PreferencesPage::builder()
        .title(&crate::core::gettext("Quick Setup"))
        .build();

    let group = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Workspace Preferences"))
        .description(&crate::core::gettext("Configure theme, units, grid, and shortcut presets"))
        .build();

    // Color Scheme
    let theme_combo = gtk4::DropDown::from_strings(&[
        &crate::core::gettext("System"),
        &crate::core::gettext("Light"),
        &crate::core::gettext("Dark"),
    ]);

    let curr_scheme = crate::core::AppSettings::color_scheme();
    match curr_scheme.as_str() {
        "light" => theme_combo.set_selected(1),
        "dark" => theme_combo.set_selected(2),
        _ => theme_combo.set_selected(0),
    }

    theme_combo.connect_selected_notify(|cb| {
        let scheme = match cb.selected() {
            1 => "light",
            2 => "dark",
            _ => "system",
        };
        crate::core::AppSettings::set_color_scheme(scheme);
        match scheme {
            "light" => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceLight),
            "dark" => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark),
            _ => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default),
        }
    });

    let row_theme = adw::ActionRow::builder()
        .title(&crate::core::gettext("Color Scheme"))
        .subtitle(&crate::core::gettext("Choose system, light, or dark mode"))
        .build();
    row_theme.add_suffix(&theme_combo);
    group.add(&row_theme);

    // Units
    let unit_combo = gtk4::DropDown::from_strings(&["px", "mm", "in", "pt"]);
    let curr_unit = crate::core::AppSettings::unit();
    let u_idx = match curr_unit.as_str() {
        "mm" => 1,
        "in" => 2,
        "pt" => 3,
        _ => 0,
    };
    unit_combo.set_selected(u_idx);

    let cv_unit = canvas.clone();
    unit_combo.connect_selected_notify(move |cb| {
        let u_str = match cb.selected() {
            1 => "mm",
            2 => "in",
            3 => "pt",
            _ => "px",
        };
        crate::core::AppSettings::set_unit(u_str);
        if let Some(ref cv) = cv_unit {
            if let Some(unit) = crate::core::Unit::from_suffix(u_str) {
                cv.set_unit(unit);
            }
        }
    });

    let row_unit = adw::ActionRow::builder()
        .title(&crate::core::gettext("Canvas Unit"))
        .subtitle(&crate::core::gettext("Default measurement unit for canvas and rulers"))
        .build();
    row_unit.add_suffix(&unit_combo);
    group.add(&row_unit);

    // Grid toggle
    let switch_grid = gtk4::Switch::builder()
        .active(crate::core::AppSettings::show_grid())
        .valign(gtk4::Align::Center)
        .build();

    let cv_grid = canvas.clone();
    switch_grid.connect_active_notify(move |sw| {
        let active = sw.is_active();
        crate::core::AppSettings::set_show_grid(active);
        if let Some(ref cv) = cv_grid {
            cv.set_grid_visible(active);
        }
    });

    let row_grid = adw::ActionRow::builder()
        .title(&crate::core::gettext("Show Grid"))
        .subtitle(&crate::core::gettext("Toggle grid overlay on the canvas"))
        .build();
    row_grid.add_suffix(&switch_grid);
    group.add(&row_grid);

    // Shortcut preset
    let sc_combo = gtk4::DropDown::from_strings(&["Default", "Figma", "Illustrator", "Inkscape"]);
    let curr_sc = crate::core::AppSettings::shortcut_preset();
    let sc_idx = match curr_sc.as_str() {
        "figma" => 1,
        "illustrator" => 2,
        "inkscape" => 3,
        _ => 0,
    };
    sc_combo.set_selected(sc_idx);

    let cv_sc = canvas;
    sc_combo.connect_selected_notify(move |cb| {
        let sc_str = match cb.selected() {
            1 => "figma",
            2 => "illustrator",
            3 => "inkscape",
            _ => "default",
        };
        crate::core::AppSettings::set_shortcut_preset(sc_str);
        if let Some(ref cv) = cv_sc {
            let sc_preset = match sc_str {
                "figma" => crate::core::ShortcutPreset::Figma,
                "illustrator" => crate::core::ShortcutPreset::Illustrator,
                "inkscape" => crate::core::ShortcutPreset::Inkscape,
                _ => crate::core::ShortcutPreset::Default,
            };
            cv.set_shortcut_preset(sc_preset);
        }
    });

    let row_sc = adw::ActionRow::builder()
        .title(&crate::core::gettext("Shortcut Preset"))
        .subtitle(&crate::core::gettext("Choose keybinding layout preset"))
        .build();
    row_sc.add_suffix(&sc_combo);
    group.add(&row_sc);

    pref_page.add(&group);
    pref_page.upcast()
}

// ─────────────────────────────────────────────────────────────
// PAGE 3: ABOUT AND GET INVOLVED (adw::PreferencesPage)
// ─────────────────────────────────────────────────────────────

fn build_page_about() -> gtk4::Widget {
    let pref_page = adw::PreferencesPage::builder()
        .title(&crate::core::gettext("About and get Involved"))
        .build();

    let group_info = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("About Paths"))
        .description(&crate::core::gettext("Paths v0.3.1 - Modern vector design editor accelerated by Skia GPU"))
        .build();

    let row_app = adw::ActionRow::builder()
        .title("Paths v0.3.1")
        .subtitle(&crate::core::gettext("Built with GTK4, Libadwaita, Skia GPU Acceleration &amp; Rust"))
        .build();
    let app_icon = gtk4::Image::from_icon_name("io.gitlab.lewisHeart.Paths");
    app_icon.set_pixel_size(32);
    row_app.add_prefix(&app_icon);
    group_info.add(&row_app);

    pref_page.add(&group_info);

    let group_community = adw::PreferencesGroup::builder()
        .title(&crate::core::gettext("Community &amp; Source"))
        .description(&crate::core::gettext("Get involved, report bugs, or contribute code"))
        .build();

    let row_repo = adw::ActionRow::builder()
        .title(&crate::core::gettext("GitLab Repository"))
        .subtitle("https://gitlab.com/lewisHeart/gnome-paths")
        .activatable(true)
        .build();

    let icon_repo = gtk4::Image::from_icon_name("web-browser-symbolic");
    row_repo.add_prefix(&icon_repo);
    row_repo.connect_activated(|_| {
        let _ = gtk4::gio::AppInfo::launch_default_for_uri(
            "https://gitlab.com/lewisHeart/gnome-paths",
            None::<&gtk4::gio::AppLaunchContext>,
        );
    });
    group_community.add(&row_repo);

    let row_issues = adw::ActionRow::builder()
        .title(&crate::core::gettext("Report an Issue"))
        .subtitle("https://gitlab.com/lewisHeart/gnome-paths/-/issues")
        .activatable(true)
        .build();

    let icon_issues = gtk4::Image::from_icon_name("help-about-symbolic");
    row_issues.add_prefix(&icon_issues);
    row_issues.connect_activated(|_| {
        let _ = gtk4::gio::AppInfo::launch_default_for_uri(
            "https://gitlab.com/lewisHeart/gnome-paths/-/issues",
            None::<&gtk4::gio::AppLaunchContext>,
        );
    });
    group_community.add(&row_issues);

    pref_page.add(&group_community);
    pref_page.upcast()
}
