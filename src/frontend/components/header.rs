use yew::prelude::*;

#[derive(Debug, Clone)]
pub struct Header {
    props: Props,
}

#[derive(Debug, PartialEq, Clone, Properties)]
pub struct Props {
    pub location: Option<String>,
}

impl Component for Header {
    type Message = ();

    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Header { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        let location = if let Some(location) = &self.props.location {
            location
        } else {
            "No Location Selected"
        };

        html! {
            <div class="header">
                <h1 class="location-header">{location}</h1>
                <img class="app-icon" src="/icon.png" alt="frost icon" />
            </div>
        }
    }
}
