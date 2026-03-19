use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::{closure::Closure, JsCast};

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
            let callback = Closure::wrap(Box::new(move || {
                if matches!(
                    session_state.get_untracked(),
                    SessionState::LoggedOut | SessionState::Checking
                ) {
                    refresh_session_state(set_session_state);
                }
            }) as Box<dyn FnMut()>);

            if window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    4000,
                )
                .is_ok()
            {
                callback.forget();
            }
        }
    });
}

pub fn use_auth_context() -> AuthContext {
    expect_context::<AuthContext>()
}
