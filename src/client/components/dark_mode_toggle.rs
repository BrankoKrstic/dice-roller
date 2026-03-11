use leptos::prelude::*;

use crate::client::context::theme::{Theme, toggle_theme, use_theme_context};

stylance::import_style!(style, "dark_mode_toggle.module.scss");

#[component]
pub fn DarkModeToggle() -> impl IntoView {
    let context = use_theme_context();
    view! {
        <label class=style::switch for="themeSwitch">
            <input
                type="checkbox"
                id="themeSwitch"
                prop:checked=move || context.get() == Theme::Dark
                name="themeSwitch"
                aria-label="Toggle theme"
                on:change=move |_| toggle_theme()
            />
            <span class=style::slider>
                <span class=format!("{} {}", style::icon_wrapper, style::sun)>
                    <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <g clipPath="url(#clip0_501_49)">
                            <circle cx="12" cy="12" r="7" />
                            <rect x="11" y="20" width="2" height="4" rx="1" />
                            <rect
                                x="16.9498"
                                y="18.364"
                                width="2"
                                height="4"
                                rx="1"
                                transform="rotate(-45 16.9498 18.364)"
                            />
                            <rect
                                x="5.63605"
                                y="16.9497"
                                width="2"
                                height="4"
                                rx="1"
                                transform="rotate(45 5.63605 16.9497)"
                            />
                            <rect x="11" width="2" height="4" rx="1" />
                            <rect
                                x="2.80762"
                                y="4.22192"
                                width="2"
                                height="4"
                                rx="1"
                                transform="rotate(-45 2.80762 4.22192)"
                            />
                            <rect
                                x="20.0001"
                                y="13"
                                width="2"
                                height="4"
                                rx="1"
                                transform="rotate(-90 20.0001 13)"
                            />
                            <rect
                                x="6.10352e-05"
                                y="13"
                                width="2"
                                height="4"
                                rx="1"
                                transform="rotate(-90 6.10352e-05 13)"
                            />
                            <rect
                                x="19.7782"
                                y="2.80762"
                                width="2"
                                height="4"
                                rx="1"
                                transform="rotate(45 19.7782 2.80762)"
                            />
                        </g>
                        <defs>
                            <clipPath id="clip0_501_49">
                                <rect width="24" height="24" />
                            </clipPath>
                        </defs>
                    </svg>
                </span>

                <span class=format!("{} {}", style::icon_wrapper, style::moon)>
                    <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <path
                            fill-rule="evenodd"
                            clip-rule="evenodd"
                            d="M20.653 18.4371C20.9265 18.1123 20.649 17.6372 20.2244 17.6372C15.2538 17.6372 11.2244 13.6078 11.2244 8.63721C11.2244 6.43079 12.0184 4.40981 13.3362 2.8444C13.6101 2.51909 13.4252 2 13 2C7.47715 2 3 6.47715 3 12C3 17.5228 7.47715 22 13 22C16.071 22 18.8186 20.6157 20.653 18.4371Z"
                            fill="#3B324A"
                        />
                    </svg>
                </span>
            </span>
        </label>
    }
}
