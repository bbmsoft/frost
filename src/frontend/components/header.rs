use super::place_picker;
use super::place_picker::PlacePicker;
use crate::frontend;
use crate::frontend::FrostApp;
use yew::prelude::*;

#[derive(Debug, Clone)]
pub struct Header {
    props: Props,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub location: Option<String>,
    pub notifications_on: bool,
    pub app_link: ComponentLink<FrostApp>,
    pub geolocation_supported: bool,
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

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let location = place_picker::Msg::PlacePicked(self.props.location.clone());
        let app_link = self.props.app_link.clone();
        let notifications_icon = if self.props.notifications_on {
            "fas fa-sync-alt"
        } else {
            "fas fa-bell-slash"
        };

        let geolocation_not_supported = !self.props.geolocation_supported;
        let get_location = self
            .props
            .app_link
            .callback(move |_| frontend::Msg::RequestDeviceLocation);
        let refresh = self.props.app_link.callback(|_| frontend::Msg::Refresh);

        html! {
            <div class="header">
                <PlacePicker state={location} app_link={app_link} />
                <button disabled={geolocation_not_supported} onclick={get_location}><i class="fas fa-map-marker-alt"></i></button>
                <div class="space"></div>
                <button disabled=true><i class={notifications_icon}></i></button>
                <button onclick={refresh}><i class="fas fa-sync-alt"></i></button>
                <img class="app-icon" src="/icon.png" alt="frost icon" />
            </div>
        }
    }
}
