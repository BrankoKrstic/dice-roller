use leptos::prelude::*;

use crate::client::{
    components::roll_editor::RollEditor,
    context::theme::{toggle_theme, use_theme_context, Theme},
};

#[component]
pub(crate) fn HomePage() -> impl IntoView {
    let theme = use_theme_context();

    view! {
        <div>
            <RollEditor />
        </div>
    }
}
