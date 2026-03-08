use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use serde::{Serialize};
use web_sys::SubmitEvent;

use crate::{client::context::auth::use_auth_context, shared::data::user::AuthUser};

#[derive(Serialize)]
struct LoginRequest {
	email: String,
	password: String,
}



pub(crate) async fn login_user_request(payload: LoginRequest) -> Result<AuthUser, String> {
	let client = reqwest::Client::new();
	let res = client.post("/api/auth/login")
		.json(&payload)
		.send()
		.await.map_err(|err| err.to_string())?;


    if !(200..300).contains(&res.status().as_u16()) {
        return Err(res.text().await.unwrap_or(String::from("Failed to register user")));
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
		<section class="page page--auth">
			<header class="page__header">
				<h1 class="page__title">"Sign In"</h1>
				<p class="page__subtitle">
					"Log in to access protected API routes and account-aware UI."
				</p>
			</header>

			<article class="auth-card">
				<form class="auth-form" on:submit=on_submit>
					<label class="editor-label" for="login-input">
						"Email or Username"
					</label>
					<input
						id="login-input"
						class="expression-input"
						type="text"
						prop:value=move || login.get()
						on:input=move |event| login.set(event_target_value(&event))
						autocomplete="username"
						required=true
					/>

					<label class="editor-label" for="login-password-input">
						"Password"
					</label>
					<input
						id="login-password-input"
						class="expression-input"
						type="password"
						prop:value=move || password.get()
						on:input=move |event| password.set(event_target_value(&event))
						autocomplete="current-password"
						required=true
					/>

					<button
						class="roll-button"
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
									<p class="auth-feedback auth-feedback--error">{message}</p>
								}
							})
					}}
				</form>

				<p class="auth-switch">
					"Need an account? " <a class="auth-switch__link" href="/register">
						"Register"
					</a>
				</p>
			</article>
		</section>
	}
}
