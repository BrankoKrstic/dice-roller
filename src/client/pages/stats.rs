use leptos::prelude::*;

use crate::client::components::roll_editor::{EditorComponent, EditorState};
stylance::import_style!(style, "stats.module.scss");

pub struct ChanceCalculatorResult {
    trials: u32,
    successes: u32,
    accumulated_result: i128,
    success_percent: f64,
    avg_dmg: f64,
    avg_dmg_per_attack: f64,
}

struct ChanceCalculator {
    trials: u32,
    successes: u32,
    accumulated_result: i128,
}

impl ChanceCalculator {
    fn new(trials: u32) -> Self {
        Self {
            trials,
            successes: 0,
            accumulated_result: 0,
        }
    }

    fn record_hit(&mut self, result: i64) {
        self.successes += 1;
        self.accumulated_result += result as i128;
    }

    fn finish(self) -> ChanceCalculatorResult {
        ChanceCalculatorResult {
            trials: self.trials,
            successes: self.successes,
            accumulated_result: self.accumulated_result,
            success_percent: self.successes as f64 / self.trials as f64,
            avg_dmg: self.accumulated_result as f64 / self.successes as f64,
            avg_dmg_per_attack: self.accumulated_result as f64 / self.trials as f64,
        }
    }
}

#[derive(Debug, Clone)]
enum CalculatorVariant {
    Ac,
    Dc,
}

#[component]
pub fn StatsPage() -> impl IntoView {
    let (variant, set_variant) = signal(CalculatorVariant::Ac);
    let (target, set_target) = signal(15);
    let to_hit_editor = RwSignal::new(EditorState::default());
    let dmg_editor = RwSignal::new(EditorState::default());
    let (running, set_running) = signal(false);

    let run_simulation = move |_| {
        set_running.set(true);
    };

    view! {
        <div class="page">
            <div class="page-tabs" role="tablist" aria-label="Calculator type">
                <button
                    class="button-secondary"
                    class:button-secondary-active=move || {
                        matches!(variant.get(), CalculatorVariant::Ac)
                    }
                    type="button"

                    on:click=move |_| set_variant.set(CalculatorVariant::Ac)
                >
                    "To-Hit"
                </button>
                <button
                    class="button-secondary"
                    class:button-secondary-active=move || {
                        matches!(variant.get(), CalculatorVariant::Dc)
                    }
                    on:click=move |_| set_variant.set(CalculatorVariant::Dc)
                    type="button"
                >
                    "Saving Throw"
                </button>
            </div>
            <section class=style::stats_card>
                <h2 class=style::stats_card_title>
                    {move || {
                        if matches!(variant.get(), CalculatorVariant::Ac) {
                            "To-hit Calculator"
                        } else {
                            "Saving throw calculator"
                        }
                    }}
                </h2>
                <p class=style::stats_card_subtitle>
                    {move || {
                        if matches!(variant.get(), CalculatorVariant::Ac) {
                            "Hit chance is the percent of rolls where to-hit is greater than or equal to target AC."
                        } else {
                            "Spell lands when save roll is below the save DC."
                        }
                    }}
                </p>

                <div class=style::stats_fields>
                    <label class=style::stats_editor_label for="to-hit-target-input">
                        {move || {
                            if matches!(variant.get(), CalculatorVariant::Ac) {
                                "Target AC"
                            } else {
                                "Target DC"
                            }
                        }}
                    </label>
                    <input
                        id="to-hit-target-input"
                        class="g-input"
                        type="number"
                        prop:value=move || target.get().to_string()
                        on:input=move |ev| {
                            set_target.set(event_target_value(&ev).parse::<i64>().unwrap_or(10));
                        }
                    />
                </div>

                <div class=style::stats_editor_grid>
                    <article class=style::stats_editor_block>
                        <div class=style::stats_editor_block_inner>
                            <h3 class=style::stats_editor_block_title>
                                {move || {
                                    if matches!(variant.get(), CalculatorVariant::Ac) {
                                        "To-hit Roll"
                                    } else {
                                        "Saving Throw Roll"
                                    }
                                }}
                            </h3>
                            <EditorComponent state=to_hit_editor />
                        </div>
                    </article>

                    <article class=style::stats_editor_block>
                        <div class=style::stats_editor_block_inner>

                            <h3 class=style::stats_editor_block_title>"Damage Roll"</h3>
                            <EditorComponent state=dmg_editor />
                        </div>
                    </article>
                </div>

                <button
                    class="button-primary"
                    type="button"
                    on:click=run_simulation
                    prop:disabled=move || running.get()
                >
                    {move || if running.get() { "Running..." } else { "Run Simulation" }}
                </button>

            </section>

        </div>
    }
}
