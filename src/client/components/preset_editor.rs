use leptos::{prelude::*, task::spawn_local};

use crate::{
    client::{
        components::dialog::Dialog,
        context::auth::use_auth_context,
        utils::{
            api::parse_error_response,
            async_state::{LoadState, MutationState},
            url::base_url,
        },
    },
    shared::data::{
        preset::{Preset, PresetRequest},
        user::AuthContext,
    },
};

stylance::import_style!(style, "preset_editor.module.scss");

const MAX_PRESETS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingDialog {
    Save,
    Delete(i64),
}

fn current_user_id(auth: &AuthContext) -> Option<i64> {
    auth.user.get_untracked().map(|user| user.id.into_inner())
}

#[derive(Clone)]
struct PresetEditorState {
    presets_state: RwSignal<LoadState<Vec<Preset>, String>>,
    dialog: RwSignal<Option<PendingDialog>>,
    pending_name: RwSignal<String>,
    save_state: RwSignal<MutationState<String>>,
    archiving_id: RwSignal<Option<i64>>,
    archive_state: RwSignal<MutationState<String>>,
    last_loaded_user_id: RwSignal<Option<i64>>,
}

impl PresetEditorState {
    fn new() -> Self {
        Self {
            presets_state: RwSignal::new(LoadState::idle()),
            dialog: RwSignal::new(None),
            pending_name: RwSignal::new(String::new()),
            save_state: RwSignal::new(MutationState::idle()),
            archiving_id: RwSignal::new(None),
            archive_state: RwSignal::new(MutationState::idle()),
            last_loaded_user_id: RwSignal::new(None),
        }
    }

    fn reset_for_signed_out(&self) {
        self.presets_state.set(LoadState::idle());
        self.dialog.set(None);
        self.pending_name.set(String::new());
        self.save_state.set(MutationState::idle());
        self.archiving_id.set(None);
        self.archive_state.set(MutationState::idle());
        self.last_loaded_user_id.set(None);
    }

    fn dismiss_dialog(&self) {
        self.dialog.set(None);
        self.pending_name.set(String::new());
        self.save_state.set(MutationState::idle());
        self.archive_state.set(MutationState::idle());
    }

    fn open_save_dialog(&self) {
        self.save_state.set(MutationState::idle());
        self.pending_name.set(String::new());
        self.dialog.set(Some(PendingDialog::Save));
    }

    fn preset_count(&self) -> usize {
        match self.presets_state.get_untracked() {
            LoadState::Ready(items) => items.len(),
            _ => 0,
        }
    }

    fn preset_count_signal(&self) -> Signal<usize> {
        let state = self.clone();
        Signal::derive(move || state.preset_count())
    }

    fn save_dialog_open_signal(&self) -> Signal<bool> {
        let state = self.clone();
        Signal::derive(move || state.dialog.get() == Some(PendingDialog::Save))
    }

    fn delete_dialog_open_signal(&self) -> Signal<bool> {
        let state = self.clone();
        Signal::derive(move || matches!(state.dialog.get(), Some(PendingDialog::Delete(_))))
    }

    fn selected_delete_preset_signal(&self) -> Signal<Option<Preset>> {
        let state = self.clone();
        Signal::derive(move || match state.dialog.get() {
            Some(PendingDialog::Delete(preset_id)) => match state.presets_state.get() {
                LoadState::Ready(items) => {
                    items.into_iter().find(|preset| preset.id.0 == preset_id)
                }
                _ => None,
            },
            _ => None,
        })
    }

    fn replace_presets(&self, items: Vec<Preset>) {
        self.presets_state.set(LoadState::ready(items));
    }

    fn clear_presets_with_error(&self, message: String) {
        self.presets_state.set(LoadState::error(message));
    }

    fn push_preset(&self, preset: Preset) {
        self.presets_state.update(|state| {
            if let LoadState::Ready(items) = state {
                items.push(preset);
            }
        });
    }

    fn remove_preset(&self, preset_id: i64) {
        self.presets_state.update(|state| {
            if let LoadState::Ready(items) = state {
                items.retain(|preset| preset.id.0 != preset_id);
            }
        });
    }
}

fn save_cta_copy(save_state: &MutationState<String>, preset_count: usize) -> &'static str {
    match save_state {
        MutationState::Pending => "Saving...",
        MutationState::Idle | MutationState::Success | MutationState::Error(_)
            if preset_count >= MAX_PRESETS =>
        {
            "Preset limit reached"
        }
        MutationState::Idle | MutationState::Success | MutationState::Error(_) => "Save as Preset",
    }
}

fn save_cta_disabled(save_state: &MutationState<String>, preset_count: usize) -> bool {
    match save_state {
        MutationState::Pending => true,
        MutationState::Idle | MutationState::Success | MutationState::Error(_) => {
            preset_count >= MAX_PRESETS
        }
    }
}

fn archive_cta_copy(archive_state: &MutationState<String>) -> &'static str {
    match archive_state {
        MutationState::Pending => "Deleting...",
        MutationState::Idle | MutationState::Success | MutationState::Error(_) => "Delete preset",
    }
}

fn archive_cta_disabled(archive_state: &MutationState<String>) -> bool {
    matches!(archive_state, MutationState::Pending)
}

fn save_dialog_submit_disabled(
    save_state: &MutationState<String>,
    pending_name: &str,
    preset_count: usize,
) -> bool {
    match save_state {
        MutationState::Pending => true,
        MutationState::Idle | MutationState::Success | MutationState::Error(_) => {
            pending_name.trim().is_empty() || preset_count >= MAX_PRESETS
        }
    }
}

fn delete_dialog_copy(selected_delete_preset: Option<Preset>) -> String {
    selected_delete_preset
        .map(|preset| format!("Delete \"{}\" from your presets?", preset.name))
        .unwrap_or_else(|| "Delete this preset?".to_string())
}

#[component]
fn PresetCard(
    preset: Preset,
    state: PresetEditorState,
    #[prop(into)] on_select: Callback<String>,
) -> impl IntoView {
    let apply_expr = preset.expr.clone();
    let delete_id = preset.id.0;

    view! {
        <article class=style::preset_card>
            <p class=style::preset_launch>
                <span class=style::preset_card_title>{preset.name.clone()}</span>
                <code class=style::preset_card_code>{preset.expr.clone()}</code>
            </p>
            <div class=style::preset_editor_button_wrap>
                <button
                    class=format!("g-button-utility {}", style::preset_archive)
                    type="button"
                    on:click=move |_| on_select.run(apply_expr.clone())
                >
                    "Load"
                </button>

                <button
                    class=format!("g-button-utility {}", style::preset_archive)
                    type="button"
                    prop:disabled=move || state.archiving_id.get() == Some(delete_id)
                    on:click=move |_| {
                        state.archive_state.set(MutationState::idle());
                        state.dialog.set(Some(PendingDialog::Delete(delete_id)));
                    }
                >
                    {move || {
                        if state.archiving_id.get() == Some(delete_id) {
                            "Deleting..."
                        } else {
                            "Delete"
                        }
                    }}
                </button>
            </div>
        </article>
    }
}

#[component]
fn SavePresetDialog(
    state: PresetEditorState,
    #[prop(into)] expression: Signal<String>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_submit: Callback<()>,
) -> impl IntoView {
    let preset_count = state.preset_count_signal();
    let input_state = state.clone();
    let disable_state = state.clone();
    let label_state = state.clone();

    view! {
        <Dialog
            open=state.save_dialog_open_signal()
            title="Save preset".to_string()
            label="Preset controls"
            summary="Name this expression so it can be dropped back into the editor in one click."
                .to_string()
            on_close
        >
            <label class="g-field-label" for="preset-name-input">
                "Preset name"
            </label>
            <input
                id="preset-name-input"
                class="g-text-input"
                type="text"
                prop:value=move || input_state.pending_name.get()
                on:input=move |event| {
                    input_state.save_state.set(MutationState::idle());
                    input_state.pending_name.set(event_target_value(&event));
                }
                maxlength="48"
                placeholder="Sneak Attack"
            />
            <div class=style::dialog_expression_preview>
                <span class="g-field-label">"Current expression"</span>
                <code class=style::dialog_expression_code>{move || expression.get()}</code>
            </div>
            {move || {
                match state.save_state.get() {
                    MutationState::Error(message) => {
                        Some(view! { <p class=style::dialog_feedback>{message}</p> })
                    }
                    MutationState::Idle | MutationState::Pending | MutationState::Success => None,
                }
            }}
            <div class=style::dialog_actions>
                <button class="g-button-ghost" type="button" on:click=move |_| on_close.run(())>
                    "Cancel"
                </button>
                <button
                    class="g-button-action"
                    type="button"
                    prop:disabled=move || {
                        save_dialog_submit_disabled(
                            &disable_state.save_state.get(),
                            &disable_state.pending_name.get(),
                            preset_count.get(),
                        )
                    }
                    on:click=move |_| on_submit.run(())
                >
                    {move || save_cta_copy(&label_state.save_state.get(), preset_count.get())}
                </button>
            </div>
        </Dialog>
    }
}

#[component]
fn DeletePresetDialog(
    state: PresetEditorState,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_confirm: Callback<()>,
) -> impl IntoView {
    let selected_delete_preset = state.selected_delete_preset_signal();
    let disable_state = state.clone();
    let label_state = state.clone();

    view! {
        <Dialog
            open=state.delete_dialog_open_signal()
            title="Delete preset".to_string()
            label="Preset controls"
            summary="Preset will be deleted from the quick access options".to_string()
            on_close
        >
            <p class=style::dialog_copy>
                {move || delete_dialog_copy(selected_delete_preset.get())}
            </p>
            {move || {
                match state.archive_state.get() {
                    MutationState::Error(message) => {
                        Some(view! { <p class=style::dialog_feedback>{message}</p> })
                    }
                    MutationState::Idle | MutationState::Pending | MutationState::Success => None,
                }
            }}
            <div class=style::dialog_actions>
                <button class="g-button-ghost" type="button" on:click=move |_| on_close.run(())>
                    "Keep preset"
                </button>
                <button
                    class=format!("g-button-utility {}", style::dialog_danger)
                    type="button"
                    prop:disabled=move || {
                        disable_state.archiving_id.get().is_some()
                            || archive_cta_disabled(&disable_state.archive_state.get())
                    }
                    on:click=move |_| on_confirm.run(())
                >
                    {move || archive_cta_copy(&label_state.archive_state.get())}
                </button>
            </div>
        </Dialog>
    }
}

async fn list_presets_request() -> Result<Vec<Preset>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/presets", base_url()))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to load presets").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

async fn save_preset_request(payload: PresetRequest) -> Result<Preset, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/presets", base_url()))
        .json(&payload)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to save preset").await);
    }

    response.json().await.map_err(|error| error.to_string())
}

async fn archive_preset_request(preset_id: i64) -> Result<(), String> {
    let client = reqwest::Client::new();
    let response = client
        .delete(format!("{}/api/presets/{}", base_url(), preset_id))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(parse_error_response(response, "Failed to delete preset").await);
    }

    Ok(())
}

#[component]
pub fn PresetEditor(
    #[prop(into)] expression: Signal<String>,
    #[prop(into)] on_select: Callback<String>,
) -> impl IntoView {
    let auth = use_auth_context();
    let state = PresetEditorState::new();

    let auth_for_effect = auth.clone();
    {
        let state = state.clone();
        Effect::new(move |_| {
            if !cfg!(feature = "hydrate") {
                return;
            }

            let auth_loading = auth_for_effect.loading.get();
            let auth_user = auth_for_effect.user.get();

            if auth_loading {
                return;
            }

            let Some(user) = auth_user else {
                state.reset_for_signed_out();
                return;
            };

            let user_id = user.id.into_inner();
            if state.last_loaded_user_id.get_untracked() == Some(user_id) {
                return;
            }

            state.last_loaded_user_id.set(Some(user_id));
            state.presets_state.set(LoadState::loading());

            let auth = auth_for_effect.clone();
            let request_state = state.clone();
            spawn_local(async move {
                let response = list_presets_request().await;

                if current_user_id(&auth) != Some(user_id) {
                    return;
                }

                match response {
                    Ok(items) => request_state.replace_presets(items),
                    Err(message) => request_state.clear_presets_with_error(message),
                }
            });
        });
    }

    let open_save_dialog = {
        let state = state.clone();
        Callback::new(move |_| {
            state.open_save_dialog();
        })
    };

    let dismiss_dialog = {
        let state = state.clone();
        Callback::new(move |_| {
            state.dismiss_dialog();
        })
    };

    let auth_for_save = auth.clone();
    let submit_save = {
        let state = state.clone();
        Callback::new(move |_| {
            if state.save_state.get_untracked().is_pending()
                || save_cta_disabled(&state.save_state.get_untracked(), state.preset_count())
            {
                return;
            }

            let name = state.pending_name.get_untracked().trim().to_string();
            if name.is_empty() {
                state
                    .save_state
                    .set(MutationState::error("Preset name is required".to_string()));
                return;
            }

            let Some(user_id) = current_user_id(&auth_for_save) else {
                state.save_state.set(MutationState::error(
                    "You need to sign in before saving presets".to_string(),
                ));
                return;
            };

            let payload = PresetRequest {
                name,
                expr: expression.get_untracked(),
            };

            state.save_state.set(MutationState::pending());

            let auth = auth_for_save.clone();
            let state = state.clone();
            spawn_local(async move {
                let response = save_preset_request(payload).await;

                if current_user_id(&auth) != Some(user_id) {
                    return;
                }

                match response {
                    Ok(preset) => {
                        state.push_preset(preset);
                        state.dialog.set(None);
                        state.pending_name.set(String::new());
                        state.save_state.set(MutationState::success());
                    }
                    Err(message) => state.save_state.set(MutationState::error(message)),
                }
            });
        })
    };

    let auth_for_archive = auth.clone();
    let confirm_archive = {
        let state = state.clone();
        Callback::new(move |_| {
            let Some(PendingDialog::Delete(preset_id)) = state.dialog.get_untracked() else {
                return;
            };

            if state.archiving_id.get_untracked().is_some() {
                return;
            }

            let Some(user_id) = current_user_id(&auth_for_archive) else {
                state.archive_state.set(MutationState::error(
                    "You need to sign in before archiving presets".to_string(),
                ));
                return;
            };

            state.archiving_id.set(Some(preset_id));
            state.archive_state.set(MutationState::pending());

            let auth = auth_for_archive.clone();
            let state = state.clone();
            spawn_local(async move {
                let response = archive_preset_request(preset_id).await;

                if current_user_id(&auth) != Some(user_id) {
                    return;
                }

                match response {
                    Ok(()) => {
                        state.remove_preset(preset_id);
                        state.dialog.set(None);
                        state.archive_state.set(MutationState::success());
                    }
                    Err(message) => state.archive_state.set(MutationState::error(message)),
                }

                state.archiving_id.set(None);
            });
        })
    };

    let preset_count = state.preset_count_signal();

    let auth_for_view = auth.clone();
    let view_state = state.clone();
    let view_on_select = on_select.clone();
    view! {
        {move || {
            if auth_for_view.loading.get() || auth_for_view.user.get().is_none() {
                return None;
            }
            let presets_state = view_state.clone();
            let footer_state = view_state.clone();
            let save_dialog_state = view_state.clone();
            let delete_dialog_state = view_state.clone();
            let on_select = view_on_select.clone();
            let dismiss_dialog = dismiss_dialog.clone();
            let submit_save = submit_save.clone();
            let confirm_archive = confirm_archive.clone();
            Some(
                view! {
                    <section class=style::preset_section>
                        <div class=style::preset_header>
                            <div class=style::preset_heading>
                                <p class="g-section-label">"Presets"</p>
                                <h2 class=style::preset_title>"Your saved rolls"</h2>
                                <p class=style::preset_summary>
                                    "Save your preset rolls here for easy access"
                                </p>
                            </div>

                            <div class=style::preset_meta>
                                <span class=style::preset_count>
                                    {move || format!("{} / {}", preset_count.get(), MAX_PRESETS)}
                                </span>
                            </div>
                        </div>

                        {move || {
                            match view_state.presets_state.get() {
                                LoadState::Error(message) => {
                                    Some(view! { <p class=style::preset_feedback>{message}</p> })
                                }
                                LoadState::Idle | LoadState::Loading | LoadState::Ready(_) => None,
                            }
                        }}

                        <div class=style::preset_rail>
                            {move || {
                                match presets_state.presets_state.get() {
                                    LoadState::Idle | LoadState::Loading => {
                                        view! {
                                            <p class=format!(
                                                "g-result-hint {}",
                                                style::preset_hint_card,
                                            )>"Loading your saved presets..."</p>
                                        }
                                            .into_any()
                                    }
                                    LoadState::Error(_) => {
                                        view! {
                                            <div class=style::preset_empty>
                                                <span class=style::preset_empty_title>
                                                    "No presets saved yet."
                                                </span>
                                                <p class="g-result-hint">"Saved rolls will appear here."</p>
                                            </div>
                                        }
                                            .into_any()
                                    }
                                    LoadState::Ready(ref items) if items.is_empty() => {
                                        view! {
                                            <div class=style::preset_empty>
                                                <span class=style::preset_empty_title>
                                                    "No presets saved yet."
                                                </span>
                                                <p class="g-result-hint">"Saved rolls will appear here."</p>
                                            </div>
                                        }
                                            .into_any()
                                    }
                                    LoadState::Ready(items) => {
                                        let card_state = presets_state.clone();
                                        let on_select = on_select.clone();
                                        items
                                            .into_iter()
                                            .map(move |preset| {
                                                let state = card_state.clone();
                                                let on_select = on_select.clone();
                                                view! { <PresetCard preset state on_select /> }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }
                                }
                            }}
                        </div>

                        <div class=style::preset_editor_footer>
                            <div class=style::preset_editor_preview>
                                <span class="g-field-label">"Current expression"</span>
                                <code class=style::preset_editor_preview_code>
                                    {move || expression.get()}
                                </code>
                            </div>
                            <button
                                class="g-button-action"
                                type="button"
                                prop:disabled=move || {
                                    save_cta_disabled(
                                        &footer_state.save_state.get(),
                                        preset_count.get(),
                                    )
                                }
                                on:click=move |_| open_save_dialog.run(())
                            >
                                {move || {
                                    save_cta_copy(
                                        &footer_state.save_state.get(),
                                        preset_count.get(),
                                    )
                                }}
                            </button>
                        </div>

                        <SavePresetDialog
                            state=save_dialog_state
                            expression
                            on_close=dismiss_dialog
                            on_submit=submit_save
                        />

                        <DeletePresetDialog
                            state=delete_dialog_state
                            on_close=dismiss_dialog.clone()
                            on_confirm=confirm_archive
                        />
                    </section>
                },
            )
        }}
    }
}
