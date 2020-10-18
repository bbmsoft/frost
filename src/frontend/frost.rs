use super::super::common::*;
use super::super::common::{BackendError, LocationStatus, WeatherDataStatus};
use super::record::*;
use yew::format::Nothing;
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::virtual_dom::VNode;

pub struct Frost {
    link: ComponentLink<Self>,
    props: Props,
    fetch_task: Option<FetchTask>,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub location: LocationStatus,
    pub weather: Option<WeatherDataStatus>,
}

impl Frost {
    fn new(link: ComponentLink<Self>, props: Props) -> Self {
        Frost {
            link,
            props,
            fetch_task: None,
        }
    }
}

pub enum Msg {
    WeatherUpdate(WeatherDataStatus),
}

impl Component for Frost {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut frost = Frost::new(link.clone(), props.clone());
        check_for_weather_update(&mut frost, props.location, link);
        frost
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::WeatherUpdate(weather) => self.props.weather = Some(weather),
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let old_location = self.props.location.clone();
        self.props = props;

        if self.props.location != old_location {
            check_for_weather_update(self, self.props.location.clone(), self.link.clone());
        }

        true
    }

    fn view(&self) -> Html {
        match &self.props.weather {
            Some(WeatherDataStatus::WeatherDataRetrieved(data)) => match data {
                Ok(data) => {
                    let records: Vec<VNode> = data.iter().map(to_record).collect();
                    html! {
                        <div class="records">
                            { records }
                        </div>
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
                <div>
                    {"Fetching weather data..."}
                </div>
            },
            Some(WeatherDataStatus::FetchError(e)) => html! {
                <div>
                    {"Error fetching weather data: "} {e}
                </div>
            },
            Some(WeatherDataStatus::ParseError(e)) => html! {
                <div>
                    {"Error parsing weather data: "} {e}
                </div>
            },
            None => match &self.props.location {
                LocationStatus::WaitingForLocation | LocationStatus::LocationRetrieved(_, _) => {
                    html! {
                        <div>
                            {"Waiting for access to device location..."}
                        </div>
                    }
                }
                LocationStatus::LocationFailed(_code, msg) => html! {
                    <div>
                        {"Device location could not be determined: "} {msg}
                    </div>
                },
                LocationStatus::LocationDisabled => html! {
                    <div>
                        {"TODO: enter location manually"}
                    </div>
                },
            },
        }
    }
}

fn check_for_weather_update(
    frost: &mut Frost,
    location: LocationStatus,
    link: ComponentLink<Frost>,
) {
    if let LocationStatus::LocationRetrieved(lat, lon) = location {
        match fetch_weather_data(lat, lon, link.clone()) {
            Ok(fetch_task) => {
                // prevent fetch task from being dropped / cancelled
                frost.fetch_task = Some(fetch_task);
                link.send_message(Msg::WeatherUpdate(WeatherDataStatus::WaitingForWeatherData))
            }
            Err(e) => link.send_message(Msg::WeatherUpdate(WeatherDataStatus::FetchError(
                e.to_string(),
            ))),
        }
    }
}

fn fetch_weather_data(
    lat: f32,
    lon: f32,
    link: ComponentLink<Frost>,
) -> Result<FetchTask, BackendError> {
    let callback = move |response: Response<Result<String, anyhow::Error>>| {
        let data = response.body();
        let status = match data {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(response) => WeatherDataStatus::WeatherDataRetrieved(response),
                Err(e) => WeatherDataStatus::ParseError(e.to_string()),
            },
            Err(e) => WeatherDataStatus::FetchError(e.to_string()),
        };
        Msg::WeatherUpdate(status)
    };

    let uri = format!(
        "/weather?lat={}&lon={}&warning_threshold=5.0&danger_threshold=0.0",
        lat, lon
    );
    let callback = link.callback(callback);

    let request = Request::get(uri).body(Nothing)?;
    let fetch_task = convert_err(FetchService::fetch(request, callback));
    Ok(fetch_task?)
}

fn to_record(phase: &ColdPhase) -> VNode {
    html! {
        <Record phase={phase} />
    }
}

fn convert_err(
    result: Result<FetchTask, anyhow::Error>,
) -> Result<FetchTask, Box<dyn std::error::Error>> {
    Ok(result?)
}

impl From<http::Error> for BackendError {
    fn from(e: http::Error) -> Self {
        BackendError::NetworkError(e.to_string())
    }
}
