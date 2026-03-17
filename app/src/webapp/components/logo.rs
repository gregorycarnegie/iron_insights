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
            <rect x="8" y="8" width="112" height="112" rx="24" fill="#c8f135" />
            <path d="M22 54h84v20H22z" fill="#08080a" />
            <rect x="10" y="49" width="16" height="30" rx="6" fill="#08080a" />
            <rect x="102" y="49" width="16" height="30" rx="6" fill="#08080a" />
            <rect x="42" y="38" width="12" height="52" rx="6" fill="#08080a" />
            <rect x="74" y="38" width="12" height="52" rx="6" fill="#08080a" />
            <path
                d="M8 26c14-10 29-14 44-14h68v18H55c-18 0-33 3-47 12z"
                fill="#ffffff"
                fill-opacity="0.18"
            />
        </svg>
    }
}
