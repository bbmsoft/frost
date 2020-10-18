use super::super::common::ColdPhase;
use chrono::prelude::*;
use yew::prelude::*;

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub phase: ColdPhase,
}

pub struct Record {
    props: Props,
}

impl Component for Record {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Record { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let phase = &self.props.phase;

        let location = &phase.location;
        let date = format_date(&phase);
        let class = phase.record_type.to_string();
        let type_text = class.to_uppercase();
        let temp = format!("Temperature drops as low as {} °C", phase.min_temp);
        let timestamp = format_time(&phase);

        html! {
            <div class="record">
                <span class="location">{location}</span>
                <span class="date">{date}</span>
                <span class={class}>{type_text}{":"}</span>
                <span class="temperature">{temp}</span>
                <span class="time">{timestamp}</span>
            </div>
        }
    }
}

fn format_date(phase: &ColdPhase) -> String {
    if phase.start.date() == phase.end.date() {
        phase.start.format("%Y-%m-%d").to_string()
    } else {
        format!(
            "{} - {}",
            phase.start.format("%Y-%m-%d"),
            phase.end.format("%Y-%m-%d")
        )
    }
}

fn format_time(phase: &ColdPhase) -> String {
    if phase.end.day() - phase.start.day() < 2 {
        format!(
            "between {} and {}",
            phase.start.format("%H:%M"),
            phase.end.format("%H:%M")
        )
    } else {
        format!(
            "between {} and {}",
            phase.start.format("%Y-%m-%d %H:%M"),
            phase.end.format("%Y-%m-%d %H:%M")
        )
    }
}
