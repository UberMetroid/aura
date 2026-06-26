use crate::i18n::{Locale, TransKey, translate};
use crate::types::TextSearchResult;
use leptos::*;

#[component]
pub fn SearchPanel<F>(
    locale: ReadSignal<Locale>,
    query_input: ReadSignal<String>,
    set_query_input: WriteSignal<String>,
    is_loading: ReadSignal<bool>,
    ai_response: ReadSignal<String>,
    is_generating: ReadSignal<bool>,
    text_results: ReadSignal<Vec<TextSearchResult>>,
    on_search: F,
    invoke_image: ReadSignal<Option<String>>,
    invoke_loading: ReadSignal<bool>,
    invoke_error: ReadSignal<Option<String>>,
) -> impl IntoView
where
    F: Fn(ev::SubmitEvent) + 'static + Copy,
{
    let has_ai = move || !ai_response.get().is_empty() || is_generating.get();

    view! {
        <main class="main-content">
            // AI Response Section (AI Overview and AI Image side-by-side)
            {move || has_ai().then(|| view! {
                <section class="ai-response-section">
                    <div class="ai-split-layout">
                        <div class="ai-response-card">
                            <div class="ai-text">
                                {move || if ai_response.get().is_empty() && is_generating.get() {
                                    view! {
                                        <span class="ai-thinking">
                                            {translate(TransKey::ThinkingMsg, locale.get())}
                                            <span class="dot-bounce">"."</span>
                                            <span class="dot-bounce dot-2">"."</span>
                                            <span class="dot-bounce dot-3">"."</span>
                                        </span>
                                    }.into_view()
                                 } else {
                                    view! { {ai_response.get()} }.into_view()
                                 }}
                                {move || is_generating.get().then(|| view! { <span class="cursor-blink">"▌"</span> })}
                            </div>
                        </div>

                        // InvokeAI Image Card
                        {move || {
                            let loading = invoke_loading.get();
                            let img_opt = invoke_image.get();
                            let err_opt = invoke_error.get();
                            (loading || img_opt.is_some() || err_opt.is_some()).then(|| view! {
                                <div class="invoke-image-card">
                                    {if loading {
                                        view! {
                                            <div class="invoke-loading-container">
                                                <div class="invoke-spinner"></div>
                                                <span class="invoke-loading-text">"Generating image..."</span>
                                            </div>
                                        }.into_view()
                                    } else if let Some(err) = err_opt {
                                        view! { <div class="invoke-error-text">{err}</div> }.into_view()
                                    } else if let Some(img_url) = img_opt {
                                        view! {
                                            <a href=img_url.clone() target="_blank">
                                                <img src=img_url.clone() alt="AI Generated Illustration" class="invoke-img" />
                                            </a>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }}
                                </div>
                            })
                        }}
                    </div>
                </section>
            })}

            // Search Box
            <div class="search-section">
                <form class="search-form" on:submit=on_search>
                    <div class="search-input-group">
                        <input
                            type="text"
                            placeholder=move || translate(TransKey::SearchPlaceholder, locale.get())
                            class="search-input"
                            prop:value=query_input
                            on:input=move |ev| set_query_input.set(event_target_value(&ev))
                            disabled=is_loading
                        />
                        <button type="submit" class="search-submit-btn" disabled=is_loading>
                            {move || if is_loading.get() { translate(TransKey::SearchingBtn, locale.get()) } else { translate(TransKey::SearchBtn, locale.get()) }}
                        </button>
                    </div>
                </form>
            </div>

            // Web Text Search Results
            {move || (!text_results.get().is_empty()).then(|| view! {
                <section class="search-results-list">
                    <h3>{move || translate(TransKey::WebResultsTitle, locale.get())}</h3>
                    <div class="results-grid">
                        {move || text_results.get().into_iter().map(|res| view! {
                            <div class="result-item">
                                <a href=res.url.clone() target="_blank" class="result-title">{res.title}</a>
                                <div class="result-url">{res.url}</div>
                                <p class="result-snippet">{res.snippet}</p>
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                </section>
            })}
        </main>
    }
}
