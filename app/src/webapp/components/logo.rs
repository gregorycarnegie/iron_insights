use leptos::prelude::*;

#[component]
pub(in crate::webapp) fn LogoMark() -> impl IntoView {
    view! {
        <svg
            class="brand-logo"
            viewBox="0 0 128 128"
            width="40"
            height="40"
            role="img"
            aria-label="Iron Insights logo"
            xmlns="http://www.w3.org/2000/svg"
        >
            <rect x="8" y="8" width="112" height="112" rx="24" fill="#0f2e24" />
            <path
                d="M30 69h68v10H30zm8-14h52v10H38zm-8 28h68v10H30z"
                fill="#f6f2e8"
            />
            <circle cx="22" cy="74" r="8" fill="#d4c6a9" />
            <circle cx="106" cy="74" r="8" fill="#d4c6a9" />
            <path d="M64 28 79 44H49z" fill="#d4c6a9" />
        </svg>
    }
}
