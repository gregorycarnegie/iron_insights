use crate::AssetManifest;
use maud::{Markup, html};

pub mod calculations;
pub mod charts;
pub mod data;
pub mod init;
pub mod main;
pub mod ui;
pub mod utils;
pub mod websocket;

pub fn render_scripts(manifest: &AssetManifest) -> Markup {
    let app_js_path = manifest.get("app.js");

    html! {
        script src=(app_js_path) defer {}
    }
}
