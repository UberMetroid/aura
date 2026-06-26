use crate::i18n::{Locale, TransKey, translate};
use leptos::*;

#[component]
pub fn Login<F>(
    locale: ReadSignal<Locale>,
    is_loading: ReadSignal<bool>,
    access_key_input: ReadSignal<String>,
    set_access_key_input: WriteSignal<String>,
    error_message: ReadSignal<String>,
    on_submit: F,
) -> impl IntoView
where
    F: Fn(ev::SubmitEvent) + 'static,
{
    view! {
        <div class="login-container">
            <div class="login-box">
                <div class="pin-header">
                    <h2 id="pin-description">{move || translate(TransKey::EnterPin, locale.get())}</h2>
                    <p>{move || translate(TransKey::PinDescription, locale.get())}</p>
                </div>
                <form id="pin-form" on:submit=on_submit>
                    <div class="pin-wrapper">
                        <input
                            type="password"
                            class="pin-input-field"
                            placeholder=move || translate(TransKey::PinInputPlaceholder, locale.get())
                            prop:value=access_key_input
                            on:input=move |ev| set_access_key_input.set(event_target_value(&ev))
                            disabled=is_loading
                            autofocus=true
                        />
                    </div>
                    <button type="submit" class="auth-submit-btn" disabled=is_loading style="margin-top: 1.25rem;">
                        {move || if is_loading.get() { translate(TransKey::SearchingBtn, locale.get()) } else { translate(TransKey::SearchBtn, locale.get()) }}
                    </button>
                </form>
                {move || (!error_message.get().is_empty()).then(|| view! {
                    <div class="pin-status">
                        <p class="pin-error" style="display: block;">{error_message.get()}</p>
                    </div>
                })}
            </div>
        </div>
    }
}
