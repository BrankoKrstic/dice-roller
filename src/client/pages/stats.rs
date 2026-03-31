use leptos::prelude::*;

use crate::{
    ChanceResult, WorkerSimulationRequest, WorkerSimulationResponse,
    client::components::roll_editor::{EditorComponent, EditorState},
};

#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, JsValue};

stylance::import_style!(style, "stats.module.scss");

use web_sys::{MessageEvent, Worker};

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
            {move || {
                if let Some(result) = result.get() {
                    view! {
                        <article class=style::stats_card>
                            <h3 class=style::stats_card_label>"Simulation result"</h3>
                            <p class=style::stats_card_total>
                                {format!("{:.2}%", success_percent() * 100.0)}
                            </p>
                            <pre class=style::stats_card_breakdown>
                                {format!(
                                    "Average damage per attempt: {:.3}\nAverage damage on success: {:.3}",
                                    if result.success_count == 0 {
                                        0.0
                                    } else {
                                        result.dmg as f32 / result.success_count as f32
                                    },
                                    result.dmg as f32 / result.trials as f32,
                                )}
                            </pre>
                        </article>
                    }
                        .into_any()
                } else if let Some(message) = error.get() {
                    view! {
                        <article class=format!("{} {}", style::stats_card, style::stats_card_error)>
                            <h3 class=style::stats_card_label>"Simulation error"</h3>
                            <p class=style::stats_card_error_inner>{message}</p>
                        </article>
                    }
                        .into_any()
                } else {
                    view! {
                        <Show
                            when=move || running.get()
                            fallback=move || {
                                view! {
                                    <article class=style::stats_card>
                                        <h3 class=style::stats_card_label>"Ready"</h3>
                                        <p class=style::stats_card_hint>
                                            "Set the target, draft the two rolls, then run the ledger."
                                        </p>
                                    </article>
                                }
                            }
                        >
                            <div
                                class=style::stats_loader
                                role="status"
                                aria-label="Simulation in progress"
                            >
                                <div class=style::stats_loader_spinner></div>
                                <p class=style::stats_loader_text>
                                    "Running one million trials..."
                                </p>
                            </div>
                        </Show>
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
        <div class=format!("g-page g-page-shell g-page-shell-split {}", style::stats_shell)>
            <section class=style::stats_column>
                <section class="g-panel g-panel-strong">
                    <p class="g-section-label">"Analysis mode"</p>
                    <h1 class="g-section-title">"Probability Ledger"</h1>
                    <p class="g-section-summary">
                        "Draft the two commands, set the target, and read the result as a clean numeric report."
                    </p>
                </section>

                <section class=format!("g-panel g-panel-strong {}", style::stats_workbench)>
                    <div class=style::stats_toolbar>
                        <div
                            class=format!("g-roll-editor-mode-switch {}", style::stats_mode_switch)
                            role="tablist"
                            aria-label="Calculator type"
                        >
                            <button
                                class="g-button-mode"
                                class:g-button-mode-active=move || {
                                    matches!(variant.get(), CalculatorVariant::Ac)
                                }
                                type="button"
                                on:click=move |_| set_variant.set(CalculatorVariant::Ac)
                            >
                                "To-hit"
                            </button>
                            <button
                                class="g-button-mode"
                                class:g-button-mode-active=move || {
                                    matches!(variant.get(), CalculatorVariant::Dc)
                                }
                                on:click=move |_| set_variant.set(CalculatorVariant::Dc)
                                type="button"
                            >
                                "Saving throw"
                            </button>
                        </div>

                        <div class=style::stats_target>
                            <label class="g-field-label" for="to-hit-target-input">
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
                                class=format!("g-text-input {}", style::stats_target_input)
                                type="number"
                                prop:value=move || target.get().to_string()
                                on:input=move |ev| {
                                    set_target
                                        .set(event_target_value(&ev).parse::<i64>().unwrap_or(10));
                                }
                            />
                        </div>
                    </div>

                    <div>
                        <h2 class=style::stats_card_title>
                            {move || {
                                if matches!(variant.get(), CalculatorVariant::Ac) {
                                    "Measure hit chance against armor."
                                } else {
                                    "Measure failure rate against a saving throw."
                                }
                            }}
                        </h2>
                        <p class=style::stats_card_subtitle>
                            {move || {
                                if matches!(variant.get(), CalculatorVariant::Ac) {
                                    "Draft the attack roll and paired damage roll, then estimate how often the total lands against the target AC."
                                } else {
                                    "Draft the save expression and paired damage roll, then estimate how often the save misses the target DC."
                                }
                            }}
                        </p>
                    </div>

                    <div class=style::stats_editor_grid>
                        <article class=style::stats_editor_block>
                            <p class="g-section-label">"Step 1"</p>
                            <h3 class=style::stats_editor_block_title>
                                {move || {
                                    if matches!(variant.get(), CalculatorVariant::Ac) {
                                        "Attack roll"
                                    } else {
                                        "Saving throw roll"
                                    }
                                }}
                            </h3>
                            <div class=style::stats_editor_body>
                                <EditorComponent state=to_hit_editor />
                            </div>
                        </article>

                        <article class=style::stats_editor_block>
                            <p class="g-section-label">"Step 2"</p>
                            <h3 class=style::stats_editor_block_title>"Damage roll"</h3>
                            <div class=style::stats_editor_body>
                                <EditorComponent state=dmg_editor />
                            </div>
                        </article>
                    </div>

                    <button
                        class="g-button-action"
                        type="button"
                        on:click=run_simulation
                        prop:disabled=move || running.get()
                    >
                        {move || if running.get() { "Running..." } else { "Run Simulation" }}
                    </button>
                </section>
            </section>

            <aside class=style::stats_rail>
                <StatsResultPanel running=running result=result error=error />
            </aside>
        </div>
    }
}
