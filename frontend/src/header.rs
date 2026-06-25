use crate::i18n::{translate, Locale, TransKey};
use leptos::*;

#[component]
pub fn Header<F, G>(
    locale: ReadSignal<Locale>,
    set_locale: WriteSignal<Locale>,
    theme: ReadSignal<String>,
    is_authorized: ReadSignal<bool>,
    enable_translation: ReadSignal<bool>,
    enable_themes: ReadSignal<bool>,
    enable_print: ReadSignal<bool>,
    pin_required: ReadSignal<bool>,
    on_logout: F,
    on_toggle_theme: G,
) -> impl IntoView
where
    F: Fn() + 'static + Clone,
    G: Fn() + 'static + Clone,
{
    let on_select_lang = move |e: ev::Event| {
        use wasm_bindgen::JsCast;
        let select = e
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlSelectElement>()
            .unwrap();
        let new_loc = Locale::from_str(&select.value());
        set_locale.set(new_loc);
    };

    let on_logout_click = on_logout.clone();
    let on_toggle_theme_click = on_toggle_theme.clone();

    view! {
        <header>
            <div id="header-title">
                <h1>"RustSearch"</h1>
            </div>
            <div class="header-right">
                {move || if enable_translation.get() {
                    view! {
                        <div class="language-select-container">
                            <select
                                class="language-select"
                                id="language-select"
                                on:change=on_select_lang
                                aria-label="Select language"
                            >
                                {Locale::all().iter().map(|loc| {
                                    let selected = move || locale.get() == *loc;
                                    view! {
                                        <option value=loc.to_str() selected=selected>
                                            {loc.display_label()}
                                        </option>
                                    }
                                }).collect::<Vec<_>>()}
                            </select>
                        </div>
                    }.into_view()
                } else {
                    "".into_view()
                }}
                {
                    let on_toggle_theme_click = on_toggle_theme_click.clone();
                    move || if enable_themes.get() {
                        let on_toggle_theme_click = on_toggle_theme_click.clone();
                        view! {
                            <button
                                id="theme-toggle"
                                class="icon-button"
                                aria-label="Toggle theme"
                                on:click=move |_| on_toggle_theme_click()
                                title=move || translate(TransKey::ToggleThemeTooltip, locale.get())
                            >
                                {move || match theme.get().as_str() {
                                    "brinstar" => view! {
                                        <svg id="leaf-icon" class="leaf" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 3.5 1 9.8a7 7 0 0 1-9 8.2Z" /><path d="M19 2 9.8 11.5" /></svg>
                                    }.into_view(),
                                    "norfair" => view! {
                                        <svg id="flame-icon" class="flame" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z" /></svg>
                                    }.into_view(),
                                    "wrecked_ship" => view! {
                                        <svg id="ghost-icon" class="ghost" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 10h.01"/><path d="M15 10h.01"/><path d="M12 2a8 8 0 0 0-8 8v12l3-3 2.5 2.5L12 19l2.5 2.5L17 19l3 3V10a8 8 0 0 0-8-8z"/></svg>
                                    }.into_view(),
                                    "maridia" => view! {
                                        <svg id="waves-icon" class="waves" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 6c.6 0 1.2-.4 1.8-1 1.2-1.2 2.4-1.2 3.6 0 .6.6 1.2 1 1.8 1 .6 0 1.2-.4 1.8-1 1.2-1.2 2.4-1.2 3.6 0 .6.6 1.2 1 1.8 1 .6 0 1.2-.4 1.8-1 1.2-1.2 2.4-1.2 3.6 0 .6.6 1.2 1 1.8 1" /><path d="M2 12c.6 0 1.2-.4 1.8-1 1.2-1.2 2.4-1.2 3.6 0 .6.6 1.2 1 1.8 1 .6 0 1.2-.4 1.8-1 1.2-1.2 2.4-1.2 3.6 0 .6.6 1.2 1 1.8 1 .6 0 1.2-.4 1.8-1 1.2-1.2 2.4-1.2 3.6 0 .6.6 1.2 1 1.8 1" /><path d="M2 18c.6 0 1.2-.4 1.8-1 1.2-1.2 2.4-1.2 3.6 0 .6.6 1.2 1 1.8 1 .6 0 1.2-.4 1.8-1 1.2-1.2 2.4-1.2 3.6 0 .6.6 1.2 1 1.8 1 .6 0 1.2-.4 1.8-1 1.2-1.2 2.4-1.2 3.6 0 .6.6 1.2 1 1.8 1" /></svg>
                                    }.into_view(),
                                    "tourian" => view! {
                                        <svg id="target-icon" class="target" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><circle cx="12" cy="12" r="6" /><circle cx="12" cy="12" r="2" /></svg>
                                    }.into_view(),
                                    _ => view! {
                                        <svg id="cloud-rain-icon" class="cloud-rain" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 17.58A5 5 0 0 0 18 8h-1.26A8 8 0 1 0 4 16.25" /><path d="M8 20v2" /><path d="M12 20v2" /><path d="M16 20v2" /></svg>
                                    }.into_view(),
                                }}
                            </button>
                        }.into_view()
                    } else {
                        "".into_view()
                    }
                }
                {move || if enable_print.get() {
                    view! {
                        <button
                            id="print-button"
                            class="icon-button"
                            on:click=move |_| {
                                if let Some(win) = web_sys::window() {
                                    let _ = win.print();
                                }
                            }
                            title=move || translate(TransKey::PrintTooltip, locale.get())
                        >
                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <polyline points="6 9 6 2 18 2 18 9" />
                                <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2" />
                                <rect x="6" y="14" width="12" height="8" />
                            </svg>
                        </button>
                    }.into_view()
                } else {
                    "".into_view()
                }}
                {
                    let on_logout_click = on_logout_click.clone();
                    move || if pin_required.get() {
                        let on_logout_click = on_logout_click.clone();
                        view! {
                            <button
                                id="logout-button"
                                class="icon-button"
                                on:click=move |_| on_logout_click()
                                disabled=move || !is_authorized.get()
                                title=move || translate(TransKey::SignOut, locale.get())
                            >
                                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
                                    <polyline points="16 17 21 12 16 7" />
                                    <line x1="21" y1="12" x2="9" y2="12" />
                                </svg>
                            </button>
                        }.into_view()
                    } else {
                        "".into_view()
                    }
                }
            </div>
        </header>
    }
}
