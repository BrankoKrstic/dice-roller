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

    pub fn load_expression(&mut self, expr: &str) {
        self.mode = EditorMode::Expression;
        self.expr.set(expr.to_string());
    }

    fn show_builder(&mut self) {
        self.mode = EditorMode::Builder;
    }

    fn show_expression(&mut self) {
        self.mode = EditorMode::Expression;
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
        <article
            class=style::die_card
            data-active=move || if count.get() != 0 { "true" } else { "false" }
        >
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

pub fn current_expression_signal(state: RwSignal<EditorState>) -> Signal<String> {
    Signal::derive(move || state.get().get_expr())
}

#[component]
pub fn EditorExpressionPreview(#[prop(into)] expression: Signal<String>) -> impl IntoView {
    view! {
        <div class=style::roll_editor_preview>
            <span class="g-field-label">"Current expression"</span>
            <code class=style::roll_editor_preview_code>{move || expression.get()}</code>
        </div>
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
fn ExpressionEditor(expr: RwSignal<String>, input_id: String) -> impl IntoView {
    let label_target = input_id.clone();

    view! {
        <div class=style::roll_editor_panel>
            <label class="g-field-label" for=label_target>
                "Expression"
            </label>
            <input
                id=input_id
                type="text"
                class=format!("g-text-input {}", style::expression_editor_input)
                prop:value=move || expr.get()
                on:input=move |event| expr.set(event_target_value(&event))
            />
        </div>
    }
}

#[component]
pub fn EditorComponent(state: RwSignal<EditorState>, expression_input_id: String) -> impl IntoView {
    view! {
        <div class="g-roll-editor-mode-switch">
            <button
                on:click=move |_| {
                    state.update(|editor| editor.show_builder());
                }
                class="g-button-mode"
                class=("g-button-mode-active", move || state.get().mode == EditorMode::Builder)
            >
                "Dice Bench"
            </button>
            <button
                class="g-button-mode"
                class=("g-button-mode-active", move || state.get().mode == EditorMode::Expression)
                on:click=move |_| {
                    state.update(|editor| editor.show_expression());
                }
            >
                "Expression"
            </button>

        </div>
        <div>
            {move || {
                if matches!(state.get().mode, EditorMode::Builder) {
                    view! { <BuilderEditor builder=state.get().builder /> }.into_any()
                } else {
                    view! {
                        <ExpressionEditor
                            expr=state.get().expr
                            input_id=expression_input_id.clone()
                        />
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn RollEditorPanel(
    state: RwSignal<EditorState>,
    expression_input_id: String,
    #[prop(optional)] show_heading: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let current_expression = current_expression_signal(state);

    view! {
        <section class=style::roll_editor>
            <Show when=move || show_heading>
                <div class=style::roll_editor_heading>
                    <p class="g-section-label">"Editor"</p>
                    <h1 class=style::roll_editor_title>"Build a roll"</h1>
                    <p class=style::roll_editor_summary>
                        "Use the bench to quick-draft a roll, or unlock advanced modifiers in the expression editor."
                    </p>
                </div>
            </Show>

            <EditorComponent state=state expression_input_id=expression_input_id />
            <div class=style::roll_editor_footer>
                <EditorExpressionPreview expression=current_expression />
                <div class=style::roll_editor_actions>{children()}</div>
            </div>
        </section>
    }
}

#[component]
pub fn RollEditor(
    #[prop(into)] on_roll: Callback<String>,
    #[prop(optional)] state: Option<RwSignal<EditorState>>,
    #[prop(optional, into)] expression_input_id: MaybeProp<String>,
) -> impl IntoView {
    let state = state.unwrap_or(RwSignal::new(EditorState::default()));
    let current_expression = current_expression_signal(state);
    let expression_input_id = expression_input_id
        .get()
        .unwrap_or_else(|| "expression-editor-input".to_string());

    view! {
        <RollEditorPanel state=state expression_input_id=expression_input_id show_heading=true>
            <a class="g-button-ghost" href="/reference">
                "Open reference"
            </a>
            <button
                class="g-button-action"
                type="button"
                on:click=move |_| {
                    on_roll.run(state.get().get_expr());
                }
            >
                "Roll to ledger"
            </button>
        </RollEditorPanel>
        <PresetEditor
            expression=current_expression
            on_select=Callback::new(move |expr: String| {
                state.update(|editor| editor.load_expression(&expr));
            })
        />
    }
}
