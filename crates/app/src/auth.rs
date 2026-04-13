use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::spawn_local;

use crate::api;

#[derive(Clone)]
pub enum SessionState {
    Checking,
    LoggedOut,
    LoggedIn(api::CurrentUser),
    Error(String),
}

#[derive(Clone, Copy)]
pub struct AuthContext {
    pub session_state: ReadSignal<SessionState>,
    pub set_session_state: WriteSignal<SessionState>,
    pub is_logging_in: ReadSignal<bool>,
    pub set_logging_in: WriteSignal<bool>,
    pub is_logging_out: ReadSignal<bool>,
    pub set_logging_out: WriteSignal<bool>,
}

pub fn refresh_session_state(set_session_state: WriteSignal<SessionState>) {
    spawn_local(async move {
        match api::get_current_user().await {
            Ok(user) => set_session_state.set(SessionState::LoggedIn(user)),
            Err(err) => {
                if err.contains("Not logged in") {
                    set_session_state.set(SessionState::LoggedOut);
                } else {
                    set_session_state.set(SessionState::Error(err));
                }
            }
        }
    });
}

/// Publishes an `AuthContext` into the Leptos context and starts reactive session management.
///
/// The function creates and provides signals for session state and login/logout flags, triggers
/// an immediate session refresh, schedules a 9000ms fallback that marks the session check as an
/// error if it remains `SessionState::Checking`, and re-checks the session when the window gains
/// focus or the document's visibility changes.
///
/// # Examples
///
/// ```
/// use leptos::mount_to_body;
/// use crate::auth::provide_auth_context;
///
/// fn main() {
///     provide_auth_context();
///     // mount app...
///     mount_to_body(|cx| view! { cx, <div>"App"</div> });
/// }
/// ```
pub fn provide_auth_context() {
    let (session_state, set_session_state) = signal(SessionState::Checking);
    let (is_logging_in, set_logging_in) = signal(false);
    let (is_logging_out, set_logging_out) = signal(false);

    provide_context(AuthContext {
        session_state,
        set_session_state,
        is_logging_in,
        set_logging_in,
        is_logging_out,
        set_logging_out,
    });

    Effect::new(move |_| {
        refresh_session_state(set_session_state);

        if let Some(window) = web_sys::window() {
            let timeout_callback = Closure::wrap(Box::new(move || {
                if matches!(session_state.get_untracked(), SessionState::Checking) {
                    set_session_state.set(SessionState::Error(
                        "Session check timed out. Verify API is reachable, then retry.".to_string(),
                    ));
                }
            }) as Box<dyn FnMut()>);

            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout_callback.as_ref().unchecked_ref(),
                9000,
            );
            timeout_callback.forget();
        }

        if let Some(window) = web_sys::window() {
            let focus_refresh = Closure::wrap(Box::new(move || {
                refresh_session_state(set_session_state);
            }) as Box<dyn FnMut()>);

            let _ = window
                .add_event_listener_with_callback("focus", focus_refresh.as_ref().unchecked_ref());
            focus_refresh.forget();

            if let Some(document) = window.document() {
                let visibility_refresh = Closure::wrap(Box::new(move || {
                    refresh_session_state(set_session_state);
                }) as Box<dyn FnMut()>);

                let _ = document.add_event_listener_with_callback(
                    "visibilitychange",
                    visibility_refresh.as_ref().unchecked_ref(),
                );
                visibility_refresh.forget();
            }
        }
    });
}

pub fn use_auth_context() -> AuthContext {
    expect_context::<AuthContext>()
}
