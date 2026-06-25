use leptos::*;

mod api;
mod header;
mod i18n;
mod login;
mod search_panel;
mod theme;
mod types;

use header::Header;
use i18n::{translate, Locale, TransKey};
use login::Login;
use search_panel::SearchPanel;
use types::{ChatCompletionRequest, ChatMessage, ImageSearchResult, TextSearchResult};

#[component]
pub fn App() -> impl IntoView {
    let (access_key_required, set_access_key_required) = create_signal(false);
    let (is_authorized, set_is_authorized) = create_signal(false);
    let (access_key_input, set_access_key_input) = create_signal(String::new());

    let (query_input, set_query_input) = create_signal(String::new());
    let (search_type, set_search_type) = create_signal(String::from("text"));

    let (text_results, set_text_results) = create_signal(Vec::<TextSearchResult>::new());
    let (image_results, set_image_results) = create_signal(Vec::<ImageSearchResult>::new());

    let (ai_response, set_ai_response) = create_signal(String::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (is_generating, set_is_generating) = create_signal(false);
    let (error_message, set_error_message) = create_signal(String::new());

    let (enable_translation, set_enable_translation) = create_signal(true);

    // Theme state
    let (theme, _set_theme, on_toggle_theme) = theme::use_theme();

    // Locale state
    let (locale, set_locale) = create_signal({
        let storage = web_sys::window().and_then(|win| win.local_storage().ok().flatten());
        let lang = storage
            .and_then(|s| s.get_item("lang").ok().flatten())
            .unwrap_or_else(|| "en".to_string());
        Locale::from_str(&lang)
    });

    // Footer notification state
    let (footer_notification, set_footer_notification) = create_signal(None::<(String, String)>);

    // Save language setting when it changes
    create_effect(move |_| {
        if let Some(storage) = web_sys::window().and_then(|win| win.local_storage().ok().flatten())
        {
            let _ = storage.set_item("lang", locale.get().to_str());
        }
    });

    // Show toast function
    let show_toast = move |message: String, is_error: bool| {
        let cls = if is_error {
            "error".to_string()
        } else {
            "success".to_string()
        };
        set_footer_notification.set(Some((message, cls)));
        gloo_timers::callback::Timeout::new(3000, move || {
            set_footer_notification.set(None);
        })
        .forget();
    };

    // Check auth status on startup
    create_effect(move |_| {
        spawn_local(async move {
            match api::check_auth_status().await {
                Ok(status) => {
                    set_access_key_required.set(status.pin_required);
                    set_is_authorized.set(status.is_authorized);
                    set_enable_translation.set(status.enable_translation);
                }
                Err(_) => {
                    set_error_message.set(translate(TransKey::ConnectionError, locale.get()));
                    show_toast(translate(TransKey::ConnectionError, locale.get()), true);
                }
            }
        });
    });

    // Handle PIN validation submit
    let on_submit_password = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let password = access_key_input.get();
        if password.is_empty() {
            return;
        }
        spawn_local(async move {
            set_is_loading.set(true);
            set_error_message.set(String::new());
            match api::verify_pin(&password).await {
                Ok(true) => {
                    set_is_authorized.set(true);
                    show_toast(
                        translate(TransKey::AuthenticatedSuccess, locale.get()),
                        false,
                    );
                }
                Ok(false) => {
                    let err_msg = translate(TransKey::InvalidPassword, locale.get());
                    set_error_message.set(err_msg.clone());
                    show_toast(err_msg, true);
                }
                Err(err_msg) => {
                    set_error_message.set(err_msg.clone());
                    show_toast(err_msg, true);
                }
            }
            set_is_loading.set(false);
        });
    };

    // Run the search and start text streaming
    let on_search = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let query = query_input.get();
        if query.trim().is_empty() {
            return;
        }
        let search_mode = search_type.get();
        spawn_local(async move {
            set_is_loading.set(true);
            set_error_message.set(String::new());
            set_text_results.set(Vec::new());
            set_image_results.set(Vec::new());
            set_ai_response.set(String::new());

            if search_mode == "text" {
                match api::search_text(&query).await {
                    Ok(mapped) => {
                        set_text_results.set(mapped.clone());
                        set_is_loading.set(false);
                        if !mapped.is_empty() {
                            set_is_generating.set(true);
                            let mut context = String::from("You are a direct, concise AI assistant. Answer the user's query briefly using ONLY the search results below.\n\n\
                            Rules:\n\
                            1. Do not use conversational filler, greetings, pleasantries, or introductory preamble.\n\
                            2. Keep the answer extremely brief and to the point.\n\
                            3. CRITICAL: Do not ask the user any follow-up questions, suggest next steps, or prompt for more input. Stop speaking immediately after answering the query.\n\n\
                            Search results:\n\n");
                            for (i, res) in mapped.iter().enumerate() {
                                context.push_str(&format!(
                                    "{}. [{}]({}): {}\n",
                                    i + 1,
                                    res.title,
                                    res.url,
                                    res.snippet
                                ));
                            }
                            let chat_req = ChatCompletionRequest {
                                messages: vec![
                                    ChatMessage {
                                        role: "system".to_string(),
                                        content: context,
                                    },
                                    ChatMessage {
                                        role: "user".to_string(),
                                        content: query.clone(),
                                    },
                                ],
                            };
                            if let Err(e) = api::stream_inference(&chat_req, move |content| {
                                let current = ai_response.get();
                                set_ai_response.set(format!("{}{}", current, content));
                            })
                            .await
                            {
                                if e == "Unauthorized" {
                                    set_is_authorized.set(false);
                                } else {
                                    show_toast(e, true);
                                }
                            }
                            set_is_generating.set(false);
                        }
                    }
                    Err(e) => {
                        if e == "Unauthorized" {
                            set_is_authorized.set(false);
                        } else {
                            show_toast(translate(TransKey::ConnectionError, locale.get()), true);
                        }
                        set_is_loading.set(false);
                    }
                }
            } else {
                match api::search_images(&query).await {
                    Ok(mapped) => {
                        set_image_results.set(mapped);
                    }
                    Err(e) => {
                        if e == "Unauthorized" {
                            set_is_authorized.set(false);
                        } else {
                            show_toast("Failed to execute image search.".to_string(), true);
                        }
                    }
                }
                set_is_loading.set(false);
            }
        });
    };

    let logout = move || {
        spawn_local(async move {
            let _ = api::logout().await;
            set_is_authorized.set(false);
            show_toast(translate(TransKey::LoggedOutSuccess, locale.get()), false);
        });
    };

    view! {
        <div class="app-container">
            <Header
                locale=locale set_locale=set_locale theme=theme
                is_authorized=is_authorized enable_translation=enable_translation
                on_logout=logout on_toggle_theme=on_toggle_theme
            />
            <div class="container">
                {move || if access_key_required.get() && !is_authorized.get() {
                    view! {
                        <Login
                            locale=locale is_loading=is_loading access_key_input=access_key_input
                            set_access_key_input=set_access_key_input error_message=error_message on_submit=on_submit_password
                        />
                    }.into_view()
                } else {
                    view! {
                        <SearchPanel
                            locale=locale query_input=query_input set_query_input=set_query_input
                            search_type=search_type set_search_type=set_search_type is_loading=is_loading
                            ai_response=ai_response is_generating=is_generating text_results=text_results
                            image_results=image_results on_search=on_search
                        />
                    }.into_view()
                }}
            </div>
            <footer class="layout-footer">
                <div class=move || format!("footer-status-text {}", footer_notification.get().map(|(_, cls)| cls).unwrap_or_else(|| "success".to_string()))>
                    {move || match footer_notification.get() {
                        Some((msg, _)) => msg,
                        None => translate(TransKey::Ready, locale.get()),
                    }}
                </div>
            </footer>
        </div>
    }
}
