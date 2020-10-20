use self::components::frost::Frost;
use self::components::status::StatusBar;
use super::common::*;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod components;
pub mod js;

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
    Refresh,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub location: LocationStatus,
    pub geolocation_supported: bool,
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
                if let &LocationStatus::LocationRetrieved(lat, lon) = &self.props.location {
                    let value = serde_json::to_string(&(lat, lon)).expect("can't fail");
                    debug!("Setting location cookie: {}", value);
                    js::set_cookie("location", &value, 30);
                }
                true
            }
            Msg::Refresh => {
                // TODO
                debug!("Refreshing weather data...");
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let get_location = self.link.callback(move |_| Msg::RequestDeviceLocation);
        let geolocation_not_supported = !self.props.geolocation_supported;
        let location = self.props.location.clone();
        let refresh = self.link.callback(|_| Msg::Refresh);
        html! {
            <div class="app">
                <Frost location={location} weather={None} />
                <StatusBar />
                <div class="controls">
                    <button disabled={geolocation_not_supported} onclick={get_location}>{"Use current location"}</button>
                    <button disabled=true>{"Select location"}</button>
                    <button onclick={refresh}>{"Refresh"}</button>
                </div>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    info!("WASM successfully loaded!");

    let geolocation_supported = js::is_geolocation_available();

    let location = if let Some(value) = js::get_cookie(LOCATION_COOKIE) {
        if let Ok((lat, lon)) = serde_json::from_str(&value) {
            LocationStatus::LocationRetrieved(lat, lon)
        } else {
            warn!("Location cookie invalid.");
            LocationStatus::WaitingForLocation
        }
    } else {
        debug!("Location cookie not set.");
        LocationStatus::WaitingForLocation
    };

    let thresholds = (5.0, 0.0);
    let value = serde_json::to_string(&thresholds).expect("can't fail");
    js::set_cookie(THRESHOLD_COOKIE, &value, 30);

    let props = Props {
        location,
        geolocation_supported,
    };

    App::<Model>::new().mount_to_body_with_props(props);

    Ok(())
}
