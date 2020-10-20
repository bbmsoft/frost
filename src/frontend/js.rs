use chrono::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/app.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn set_cookie_js(cookie: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn get_cookies_js() -> Result<String, JsValue>;
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
    let cookie = format_cookie(key, value, days_valid);
    unsafe {
        if let Err(e) = set_cookie_js(&cookie) {
            error!("Error setting cookie: {:?}", e);
        }
    }
}

#[allow(unused_unsafe)]
pub fn get_cookie(key: &str) -> Option<String> {
    unsafe {
        match get_cookies_js() {
            Ok(cookies) => extract_cookie(key, cookies),
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

fn format_cookie(key: &str, value: &str, days_valid: usize) -> String {
    let expiry_date = Utc::now() + chrono::Duration::days(days_valid as i64);
    let expiry_date = expiry_date.to_rfc3339();
    format!(
        "{}={};expires={};secure;samesite=strict",
        key, value, expiry_date
    )
}

fn extract_cookie(key: &str, cookies: String) -> Option<String> {
    let individual_cookies = cookies.split(";");
    let key_value_pairs = individual_cookies.map(|c| c.trim().split("="));
    let mut cookies_with_matching_keys = key_value_pairs.filter_map(|mut c| match c.next() {
        Some(k) if k == key => Some(c.next().unwrap_or("")),
        _ => None,
    });
    cookies_with_matching_keys.next().map(|v| v.to_owned())
}
