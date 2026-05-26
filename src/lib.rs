pub mod app;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    #[allow(unused_imports)]
    use crate::app::*;
    #[cfg(feature = "hydrate-diagnostics")]
    console_error_panic_hook::set_once();
    #[cfg(feature = "hydrate-diagnostics")]
    tracing_wasm::set_as_global_default();
    leptos::mount::hydrate_body(App);
}
