use super::record::*;
use crate::common::WeatherDataStatus;
use crate::common::*;
use yew::prelude::*;
use yew::virtual_dom::VNode;

pub struct Frost {
    props: Props,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub weather: Option<WeatherDataStatus>,
}

impl Component for Frost {
    type Message = ();

    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Frost { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        if let Some(WeatherDataStatus::WeatherDataRetrieved(Ok(data))) = &self.props.weather {
            let records: Vec<VNode> = data.cold_phases.iter().map(to_record).collect();
            if records.is_empty() {
                html! {
                    <div class="records">
                        <div class="record">
                            <span class="temperature">{"Looks like it's going to be warm the next few days."}</span>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div class="records">
                        { records }
                    </div>
                }
            }
        } else {
            html! {
                <div class="records" />
            }
        }
    }
}

fn to_record(phase: &ColdPhase) -> VNode {
    html! {
        <Record phase={phase} />
    }
}
