use super::common::*;
use frost::Frost;
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
    props: Props,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub location: LocationStatus,
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
    set_panic_hook();
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

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
