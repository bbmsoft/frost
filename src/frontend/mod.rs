use self::frost::Frost;
use super::common::*;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub mod frost;
pub mod js;
pub mod record;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct Model {
    link: ComponentLink<Model>,
    props: Props,
    on_location_success: Closure<dyn Fn(f32, f32)>,
    on_location_error: Closure<dyn Fn(u16, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    RequestDeviceLocation,
    LocationUpdate(LocationStatus),
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub location: LocationStatus,
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link_success = link.clone();
        let link_error = link.clone();
        let on_location_success = Closure::new(move |lat, lon| {
            link_success.send_message(Msg::LocationUpdate(LocationStatus::LocationRetrieved(
                lat, lon,
            )))
        });
        let on_location_error = Closure::new(move |code, msg| {
            link_error.send_message(Msg::LocationUpdate(LocationStatus::LocationFailed(
                code, msg,
            )))
        });

        Model {
            link,
            props,
            on_location_success,
            on_location_error,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestDeviceLocation => {
                js::get_location(&self.on_location_success, &self.on_location_error);
                false
            }
            Msg::LocationUpdate(status) => {
                self.props.location = status;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let click_callback = self.link.callback(move |_| Msg::RequestDeviceLocation);
        let geolocation_not_supported = self.props.location == LocationStatus::LocationNotSupported;
        let location = self.props.location.clone();
        html! {
            <div class="app">
                <Frost location={location} weather={None} />
                <div class="controls">
                    <button disabled={geolocation_not_supported} onclick={click_callback}>{"Use current location"}</button>
                    <button disabled=true>{"Select location"}</button>
                </div>
            </div>
        }
    }
}

#[wasm_bindgen]
pub fn start(geolocation_supported: bool) {
    set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());

    let location = if !geolocation_supported {
        LocationStatus::LocationNotSupported
    } else {
        // TODO try to get location from cookies
        LocationStatus::WaitingForLocation
    };

    let props = Props { location };
    App::<Model>::new().mount_to_body_with_props(props);
}

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
