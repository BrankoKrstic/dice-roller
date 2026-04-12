use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use serde::Serialize;
use web_sys::SubmitEvent;

use crate::{
    client::{
        context::{auth::use_auth_context, page_title::use_static_page_title},
        utils::{
            api::parse_error_response,
            rooms::{MAX_USERNAME_LENGTH, validate_username_input},
            url::base_url,
        },
    },
    shared::data::user::AuthUser,
};

stylance::import_style!(style, "register.module.scss");

#[derive(Serialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

async fn register_user_request(payload: RegisterRequest) -> Result<AuthUser, String> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/api/auth/register", base_url()))
        .json(&payload)
        .send()
        .await
        .map_err(|err| {
            leptos::logging::debug_error!("{:?}", err);
            err.to_string()
        })?;

    if !(200..300).contains(&res.status().as_u16()) {
        return Err(parse_error_response(res, "Failed to register user").await);
    }

    let payload: AuthUser = res.json().await.map_err(|error| error.to_string())?;
    Ok(payload)
}

#[component]
pub(super) fn RegisterPage() -> impl IntoView {
    use_static_page_title("Register");

    let auth = use_auth_context();
    let navigate = use_navigate();

    let username = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let submitting = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let redirect_home = navigate.clone();
    Effect::new(move |_| {
        if auth.loading.get() || auth.user.get().is_none() {
            return;
        }

        redirect_home("/", Default::default());
    });

    let on_submit = move |event: SubmitEvent| {
        event.prevent_default();
        if submitting.get_untracked() {
            return;
        }

        submitting.set(true);
        error.set(None);

        let username_value = match validate_username_input(&username.get_untracked()) {
            Ok(username) if username.len() >= 2 => username,
            Ok(_) => {
                submitting.set(false);
                error.set(Some("User names need at least 2 characters.".to_string()));
                return;
            }
            Err(message) => {
                submitting.set(false);
                error.set(Some(message));
                return;
            }
        };

        let auth = auth.clone();
        let navigate = navigate.clone();

        let payload = RegisterRequest {
            username: username_value,
            email: email.get_untracked(),
            password: password.get_untracked(),
        };

        spawn_local(async move {
            match register_user_request(payload).await {
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
                    <p class="g-section-label">"New account"</p>
                    <h1 class="g-section-title">"Open a table identity."</h1>
                    <p class="g-section-summary">
                        "Create an account so your name can stay attached to protected routes and shared session history."
                    </p>
                </header>

                <article class=style::auth_card>
                    <p class="g-section-label">"Register"</p>
                    <form class=style::auth_form on:submit=on_submit>
                        <label class="g-field-label" for="register-username-input">
                            "Username"
                        </label>
                        <input
                            id="register-username-input"
                            class="g-text-input"
                            type="text"
                            maxlength=MAX_USERNAME_LENGTH
                            prop:value=move || username.get()
                            on:input=move |event| {
                                error.set(None);
                                username.set(event_target_value(&event));
                            }
                            autocomplete="username"
                            required=true
                        />

                        <label class="g-field-label" for="register-email-input">
                            "Email"
                        </label>
                        <input
                            id="register-email-input"
                            class="g-text-input"
                            type="email"
                            prop:value=move || email.get()
                            on:input=move |event| email.set(event_target_value(&event))
                            autocomplete="email"
                            required=true
                        />

                        <label class="g-field-label" for="register-password-input">
                            "Password"
                        </label>
                        <input
                            id="register-password-input"
                            class="g-text-input"
                            type="password"
                            prop:value=move || password.get()
                            on:input=move |event| password.set(event_target_value(&event))
                            autocomplete="new-password"
                            required=true
                        />

                        <button
                            class="g-button-action"
                            type="submit"
                            prop:disabled=move || submitting.get()
                        >
                            {move || if submitting.get() { "Creating account..." } else { "Register" }}
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

                    <ul class=style::auth_side_list>
                        <li>"Usernames become the visible voice in the roll history."</li>
                        <li>"Email stays attached to recovery and auth flows."</li>
                        <li>"You can jump straight back to the roller after registration."</li>
                    </ul>

                    <p class=style::auth_switch>
                        "Already have an account? " <a class=style::auth_switch_link href="/login">
                            "Sign in"
                        </a>
                    </p>
                </article>
            </div>
        </section>
    }
}
