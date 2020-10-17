use super::record::*;
use chrono::prelude::*;
use yew::format::Nothing;
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::virtual_dom::VNode;

pub struct Frost {
    link: ComponentLink<Self>,
    waiting_for_location: bool,
    pos: Option<(f32, f32)>,
    weather_data: Option<brtsky::Response>,
    location_error: Option<String>,
    fetch_error: Option<String>,
    parse_error: Option<String>,
    fetch_task: Option<FetchTask>,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    pub parent_link: ComponentLink<super::super::Model>,
}

impl Frost {
    fn new(link: ComponentLink<Self>) -> Self {
        Frost {
            link,
            waiting_for_location: false,
            pos: None,
            weather_data: None,
            location_error: None,
            fetch_error: None,
            parse_error: None,
            fetch_task: None,
        }
    }
}

pub enum Msg {
    WaitingForLocation,
    LocationFailed(String),
    GotLocation((f32, f32)),
    WeatherDataArrived(brtsky::Response),
    WeatherDataFetchError(String),
    WeatherDataParseError(String),
}

impl Component for Frost {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        props
            .parent_link
            .send_message(super::super::Msg::FrostCreated(link.clone()));
        Frost::new(link)
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::WaitingForLocation => {
                self.pos = None;
                self.waiting_for_location = true;
                self.weather_data = None;
                self.location_error = None;
                self.fetch_error = None;
                self.parse_error = None;
                self.fetch_task = None;
            }
            Msg::LocationFailed(msg) => {
                self.pos = None;
                self.waiting_for_location = false;
                self.location_error = Some(msg);
                self.weather_data = None;
                self.fetch_error = None;
                self.parse_error = None;
                self.fetch_task = None;
            }
            Msg::GotLocation((lat, lon)) => {
                self.pos = Some((lat, lon));
                self.waiting_for_location = false;
                self.weather_data = None;
                self.location_error = None;
                self.fetch_error = None;
                self.parse_error = None;
                match fetch_weather_data(lat, lon, self.link.clone()) {
                    Ok(fetch_tak) => {
                        self.fetch_task = Some(fetch_tak);
                    }
                    Err(e) => self
                        .link
                        .send_message(Msg::WeatherDataFetchError(e.to_string())),
                }
            }
            Msg::WeatherDataArrived(weather_data) => {
                self.pos = None;
                self.waiting_for_location = false;
                self.weather_data = Some(weather_data);
                self.location_error = None;
                self.fetch_error = None;
                self.parse_error = None;
                self.fetch_task = None;
            }
            Msg::WeatherDataFetchError(e) => {
                self.pos = None;
                self.waiting_for_location = false;
                self.weather_data = None;
                self.location_error = None;
                self.fetch_error = Some(e);
                self.parse_error = None;
                self.fetch_task = None;
            }
            Msg::WeatherDataParseError(e) => {
                self.pos = None;
                self.waiting_for_location = false;
                self.weather_data = None;
                self.location_error = None;
                self.fetch_error = None;
                self.parse_error = Some(e);
                self.fetch_task = None;
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if let Some(_) = self.pos {
            html! {
                <div>
                    {"Fetching weather data..."}
                </div>
            }
        } else if let Some(e) = &self.fetch_error {
            html! {
                <div>
                    {"Error fetching weather data: "} {e}
                </div>
            }
        } else if let Some(e) = &self.parse_error {
            html! {
                <div>
                    {"Error parsing weather data: "} {e}
                </div>
            }
        } else if let Some(data) = &self.weather_data {
            let records = format_weather_data(data);
            html! {
                <div>
                    { records }
                </div>
            }
        } else if self.waiting_for_location {
            html! {
                <div>
                    {"Waiting for access to device location..."}
                </div>
            }
        } else if let Some(msg) = &self.location_error {
            html! {
                <div>
                    {"Device location could not be determined: "} {msg}
                </div>
            }
        } else {
            html! {
                <div>
                    {"Device location could not be determined."}
                </div>
            }
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
                    Ok(response) => Msg::WeatherDataArrived(response),
                    Err(e) => Msg::WeatherDataParseError(e.to_string()),
                },
                Err(e) => Msg::WeatherDataParseError(e.to_string()),
            }
        } else {
            Msg::WeatherDataFetchError(meta.status.as_str().to_owned())
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
