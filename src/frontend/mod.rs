use self::frost::Frost;
use super::common::*;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub mod frost;
pub mod record;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct Model {
    link: ComponentLink<Model>,
    props: Props,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    RequestDeviceLocation(Option<js_sys::Function>),
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub location: LocationStatus,
    pub get_current_location: Option<js_sys::Function>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestDeviceLocation(callback) => {
                if let Some(callback) = callback {
                    let this = JsValue::null();
                    if let Err(e) = callback.call0(&this) {
                        error!("{:?}", e);
                    }
                } else {
                    warn!("Geolocation not available!");
                }
            }
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let get_current_location = &self.props.get_current_location;
        let click_callback = wrap_js_sys_function(get_current_location, self.link.clone());
        html! {
            <div class="app">
                <Frost location={self.props.location.clone()} weather={None} />
                <div class="controls">
                    <button disabled={get_current_location.is_none()} onclick={click_callback}>{"Use current location"}</button>
                    <button disabled=true>{"Select location"}</button>
                </div>
            </div>
        }
    }
}

#[wasm_bindgen]
pub fn start(get_current_location: Option<js_sys::Function>) {
    set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());
    let props = Props {
        location: LocationStatus::WaitingForLocation,
        get_current_location,
    };
    App::<Model>::new().mount_to_body_with_props(props);
}

#[wasm_bindgen]
pub fn set_location(lat: f32, lon: f32, get_current_location: Option<js_sys::Function>) {
    let props = Props {
        location: LocationStatus::LocationRetrieved(lat, lon),
        get_current_location,
    };
    App::<Model>::new().mount_to_body_with_props(props);
}

#[wasm_bindgen]
pub fn get_location_failed(code: u16, msg: String, get_current_location: Option<js_sys::Function>) {
    let props = Props {
        location: LocationStatus::LocationFailed(code, msg),
        get_current_location,
    };
    App::<Model>::new().mount_to_body_with_props(props);
}

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

fn wrap_js_sys_function(
    f: &Option<js_sys::Function>,
    link: ComponentLink<Model>,
) -> Callback<MouseEvent> {
    let msg = Msg::RequestDeviceLocation(f.clone());
    link.callback(move |_| msg.clone())
}
