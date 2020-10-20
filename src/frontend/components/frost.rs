use super::record::*;
use crate::common::WeatherDataStatus;
use crate::common::*;
use yew::prelude::*;
use yew::virtual_dom::VNode;

pub struct Frost {
    link: ComponentLink<Self>,
    props: Props,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub weather: Option<WeatherDataStatus>,
}

pub enum Msg {
    WeatherUpdate(WeatherDataStatus),
}

impl Component for Frost {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Frost { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::WeatherUpdate(weather) => self.props.weather = Some(weather),
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        match &self.props.weather {
            Some(WeatherDataStatus::WeatherDataRetrieved(data)) => match data {
                Ok(data) => {
                    let records: Vec<VNode> = data.iter().map(to_record).collect();
                    if records.is_empty() {
                        html! {
                            <div class="records">
                                {"Looks like it's going to be warm the next few days."}
                            </div>
                        }
                    } else {
                        html! {
                            <div class="records">
                                { records }
                            </div>
                        }
                    }
                }
                Err(e) => html! {
                    <div class="error">
                        <span class="error-header">{"there was an error getting the current weather data:"}</span>
                        <span class="error-body">{e}</span>
                    </div>
                },
            },
            Some(WeatherDataStatus::WaitingForWeatherData) => html! {
                <div class="records">
                    {"Fetching weather data..."}
                </div>
            },
            Some(WeatherDataStatus::FetchError(e)) => html! {
                <div class="records">
                    {"Error fetching weather data: "} {e}
                </div>
            },
            Some(WeatherDataStatus::ParseError(e)) => html! {
                <div class="records">
                    {"Error parsing weather data: "} {e}
                </div>
            },
            None => html! {},
        }
    }
}

fn to_record(phase: &ColdPhase) -> VNode {
    html! {
        <Record phase={phase} />
    }
}
