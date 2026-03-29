use leptos::{html, prelude::*};

use crate::client::context::scroll_lock::use_scroll_lock_context;

stylance::import_style!(style, "dialog.module.scss");

#[component]
pub fn Dialog(
    #[prop(into)] open: Signal<bool>,
    title: String,
    #[prop(optional)] summary: Option<String>,
    #[prop(into)] on_close: Callback<()>,
    children: ChildrenFn,
) -> impl IntoView {
    let dialog_ref = NodeRef::<html::Dialog>::new();
    let scroll_lock = use_scroll_lock_context();
    let registered_lock = RwSignal::new(false);

    Effect::new(move |_| {
        let is_open = open.get();
        let is_registered = registered_lock.get();

        if is_open && !is_registered {
            scroll_lock.lock();
            registered_lock.set(true);
        } else if !is_open && is_registered {
            scroll_lock.unlock();
            registered_lock.set(false);
        }

        if !cfg!(feature = "hydrate") {
            return;
        }

        if is_open {
            if let Some(dialog) = dialog_ref.get()
                && !dialog.open()
            {
                let _ = dialog.show_modal();
            }
        } else {
            if let Some(dialog) = dialog_ref.get()
                && dialog.open()
            {
                dialog.close();
            }
        }
    });

    on_cleanup(move || {
        if registered_lock.get_untracked() {
            scroll_lock.unlock();
        }
    });

    view! {
        {move || {
            open.get()
                .then(|| {
                    let close = on_close.clone();
                    let dialog_ref = dialog_ref.clone();

                    view! {
                        <dialog
                            node_ref=dialog_ref
                            class=style::dialog_shell
                            aria-labelledby="dialog-title"
                            on:close=move |_| {
                                close.run(());
                            }
                        >
                            <div class=format!("g-panel g-panel-strong {}", style::dialog_panel)>
                                <div class=style::dialog_header>
                                    <div class=style::dialog_heading>
                                        <p class="g-section-label">"Preset controls"</p>
                                        <h2 id="dialog-title" class="g-section-title">
                                            {title.clone()}
                                        </h2>
                                        {summary
                                            .as_ref()
                                            .map(|summary| {
                                                view! { <p class="g-section-summary">{summary.clone()}</p> }
                                            })}
                                    </div>
                                    <button
                                        class=format!("g-button-ghost {}", style::dialog_close)
                                        type="button"
                                        on:click=move |_| {
                                            if let Some(dialog) = dialog_ref.get() {
                                                dialog.close();
                                            }
                                        }
                                    >
                                        "Close"
                                    </button>
                                </div>
                                <div class=style::dialog_body>{children()}</div>
                            </div>
                        </dialog>
                    }
                        .into_any()
                })
        }}
    }
}
