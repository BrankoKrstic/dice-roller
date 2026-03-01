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

#[component]
pub fn RollFeed(
    #[prop(into)] feed: Signal<DiceRollFeed>,
    #[prop(into)] loading_more: Signal<bool>,
    #[prop(into)] load_older_rolls: Callback<()>,
) -> impl IntoView {
    let unread_rolls = RwSignal::new(0);

    let roll_feed_ref = NodeRef::<html::Div>::new();

    let on_roll_feed_scroll = move |_| {
        if feed.get_untracked().has_more && !loading_more.get_untracked() {
            return;
        }

        if let Some(container) = roll_feed_ref.get() {
            if container.scroll_top() <= 80 {
                load_older_rolls.run(());
            }
            if is_scroll_near_bottom(&roll_feed_ref) {
                unread_rolls.set(0);
            }
        }
    };
    view! {
        <section class=style::room_top_grid>
            <article class=style::rooms_card>
                <div class=style::rooms_card_header>
                    <h2 style::rooms_card_title>"Roll Feed"</h2>
                    <Show when=move || unread_rolls.get() != 0>
                        <button
                            class=format!("button-secondary {}", style::roll_feed_jump)
                            type="button"
                            on:click=move |_| {
                                scroll_to_bottom(&roll_feed_ref);
                                unread_rolls.set(0);
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
                        <p class="result-card__hint">"Loading older rolls..."</p>
                    </Show>
                    {move || {
                        let roll_count = feed.get().rolls.len();
                        if roll_count == 0 && !feed.get().has_more {
                            view! { <p class="result-card__hint">"No rolls yet."</p> }.into_any()
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
                                                            {roll.user_name.clone()}
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
                        <p class="result-card__hint">"Scroll up to load older rolls."</p>
                    </Show>
                </div>
            </article>
        </section>
    }
}
