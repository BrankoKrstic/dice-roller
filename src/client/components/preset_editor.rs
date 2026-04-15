use leptos::{prelude::*, task::spawn_local};

use crate::{
    client::{
        components::dialog::Dialog,
        context::auth::use_auth_context,
        utils::{api::parse_error_response, url::base_url},
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

fn save_disabled(presets_len: usize, saving: bool) -> bool {
    saving || presets_len >= MAX_PRESETS
}

fn reset_for_signed_out(
    presets: RwSignal<Vec<Preset>>,
    loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
    dialog: RwSignal<Option<PendingDialog>>,
    pending_name: RwSignal<String>,
    saving: RwSignal<bool>,
    save_error: RwSignal<Option<String>>,
    archiving_id: RwSignal<Option<i64>>,
    archive_error: RwSignal<Option<String>>,
    last_loaded_user_id: RwSignal<Option<i64>>,
) {
    presets.set(Vec::new());
    loading.set(false);
    load_error.set(None);
    dialog.set(None);
    pending_name.set(String::new());
    saving.set(false);
    save_error.set(None);
    archiving_id.set(None);
    archive_error.set(None);
    last_loaded_user_id.set(None);
}

fn dismiss_dialog_state(
    dialog: RwSignal<Option<PendingDialog>>,
    pending_name: RwSignal<String>,
    save_error: RwSignal<Option<String>>,
    archive_error: RwSignal<Option<String>>,
) {
    dialog.set(None);
    pending_name.set(String::new());
    save_error.set(None);
    archive_error.set(None);
}

fn save_cta_copy(saving: bool, preset_count: usize) -> &'static str {
    if saving {
        "Saving..."
    } else if preset_count >= MAX_PRESETS {
        "Preset limit reached"
    } else {
        "Save as Preset"
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
    archiving_id: RwSignal<Option<i64>>,
    archive_error: RwSignal<Option<String>>,
    dialog: RwSignal<Option<PendingDialog>>,
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
                    prop:disabled=move || archiving_id.get() == Some(delete_id)
                    on:click=move |_| {
                        archive_error.set(None);
                        dialog.set(Some(PendingDialog::Delete(delete_id)));
                    }
                >
                    {move || {
                        if archiving_id.get() == Some(delete_id) {
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
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] expression: Signal<String>,
    pending_name: RwSignal<String>,
    save_error: RwSignal<Option<String>>,
    #[prop(into)] saving: Signal<bool>,
    #[prop(into)] preset_count: Signal<usize>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_submit: Callback<()>,
) -> impl IntoView {
    view! {
        <Dialog
            open
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
                prop:value=move || pending_name.get()
                on:input=move |event| {
                    save_error.set(None);
                    pending_name.set(event_target_value(&event));
                }
                maxlength="48"
                placeholder="Sneak Attack"
            />
            <div class=style::dialog_expression_preview>
                <span class="g-field-label">"Current expression"</span>
                <code class=style::dialog_expression_code>{move || expression.get()}</code>
            </div>
            {move || {
                save_error
                    .get()
                    .map(|message| view! { <p class=style::dialog_feedback>{message}</p> })
            }}
            <div class=style::dialog_actions>
                <button
                    class="g-button-ghost"
                    type="button"
                    on:click=move |_| on_close.run(())
                >
                    "Cancel"
                </button>
                <button
                    class="g-button-action"
                    type="button"
                    prop:disabled=move || {
                        saving.get() || pending_name.get().trim().is_empty()
                            || preset_count.get() >= MAX_PRESETS
                    }
                    on:click=move |_| on_submit.run(())
                >
                    {move || if saving.get() { "Saving..." } else { "Save preset" }}
                </button>
            </div>
        </Dialog>
    }
}

#[component]
fn DeletePresetDialog(
    #[prop(into)] open: Signal<bool>,
    selected_delete_preset: Signal<Option<Preset>>,
    archive_error: RwSignal<Option<String>>,
    #[prop(into)] archiving: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_confirm: Callback<()>,
) -> impl IntoView {
    view! {
        <Dialog
            open
            title="Delete preset".to_string()
            label="Preset controls"
            summary="Preset will be deleted from the quick access options".to_string()
            on_close
        >
            <p class=style::dialog_copy>
                {move || delete_dialog_copy(selected_delete_preset.get())}
            </p>
            {move || {
                archive_error
                    .get()
                    .map(|message| view! { <p class=style::dialog_feedback>{message}</p> })
            }}
            <div class=style::dialog_actions>
                <button
                    class="g-button-ghost"
                    type="button"
                    on:click=move |_| on_close.run(())
                >
                    "Keep preset"
                </button>
                <button
                    class=format!("g-button-utility {}", style::dialog_danger)
                    type="button"
                    prop:disabled=move || archiving.get()
                    on:click=move |_| on_confirm.run(())
                >
                    {move || if archiving.get() { "Deleting..." } else { "Delete preset" }}
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
    let presets = RwSignal::new(Vec::<Preset>::new());
    let loading = RwSignal::new(false);
    let load_error = RwSignal::new(None::<String>);
    let dialog = RwSignal::new(None::<PendingDialog>);
    let pending_name = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let save_error = RwSignal::new(None::<String>);
    let archiving_id = RwSignal::new(None::<i64>);
    let archive_error = RwSignal::new(None::<String>);
    let last_loaded_user_id = RwSignal::new(None::<i64>);

    let auth_for_effect = auth.clone();
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
            reset_for_signed_out(
                presets,
                loading,
                load_error,
                dialog,
                pending_name,
                saving,
                save_error,
                archiving_id,
                archive_error,
                last_loaded_user_id,
            );
            return;
        };

        let user_id = user.id.into_inner();
        if last_loaded_user_id.get_untracked() == Some(user_id) {
            return;
        }

        last_loaded_user_id.set(Some(user_id));
        loading.set(true);
        load_error.set(None);

        let auth = auth_for_effect.clone();
        spawn_local(async move {
            let response = list_presets_request().await;

            if current_user_id(&auth) != Some(user_id) {
                return;
            }

            match response {
                Ok(items) => {
                    presets.set(items);
                    load_error.set(None);
                }
                Err(message) => {
                    presets.set(Vec::new());
                    load_error.set(Some(message));
                }
            }

            loading.set(false);
        });
    });

    let open_save_dialog = Callback::new(move |_| {
        save_error.set(None);
        pending_name.set(String::new());
        dialog.set(Some(PendingDialog::Save));
    });

    let dismiss_dialog = Callback::new(move |_| {
        dismiss_dialog_state(dialog, pending_name, save_error, archive_error);
    });

    let auth_for_save = auth.clone();
    let submit_save = Callback::new(move |_| {
        if saving.get_untracked() || save_disabled(presets.get_untracked().len(), false) {
            return;
        }

        let name = pending_name.get_untracked().trim().to_string();
        if name.is_empty() {
            save_error.set(Some("Preset name is required".to_string()));
            return;
        }

        let Some(user_id) = current_user_id(&auth_for_save) else {
            save_error.set(Some(
                "You need to sign in before saving presets".to_string(),
            ));
            return;
        };

        let payload = PresetRequest {
            name,
            expr: expression.get_untracked(),
        };

        saving.set(true);
        save_error.set(None);

        let auth = auth_for_save.clone();
        spawn_local(async move {
            let response = save_preset_request(payload).await;

            if current_user_id(&auth) != Some(user_id) {
                return;
            }

            match response {
                Ok(preset) => {
                    presets.update(|items| items.push(preset));
                    dialog.set(None);
                    pending_name.set(String::new());
                }
                Err(message) => save_error.set(Some(message)),
            }

            saving.set(false);
        });
    });

    let auth_for_archive = auth.clone();
    let confirm_archive = Callback::new(move |_| {
        let Some(PendingDialog::Delete(preset_id)) = dialog.get_untracked() else {
            return;
        };

        if archiving_id.get_untracked().is_some() {
            return;
        }

        let Some(user_id) = current_user_id(&auth_for_archive) else {
            archive_error.set(Some(
                "You need to sign in before archiving presets".to_string(),
            ));
            return;
        };

        archiving_id.set(Some(preset_id));
        archive_error.set(None);

        let auth = auth_for_archive.clone();
        spawn_local(async move {
            let response = archive_preset_request(preset_id).await;

            if current_user_id(&auth) != Some(user_id) {
                return;
            }

            match response {
                Ok(()) => {
                    presets.update(|items| items.retain(|preset| preset.id.0 != preset_id));
                    dialog.set(None);
                }
                Err(message) => archive_error.set(Some(message)),
            }

            archiving_id.set(None);
        });
    });

    let save_dialog_open = Signal::derive(move || dialog.get() == Some(PendingDialog::Save));
    let delete_dialog_open =
        Signal::derive(move || matches!(dialog.get(), Some(PendingDialog::Delete(_))));
    let preset_count = Signal::derive(move || presets.get().len());
    let save_cta_disabled = Signal::derive(move || save_disabled(preset_count.get(), saving.get()));
    let selected_delete_preset = Signal::derive(move || match dialog.get() {
        Some(PendingDialog::Delete(preset_id)) => presets
            .get()
            .into_iter()
            .find(|preset| preset.id.0 == preset_id),
        _ => None,
    });

    let auth_for_view = auth.clone();
    view! {
        {move || {
            if auth_for_view.loading.get() || auth_for_view.user.get().is_none() {
                return None;
            }
            Some(view! {
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
                        load_error
                            .get()
                            .map(|message| view! { <p class=style::preset_feedback>{message}</p> })
                    }}

                    <div class=style::preset_rail>
                        {move || {
                            if loading.get() {
                                view! {
                                    <p class=format!("g-result-hint {}", style::preset_hint_card)>
                                        "Loading your saved presets..."
                                    </p>
                                }
                                    .into_any()
                            } else if presets.get().is_empty() {
                                view! {
                                    <div class=style::preset_empty>
                                        <span class=style::preset_empty_title>
                                            "No presets saved yet."
                                        </span>
                                        <p class="g-result-hint">"Saved rolls will appear here."</p>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                presets
                                    .get()
                                    .into_iter()
                                    .map(|preset| {
                                        view! {
                                            <PresetCard
                                                preset
                                                archiving_id
                                                archive_error
                                                dialog
                                                on_select
                                            />
                                        }
                                    })
                                    .collect_view()
                                    .into_any()
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
                            prop:disabled=move || save_cta_disabled.get()
                            on:click=move |_| open_save_dialog.run(())
                        >
                            {move || save_cta_copy(saving.get(), preset_count.get())}
                        </button>
                    </div>

                    <SavePresetDialog
                        open=save_dialog_open
                        expression
                        pending_name
                        save_error
                        saving
                        preset_count
                        on_close=dismiss_dialog
                        on_submit=submit_save
                    />

                    <DeletePresetDialog
                        open=delete_dialog_open
                        selected_delete_preset
                        archive_error
                        archiving=Signal::derive(move || archiving_id.get().is_some())
                        on_close=dismiss_dialog
                        on_confirm=confirm_archive
                    />
                </section>
            })
        }}
    }
}
