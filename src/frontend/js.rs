use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/app.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn set_cookie_js(key: &str, value: &str, days_valid: usize) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn get_cookie_js(key: &str) -> Result<Option<String>, JsValue>;
    #[wasm_bindgen(catch)]
    fn get_location_js(
        on_success: &Closure<dyn Fn(f32, f32)>,
        on_error: &Closure<dyn Fn(u16, String)>,
    ) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn is_geolocation_available_js() -> Result<bool, JsValue>;
}

// apparently this is not actually necessary
// compiling and running code that calls the js functions without an unsafe block works just fine
// however VS code with rust-analyzer shows an error, which is very annoying in development,
// hence this workaround

#[allow(unused_unsafe)]
pub fn set_cookie(key: &str, value: &str, days_valid: usize) {
    unsafe {
        if let Err(e) = set_cookie_js(key, value, days_valid) {
            error!("Error setting cookie: {:?}", e);
        }
    }
}

#[allow(unused_unsafe)]
pub fn get_cookie(key: &str) -> Option<String> {
    unsafe {
        match get_cookie_js(key) {
            Ok(value) => value,
            Err(e) => {
                error!("Error getting cookie value: {:?}", e);
                None
            }
        }
    }
}

#[allow(unused_unsafe)]
pub fn get_location(
    on_success: &Closure<dyn Fn(f32, f32)>,
    on_error: &Closure<dyn Fn(u16, String)>,
) {
    unsafe {
        if let Err(e) = get_location_js(on_success, on_error) {
            error!("Error getting location: {:?}", e);
        }
    }
}

#[allow(unused_unsafe)]
pub fn is_geolocation_available() -> bool {
    unsafe { is_geolocation_available_js().unwrap_or(false) }
}
