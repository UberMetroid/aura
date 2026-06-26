use leptos::*;

mod api;
mod api_invoke;
mod header;
mod i18n;
mod login;
mod search_panel;
mod theme;
mod types;

use header::Header;
use i18n::{Locale, TransKey, translate};
use login::Login;
use search_panel::SearchPanel;
use types::{ChatCompletionRequest, ChatMessage, TextSearchResult};

#[component]
pub fn App() -> impl IntoView {
    let (access_key_required, set_access_key_required) = create_signal(false);
    let (is_authorized, set_is_authorized) = create_signal(false);
    let (access_key_input, set_access_key_input) = create_signal(String::new());

    let (query_input, set_query_input) = create_signal(String::new());
    let (text_results, set_text_results) = create_signal(Vec::<TextSearchResult>::new());

    let (ai_response, set_ai_response) = create_signal(String::new());
    let (invoke_image, set_invoke_image) = create_signal(None::<String>);
    let (invoke_loading, set_invoke_loading) = create_signal(false);
    let (invoke_error, set_invoke_error) = create_signal(None::<String>);
    let (is_loading, set_is_loading) = create_signal(false);
    let (is_generating, set_is_generating) = create_signal(false);
    let (error_message, set_error_message) = create_signal(String::new());

    let (enable_translation, set_enable_translation) = create_signal(true);
    let (enable_themes, set_enable_themes) = create_signal(true);
    let (enable_print, set_enable_print) = create_signal(true);
    let (theme, set_theme, on_toggle_theme) = theme::use_theme();

    let (locale, set_locale) = create_signal({
        let storage = web_sys::window().and_then(|win| win.local_storage().ok().flatten());
        let lang = storage
            .and_then(|s| s.get_item("lang").ok().flatten())
            .unwrap_or_else(|| "en".to_string());
        Locale::from_str(&lang)
    });

    let (footer_notification, set_footer_notification) = create_signal(None::<(String, String)>);
    let (last_search, set_last_search) = create_signal(String::new());

    create_effect(move |_| {
        if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = s.set_item("lang", locale.get().to_str());
        }
    });

    let show_toast = move |msg: String, is_err: bool| {
        set_footer_notification.set(Some((
            msg,
            if is_err { "error" } else { "success" }.to_string(),
        )));
        gloo_timers::callback::Timeout::new(3000, move || set_footer_notification.set(None))
            .forget();
    };

    create_effect(move |_| {
        let set_theme = set_theme;
        spawn_local(async move {
            if let Ok(st) = api::check_auth_status().await {
                set_access_key_required.set(st.pin_required);
                set_is_authorized.set(st.is_authorized);
                set_enable_translation.set(st.enable_translation);
                set_enable_themes.set(st.enable_themes);
                set_enable_print.set(st.enable_print);
                if !st.enable_themes {
                    set_theme.set("tourian".to_string());
                }
            } else {
                let err = translate(TransKey::ConnectionError, locale.get());
                set_error_message.set(err.clone());
                show_toast(err, true);
            }
        });
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

    let perform_search = move || {
        let query = query_input.get_untracked();
        if query.trim().is_empty() || query == last_search.get_untracked() {
            return;
        }
        spawn_local(async move {
            set_is_loading.set(true);
            set_error_message.set(String::new());
            set_text_results.set(Vec::new());
            set_ai_response.set(String::new());
            set_invoke_image.set(None);
            set_invoke_loading.set(false);
            set_invoke_error.set(None);
            set_last_search.set(query.clone());

            if invoke_image.get_untracked().is_none() && !invoke_loading.get_untracked() {
                api_invoke::spawn_invoke_generation(
                    query.clone(),
                    set_invoke_image,
                    set_invoke_loading,
                    set_invoke_error,
                );
            }
            match api::search_text(&query).await {
                Ok(mapped) => {
                    set_text_results.set(mapped.clone());
                    set_is_loading.set(false);
                    if !mapped.is_empty() {
                        set_is_generating.set(true);
                        let mut ctx = String::from(
                            "You are a direct, concise AI assistant. Answer the user's query briefly by providing an overview using ONLY the search results below.\n\n\
                        Rules:\n\
                        1. Do not use conversational filler, greetings, pleasantries, or introductory preamble.\n\
                        2. Keep the answer extremely brief and to the point.\n\
                        3. CRITICAL: Do not ask the user any follow-up questions, suggest next steps, or prompt for more input. Stop speaking immediately after answering the query.\n\
                        4. Do NOT prepend your response with 'AI Assistant:', 'AI:', 'Overview:', or any other speaker label prefix.\n\
                        5. Do NOT use any Markdown formatting (such as bold asterisks or lists) in your response.\n\n\
                        Search results:\n\n",
                        );
                        for (i, res) in mapped.iter().enumerate() {
                            ctx.push_str(&format!(
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
                                    content: ctx,
                                },
                                ChatMessage {
                                    role: "user".to_string(),
                                    content: query.clone(),
                                },
                            ],
                        };
                        if let Err(e) = api::stream_inference(&chat_req, move |content| {
                            let mut current = ai_response.get();
                            current.push_str(content);
                            set_ai_response.set(api_invoke::strip_speaker_labels(&current));
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
        });
    };

    let on_search = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        perform_search();
    };

    let logout = move || {
        spawn_local(async move {
            let _ = api::logout().await;
            set_is_authorized.set(false);
            show_toast(translate(TransKey::LoggedOutSuccess, locale.get()), false);
        })
    };

    view! {
        <div class="app-container">
            <Header
                locale=locale set_locale=set_locale theme=theme
                is_authorized=is_authorized enable_translation=enable_translation
                enable_themes=enable_themes enable_print=enable_print
                pin_required=access_key_required
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
                            is_loading=is_loading ai_response=ai_response is_generating=is_generating
                            text_results=text_results on_search=on_search
                            invoke_image=invoke_image invoke_loading=invoke_loading
                            invoke_error=invoke_error
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
