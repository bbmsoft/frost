use crate::common::ColdPhase;
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

        // let location = &phase.location;
        let date_start = phase.start.format("%Y-%m-%d");
        let class = phase.record_type.to_string();
        let type_text = class.to_uppercase();
        let explanation = match phase.record_type {
            crate::common::RecordType::Warning => format!("< {} °C", phase.warning_threshold),
            crate::common::RecordType::Danger => format!("< {} °C", phase.danger_threshold),
        };
        let temp = format!("Temperature drops as low as {} °C", phase.min_temp);
        let timestamp = format_time(&phase);

        let date2 = if phase.start.date() != phase.end.date() {
            let date_end = phase.end.format("%Y-%m-%d");
            html! {<span class="date2">{"- "}{date_end}</span>}
        } else {
            html! {}
        };

        html! {
            <div class="record">
                // <span class="location">{location}</span>
                <span class="date">{date_start}</span>
                {date2}
                <span class={class}>{type_text}{": "}{explanation}</span>
                <span class="temperature">{temp}</span>
                <span class="time">{timestamp}</span>
            </div>
        }
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
