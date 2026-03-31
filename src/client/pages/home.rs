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

#[component]
pub(crate) fn HomePage() -> impl IntoView {
    let feed = RwSignal::new(DiceRollFeed::new());

    let load_older_rolls = || {};

    let process_roll = move |expr: String| {
        let result = parse_and_roll(&expr).unwrap();
        let new_roll = DiceRoll {
            id: String::new(),
            user_id: String::new(),
            username: String::from("You"),
            ts: format_timestamp(Utc::now()),
            expr,
            result: result.total(),
            breakdown: result.to_string(),
        };

        feed.write().add_roll(new_roll);
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
