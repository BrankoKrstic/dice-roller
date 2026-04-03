use leptos::{html, prelude::*};

use crate::client::utils::roll_feed::DiceRollFeed;

stylance::import_style!(style, "roll_feed.module.scss");

fn scroll_to_bottom(node_ref: &NodeRef<html::Div>) -> bool {
    if let Some(node) = node_ref.get() {
        node.set_scroll_top(node.scroll_height());
        return true;
    }

    false
}

fn is_scroll_near_bottom(node_ref: &NodeRef<html::Div>) -> bool {
    if let Some(node) = node_ref.get() {
        let max_scroll_top = node.scroll_height().saturating_sub(node.client_height());
        return max_scroll_top.saturating_sub(node.scroll_top()) <= 24;
    }
    true
}

fn should_follow_new_roll(previous_roll_count: usize, was_near_bottom: bool) -> bool {
    previous_roll_count == 0 || was_near_bottom
}

#[component]
pub fn RollFeed(
    #[prop(into)] feed: Signal<DiceRollFeed>,
    #[prop(into)] loading_more: Signal<bool>,
    #[prop(into)] load_older_rolls: Callback<()>,
) -> impl IntoView {
    let unread_rolls = RwSignal::new(0);
    let roll_feed_ref = NodeRef::<html::Div>::new();
    let last_roll_count = RwSignal::new(0usize);
    let pending_restore_height = RwSignal::new(None::<i32>);
    let was_near_bottom = RwSignal::new(true);

    let on_roll_feed_scroll = move |_| {
        if let Some(container) = roll_feed_ref.get() {
            if feed.get_untracked().has_more
                && !loading_more.get_untracked()
                && container.scroll_top() <= 80
            {
                pending_restore_height.set(Some(container.scroll_height()));
                load_older_rolls.run(());
            }

            if is_scroll_near_bottom(&roll_feed_ref) {
                unread_rolls.set(0);
            }

            was_near_bottom.set(is_scroll_near_bottom(&roll_feed_ref));
        }
    };

    Effect::new(move |_| {
        let roll_count = feed.get().rolls.len();
        let is_loading_more = loading_more.get();

        if let Some(container) = roll_feed_ref.get() {
            let previous_roll_count = last_roll_count.get();
            let previous_height = pending_restore_height.get();
            let near_bottom_before_update = was_near_bottom.get();

            if roll_count != previous_roll_count {
                if let Some(previous_height) = previous_height {
                    if !is_loading_more && roll_count > previous_roll_count {
                        let height_delta =
                            container.scroll_height().saturating_sub(previous_height);
                        container
                            .set_scroll_top(container.scroll_top().saturating_add(height_delta));
                        pending_restore_height.set(None);
                    }
                } else if should_follow_new_roll(previous_roll_count, near_bottom_before_update) {
                    scroll_to_bottom(&roll_feed_ref);
                    unread_rolls.set(0);
                } else if roll_count > previous_roll_count {
                    unread_rolls
                        .update(|count| *count += roll_count.saturating_sub(previous_roll_count));
                }

                was_near_bottom.set(is_scroll_near_bottom(&roll_feed_ref));
                last_roll_count.set(roll_count);
            }
        } else if roll_count != last_roll_count.get() {
            last_roll_count.set(roll_count);
        }
    });

    view! {
        <section class=style::room_top_grid>
            <article class=style::rooms_card>
                <div class=style::rooms_card_header>
                    <p class="g-section-label">"Activity"</p>
                    <h2 class=style::rooms_card_title>"Room Activity"</h2>
                    <p class=style::rooms_card_summary>"Recent rolls show up here."</p>
                    <Show when=move || unread_rolls.get() != 0>
                        <button
                            class=format!("g-button-utility {}", style::roll_feed_jump)
                            type="button"
                            on:click=move |_| {
                                scroll_to_bottom(&roll_feed_ref);
                                unread_rolls.set(0);
                                was_near_bottom.set(true);
                            }
                        >
                            {move || {
                                let count = unread_rolls.get();
                                if count == 1 {
                                    "1 new roll".to_string()
                                } else {
                                    format!("{count} new rolls")
                                }
                            }}
                        </button>
                    </Show>
                </div>
                <div
                    class=style::roll_feed_scroll
                    node_ref=roll_feed_ref
                    on:scroll=on_roll_feed_scroll
                >
                    <Show when=move || loading_more.get()>
                        <p class="g-result-hint">"Loading earlier entries..."</p>
                    </Show>
                    {move || {
                        let roll_count = feed.get().rolls.len();
                        if roll_count == 0 && !feed.get().has_more {
                            view! { <p class="g-result-hint">"No rolls recorded yet."</p> }
                                .into_any()
                        } else {
                            view! {
                                <ul class=style::roll_feed_item_list>
                                    <For
                                        each=move || { feed.get().rolls.clone() }
                                        key=|roll| roll.id.clone()
                                        children=move |roll| {
                                            view! {
                                                <li class=style::roll_feed_item>
                                                    <div class=style::roll_feed_item_header>
                                                        <strong class=style::roll_feed_item_user>
                                                            {roll.username.clone()}
                                                        </strong>
                                                        <span class=style::roll_feed_item_total>{roll.result}</span>
                                                    </div>
                                                    <p class=style::roll_feed_item_meta>{roll.ts}</p>
                                                    <p class=style::roll_feed_item_expression>
                                                        <code>{roll.expr}</code>
                                                    </p>
                                                    <details class=style::roll_feed_item_details>
                                                        <summary>"Breakdown"</summary>
                                                        <pre class=style::roll_feed_item_breakdown>
                                                            {roll.breakdown}
                                                        </pre>
                                                    </details>
                                                </li>
                                            }
                                        }
                                    />
                                </ul>
                            }
                                .into_any()
                        }
                    }}

                    <Show when=move || { feed.get().has_more && !loading_more.get() }>
                        <p class="g-result-hint">"Scroll upward to pull earlier room activity."</p>
                    </Show>
                </div>
            </article>
        </section>
    }
}
