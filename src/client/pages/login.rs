use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use serde::Serialize;
use web_sys::SubmitEvent;

use crate::{
    client::{
        context::auth::use_auth_context,
        utils::{api::parse_error_response, url::base_url},
    },
    shared::data::user::AuthUser,
};

stylance::import_style!(style, "login.module.scss");

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

async fn login_user_request(payload: LoginRequest) -> Result<AuthUser, String> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/api/auth/login", base_url()))
        .json(&payload)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if !(200..300).contains(&res.status().as_u16()) {
        return Err(parse_error_response(res, "Failed to sign in").await);
    }

    let payload: AuthUser = res.json().await.map_err(|error| error.to_string())?;
    Ok(payload)
}

#[component]
pub(super) fn LoginPage() -> impl IntoView {
    let auth = use_auth_context();
    let navigate = use_navigate();

    let login = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let submitting = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let on_submit = move |event: SubmitEvent| {
        event.prevent_default();
        if submitting.get_untracked() {
            return;
        }

        submitting.set(true);
        error.set(None);

        let auth = auth.clone();
        let navigate = navigate.clone();
        let payload = LoginRequest {
            email: login.get_untracked(),
            password: password.get_untracked(),
        };

        spawn_local(async move {
            match login_user_request(payload).await {
                Ok(user) => {
                    auth.user.set(Some(user));
                    auth.loading.set(false);
                    navigate("/", Default::default());
                }
                Err(message) => {
                    error.set(Some(message));
                }
            }
            submitting.set(false);
        });
    };

    view! {
        <section class=format!("g-page g-page-shell {}", style::page_auth)>
            <div class=style::auth_layout>
                <header class="g-panel g-panel-strong">
                    <p class="g-section-label">"Account access"</p>
                    <h1 class="g-section-title">"Return to your table."</h1>
                    <p class="g-section-summary">
                        "Sign in for protected room routes and account-aware history."
                    </p>
                </header>

                <article class=style::auth_card>
                    <p class="g-section-label">"Sign in"</p>
                    <form class=style::auth_form on:submit=on_submit>
                        <label class="g-field-label" for="login-input">
                            "Email"
                        </label>
                        <input
                            id="login-input"
                            class="g-text-input"
                            type="text"
                            prop:value=move || login.get()
                            on:input=move |event| login.set(event_target_value(&event))
                            autocomplete="username"
                            required=true
                        />

                        <label class="g-field-label" for="login-password-input">
                            "Password"
                        </label>
                        <input
                            id="login-password-input"
                            class="g-text-input"
                            type="password"
                            prop:value=move || password.get()
                            on:input=move |event| password.set(event_target_value(&event))
                            autocomplete="current-password"
                            required=true
                        />

                        <button
                            class="g-button-action"
                            type="submit"
                            prop:disabled=move || submitting.get()
                        >
                            {move || if submitting.get() { "Signing in..." } else { "Sign In" }}
                        </button>

                        {move || {
                            error
                                .get()
                                .map(|message| {
                                    view! {
                                        <p class=format!(
                                            "{} {}",
                                            style::auth_feedback,
                                            style::auth_feedback_error,
                                        )>{message}</p>
                                    }
                                })
                        }}
                    </form>

                    <p class=style::auth_switch>
                        "Need an account? " <a class=style::auth_switch_link href="/register">
                            "Register"
                        </a>
                    </p>
                </article>
            </div>
        </section>
    }
}
