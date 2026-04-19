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
mod share;
mod slices;
mod state;
mod ui;

pub(super) use app::AppPage;

use self::app::App;
use leptos::mount::mount_to;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

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
