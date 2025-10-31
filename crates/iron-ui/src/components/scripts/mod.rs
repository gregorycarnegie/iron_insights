use maud::{Markup, PreEscaped};

pub mod calculations;
pub mod charts;
pub mod data;
pub mod init;
pub mod main;
pub mod ui;
pub mod utils;
pub mod websocket;

pub use calculations::render_calculation_scripts;
pub use charts::render_chart_scripts;
pub use data::render_data_scripts;
pub use init::render_init_scripts;
pub use main::render_main_scripts;
pub use ui::render_ui_scripts;
pub use utils::render_utility_scripts;
pub use websocket::render_websocket_scripts;

pub fn render_scripts() -> Markup {
    PreEscaped(format!(
        "<script>{}{}{}{}{}{}{}{}</script>",
        render_init_scripts().into_string(),
        render_websocket_scripts().into_string(),
        render_data_scripts().into_string(),
        render_chart_scripts().into_string(),
        render_ui_scripts().into_string(),
        render_calculation_scripts().into_string(),
        render_utility_scripts().into_string(),
        render_main_scripts().into_string()
    ))
}
