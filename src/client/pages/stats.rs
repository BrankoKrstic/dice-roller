use leptos::prelude::*;

use crate::{
    client::components::roll_editor::{EditorComponent, EditorState},
    ChanceResult, WorkerSimulationRequest, WorkerSimulationResponse,
};

#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, JsValue};

stylance::import_style!(style, "stats.module.scss");

use web_sys::{MessageEvent, Worker};

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

#[cfg(feature = "hydrate")]
fn spawn_chance_worker(
    set_result: WriteSignal<Option<ChanceResult>>,
    set_error: WriteSignal<Option<String>>,
    set_running: WriteSignal<bool>,
) -> Worker {
    let options = web_sys::WorkerOptions::new();
    options.set_type(web_sys::WorkerType::Module);

    let worker = web_sys::Worker::new_with_options("/workers/stats-worker.js", &options)
        .expect("Worker should be there");

    let on_message = wasm_bindgen::closure::Closure::<dyn FnMut(MessageEvent)>::new(
        move |message: MessageEvent| {
            let response = serde_json::from_str::<WorkerSimulationResponse>(
                &message.data().as_string().unwrap(),
            );

            let response = match response {
                Ok(result) => result,
                Err(e) => {
                    set_error.set(Some(format!("Failed to parse result {}", e)));
                    set_running.set(false);
                    return;
                }
            };

            match response {
                WorkerSimulationResponse::Result(chance_result) => {
                    set_result.set(Some(chance_result))
                }
                WorkerSimulationResponse::Error(err) => set_error.set(Some(err)),
            }
            set_running.set(false);
        },
    );

    worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    worker
}

#[component]
fn StatsResultPanel(
    running: ReadSignal<bool>,
    result: ReadSignal<Option<ChanceResult>>,
    error: ReadSignal<Option<String>>,
) -> impl IntoView {
    let success_percent = move || {
        if let Some(res) = result.get() {
            res.success_count as f32 / res.trials as f32
        } else {
            0.0
        }
    };
    view! {
        <section class=style::stats_result aria-live="polite">
            <Show when=move || running.get()>
                <div class=style::stats_loader role="status" aria-label="Simulation in progress">
                    <div class=style::stats_loader_spinner></div>
                    <p class=style::stats_loader_text>"Crunching..."</p>
                </div>
            </Show>

            {move || {
                if let Some(result) = result.get() {
                    view! {
                        <article class=style::stats_card>
                            <h3>"Simulation Result"</h3>
                            <p>{format!("{:.2}%", success_percent() * 100.0)}</p>
                            <pre>
                                {format!(
                                    "Average damage per attempt: {:.3}\nAverage damage on success: {:.3}",
                                    result.dmg as f32 / result.success_count as f32,
                                    result.dmg as f32 / result.trials as f32,
                                )}
                            </pre>
                        </article>
                    }
                        .into_any()
                } else if let Some(message) = error.get() {
                    view! {
                        <article class=style::stats_card>
                            <h3>"Simulation Error"</h3>
                            <p>{message}</p>
                        </article>
                    }
                        .into_any()
                } else {
                    view! {
                        <article class="result-card result-card--empty">
                            <h3 class="result-card__label">"Ready"</h3>
                            <p class="result-card__hint">"Configure rolls and run a simulation."</p>
                        </article>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}

#[component]
pub fn StatsPage() -> impl IntoView {
    let (variant, set_variant) = signal(CalculatorVariant::Ac);
    let (target, set_target) = signal(15);
    let to_hit_editor = RwSignal::new(EditorState::default());
    let dmg_editor = RwSignal::new(EditorState::default());
    let (running, set_running) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (result, set_result) = signal::<Option<ChanceResult>>(None);

    #[cfg(feature = "hydrate")]
    let worker = spawn_chance_worker(set_result, set_error, set_running);

    #[cfg(feature = "hydrate")]
    let run_simulation = move |_| {
        set_running.set(true);
        set_result.set(None);
        set_error.set(None);
        worker
            .post_message(&JsValue::from_str(
                &serde_json::to_string(&WorkerSimulationRequest {
                    to_hit_expression: to_hit_editor.get().get_expr(),
                    damage_expression: dmg_editor.get().get_expr(),
                    target: target.get(),
                    trials: 1_000_000,
                    ac_mode: matches!(variant.get(), CalculatorVariant::Ac),
                })
                .unwrap(),
            ))
            .expect("Can post message to worker");
    };

    #[cfg(not(feature = "hydrate"))]
    let run_simulation = |_| {};

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

                <StatsResultPanel running=running result=result error=error />
            </section>

        </div>
    }
}
