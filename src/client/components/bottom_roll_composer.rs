use leptos::prelude::*;

use crate::client::components::{
    dialog::{Dialog, DialogPresentation},
    preset_editor::PresetEditor,
    roll_editor::{RollEditorController, RollEditorPanel},
};

stylance::import_style!(style, "bottom_roll_composer.module.scss");

#[component]
pub fn BottomRollComposer(
    controller: RollEditorController,
    expression_input_id: String,
    #[prop(into)] on_roll: Callback<String>,
    #[prop(into)] error: Signal<Option<String>>,
    dialog_title: String,
    dialog_summary: String,
) -> impl IntoView {
    let composer_open = RwSignal::new(false);
    let current_expression = controller.current_expression_signal();
    let dialog_expression_input_id = expression_input_id.clone();

    view! {
        <div class=style::mobile_composer_shell>
            <Show when=move || error.get().is_some()>
                <p class=style::mobile_composer_feedback>
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>

            <div class=style::mobile_composer_bar>
                <button
                    class=style::mobile_expression_trigger
                    type="button"
                    on:click=move |_| composer_open.set(true)
                >
                    <span class="g-field-label">"Current expression"</span>
                    <code class=style::mobile_expression_code>{move || current_expression.get()}</code>
                    <span class=style::mobile_expression_cta>"click to edit"</span>
                </button>

                <button
                    class="g-button-action"
                    type="button"
                    on:click=move |_| {
                        controller.submit_roll(on_roll);
                    }
                >
                    "Roll"
                </button>
            </div>
        </div>

        <Dialog
            open=move || composer_open.get()
            title=dialog_title
            label="Editor".to_string()
            summary=dialog_summary
            presentation=DialogPresentation::Fullscreen
            show_close_button=false
            on_close=Callback::new(move |_| composer_open.set(false))
        >
            <div class=style::mobile_dialog_body>
                <RollEditorPanel
                    controller=controller
                    expression_input_id=dialog_expression_input_id.clone()
                    show_heading=false
                >
                    <button
                        class=format!("g-button-action {}", style::mobile_confirm_button)
                        type="button"
                        on:click=move |_| composer_open.set(false)
                    >
                        "Confirm"
                    </button>
                </RollEditorPanel>
                <PresetEditor expression=current_expression on_select=controller.preset_select_callback() />
            </div>
        </Dialog>
    }
}
