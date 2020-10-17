use super::super::{LocationStatus, WeatherDataStatus};
use super::record::*;
use chrono::prelude::*;
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
            Some(WeatherDataStatus::WeatherDataRetrieved(data)) => {
                let records = format_weather_data(&data);
                html! {
                    <div>
                        { records }
                    </div>
                }
            }
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
) -> Result<FetchTask, Box<dyn std::error::Error>> {
    let callback = move |response: Response<Result<String, anyhow::Error>>| {
        let (meta, data) = response.into_parts();
        if meta.status.is_success() {
            match data {
                Ok(data) => match data.parse() {
                    Ok(response) => {
                        Msg::WeatherUpdate(WeatherDataStatus::WeatherDataRetrieved(response))
                    }
                    Err(e) => Msg::WeatherUpdate(WeatherDataStatus::ParseError(e.to_string())),
                },
                Err(e) => Msg::WeatherUpdate(WeatherDataStatus::ParseError(e.to_string())),
            }
        } else {
            Msg::WeatherUpdate(WeatherDataStatus::FetchError(meta.status.to_string()))
        }
    };

    let uri = format!("/weather?lat={}&lon={}", lat, lon);
    let callback = link.callback(callback);

    let request = Request::get(uri).body(Nothing)?;

    Ok(FetchService::fetch(request, callback)?)
}

fn format_weather_data(response: &brtsky::Response) -> Vec<VNode> {
    let now: DateTime<Utc> = Utc::now();

    let mut out = Vec::new();

    for data in response
        .weather_data_sets()
        .filter(|w| w.weather_data().timestamp > now)
    {
        let temp = data.weather_data().temperature;
        let timestamp = data.weather_data().timestamp;
        let location = &data.source().station_name;

        if data.weather_data().temperature <= 0.0 {
            let record = html! {
                <Record record_type={Type::Danger} temp={temp} timestamp={timestamp} location={location} />
            };
            out.push(record);
        } else if data.weather_data().temperature < 5.0 {
            let record = html! {
                <Record record_type={Type::Warning} temp={temp} timestamp={timestamp} location={location} />
            };
            out.push(record);
        }
    }

    out
}
