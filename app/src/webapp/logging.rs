#[cfg(debug_assertions)]
pub(super) fn debug_log(message: &str) {
    web_sys::console::debug_1(&message.into());
}

#[cfg(not(debug_assertions))]
pub(super) fn debug_log(_message: &str) {}
