use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;
use leptos::prelude::*;

use crate::{
    client::{
        components::{roll_editor::RollEditor, roll_feed::RollFeed},
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
                <section class=format!("g-panel g-panel-strong {}", style::intro_card)>
                    <p class="g-section-label">"Session ledger"</p>
                    <h1 class="g-section-title">"Room-first rolling, live by default."</h1>
                    <p class="g-section-summary">
                        "Draft the command, send it to the shared history, and keep the table reading one running ledger instead of separate utility panes."
                    </p>
                </section>

                <RollEditor on_roll=process_roll />
                <Show when=move || roll_error.get().is_some()>
                    <p class="g-result-hint">{move || roll_error.get().unwrap_or_default()}</p>
                </Show>
            </section>

            <aside class=style::home_rail>
                <section class=format!("g-panel g-panel-strong {}", style::session_card)>
                    <p class="g-section-label">"Current Table"</p>
                    <h2 class="g-section-title">"Solo table live, shared rooms next."</h2>
                    <p class="g-section-summary">
                        "You are drafting against the local ledger today. The shell already speaks in room language so multiplayer flows can slot in without another visual reset."
                    </p>
                    <ul class=style::session_list>
                        <li>"Rolls append immediately to the activity rail."</li>
                        <li>"Notation help lives on the dedicated reference route."</li>
                        <li>
                            "Chance analysis stays adjacent instead of competing with the main composer."
                        </li>
                    </ul>
                </section>

                <RollFeed feed=feed loading_more=false load_older_rolls=load_older_rolls />
            </aside>
        </div>
    }
}
