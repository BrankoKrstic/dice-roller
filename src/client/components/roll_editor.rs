use leptos::prelude::*;

use crate::client::components::preset_editor::PresetEditor;

stylance::import_style!(style, "roll_editor.module.scss");

#[derive(Debug, Default, Clone, PartialEq, Eq)]
enum EditorMode {
    #[default]
    Builder,
    Expression,
}

struct DiceDef {
    label: &'static str,
    token: &'static str,
}

const DICE_DEFS: [DiceDef; 9] = [
    DiceDef {
        label: "d20",
        token: "d20",
    },
    DiceDef {
        label: "d12",
        token: "d12",
    },
    DiceDef {
        label: "d10",
        token: "d10",
    },
    DiceDef {
        label: "d8",
        token: "d8",
    },
    DiceDef {
        label: "d6",
        token: "d6",
    },
    DiceDef {
        label: "d4",
        token: "d4",
    },
    DiceDef {
        label: "Mod",
        token: "",
    },
    DiceDef {
        label: "d%",
        token: "d%",
    },
    DiceDef {
        label: "dF",
        token: "dF",
    },
];

#[derive(Debug, Clone)]
struct DiceCounts {
    dice: [i16; DICE_DEFS.len()],
}

impl DiceCounts {
    pub fn new() -> Self {
        Self {
            dice: [0; DICE_DEFS.len()],
        }
    }
    pub fn add_dice(&mut self, i: usize) {
        self.dice[i] = self.dice[i].saturating_add(1);
    }
    pub fn subtract_dice(&mut self, i: usize) {
        if self.dice[i] > i16::MIN + 1 {
            self.dice[i] -= 1;
        }
    }
}

#[derive(Debug, Clone)]
enum RollType {
    Adv,
    Dis,
    Counts(DiceCounts),
}

#[derive(Debug, Clone)]
struct RollBuilder {
    roll: RollType,
}

impl RollBuilder {
    fn new() -> Self {
        Self {
            roll: RollType::Counts(DiceCounts::new()),
        }
    }
    fn add_dice(&mut self, i: usize) {
        match &mut self.roll {
            RollType::Counts(dice_counts) => dice_counts.add_dice(i),
            _ => {
                let mut new_dice_counts = DiceCounts::new();
                new_dice_counts.add_dice(i);
                self.roll = RollType::Counts(new_dice_counts);
            }
        }
    }
    fn sub_dice(&mut self, i: usize) {
        match &mut self.roll {
            RollType::Counts(dice_counts) => dice_counts.subtract_dice(i),
            _ => {
                let mut new_dice_counts = DiceCounts::new();
                new_dice_counts.subtract_dice(i);
                self.roll = RollType::Counts(new_dice_counts);
            }
        }
    }
    fn adv_roll(&mut self) {
        self.roll = RollType::Adv
    }
    fn dis_roll(&mut self) {
        self.roll = RollType::Dis
    }
    fn clear(&mut self) {
        self.roll = RollType::Counts(DiceCounts::new());
    }
    fn get_die_count(&self, i: usize) -> i16 {
        match &self.roll {
            RollType::Adv => 0,
            RollType::Dis => 0,
            RollType::Counts(dice_counts) => dice_counts.dice[i],
        }
    }
    fn to_expr(&self) -> String {
        let mut out = String::new();

        match &self.roll {
            RollType::Adv => out.push_str("d20adv"),
            RollType::Dis => out.push_str("d20dis"),
            RollType::Counts(dice_counts) => {
                for (i, &count) in dice_counts.dice.iter().enumerate() {
                    if count == 0 {
                        continue;
                    }
                    if count < 0 {
                        if out.is_empty() {
                            out.push('-');
                        } else {
                            out.push_str(" - ");
                        }
                    } else if !out.is_empty() {
                        out.push_str(" + ");
                    }
                    out.push_str(count.abs().to_string().as_str());
                    out.push_str(DICE_DEFS[i].token);
                }
            }
        }
        if out.is_empty() {
            out.push('0');
        }
        out
    }
}

#[component]
fn DieCard(
    label: &'static str,
    #[prop(into)] count: Signal<i16>,
    #[prop(into)] on_add_click: Callback<()>,
    #[prop(into)] on_sub_click: Callback<()>,
) -> impl IntoView {
    view! {
        <article class=style::die_card>
            <button
                class=style::die_card_face
                type="button"
                on:click=move |_| on_add_click.run(())
                on:contextmenu=move |ev| {
                    ev.prevent_default();
                    on_sub_click.run(());
                }
                title="Left click to add, right click to remove"
            >
                {label}
            </button>
            <div class=style::die_card_controls>
                <button
                    class=style::die_card_step
                    type="button"
                    on:click=move |_| on_sub_click.run(())
                >
                    "-"
                </button>
                <output class=style::die_card_count>{count}</output>
                <button
                    class=style::die_card_step
                    type="button"
                    on:click=move |_| on_add_click.run(())
                >
                    "+"
                </button>
            </div>
        </article>
    }
}

#[component]
fn BuilderEditor(builder: RwSignal<RollBuilder>) -> impl IntoView {
    view! {
        <div class=style::roll_editor_panel>
            <div class=style::roll_editor_grid>
                {(0..DICE_DEFS.len())
                    .map(|i| {
                        view! {
                            <DieCard
                                label=DICE_DEFS[i].label
                                count=move || builder.get().get_die_count(i)
                                on_add_click=move |_| builder.write().add_dice(i)
                                on_sub_click=move |_| builder.write().sub_dice(i)
                            />
                        }
                    })
                    .collect_view()}
            </div>
            <div class=style::quick_actions>
                <button
                    class="g-button-mode"
                    class:g-button-mode-active=move || {
                        matches!(builder.get().roll, RollType::Adv)
                    }

                    on:click=move |_| builder.write().adv_roll()
                >
                    d20adv
                </button>
                <button
                    class="g-button-mode"
                    class:g-button-mode-active=move || {
                        matches!(builder.get().roll, RollType::Dis)
                    }

                    on:click=move |_| builder.write().dis_roll()
                >
                    d20dis
                </button>
                <button class="g-button-utility" on:click=move |_| builder.write().clear()>
                    Clear
                </button>

            </div>
        </div>
    }
}

#[component]
fn ExpressionEditor(expr: RwSignal<String>) -> impl IntoView {
    view! {
        <div class=style::roll_editor_panel>
            <label class="g-field-label" for="expression-editor-input">
                "Expression"
            </label>
            <input
                id="expression-editor-input"
                type="text"
                class=format!("g-text-input {}", style::expression_editor_input)
                prop:value=move || expr.get()
                on:input=move |event| expr.set(event_target_value(&event))
            />
            <p class="g-result-hint">
                "Paste exact notation here when the table already knows the command."
            </p>
        </div>
    }
}

#[derive(Debug, Clone)]
pub struct EditorState {
    mode: EditorMode,
    expr: RwSignal<String>,
    builder: RwSignal<RollBuilder>,
}
impl Default for EditorState {
    fn default() -> Self {
        Self {
            mode: EditorMode::default(),
            expr: RwSignal::new(String::from("2d10 + 1d6 + 5")),
            builder: RwSignal::new(RollBuilder::new()),
        }
    }
}
impl EditorState {
    pub fn get_expr(&self) -> String {
        match &self.mode {
            EditorMode::Builder => self.builder.get().to_expr(),
            EditorMode::Expression => self.expr.get().to_string(),
        }
    }

    fn load_expression(&mut self, expr: &str) {
        self.mode = EditorMode::Expression;
        self.expr.set(expr.to_string());
    }
}

#[component]
pub fn EditorComponent(state: RwSignal<EditorState>) -> impl IntoView {
    view! {
        <div class="g-roll-editor-mode-switch">
            <button
                on:click=move |_| { state.write().mode = EditorMode::Builder }
                class="g-button-mode"
                class=("g-button-mode-active", move || state.get().mode == EditorMode::Builder)
            >
                "Dice Bench"
            </button>
            <button
                class="g-button-mode"
                class=("g-button-mode-active", move || state.get().mode == EditorMode::Expression)
                on:click=move |_| { state.write().mode = EditorMode::Expression }
            >
                "Expression Editor"
            </button>

        </div>
        <div>
            {move || {
                if matches!(state.get().mode, EditorMode::Builder) {
                    view! { <BuilderEditor builder=state.get().builder /> }.into_any()
                } else {
                    view! { <ExpressionEditor expr=state.get().expr /> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn RollEditor(#[prop(into)] on_roll: Callback<String>) -> impl IntoView {
    let state = RwSignal::new(EditorState::default());
    let current_expression = Signal::derive(move || state.get().get_expr());

    view! {
        <section class=style::roll_editor>
            <div class=style::roll_editor_heading>
                <p class="g-section-label">"Composer"</p>
                <h1 class=style::roll_editor_title>"Compose the next throw."</h1>
                <p class=style::roll_editor_summary>
                    "Use the bench for tactile drafting or jump to raw notation when the table already knows the move."
                </p>
            </div>

            <PresetEditor
                expression=current_expression
                on_select=Callback::new(move |expr: String| {
                    state.update(|editor| editor.load_expression(&expr));
                })
            />
            <EditorComponent state=state />
            <div class=style::roll_editor_footer>
                <div class=style::roll_editor_preview>
                    <span class="g-field-label">"Current expression"</span>
                    <code class=style::roll_editor_preview_code>
                        {move || state.get().get_expr()}
                    </code>
                    <p class=style::roll_editor_preview_note>
                        "Need a reminder on modifiers, rerolls, or keep/drop syntax in the expression editor?"
                    </p>
                </div>

                <div class=style::roll_editor_actions>
                    <a class="g-button-ghost" href="/reference">
                        "Open reference"
                    </a>
                    <button
                        class="g-button-action"
                        type="button"
                        on:click=move |_| { on_roll.run(state.get().get_expr()) }
                    >
                        "Roll to ledger"
                    </button>
                </div>
            </div>
        </section>
    }
}
