use crate::common::*;
use yew::prelude::*;

pub struct StatusBar {
    // link: ComponentLink<Self>,
    props: Props,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub status: Option<Status>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    StatusUpdate(Option<Status>),
}

impl Component for StatusBar {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        StatusBar {
            // link,
            props,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StatusUpdate(update) => {
                self.props.status = update;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match &self.props.status {
            Some(Status::Info(text)) => html! {<div class="status-bar info">{text}</div>},
            Some(Status::Progress(name)) => html! {<div class="status-bar progress">{name}</div>},
            Some(Status::Warning { title, body }) => html! {
            <div class="status-bar warning">
                <div class="warning-title">{title}</div>
                <div class="warning-body">{body}</div>
            </div>},
            Some(Status::Error { title, body }) => html! {
            <div class="status-bar error">
                <div class="error-title">{title}</div>
                <div class="error-body">{body}</div>
            </div>},
            None => html! {},
        }
    }
}
