use crate::i18n::{translate, Locale, TransKey};
use crate::types::{ImageSearchResult, TextSearchResult};
use leptos::*;

#[component]
pub fn SearchPanel<F>(
    locale: ReadSignal<Locale>,
    query_input: ReadSignal<String>,
    set_query_input: WriteSignal<String>,
    search_type: ReadSignal<String>,
    set_search_type: WriteSignal<String>,
    is_loading: ReadSignal<bool>,
    ai_response: ReadSignal<String>,
    is_generating: ReadSignal<bool>,
    text_results: ReadSignal<Vec<TextSearchResult>>,
    image_results: ReadSignal<Vec<ImageSearchResult>>,
    on_search: F,
) -> impl IntoView
where
    F: Fn(ev::SubmitEvent) + 'static + Copy,
{
    let has_ai = move || !ai_response.get().is_empty() || is_generating.get();

    view! {
        <main class="main-content">
            // AI Response Section
            {move || has_ai().then(|| view! {
                <section class="ai-response-section">
                    <div class="ai-response-card">

                        <div class="ai-text">
                            {move || if ai_response.get().is_empty() && is_generating.get() {
                                view! { <span class="ai-thinking">{translate(TransKey::ThinkingMsg, locale.get())}</span> }.into_view()
                             } else {
                                view! { {ai_response.get()} }.into_view()
                             }}
                            {move || is_generating.get().then(|| view! { <span class="cursor-blink">"▌"</span> })}
                        </div>
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
                    <div class="search-type-selector">
                        <label class=move || if search_type.get() == "text" { "active" } else { "" }>
                            <input
                                type="radio"
                                name="search_type"
                                value="text"
                                checked=move || search_type.get() == "text"
                                on:change=move |_| set_search_type.set("text".to_string())
                            />
                            {move || translate(TransKey::WebSearchOption, locale.get())}
                        </label>
                        <label class=move || if search_type.get() == "images" { "active" } else { "" }>
                            <input
                                type="radio"
                                name="search_type"
                                value="images"
                                checked=move || search_type.get() == "images"
                                on:change=move |_| set_search_type.set("images".to_string())
                            />
                            {move || translate(TransKey::ImageSearchOption, locale.get())}
                        </label>
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

            // Image Search Results
            {move || (!image_results.get().is_empty()).then(|| view! {
                <section class="image-results-list">
                    <h3>{move || translate(TransKey::ImageResultsTitle, locale.get())}</h3>
                    <div class="image-grid">
                        {move || image_results.get().into_iter().map(|res| view! {
                            <div class="image-item">
                                <a href=res.url target="_blank" title=res.title.clone()>
                                    <img src=res.thumbnail alt=res.title.clone() class="image-thumbnail" />
                                </a>
                                <a href=res.source_url target="_blank" class="image-source-link">{move || translate(TransKey::SourceLink, locale.get())}</a>
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                </section>
            })}
        </main>
    }
}
