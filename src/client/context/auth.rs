use leptos::prelude::*;

use crate::shared::data::user::AuthUser;

#[derive(Clone)]
pub(crate) struct AuthContext {
    pub(crate) user: RwSignal<Option<AuthUser>>,
    pub(crate) loading: RwSignal<bool>,
}

pub fn provide_auth_context() {
    let ctx = AuthContext {
        user: RwSignal::new(None),
        loading: RwSignal::new(false),
    };
    provide_context::<AuthContext>(ctx);
}

pub fn use_auth_context() -> AuthContext {
    use_context::<AuthContext>().expect("Signal shoudl be provided")
}
