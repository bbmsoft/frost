use components::frost::Frost;
use std::fmt;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod components;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug, Clone, PartialEq)]
pub enum RecordType {
    Warning,
    Danger,
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordType::Warning => write!(f, "warning"),
            RecordType::Danger => write!(f, "danger"),
        }
    }
}

pub struct Model {
    props: Props,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub location: LocationStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocationStatus {
    WaitingForLocation,
    LocationFailed(u16, String),
    LocationRetrieved(f32, f32),
    LocationDisabled,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WeatherDataStatus {
    WaitingForWeatherData,
    FetchError(String),
    ParseError(String),
    WeatherDataRetrieved(brtsky::Response),
}

impl Component for Model {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Model { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <Frost location={self.props.location.clone()} weather={None} />
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
    let props = Props {
        location: LocationStatus::WaitingForLocation,
    };
    App::<Model>::new().mount_to_body_with_props(props);
}

#[wasm_bindgen]
pub async fn set_location(lat: f32, lon: f32) {
    let props = Props {
        location: LocationStatus::LocationRetrieved(lat, lon),
    };
    App::<Model>::new().mount_to_body_with_props(props);
}

#[wasm_bindgen]
pub async fn get_location_failed(code: u16, msg: String) {
    let props = Props {
        location: LocationStatus::LocationFailed(code, msg),
    };
    App::<Model>::new().mount_to_body_with_props(props);
}
