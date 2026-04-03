use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;
use leptos::prelude::*;

use crate::{
    client::{
        components::{roll_editor::RollEditor, roll_feed::RollFeed},
        context::auth::use_auth_context,
        utils::roll_feed::{DiceRoll, DiceRollFeed},
    },
    dsl::parse_and_roll,
    shared::utils::time::format_timestamp,
};

stylance::import_style!(style, "home.module.scss");

static LOCAL_ROLL_ID: AtomicU64 = AtomicU64::new(1);

fn next_local_roll_id() -> String {
    format!("local-{}", LOCAL_ROLL_ID.fetch_add(1, Ordering::Relaxed))
}

fn build_local_roll(expr: String, result: crate::dsl::interpreter::EvalResult) -> DiceRoll {
    DiceRoll {
        id: next_local_roll_id(),
        user_id: String::new(),
        username: String::from("You"),
        ts: format_timestamp(Utc::now()),
        expr,
        result: result.total(),
        breakdown: result.to_string(),
    }
}

#[component]
pub(crate) fn HomePage() -> impl IntoView {
    let auth = use_auth_context();
    let feed = RwSignal::new(DiceRollFeed::new());
    let roll_error = RwSignal::new(None::<String>);

    let load_older_rolls = || {};

    let process_roll = move |expr: String| {
        let result = match parse_and_roll(&expr) {
            Ok(result) => result,
            Err(error) => {
                roll_error.set(Some(error.to_string()));
                return;
            }
        };

        roll_error.set(None);
        feed.write().add_roll(build_local_roll(expr, result));
    };
    view! {
        <div class=format!("g-page g-page-shell g-page-shell-split {}", style::home_shell)>
            <section class=style::home_column>
                <RollEditor on_roll=process_roll />
                <Show when=move || roll_error.get().is_some()>
                    <p class="g-result-hint">{move || roll_error.get().unwrap_or_default()}</p>
                </Show>
            </section>

            <aside class=style::home_rail>
                <section class=format!("g-panel g-panel-strong {}", style::intro_card)>
                    <p class="g-section-label">"Local ledger"</p>
                    <h1 class="g-section-title">"Your private bench"</h1>
                    <p class="g-section-summary">
                        "Draft a roll, run it in your local ledger, and s ave a preset for later."
                    </p>
                    <ul class=style::session_list>
                        <li>"Rolls immediately append to the activity feed."</li>
                        <li>
                            "Visit the "<a href="/reference">"reference page"</a>
                            " for help with the expressio nnotation."
                        </li>
                        <Show when=move || auth.user.get().is_none()>
                            <li>
                                <a href="/register">"Create an account"</a>
                                " to save rolls as presets."
                            </li>
                        </Show>
                    </ul>
                </section>

                <RollFeed feed=feed loading_more=false load_older_rolls=load_older_rolls />
            </aside>
        </div>
    }
}
