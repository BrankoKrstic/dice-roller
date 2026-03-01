use leptos::{prelude::*, server::codee::string::JsonSerdeCodec};
use leptos_use::storage::{use_local_storage, use_local_storage_with_options, UseStorageOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

pub fn provide_theme_context() {
    let (read_signal, ..) = use_local_storage_with_options::<Theme, JsonSerdeCodec>(
        "theme",
        UseStorageOptions::default().delay_during_hydration(true),
    );
    provide_context::<Signal<Theme>>(read_signal);
}

pub fn toggle_theme() {
    let (read_signal, write_signal, _) = use_local_storage::<Theme, JsonSerdeCodec>("theme");
    write_signal.set(match read_signal.get() {
        Theme::Light => Theme::Dark,
        Theme::Dark => Theme::Light,
    });
}

pub fn use_theme_context() -> Signal<Theme> {
    use_context::<Signal<Theme>>().expect("Signal shoudl be provided")
}
