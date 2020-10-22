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

#[wasm_bindgen(module = "/js/notifications.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn are_notifications_supported_js() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    fn request_notification_permission_js(
        on_result: &Closure<dyn Fn(JsValue)>,
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

#[wasm_bindgen(module = "/js/web-storage.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn store_js(key: &str, value: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn get_stored_js(key: &str) -> Result<Option<String>, JsValue>;
}

// apparently this is not actually necessary
// compiling and running code that calls the js functions without an unsafe block works just fine
// however VS code with rust-analyzer shows an error, which is very annoying in development,
// hence this workaround

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
pub fn request_notification_permission(on_result: &Closure<dyn Fn(JsValue)>) {
    unsafe {
        if let Err(e) = request_notification_permission_js(on_result) {
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

#[allow(unused_unsafe)]
pub fn store(key: &str, value: &str) {
    unsafe {
        if let Err(e) = store_js(key, value) {
            error!("Error writing to web storage: {:?}", e);
        }
    }
}

#[allow(unused_unsafe)]
pub fn get_stored(key: &str) -> Option<String> {
    unsafe {
        match get_stored_js(key) {
            Ok(Some(value)) => Some(value),
            Ok(None) => None,
            Err(e) => {
                error!("Error getting value from web storage: {:?}", e);
                None
            }
        }
    }
}
