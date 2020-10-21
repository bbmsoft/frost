use chrono::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/location.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn is_geolocation_supported_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    fn get_location_js(
        on_success: &Closure<dyn Fn(f32, f32)>,
        on_error: &Closure<dyn Fn(u16, String)>,
    ) -> Result<(), JsValue>;
}

#[wasm_bindgen(module = "/js/cookies.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn set_cookie_js(cookie: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn get_cookies_js() -> Result<String, JsValue>;
}

#[wasm_bindgen(module = "/js/notifications.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn are_notifications_supported_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    fn request_notification_permission_js(
        on_granted: &Closure<dyn Fn()>,
        on_denied: &Closure<dyn Fn()>,
        on_error: &Closure<dyn Fn()>,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    fn show_notification_js(
        title: &str,
        text: &str,
        icon: Option<&str>,
        tag: Option<&str>,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    fn show_notification_with_callbacks_js(
        title: &str,
        text: &str,
        icon: Option<&str>,
        tag: Option<&str>,
        on_click: &Closure<dyn Fn()>,
        on_error: &Closure<dyn Fn()>,
    ) -> Result<(), JsValue>;
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
pub fn is_geolocation_supported() -> bool {
    unsafe { is_geolocation_supported_js().unwrap_or(false) }
}

#[allow(unused_unsafe)]
pub fn are_notifications_supported() -> bool {
    unsafe { are_notifications_supported_js().unwrap_or(false) }
}

#[allow(unused_unsafe)]
pub fn request_notification_permission(
    on_granted: &Closure<dyn Fn()>,
    on_denied: &Closure<dyn Fn()>,
    on_error: &Closure<dyn Fn()>,
) {
    unsafe {
        if let Err(e) = request_notification_permission_js(on_granted, on_denied, on_error) {
            error!("Error getting location: {:?}", e);
        }
    }
}

#[allow(unused_unsafe)]
pub fn show_notification(title: &str, text: &str, icon: Option<&str>, tag: Option<&str>) {
    unsafe {
        if let Err(e) = show_notification_js(title, text, icon, tag) {
            error!("Error getting location: {:?}", e);
        }
    }
}

#[allow(unused_unsafe)]
pub fn show_notification_with_callbacks(
    title: &str,
    text: &str,
    icon: Option<&str>,
    tag: Option<&str>,
    on_click: &Closure<dyn Fn()>,
    on_error: &Closure<dyn Fn()>,
) {
    unsafe {
        if let Err(e) =
            show_notification_with_callbacks_js(title, text, icon, tag, on_click, on_error)
        {
            error!("Error getting location: {:?}", e);
        }
    }
}

fn format_cookie(key: &str, value: &str, days_valid: usize) -> String {
    let expiry_date = Utc::now() + chrono::Duration::days(days_valid as i64);
    let expiry_date = expiry_date.to_rfc3339();
    format!(
        "{}={};expires={};secure;samesite=Strict",
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
