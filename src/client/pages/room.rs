#[cfg(feature = "hydrate")]
use std::{cell::RefCell, rc::Rc};

use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_params_map;

#[cfg(feature = "hydrate")]
use crate::{
    client::utils::rooms::{
        IntervalHandle, RoomEventStream, append_live_room_roll, room_roll_feed_from_page,
    },
    shared::data::room::RoomStreamEvent,
};
use crate::{
    client::{
        components::{
            active_user_feed::ActiveUserFeed, dialog::Dialog, roll_editor::RollEditor,
            roll_feed::RollFeed,
        },
        utils::{
            roll_feed::DiceRollFeed,
            rooms::{
                add_room_roll_request, allow_member_request, get_room_access_request,
                kick_member_request, list_room_rolls_request, prepend_room_roll_page,
            },
        },
    },
    shared::data::{
        room::{
            Room, RoomId, RoomMemberSummary, RoomRollId, RoomRosterMember, RoomViewerState,
            RoomViewerStatus,
        },
        user::UserId,
    },
};

stylance::import_style!(style, "room.module.scss");

#[derive(Debug, Clone, PartialEq)]
enum RoomAccessState {
    Loading,
    Ready(RoomViewerState),
    Error(String),
}

#[cfg(feature = "hydrate")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoomSubscriptionTarget {
    Stream(RoomId),
    Pending(RoomId),
    None,
}

fn room_page_content(
    access_state: Signal<RoomAccessState>,
    room: Signal<Option<Room>>,
    roster_members: Signal<Vec<RoomRosterMember>>,
    roll_feed: Signal<DiceRollFeed>,
    loading_more: Signal<bool>,
    stream_connected: Signal<bool>,
    live_ready: Signal<bool>,
    stream_error: Signal<Option<String>>,
    roll_error: Signal<Option<String>>,
    action_error: Signal<Option<String>>,
    action_busy_user_id: Signal<Option<i64>>,
    kick_dialog_member: Signal<Option<RoomMemberSummary>>,
    load_older_rolls: Callback<()>,
    on_roll: Callback<String>,
    on_allow_member: Callback<UserId>,
    on_request_kick: Callback<UserId>,
    on_cancel_kick: Callback<()>,
    on_confirm_kick: Callback<()>,
) -> impl IntoView {
    view! {
        <section class="g-page g-page-shell">
            <section class=format!("g-panel g-panel-strong {}", style::room_shell)>
                <div class="g-page-meta">
                    <a class="g-button-utility" href="/rooms">
                        "Back to rooms"
                    </a>
                </div>

                {move || match access_state.get() {
                    RoomAccessState::Loading => {
                        view! {
                            <section class=format!("g-panel g-panel-strong {}", style::state_card)>
                                <p class="g-section-label">"Room access"</p>
                                <h1 class=style::room_title>"Loading room..."</h1>
                                <p class=style::room_summary>
                                    "Checking your access and opening the live room stream."
                                </p>
                            </section>
                        }
                            .into_any()
                    }
                    RoomAccessState::Error(message) => {
                        view! {
                            <section class=format!("g-panel g-panel-strong {}", style::state_card)>
                                <p class="g-section-label">"Room lookup"</p>
                                <h1 class=style::room_title>"Room unavailable."</h1>
                                <p class=style::room_summary>{message}</p>
                            </section>
                        }
                            .into_any()
                    }
                    RoomAccessState::Ready(viewer) => {
                        match viewer.viewer_status {
                            RoomViewerStatus::Pending => {
                                view! {
                                    <section class=format!(
                                        "g-panel g-panel-strong {}",
                                        style::state_card,
                                    )>
                                        <p class="g-section-label">"Waiting for approval"</p>
                                        <h1 class=style::room_title>{viewer.room.name.clone()}</h1>
                                        <p class=style::room_summary>
                                            "Your join request is in. Stay on this page while the room admin approves access."
                                        </p>
                                        <div class=style::room_header_meta>
                                            <span class=style::room_id_badge>
                                                {format!("#{}", viewer.room.id.into_inner())}
                                            </span>
                                            <span class=style::room_note_badge>"Pending"</span>
                                        </div>
                                        {move || {
                                            stream_error
                                                .get()
                                                .map(|message| {
                                                    view! { <p class=style::page_feedback>{message}</p> }
                                                })
                                        }}
                                    </section>
                                }
                                    .into_any()
                            }
                            RoomViewerStatus::Kicked => {
                                view! {
                                    <section class=format!(
                                        "g-panel g-panel-strong {}",
                                        style::state_card,
                                    )>
                                        <p class="g-section-label">"Room access revoked"</p>
                                        <h1 class=style::room_title>{viewer.room.name.clone()}</h1>
                                        <p class=style::room_summary>
                                            "You no longer have access to this room. Head back to the rooms board for another table."
                                        </p>
                                        <div class=style::room_header_meta>
                                            <span class=style::room_id_badge>
                                                {format!("#{}", viewer.room.id.into_inner())}
                                            </span>
                                            <span class=style::room_note_badge>"Kicked"</span>
                                        </div>
                                    </section>
                                }
                                    .into_any()
                            }
                            RoomViewerStatus::Creator | RoomViewerStatus::Joined => {
                                let room = room.get().unwrap_or(viewer.room.clone());

                                view! {
                                    <div class=format!("g-page-shell-split {}", style::room_layout)>
                                        <section class=style::room_main>
                                            <section class=format!(
                                                "g-panel g-panel-strong {}",
                                                style::room_header,
                                            )>
                                                <div class=style::room_header_top>
                                                    <p class="g-section-label">"Table ledger"</p>
                                                    <h1 class=style::room_title>{room.name.clone()}</h1>
                                                    <p class=style::room_summary>
                                                        "Keep live presence, shared room activity, and the next throw in the same place so the table stays synchronized."
                                                    </p>
                                                </div>

                                                <div class=style::room_header_meta>
                                                    <span class=style::room_id_badge>
                                                        {format!("#{}", room.id.into_inner())}
                                                    </span>
                                                    <span class=style::room_note_badge>
                                                        {if viewer.can_manage_members {
                                                            "Admin".to_string()
                                                        } else {
                                                            "Joined".to_string()
                                                        }}
                                                    </span>
                                                    <span
                                                        class=style::stream_badge
                                                        data-connected=move || {
                                                            if stream_connected.get() { "true" } else { "false" }
                                                        }
                                                    >
                                                        {move || {
                                                            if stream_connected.get() {
                                                                "Live stream open".to_string()
                                                            } else {
                                                                "Live stream reconnecting".to_string()
                                                            }
                                                        }}
                                                    </span>
                                                </div>
                                            </section>

                                            {move || {
                                                if !live_ready.get() {
                                                    view! {
                                                        <section class=format!(
                                                            "g-panel g-panel-strong {}",
                                                            style::state_card,
                                                        )>
                                                            <p class="g-section-label">"Opening stream"</p>
                                                            <h2 class=style::state_title>
                                                                "Waiting for the initial room snapshot."
                                                            </h2>
                                                            <p class=style::room_summary>
                                                                "The page switches to the live room as soon as the server sends the current roster and roll history."
                                                            </p>
                                                        </section>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! {
                                                        <>
                                                            {move || {
                                                                stream_error
                                                                    .get()
                                                                    .map(|message| {
                                                                        view! { <p class=style::page_feedback>{message}</p> }
                                                                    })
                                                            }}
                                                            {move || {
                                                                roll_error
                                                                    .get()
                                                                    .map(|message| {
                                                                        view! { <p class=style::page_feedback>{message}</p> }
                                                                    })
                                                            }} <RollEditor on_roll=on_roll />
                                                        </>
                                                    }
                                                        .into_any()
                                                }
                                            }}
                                        </section>

                                        <aside class=style::room_rail>
                                            <ActiveUserFeed
                                                roster_members=roster_members
                                                connected=stream_connected
                                                can_manage_members=viewer.can_manage_members
                                                busy_user_id=action_busy_user_id
                                                action_error=action_error
                                                on_allow=on_allow_member
                                                on_request_kick=on_request_kick
                                            />

                                            <RollFeed
                                                feed=roll_feed
                                                loading_more=loading_more
                                                load_older_rolls=load_older_rolls
                                            />
                                        </aside>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }
                }}
            </section>

            <Dialog
                open=move || kick_dialog_member.get().is_some()
                label="Member controls"
                title="Confirm kick".to_string()
                summary="Kicked users lose access immediately and stay visible in the kicked list so they can be reinstated later."
                    .to_string()
                on_close=on_cancel_kick
            >
                <p class=style::room_summary>
                    {move || {
                        kick_dialog_member
                            .get()
                            .map(|member| {
                                format!(
                                    "{} will lose access to the room immediately.",
                                    member.username.as_str(),
                                )
                            })
                            .unwrap_or_default()
                    }}
                </p>
                <div class=style::dialog_actions>
                    <button
                        class="g-button-ghost"
                        type="button"
                        on:click=move |_| on_cancel_kick.run(())
                    >
                        "Cancel"
                    </button>
                    <button
                        class="g-button-action"
                        type="button"
                        on:click=move |_| on_confirm_kick.run(())
                    >
                        "Confirm kick"
                    </button>
                </div>
            </Dialog>
        </section>
    }
}

#[component]
pub fn RoomPage() -> impl IntoView {
    let params = use_params_map();
    let access_state = RwSignal::new(RoomAccessState::Loading);
    let current_room_id = RwSignal::new(None::<RoomId>);
    let room = RwSignal::new(None::<Room>);
    let roster_members = RwSignal::new(Vec::<RoomRosterMember>::new());
    let roll_feed = RwSignal::new(DiceRollFeed::new());
    let next_before_id = RwSignal::new(None::<RoomRollId>);
    let loading_more = RwSignal::new(false);
    let stream_connected = RwSignal::new(false);
    let live_ready = RwSignal::new(false);
    let stream_error = RwSignal::new(None::<String>);
    let roll_error = RwSignal::new(None::<String>);
    let action_error = RwSignal::new(None::<String>);
    let action_busy_user_id = RwSignal::new(None::<i64>);
    let kick_dialog_member = RwSignal::new(None::<RoomMemberSummary>);

    #[cfg(feature = "hydrate")]
    let room_stream = Rc::new(RefCell::new(None::<RoomEventStream>));
    #[cfg(feature = "hydrate")]
    let pending_poll = Rc::new(RefCell::new(None::<IntervalHandle>));
    #[cfg(feature = "hydrate")]
    let subscription_target = RwSignal::new(RoomSubscriptionTarget::None);

    Effect::new(move |_| {
        let raw_room_id = params.get().get("roomId").unwrap_or_default();

        current_room_id.set(None);
        room.set(None);
        roster_members.set(Vec::new());
        roll_feed.set(DiceRollFeed::new());
        next_before_id.set(None);
        loading_more.set(false);
        stream_connected.set(false);
        live_ready.set(false);
        stream_error.set(None);
        roll_error.set(None);
        action_error.set(None);
        action_busy_user_id.set(None);
        kick_dialog_member.set(None);

        let trimmed_room_id = raw_room_id.trim().to_string();
        if trimmed_room_id.is_empty() {
            access_state.set(RoomAccessState::Error("Room not found.".to_string()));
            return;
        }

        let Ok(room_id) = trimmed_room_id.parse::<i64>() else {
            access_state.set(RoomAccessState::Error(
                "Room IDs use digits only.".to_string(),
            ));
            return;
        };

        let room_id = RoomId(room_id);
        current_room_id.set(Some(room_id));
        access_state.set(RoomAccessState::Loading);

        spawn_local(async move {
            match get_room_access_request(room_id.into_inner()).await {
                Ok(viewer_state) => {
                    room.set(Some(viewer_state.room.clone()));
                    access_state.set(RoomAccessState::Ready(viewer_state));
                }
                Err(message) => access_state.set(RoomAccessState::Error(message)),
            }
        });
    });

    #[cfg(feature = "hydrate")]
    Effect::new({
        let room_stream = room_stream.clone();
        let pending_poll = pending_poll.clone();

        move |_| {
            let next_target = match (current_room_id.get(), access_state.get()) {
                (Some(room_id), RoomAccessState::Ready(viewer_state)) => {
                    match viewer_state.viewer_status {
                        RoomViewerStatus::Creator | RoomViewerStatus::Joined => {
                            RoomSubscriptionTarget::Stream(room_id)
                        }
                        RoomViewerStatus::Pending => RoomSubscriptionTarget::Pending(room_id),
                        RoomViewerStatus::Kicked => RoomSubscriptionTarget::None,
                    }
                }
                _ => RoomSubscriptionTarget::None,
            };

            if subscription_target.get_untracked() == next_target {
                return;
            }

            room_stream.borrow_mut().take();
            pending_poll.borrow_mut().take();
            subscription_target.set(next_target);

            match next_target {
                RoomSubscriptionTarget::Stream(room_id) => {
                    live_ready.set(false);
                    stream_error.set(None);

                    let stream = RoomEventStream::connect(
                        room_id,
                        {
                            let room = room;
                            let roster_members = roster_members;
                            let roll_feed = roll_feed;
                            let next_before_id = next_before_id;
                            let live_ready = live_ready;
                            let stream_connected = stream_connected;
                            let access_state = access_state;

                            move |event| match event {
                                RoomStreamEvent::Snapshot { snapshot } => {
                                    room.set(Some(snapshot.room.clone()));
                                    roster_members.set(snapshot.roster_members);
                                    next_before_id.set(snapshot.recent_rolls.next_before_id);
                                    roll_feed.set(room_roll_feed_from_page(&snapshot.recent_rolls));
                                    live_ready.set(true);
                                    stream_connected.set(true);

                                    access_state.update(|state| {
                                        if let RoomAccessState::Ready(viewer_state) = state {
                                            viewer_state.room = snapshot.room.clone();
                                            viewer_state.can_manage_members =
                                                snapshot.can_manage_members;
                                        }
                                    });
                                }
                                RoomStreamEvent::RosterChanged {
                                    roster_members: next_members,
                                } => {
                                    roster_members.set(next_members);
                                    stream_connected.set(true);
                                }
                                RoomStreamEvent::RollCreated { roll } => {
                                    roll_feed.update(|feed| append_live_room_roll(feed, &roll));
                                    stream_connected.set(true);
                                }
                                RoomStreamEvent::AccessRevoked { reason } => {
                                    stream_connected.set(false);
                                    stream_error.set(Some(reason.clone()));
                                    live_ready.set(false);
                                    roster_members.set(Vec::new());
                                    roll_feed.set(DiceRollFeed::new());
                                    next_before_id.set(None);

                                    if let Some(room) = room.get_untracked() {
                                        access_state.set(RoomAccessState::Ready(RoomViewerState {
                                            room,
                                            viewer_status: RoomViewerStatus::Kicked,
                                            can_manage_members: false,
                                        }));
                                    } else {
                                        access_state.set(RoomAccessState::Error(reason));
                                    }
                                }
                            }
                        },
                        move |connected| {
                            stream_connected.set(connected);
                            if connected {
                                stream_error.set(None);
                            }
                        },
                        move |message| {
                            stream_connected.set(false);
                            stream_error.set(Some(message));
                        },
                    );

                    match stream {
                        Ok(stream) => {
                            room_stream.borrow_mut().replace(stream);
                        }
                        Err(message) => {
                            stream_connected.set(false);
                            stream_error.set(Some(message));
                        }
                    }
                }
                RoomSubscriptionTarget::Pending(room_id) => {
                    stream_connected.set(false);
                    live_ready.set(false);

                    let poll = IntervalHandle::start(4_000, {
                        let access_state = access_state;
                        let room = room;
                        move || {
                            spawn_local(async move {
                                match get_room_access_request(room_id.into_inner()).await {
                                    Ok(viewer_state) => {
                                        room.set(Some(viewer_state.room.clone()));
                                        access_state.set(RoomAccessState::Ready(viewer_state));
                                        stream_error.set(None);
                                    }
                                    Err(message) => stream_error.set(Some(message)),
                                }
                            });
                        }
                    });

                    match poll {
                        Ok(poll) => {
                            pending_poll.borrow_mut().replace(poll);
                        }
                        Err(message) => stream_error.set(Some(message)),
                    }
                }
                RoomSubscriptionTarget::None => {
                    stream_connected.set(false);
                    live_ready.set(false);
                }
            }
        }
    });

    let on_roll = Callback::new(move |expression: String| {
        let Some(room_id) = current_room_id.get_untracked() else {
            return;
        };

        roll_error.set(None);
        spawn_local(async move {
            if let Err(message) = add_room_roll_request(room_id.into_inner(), &expression).await {
                roll_error.set(Some(message));
            }
        });
    });

    let load_older_rolls = Callback::new(move |_| {
        let Some(room_id) = current_room_id.get_untracked() else {
            return;
        };
        let Some(before_id) = next_before_id.get_untracked() else {
            return;
        };
        if loading_more.get_untracked() {
            return;
        }

        loading_more.set(true);
        stream_error.set(None);

        spawn_local(async move {
            match list_room_rolls_request(room_id.into_inner(), Some(before_id)).await {
                Ok(page) => {
                    next_before_id.set(page.next_before_id);
                    roll_feed.update(|feed| prepend_room_roll_page(feed, &page));
                }
                Err(message) => stream_error.set(Some(message)),
            }
            loading_more.set(false);
        });
    });

    let on_allow_member = Callback::new(move |user_id: UserId| {
        let Some(room_id) = current_room_id.get_untracked() else {
            return;
        };
        let user_id_value = user_id.into_inner();

        action_error.set(None);
        action_busy_user_id.set(Some(user_id_value));

        spawn_local(async move {
            if let Err(message) = allow_member_request(room_id.into_inner(), user_id_value).await {
                action_error.set(Some(message));
            }
            action_busy_user_id.set(None);
        });
    });

    let on_request_kick = Callback::new(move |user_id: UserId| {
        let selected_member = roster_members
            .get_untracked()
            .into_iter()
            .find(|member| !member.is_creator && member.user_id == user_id)
            .map(|member| RoomMemberSummary {
                user_id: member.user_id,
                username: member.username,
                status: member.status,
            });

        kick_dialog_member.set(selected_member);
    });

    let on_cancel_kick = Callback::new(move |_| {
        kick_dialog_member.set(None);
    });

    let on_confirm_kick = Callback::new(move |_| {
        let Some(room_id) = current_room_id.get_untracked() else {
            return;
        };
        let Some(member) = kick_dialog_member.get_untracked() else {
            return;
        };

        let user_id = member.user_id.into_inner();
        action_error.set(None);
        action_busy_user_id.set(Some(user_id));

        spawn_local(async move {
            if let Err(message) = kick_member_request(room_id.into_inner(), user_id).await {
                action_error.set(Some(message));
            } else {
                kick_dialog_member.set(None);
            }
            action_busy_user_id.set(None);
        });
    });

    room_page_content(
        access_state.into(),
        room.into(),
        roster_members.into(),
        roll_feed.into(),
        loading_more.into(),
        stream_connected.into(),
        live_ready.into(),
        stream_error.into(),
        roll_error.into(),
        action_error.into(),
        action_busy_user_id.into(),
        kick_dialog_member.into(),
        load_older_rolls,
        on_roll,
        on_allow_member,
        on_request_kick,
        on_cancel_kick,
        on_confirm_kick,
    )
}
