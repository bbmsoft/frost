use chrono::prelude::*;
use std::fmt;
use yew::prelude::*;
#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub record_type: Type,
    pub temp: f32,
    pub timestamp: DateTime<FixedOffset>,
    pub location: String,
}
#[derive(Debug, Clone)]
pub enum Type {
    Warning,
    Danger,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Warning => write!(f, "warning"),
            Type::Danger => write!(f, "danger"),
        }
    }
}

pub struct Record(Props);

impl Component for Record {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Record(props)
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let class = self.0.record_type.to_string();
        let text = class.to_uppercase();
        let temp = self.0.temp;
        let timestamp = self
            .0
            .timestamp
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M")
            .to_string();
        let location = &self.0.location;

        html! {
            <div class="record"><span class={class}>{text}{": "}</span> {temp}{" °C predicted for "}{timestamp}{" in "}{location}</div>
        }
    }
}
