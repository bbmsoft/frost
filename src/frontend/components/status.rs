use yew::prelude::*;

pub struct StatusBar {
    link: ComponentLink<Self>,
    props: Props,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {}

impl Component for StatusBar {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        StatusBar { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {}
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {<div>{"STATUSBAR!"}</div>}
    }
}
