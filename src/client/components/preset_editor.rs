use leptos::{prelude::*, task::spawn_local};
use serde::Deserialize;

use crate::{
    client::{components::dialog::Dialog, context::auth::use_auth_context, utils::url::base_url},
    shared::data::{
        preset::{Preset, PresetRequest},
        user::AuthContext,
    },
};

stylance::import_style!(style, "preset_editor.module.scss");

const MAX_PRESETS: usize = 10;

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: String,
}

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

async fn parse_error_response(response: reqwest::Response, fallback: &str) -> String {
    let status = response.status();
    let text = response
        .text()
        .await
        .unwrap_or_else(|_| fallback.to_string());

    serde_json::from_str::<ApiErrorResponse>(&text)
        .map(|payload| payload.error)
        .unwrap_or_else(|_| {
            if text.trim().is_empty() {
                format!("{fallback} ({status})")
            } else {
                text
            }
        })
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
        return Err(parse_error_response(response, "Failed to archive preset").await);
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
        dialog.set(None);
        save_error.set(None);
        archive_error.set(None);
        pending_name.set(String::new());
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
            Some(

                view! {
                    <section class=style::preset_section>
                        <div class=style::preset_header>
                            <div class=style::preset_heading>
                                <p class="g-section-label">"Saved bench"</p>
                                <h2 class=style::preset_title>"Your ready rolls"</h2>
                                <p class=style::preset_summary>
                                    "Pin your common moves here so the editor can jump straight to exact notation."
                                </p>
                            </div>

                            <div class=style::preset_meta>
                                <span class=style::preset_count>
                                    {move || format!("{} / {}", preset_count.get(), MAX_PRESETS)}
                                </span>
                                <button
                                    class="g-button-action"
                                    type="button"
                                    prop:disabled=move || save_cta_disabled.get()
                                    on:click=move |_| open_save_dialog.run(())
                                >
                                    {move || {
                                        if saving.get() {
                                            "Saving..."
                                        } else if preset_count.get() >= MAX_PRESETS {
                                            "Preset limit reached"
                                        } else {
                                            "Save Preset"
                                        }
                                    }}
                                </button>
                            </div>
                        </div>

                        {move || {
                            load_error
                                .get()
                                .map(|message| {
                                    view! { <p class=style::preset_feedback>{message}</p> }
                                })
                        }}

                        <div class=style::preset_rail>
                            {move || {
                                if loading.get() {
                                    view! {
                                        <p class=format!(
                                            "g-result-hint {}",
                                            style::preset_hint_card,
                                        )>"Loading your saved presets..."</p>
                                    }
                                        .into_any()
                                } else if presets.get().is_empty() {
                                    view! {
                                        <div class=style::preset_empty>
                                            <span class=style::preset_empty_title>
                                                "No presets saved yet."
                                            </span>
                                            <p class="g-result-hint">
                                                "Save the current expression to keep fast access to repeat rolls."
                                            </p>
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    presets
                                        .get()
                                        .into_iter()
                                        .map(|preset| {
                                            let apply_expr = preset.expr.clone();
                                            let delete_id = preset.id.0;
                                            let delete_name = preset.name.clone();

                                            view! {
                                                <article class=style::preset_card>
                                                    <button
                                                        class=style::preset_launch
                                                        type="button"
                                                        on:click=move |_| on_select.run(apply_expr.clone())
                                                    >
                                                        <span class=style::preset_card_title>
                                                            {preset.name.clone()}
                                                        </span>
                                                        <code class=style::preset_card_code>
                                                            {preset.expr.clone()}
                                                        </code>
                                                    </button>
                                                    <button
                                                        class=format!("g-button-utility {}", style::preset_archive)
                                                        type="button"
                                                        prop:disabled=move || {
                                                            archiving_id.get() == Some(delete_id)
                                                        }
                                                        on:click=move |_| {
                                                            archive_error.set(None);
                                                            dialog.set(Some(PendingDialog::Delete(delete_id)));
                                                        }
                                                    >
                                                        {move || {
                                                            if archiving_id.get() == Some(delete_id) {
                                                                "Archiving..."
                                                            } else {
                                                                "Archive"
                                                            }
                                                        }}
                                                    </button>
                                                    <span class=style::preset_sr_note>{delete_name}</span>
                                                </article>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                            }}
                        </div>

                        <Dialog
                            open=save_dialog_open
                            title="Save preset".to_string()
                            label="Preset controls"
                            summary="Name this expression so it can be dropped back into the editor in one click."
                                .to_string()
                            on_close=dismiss_dialog
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
                                <code class=style::dialog_expression_code>
                                    {move || expression.get()}
                                </code>
                            </div>
                            {move || {
                                save_error
                                    .get()
                                    .map(|message| {
                                        view! { <p class=style::dialog_feedback>{message}</p> }
                                    })
                            }}
                            <div class=style::dialog_actions>
                                <button
                                    class="g-button-ghost"
                                    type="button"
                                    on:click=move |_| dismiss_dialog.run(())
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
                                    on:click=move |_| submit_save.run(())
                                >
                                    {move || if saving.get() { "Saving..." } else { "Save preset" }}
                                </button>
                            </div>
                        </Dialog>

                        <Dialog
                            open=delete_dialog_open
                            title="Archive preset".to_string()
                            label="Preset controls"
                            summary="Archived presets leave the quick rail so the editor only keeps active moves."
                                .to_string()
                            on_close=dismiss_dialog
                        >
                            <p class=style::dialog_copy>
                                {move || {
                                    selected_delete_preset
                                        .get()
                                        .map(|preset| {
                                            format!(
                                                "Archive \"{}\" from your preset rail?",
                                                preset.name,
                                            )
                                        })
                                        .unwrap_or_else(|| "Archive this preset?".to_string())
                                }}
                            </p>
                            {move || {
                                archive_error
                                    .get()
                                    .map(|message| {
                                        view! { <p class=style::dialog_feedback>{message}</p> }
                                    })
                            }}
                            <div class=style::dialog_actions>
                                <button
                                    class="g-button-ghost"
                                    type="button"
                                    on:click=move |_| dismiss_dialog.run(())
                                >
                                    "Keep preset"
                                </button>
                                <button
                                    class=format!("g-button-utility {}", style::dialog_danger)
                                    type="button"
                                    prop:disabled=move || archiving_id.get().is_some()
                                    on:click=move |_| confirm_archive.run(())
                                >
                                    {move || {
                                        if archiving_id.get().is_some() {
                                            "Archiving..."
                                        } else {
                                            "Archive preset"
                                        }
                                    }}
                                </button>
                            </div>
                        </Dialog>
                    </section>
                },
            )
        }}
    }
}
