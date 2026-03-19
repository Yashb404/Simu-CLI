use leptos::prelude::*;

#[component]
pub fn ConfirmDialog(
    open: Signal<bool>,
    title: Signal<String>,
    message: Signal<String>,
    confirm_label: &'static str,
    processing_label: &'static str,
    cancel_label: &'static str,
    is_processing: Signal<bool>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div class="dialog-backdrop" role="presentation" on:click=move |_| {
                if !is_processing.get() {
                    on_cancel.run(());
                }
            }>
                <section
                    class="dialog-panel"
                    role="dialog"
                    aria-modal="true"
                    aria-live="polite"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <header class="dialog-header">
                        <h3>{move || title.get()}</h3>
                    </header>
                    <p class="dialog-message">{move || message.get()}</p>
                    <div class="dialog-actions">
                        <button
                            type="button"
                            class="button btn-outline"
                            disabled=move || is_processing.get()
                            on:click=move |_| on_cancel.run(())
                        >
                            {cancel_label}
                        </button>
                        <button
                            type="button"
                            class="button btn-danger"
                            disabled=move || is_processing.get()
                            on:click=move |_| on_confirm.run(())
                        >
                            {move || if is_processing.get() { processing_label } else { confirm_label }}
                        </button>
                    </div>
                </section>
            </div>
        </Show>
    }
}
