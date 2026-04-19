#[cfg(any(target_arch = "wasm32", test))]
pub mod core {
    pub use iron_insights_core::*;
}

#[cfg(any(target_arch = "wasm32", test))]
mod webapp;

#[cfg(target_arch = "wasm32")]
fn main() {
    webapp::run();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("ironscale is a wasm CSR app. Build for wasm32-unknown-unknown.");
}
