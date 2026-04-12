use leptos::prelude::*;

use crate::client::{
    context::page_title::use_static_page_title,
    pages::reference_content::{REFERENCE_SECTIONS, ReferenceEntry},
};

stylance::import_style!(style, "reference.module.scss");

fn render_notes(notes: &'static [&'static str]) -> impl IntoView {
    if notes.is_empty() {
        ().into_any()
    } else {
        view! {
            <ul class=style::reference_entry_notes>
                {notes.iter().map(|note| view! { <li>{*note}</li> }).collect_view()}
            </ul>
        }
        .into_any()
    }
}

fn render_entry(entry: &'static ReferenceEntry) -> impl IntoView {
    view! {
        <article class=style::reference_entry>
            <code class=style::reference_entry_syntax>{entry.syntax}</code>
            <p class=style::reference_entry_meaning>{entry.meaning}</p>
            {render_notes(entry.notes)}
        </article>
    }
}

#[component]
pub fn ReferencePage() -> impl IntoView {
    use_static_page_title("Reference");

    view! {
        <section class=format!("g-page g-page-shell {}", style::reference_layout)>
            <div class=format!("g-panel g-panel-strong {}", style::reference_hero)>
                <p class="g-section-label">"Expressions"</p>
                <h1 class="g-section-title">"Expression Editor Reference"</h1>
                <p class=style::reference_hero_summary>
                    "Use Dice Bench when you want a quick draft. Use Expression Editor to take advantage of the full power of the dice roll notation."
                </p>
            </div>

            {REFERENCE_SECTIONS
                .iter()
                .map(|section| {
                    view! {
                        <section class=format!("g-panel {}", style::reference_section)>
                            <p class="g-section-label">{section.label}</p>
                            <h2 class="g-section-title">{section.title}</h2>
                            <p class="g-section-summary">{section.summary}</p>
                            <div class=style::reference_entry_grid>
                                {section
                                    .entries
                                    .iter()
                                    .map(|entry| render_entry(entry))
                                    .collect_view()}
                            </div>
                        </section>
                    }
                })
                .collect_view()}
        </section>
    }
}
