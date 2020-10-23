use crate::common::Place;
use crate::frontend;
use crate::frontend::js;
use crate::frontend::FrostApp;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Debug)]
pub struct PlacePicker {
    link: ComponentLink<Self>,
    props: Props,
    on_place_selected: Closure<dyn Fn(String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    PickPlace,
    PlacePicked(Option<Place>),
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub state: Msg,
    pub app_link: ComponentLink<FrostApp>,
}

impl Component for PlacePicker {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link_place = props.app_link.clone();
        let on_place_selected = Closure::new(move |place_json: String| match serde_json::from_str(
            &place_json,
        ) {
            Ok(place) => link_place.send_message(frontend::Msg::PlaceSelected(place)),
            Err(e) => error!("Error parsing place: {}\nJson was:\n{}", e, place_json),
        });

        PlacePicker {
            link,
            props,
            on_place_selected,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.props.state = msg;
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        match self.props.state {
            Msg::PickPlace => html! {
                html! {
                    <input type="text" id="place-picker" />
                }
            },
            Msg::PlacePicked(_) => {
                let callback = self.link.callback(|_| Msg::PickPlace);
                html! {
                    <button onclick={callback}>{"Select Location"}</button>
                }
            }
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.props.state == Msg::PickPlace {
            js::init_autocomplete("place-picker", &self.on_place_selected);
        }
    }
}
