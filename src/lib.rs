use components::frost::Frost;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod components;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct Model {
    link: ComponentLink<Self>,
    pos: Option<(f32, f32)>,
    waiting_for_location: bool,
    location_error: Option<String>,
}
pub enum Msg {
    FrostCreated(ComponentLink<Frost>),
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    waiting_for_location: bool,
    location_access_failed: Option<String>,
    location: Option<(f32, f32)>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            link,
            pos: props.location,
            waiting_for_location: props.waiting_for_location,
            location_error: props.location_access_failed,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FrostCreated(frost_link) => {
                if let Some(pos) = self.pos {
                    frost_link.send_message(components::frost::Msg::GotLocation(pos))
                } else if self.waiting_for_location {
                    frost_link.send_message(components::frost::Msg::WaitingForLocation)
                } else if let Some(msg) = &self.location_error {
                    frost_link.send_message(components::frost::Msg::LocationFailed(msg.to_owned()))
                } else {
                    frost_link.send_message(components::frost::Msg::LocationFailed(
                        "unknown error occurred".to_owned(),
                    ))
                }
            }
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <Frost parent_link={self.link.clone()} />
        }
    }
}

#[wasm_bindgen(start)]
pub async fn requesting_location() {
    utils::set_panic_hook();
    let props = Props {
        waiting_for_location: true,
        location_access_failed: None,
        location: None,
    };
    App::<Model>::new().mount_to_body_with_props(props);
}

#[wasm_bindgen]
pub async fn start(lat: f32, lon: f32) {
    let props = Props {
        waiting_for_location: false,
        location_access_failed: None,
        location: Some((lat, lon)),
    };
    App::<Model>::new().mount_to_body_with_props(props);
}

#[wasm_bindgen]
pub async fn get_location_failed(code: u16) {
    let message = match code {
        1 => "User denied the request for Geolocation.",
        2 => "Location information is unavailable.",
        3 => "The request to get the user location timed out.",
        4 => "Geolocation is not supported by the browser.",
        _ => "An unknown error occurred.",
    };
    let props = Props {
        waiting_for_location: false,
        location_access_failed: Some(message.to_owned()),
        location: None,
    };
    App::<Model>::new().mount_to_body_with_props(props);
}
