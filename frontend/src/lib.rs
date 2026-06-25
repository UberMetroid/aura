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
    let (theme, _set_theme, on_toggle_theme) = theme::use_theme();

    let (locale, set_locale) = create_signal({
        let storage = web_sys::window().and_then(|win| win.local_storage().ok().flatten());
        let lang = storage
            .and_then(|s| s.get_item("lang").ok().flatten())
            .unwrap_or_else(|| "en".to_string());
        Locale::from_str(&lang)
    });

    let (footer_notification, set_footer_notification) = create_signal(None::<(String, String)>);

    create_effect(move |_| {
        if let Some(storage) = web_sys::window().and_then(|win| win.local_storage().ok().flatten())
        {
            let _ = storage.set_item("lang", locale.get().to_str());
        }
    });

    let show_toast = move |msg: String, is_err: bool| {
        let cls = if is_err { "error" } else { "success" }.to_string();
        set_footer_notification.set(Some((msg, cls)));
        gloo_timers::callback::Timeout::new(3000, move || set_footer_notification.set(None))
            .forget();
    };

    create_effect(move |_| {
        spawn_local(async move {
            if let Ok(st) = api::check_auth_status().await {
                set_access_key_required.set(st.pin_required);
                set_is_authorized.set(st.is_authorized);
                set_enable_translation.set(st.enable_translation);
            } else {
                let err = translate(TransKey::ConnectionError, locale.get());
                set_error_message.set(err.clone());
                show_toast(err, true);
            }
        })
    });

    let on_submit_password = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let password = access_key_input.get();
        if password.is_empty() {
            return;
        }
        spawn_local(async move {
            set_is_loading.set(true);
            set_error_message.set(String::new());
            let res = match api::verify_pin(&password).await {
                Ok(true) => Ok(()),
                Ok(false) => Err(translate(TransKey::InvalidPassword, locale.get())),
                Err(err) => Err(err),
            };
            if let Err(err) = &res {
                set_error_message.set(err.clone());
                show_toast(err.clone(), true);
            } else {
                set_is_authorized.set(true);
                show_toast(
                    translate(TransKey::AuthenticatedSuccess, locale.get()),
                    false,
                );
            }
            set_is_loading.set(false);
        });
    };

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
                            let wiki_grok_results: Vec<&TextSearchResult> = mapped
                                .iter()
                                .filter(|res| {
                                    res.url.contains("wikipedia.org")
                                        || res.url.contains("grokipedia.com")
                                })
                                .collect();

                            let results_to_feed = if !wiki_grok_results.is_empty() {
                                wiki_grok_results
                            } else {
                                mapped.iter().collect::<Vec<_>>()
                            };

                            let mut context = String::from("You are a direct, concise AI assistant. Answer the user's query briefly by providing an overview using ONLY the search results below.\n\n\
                            Rules:\n\
                            1. Do not use conversational filler, greetings, pleasantries, or introductory preamble.\n\
                            2. Keep the answer extremely brief and to the point.\n\
                            3. CRITICAL: Do not ask the user any follow-up questions, suggest next steps, or prompt for more input. Stop speaking immediately after answering the query.\n\
                            4. Do NOT prepend your response with 'AI Assistant:', 'AI:', 'Overview:', or any other speaker label prefix.\n\n\
                            Search results:\n\n");
                            for (i, res) in results_to_feed.into_iter().enumerate() {
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
                                let mut current = ai_response.get();
                                current.push_str(&content);

                                let mut stripped = current.as_str();
                                loop {
                                    let mut changed = false;
                                    for prefix in &[
                                        "AI Assistant:",
                                        "AI assistant:",
                                        "ai assistant:",
                                        "AI:",
                                        "ai:",
                                        "Overview:",
                                        "overview:",
                                        "Answer:",
                                        "answer:",
                                    ] {
                                        if stripped.starts_with(prefix) {
                                            stripped =
                                                stripped.strip_prefix(prefix).unwrap().trim_start();
                                            changed = true;
                                        }
                                    }
                                    if !changed {
                                        break;
                                    }
                                }
                                set_ai_response.set(stripped.to_string());
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
