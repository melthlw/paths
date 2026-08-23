use std::cell::RefCell;

thread_local! {
    static ICON_COLOR: RefCell<Option<String>> = const { RefCell::new(None) };
    static ICON_CSS_PROVIDER: RefCell<Option<gtk4::CssProvider>> = const { RefCell::new(None) };
}

pub fn current_interface_icon_color() -> String {
    ICON_COLOR.with(|c| {
        c.borrow().clone().unwrap_or_else(|| "default".to_string())
    })
}

pub fn set_interface_icon_color(color_hex: &str) {
    ICON_COLOR.with(|c| {
        *c.borrow_mut() = if color_hex == "default" {
            None
        } else {
            Some(color_hex.to_string())
        };
    });

    ICON_CSS_PROVIDER.with(|p| {
        let mut p_borrow = p.borrow_mut();
        if p_borrow.is_none() {
            let provider = gtk4::CssProvider::new();
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 20,
                );
            }
            *p_borrow = Some(provider);
        }

        if let Some(ref provider) = *p_borrow {
            if color_hex == "default" {
                provider.load_from_string("");
            } else {
                let css = format!(
                    "button image, .toolbar image, .floating-hud-card image, .layer-row image, .layer-row .layer-icon, .layers-header image, .card-header-bar image, .align-capsule image, .studio-tab-btn image, .inspector-tab-btn image, .catalog-row image, .menu-button image, .app-menu-popover image, .zoom-hud-card image, .page-hud-card image, .navigation-sidebar image, adw-action-row image {{ color: {color}; }} .pref-color-chip image {{ color: #ffffff; }} .pref-color-chip.chip-white image, .pref-color-chip.chip-gold image, .pref-color-chip.chip-paper-light image, .pref-color-chip.chip-paper-cream image {{ color: #1a1a1a; }}",
                    color = color_hex
                );
                provider.load_from_string(&css);
            }
        }
    });
}

pub fn symbolic_icon_name(icon_name_or_resource: &str) -> String {
    if icon_name_or_resource.starts_with('/') {
        icon_name_or_resource
            .rsplit('/')
            .next()
            .and_then(|s| s.strip_suffix(".svg"))
            .map(|s| {
                if s.ends_with("-symbolic") {
                    s.to_string()
                } else {
                    format!("{}-symbolic", s)
                }
            })
            .unwrap_or_else(|| icon_name_or_resource.to_string())
    } else if icon_name_or_resource.ends_with("-symbolic") {
        icon_name_or_resource.to_string()
    } else {
        format!("{}-symbolic", icon_name_or_resource)
    }
}

pub fn make_symbolic_image(icon_name_or_resource: &str, size: i32) -> gtk4::Image {
    let name = symbolic_icon_name(icon_name_or_resource);
    let img = gtk4::Image::from_icon_name(&name);
    if size > 0 {
        img.set_pixel_size(size);
    }
    img
}
