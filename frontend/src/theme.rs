use leptos::*;

pub fn use_theme() -> (ReadSignal<String>, WriteSignal<String>, impl Fn() + Clone) {
    let (theme, set_theme) = create_signal({
        let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
        let default_theme = "crateria".to_string();
        let loaded = storage
            .and_then(|s| s.get_item("theme").ok().flatten())
            .unwrap_or(default_theme);
        match loaded.as_str() {
            "light" => "brinstar".to_string(),
            "dark" => "crateria".to_string(),
            "nord" => "maridia".to_string(),
            "dracula" => "wrecked_ship".to_string(),
            "sepia" => "norfair".to_string(),
            other => other.to_string(),
        }
    });

    let toggle_theme = move || {
        let next_theme = match theme.get().as_str() {
            "crateria" => "brinstar",
            "brinstar" => "norfair",
            "norfair" => "wrecked_ship",
            "wrecked_ship" => "maridia",
            "maridia" => "tourian",
            _ => "crateria",
        };
        let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
        if let Some(s) = storage {
            let _ = s.set_item("theme", next_theme);
        }
        if let Some(win) = web_sys::window()
            && let Some(doc) = win.document()
            && let Some(el) = doc.document_element()
        {
            let _ = el.set_attribute("data-theme", next_theme);
            let _ = el.set_attribute("class", next_theme);
        }
        set_theme.set(next_theme.to_string());
    };

    // Apply active theme on boot
    create_effect(move |_| {
        let cur_theme = theme.get();
        if let Some(win) = web_sys::window()
            && let Some(doc) = win.document()
            && let Some(el) = doc.document_element()
        {
            let _ = el.set_attribute("data-theme", &cur_theme);
            let _ = el.set_attribute("class", &cur_theme);
        }
    });

    (theme, set_theme, toggle_theme)
}
