#![warn(clippy::pedantic)]
// Canvas rendering and pixel math inherently cross numeric types; the values are
// always small non-negative (CSS pixels, bin counts, histogram indices). Silencing
// these here beats dotting #[allow(...)] over every draw call.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]

mod app;
mod charts;
mod components;
mod cross_sex;
mod data;
mod helpers;
mod logging;
mod models;
mod persistence;
mod selectors;
mod state;
mod ui;

pub(super) use app::AppPage;

use self::app::App;
use leptos::{mount::mount_to, prelude::*};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

/// Bootstraps and mounts the Leptos web application to the DOM.
/// Also removes the static `#app-shell` element once the WASM app is mounted.
pub fn run() {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        mount_to_body(|| view! { <App /> });
        return;
    };

    let Some(app_root) = document
        .get_element_by_id("app")
        .and_then(|el| el.dyn_into::<HtmlElement>().ok())
    else {
        mount_to_body(|| view! { <App /> });
        return;
    };

    let owner = mount_to(app_root, || view! { <App /> });
    if let Some(shell) = document.get_element_by_id("app-shell") {
        shell.remove();
    }
    owner.forget();
}
