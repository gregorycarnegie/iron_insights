#[cfg(target_arch = "wasm32")]
mod webapp;

#[cfg(target_arch = "wasm32")]
fn main() {
    webapp::run();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("iron_insights_app is a wasm CSR app. Build for wasm32-unknown-unknown.");
}
