use leptos::prelude::*;

#[component]
pub fn TerminalUI() -> impl IntoView {
    let (input, set_input) = signal(String::new());

    view! {
        <section class="terminal-ui" aria-label="CLI simulator terminal">
            <header class="terminal-header">"CLI Demo Runtime"</header>
            <div class="terminal-output">
                <p>"Preview runtime initialized."</p>
            </div>
            <label class="sr-only" for="terminal-input">"Terminal input"</label>
            <input
                id="terminal-input"
                type="text"
                prop:value=input
                on:input=move |ev| set_input.set(event_target_value(&ev))
                placeholder="Type a command"
            />
        </section>
    }
}
