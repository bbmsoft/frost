use crate::common::*;
use crate::frontend;
use crate::frontend::js;
use crate::frontend::FrostApp;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use yew::prelude::*;

#[derive(Debug)]
pub struct PlacePicker {
    link: ComponentLink<Self>,
    props: Props,
    on_place_selected: Closure<dyn Fn(String)>,
    input_ref: NodeRef,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub location: Option<String>,
    pub app_link: ComponentLink<FrostApp>,
}

impl Component for PlacePicker {
    type Message = PlaceStatus;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link_place = props.app_link.clone();
        let on_place_selected = Closure::new(move |place_json: String| match serde_json::from_str(
            &place_json,
        ) {
            Ok(place) => link_place.send_message(frontend::Msg::PlaceUpdate(
                PlaceStatus::PlacePicked(Some(place)),
            )),
            Err(e) => error!("Error parsing place: {}\nJson was:\n{}", e, place_json),
        });

        PlacePicker {
            link,
            props,
            on_place_selected,
            input_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            PlaceStatus::PickPlace | PlaceStatus::PlacePicked(None) => self.props.location = None,
            PlaceStatus::PlacePicked(Some(place)) => self.props.location = Some(place.name.clone()),
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        match &self.props.location {
            None => html! {
                html! {
                    <input type="text" id="place-picker" ref=self.input_ref.clone() />
                }
            },
            Some(place) => {
                let callback = self.link.callback(|_| PlaceStatus::PickPlace);
                html! {
                    <button class="location-header" onclick={callback}>{place}</button>
                }
            }
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.props.location.is_none() {
            js::init_autocomplete("place-picker", &self.on_place_selected);
            if let Some(input) = self.input_ref.cast::<HtmlElement>() {
                if let Err(e) = input.focus() {
                    error!("{:#?}", e);
                }
            }
        }
    }
}
