use leptos::{logging, prelude::*};

use crate::{client::utils::url::base_url, shared::data::user::{AuthContext, AuthUser}};


pub async fn logout() {
    let ctx = use_auth_context();
    ctx.loading.set(true);
    let client = reqwest::Client::new();
    logging::log!("HEY I AM HERE");
    let res = client
        .post(format!("{}/api/auth/logout", base_url()))
        .body("")
        .send()
        .await
        .map_err(|err| err.to_string());

    if res.is_ok() {
        ctx.user.set(None);
    }
    ctx.loading.set(false);
}

#[server]
async fn load_user_data() -> Result<Option<AuthUser>, ServerFnError> {
    use crate::server::services::auth::AuthService;
    use axum::extract::State;
    
    let state = expect_context::<crate::server::api::AppState>();
    
    let (jar, State(auth)): (axum_extra::extract::CookieJar, State<AuthService>) = leptos_axum::extract_with_state(&state).await?;

    Ok(auth.check_token(jar)?)
}

pub fn provide_auth_context() {
    let ctx = AuthContext {
        user: RwSignal::new(None),
        loading: RwSignal::new(true),
    };
    provide_context::<AuthContext>(ctx.clone());

    let resource = OnceResource::new_blocking(load_user_data());

    Effect::new(move || {
        if let Some(val) = resource.get() {
            ctx.loading.set(false);
            if let Ok(user) = val {
                ctx.user.set(user);
            }
        }
    });
}


pub (crate) fn use_auth_context() -> AuthContext {
    use_context::<AuthContext>().unwrap_or(AuthContext::new(None))
}
