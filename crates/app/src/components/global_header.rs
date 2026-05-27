use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;
use crate::auth::{SessionState, use_auth_context};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CentralHeaderVariant {
    Workspace,
    Editor,
    Docs,
    Marketing,
    SharedDemo,
}

#[derive(Clone, Copy)]
pub struct HeaderSearchModel {
    pub query: ReadSignal<String>,
    pub set_query: WriteSignal<String>,
    pub placeholder: &'static str,
}

#[derive(Clone, Copy)]
pub struct HeaderInputModel {
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
    pub placeholder: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HeaderActionTone {
    Default,
    Strong,
}

#[derive(Clone)]
pub struct HeaderAction {
    pub label: &'static str,
    pub icon: &'static str,
    pub href: Option<String>,
    pub on_click: Option<Callback<()>>,
    pub tone: HeaderActionTone,
}

impl HeaderAction {
    pub fn link(label: &'static str, icon: &'static str, href: impl Into<String>) -> Self {
        Self {
            label,
            icon,
            href: Some(href.into()),
            on_click: None,
            tone: HeaderActionTone::Default,
        }
    }

    pub fn action(label: &'static str, icon: &'static str, on_click: Callback<()>) -> Self {
        Self {
            label,
            icon,
            href: None,
            on_click: Some(on_click),
            tone: HeaderActionTone::Default,
        }
    }

    pub fn strong_link(label: &'static str, icon: &'static str, href: impl Into<String>) -> Self {
        Self {
            tone: HeaderActionTone::Strong,
            ..Self::link(label, icon, href)
        }
    }

    pub fn strong_action(label: &'static str, icon: &'static str, on_click: Callback<()>) -> Self {
        Self {
            tone: HeaderActionTone::Strong,
            ..Self::action(label, icon, on_click)
        }
    }
}

fn header_nav_links(variant: CentralHeaderVariant) -> Vec<(&'static str, &'static str)> {
    match variant {
        CentralHeaderVariant::Workspace => vec![("Workspace", "/dashboard"), ("Docs", "/docs")],
        CentralHeaderVariant::Editor => vec![("Workspace", "/dashboard"), ("Docs", "/docs")],
        CentralHeaderVariant::Docs => vec![("Home", "/"), ("Docs", "/docs"), ("API", "/docs/api")],
        CentralHeaderVariant::Marketing => vec![("Features", "/#features"), ("Docs", "/docs")],
        CentralHeaderVariant::SharedDemo => vec![("Home", "/"), ("Docs", "/docs")],
    }
}

#[component]
fn HeaderActionView(action: HeaderAction) -> impl IntoView {
    let class = match action.tone {
        HeaderActionTone::Default => "central-header__button",
        HeaderActionTone::Strong => "central-header__button central-header__button--strong",
    };

    match (action.href.clone(), action.on_click) {
        (Some(href), _) => view! {
            <A attr:class=class href=href>
                <span class="material-symbols-outlined central-header__button-icon">{action.icon}</span>
                <span class="central-header__button-label">{action.label}</span>
            </A>
        }
        .into_any(),
        (None, Some(on_click)) => view! {
            <button
                type="button"
                class=class
                on:click=move |_| on_click.run(())
            >
                <span class="material-symbols-outlined central-header__button-icon">{action.icon}</span>
                <span class="central-header__button-label">{action.label}</span>
            </button>
        }
        .into_any(),
        _ => view! { <></> }.into_any(),
    }
}

#[component]
pub fn CentralHeader(
    variant: CentralHeaderVariant,
    #[prop(optional)] left_action: Option<HeaderAction>,
    #[prop(optional)] title_input: Option<HeaderInputModel>,
    #[prop(optional)] search: Option<HeaderSearchModel>,
    #[prop(optional)] status: Option<ReadSignal<String>>,
    #[prop(optional)] center_actions: Vec<HeaderAction>,
    #[prop(optional)] right_actions: Vec<HeaderAction>,
    #[prop(optional)] on_menu_toggle: Option<Callback<()>>,
) -> impl IntoView {
    let auth = use_auth_context();
    let left_action_view = left_action.clone().map(|action| view! { <HeaderActionView action /> });
    let title_input_slot = title_input;
    let search_slot = search;
    let status_slot = status;
    let has_center_actions = !center_actions.is_empty();
    let center_actions_store = StoredValue::new(center_actions.clone());
    let (mobile_actions_open, set_mobile_actions_open) = signal(false);

    view! {
        <header class="central-header">
            <div class="central-header__row">
                <div class="central-header__lane central-header__lane--left">
                    <Show when=move || on_menu_toggle.is_some()>
                        <button
                            type="button"
                            class="central-header__icon-button central-header__icon-button--mobile"
                            aria-label="Open navigation"
                            on:click=move |_| {
                                if let Some(toggle) = on_menu_toggle {
                                    toggle.run(());
                                }
                            }
                        >
                            <span class="material-symbols-outlined">"menu"</span>
                        </button>
                    </Show>

                    <A attr:class="central-header__brand" href="/">
                        <span class="central-header__brand-mark">">_"</span>
                        <span class="central-header__brand-name">"SimuCLI"</span>
                    </A>

                    <nav class="central-header__nav">
                        <For
                            each=move || header_nav_links(variant)
                            key=|(label, _)| *label
                            children=move |(label, href)| {
                                view! { <A attr:class="central-header__nav-link" href=href>{label}</A> }
                            }
                        />
                    </nav>
                    {left_action_view}
                </div>

                <div class="central-header__lane central-header__lane--center">
                    <Show when=move || search_slot.is_some()>
                        {move || {
                            search_slot.clone().map(|search| {
                                view! {
                                    <label class="central-header__search">
                                        <span class="material-symbols-outlined central-header__search-icon">"search"</span>
                                        <input
                                            class="central-header__search-input"
                                            type="search"
                                            placeholder=search.placeholder
                                            prop:value=move || search.query.get()
                                            on:input=move |event| search.set_query.set(event_target_value(&event))
                                        />
                                    </label>
                                }
                            })
                        }}
                    </Show>

                    <Show when=move || title_input_slot.is_some()>
                        {move || {
                            title_input_slot.clone().map(|title_input| {
                                view! {
                                    <label class="central-header__title-input-wrap">
                                        <input
                                            class="central-header__title-input"
                                            type="text"
                                            placeholder=title_input.placeholder
                                            prop:value=move || title_input.value.get()
                                            on:input=move |event| title_input.set_value.set(event_target_value(&event))
                                        />
                                    </label>
                                }
                            })
                        }}
                    </Show>

                    <Show when=move || status_slot.is_some()>
                        {move || {
                            status_slot.clone().map(|status| {
                                view! {
                                    <span class="central-header__status">
                                        {move || {
                                            let text = status.get();
                                            if text.trim().is_empty() {
                                                "READY".to_string()
                                            } else {
                                                text
                                            }
                                        }}
                                    </span>
                                }
                            })
                        }}
                    </Show>

                    <div class="central-header__cluster central-header__cluster--secondary">
                        <For
                            each=move || center_actions_store.get_value()
                            key=|action| action.label
                            children=move |action| view! { <HeaderActionView action /> }
                        />
                    </div>

                    <Show when=move || has_center_actions>
                        <button
                            type="button"
                            class="central-header__button central-header__button--menu"
                            aria-label="Open more actions"
                            aria-expanded=move || mobile_actions_open.get()
                            on:click=move |_| set_mobile_actions_open.update(|open| *open = !*open)
                        >
                            <span class="material-symbols-outlined central-header__button-icon">"more_horiz"</span>
                            <span class="central-header__button-label">"More Actions"</span>
                        </button>
                    </Show>
                </div>

                <div class="central-header__lane central-header__lane--right">
                    <div class="central-header__cluster">
                        <For
                            each=move || right_actions.clone()
                            key=|action| action.label
                            children=move |action| view! { <HeaderActionView action /> }
                        />
                    </div>

                    <A attr:class="central-header__icon-button" href="/dashboard">
                        <span class="material-symbols-outlined">"settings"</span>
                    </A>

                    {move || match auth.session_state.get() {
                        SessionState::LoggedIn(user) => {
                            let username = user.username;
                            let avatar_url = user.avatar_url;
                            let has_avatar = avatar_url.is_some();
                            let initial = username
                                .chars()
                                .next()
                                .unwrap_or('U')
                                .to_ascii_uppercase()
                                .to_string();

                            view! {
                                <A attr:class="central-header__profile" href="/dashboard">
                                    <span class="central-header__profile-name">{format!("@{username}")}</span>
                                    <Show
                                        when=move || has_avatar
                                        fallback=move || {
                                            view! { <span class="central-header__avatar-fallback">{initial.clone()}</span> }
                                        }
                                    >
                                        <img
                                            class="central-header__avatar"
                                            src=avatar_url.clone().unwrap_or_default()
                                            alt="User avatar"
                                        />
                                    </Show>
                                </A>
                            }
                            .into_any()
                        }
                        _ => view! {
                            <a class="central-header__button" href={api::login_url()}>
                                <span class="material-symbols-outlined central-header__button-icon">"login"</span>
                                <span>"Login"</span>
                            </a>
                        }
                        .into_any(),
                    }}
                </div>
            </div>

            <Show when=move || mobile_actions_open.get()>
                <div class="central-header__mobile-menu">
                    <div class="central-header__mobile-menu-panel">
                        <For
                            each=move || center_actions_store.get_value()
                            key=|action| action.label
                            children=move |action| view! { <HeaderActionView action /> }
                        />
                    </div>
                </div>
            </Show>
        </header>
    }
}
