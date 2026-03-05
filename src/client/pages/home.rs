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

#[component]
pub(crate) fn HomePage() -> impl IntoView {
    let feed = RwSignal::new(DiceRollFeed::new());

    let load_older_rolls = || {};

    let process_roll = move |expr: String| {
        let result = parse_and_roll(&expr).unwrap();
        let new_roll = DiceRoll {
            id: String::new(),
            user_id: String::new(),
            user_name: String::from("You"),
            ts: format_timestamp(Utc::now()),
            expr,
            result: result.total(),
            breakdown: result.to_string(),
        };

        feed.write().add_roll(new_roll);
    };
    view! {
        <div class="page">
            <RollEditor on_roll=process_roll />
            <RollFeed feed=feed loading_more=false load_older_rolls=load_older_rolls />
        </div>
    }
}
