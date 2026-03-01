use leptos::prelude::*;

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
        label: "d4",
        token: "d4",
    },
    DiceDef {
        label: "d6",
        token: "d6",
    },
    DiceDef {
        label: "d8",
        token: "d8",
    },
    DiceDef {
        label: "d10",
        token: "d10",
    },
    DiceDef {
        label: "d12",
        token: "d12",
    },
    DiceDef {
        label: "d20",
        token: "d20",
    },
    DiceDef {
        label: "d%",
        token: "d%",
    },
    DiceDef {
        label: "dF",
        token: "dF",
    },
    DiceDef {
        label: "Mod",
        token: "",
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
        if self.dice[i] < i16::MAX {
            self.dice[i] += 1;
        }
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
            x => {
                let mut new_dice_counts = DiceCounts::new();
                new_dice_counts.add_dice(i);
                self.roll = RollType::Counts(new_dice_counts);
            }
        }
    }
    fn sub_dice(&mut self, i: usize) {
        match &mut self.roll {
            RollType::Counts(dice_counts) => dice_counts.subtract_dice(i),
            x => {
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
        </div>
    }
}

#[component]
fn ExpressionEditor() -> impl IntoView {
    view! { "Expr" }
}

#[component]
pub fn RollEditor() -> impl IntoView {
    let mut editor_mode = RwSignal::new(EditorMode::default());
    let mut editor_expr = RwSignal::new(String::from("2d10 + 1d6 + 5"));
    let mut roll_builder = RwSignal::new(RollBuilder::new());

    view! {
        <section class=style::roll_editor>
            <div class=style::roll_editor_mode_switch>
                <button
                    on:click=move |_| { editor_mode.set(EditorMode::Builder) }
                    class=style::roll_editor_mode_pill
                    class=(
                        style::roll_editor_mode_pill_active,
                        move || editor_mode.get() == EditorMode::Builder,
                    )
                >
                    "Dice Editor"
                </button>
                <button
                    class=style::roll_editor_mode_pill
                    class=(
                        style::roll_editor_mode_pill_active,
                        move || editor_mode.get() == EditorMode::Expression,
                    )
                    on:click=move |_| { editor_mode.set(EditorMode::Expression) }
                >
                    "Expression Editor"
                </button>

            </div>
            <div>
                {move || {
                    if matches!(editor_mode.get(), EditorMode::Builder) {
                        view! { <BuilderEditor builder=roll_builder /> }.into_any()
                    } else {
                        view! { <ExpressionEditor /> }.into_any()
                    }
                }}
            </div>

            {move || {
                if matches!(editor_mode.get(), EditorMode::Builder) {
                    view! {
                        <div class=style::expression_editor_preview>
                            <span class=style::expression_editor_preview_label>"Expression"</span>
                            <code class=style::expression_editor_preview_code>
                                {move || roll_builder.get().to_expr()}
                            </code>
                        </div>
                    }
                        .into_any()
                } else {
                    { "" }.into_any()
                }
            }}
        </section>
    }
}
