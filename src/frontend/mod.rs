use self::components::frost::Frost;
use self::components::status::StatusBar;
use super::common::*;
use wasm_bindgen::prelude::*;
use yew::format::Nothing;
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod components;
pub mod js;

pub struct FrostApp {
    link: ComponentLink<FrostApp>,
    props: Props,
    on_location_success: Closure<dyn Fn(f32, f32)>,
    on_location_error: Closure<dyn Fn(u16, String)>,
    fetch_task: Option<FetchTask>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    RequestDeviceLocation,
    LocationUpdate(LocationStatus),
    WeatherUpdate(WeatherDataStatus),
    Refresh,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub location: LocationStatus,
    pub weather: WeatherDataStatus,
    pub status: Option<Status>,
    pub geolocation_supported: bool,
}

impl Component for FrostApp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link_success = link.clone();
        let link_error = link.clone();
        let on_location_success = Closure::new(move |lat, lon| {
            link_success.send_message(Msg::LocationUpdate(LocationStatus::LocationRetrieved(
                lat, lon,
            )))
        });
        let on_location_error = Closure::new(move |code, msg| {
            link_error.send_message(Msg::LocationUpdate(LocationStatus::LocationFailed(
                code, msg,
            )))
        });

        let mut app = FrostApp {
            link: link.clone(),
            props: props.clone(),
            on_location_success,
            on_location_error,
            fetch_task: None,
        };

        check_for_weather_update(&mut app, props.location, link);

        app
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestDeviceLocation => {
                js::get_location(&self.on_location_success, &self.on_location_error);
                false
            }
            Msg::LocationUpdate(status) => {
                match &status {
                    LocationStatus::WaitingForLocation => {
                        self.props.status = Some(Status::Progress(
                            "Waitig for location service...".to_owned(),
                        ));
                    }
                    LocationStatus::LocationFailed(_code, msg) => {
                        self.props.status = Some(Status::Error {
                            title: "Error getting location:".to_owned(),
                            body: msg.to_owned(),
                        });
                    }
                    LocationStatus::LocationRetrieved(lat, lon) => {
                        if let LocationStatus::WaitingForLocation = self.props.location {
                            self.props.status = None;
                        }
                        let value = serde_json::to_string(&(lat, lon)).expect("can't fail");
                        debug!("Setting location cookie: {}", value);
                        js::set_cookie("location", &value, 30);
                    }
                    LocationStatus::LocationDisabled => {}
                }

                self.props.location = status;

                true
            }
            Msg::WeatherUpdate(data) => {
                match &data {
                    WeatherDataStatus::WaitingForWeatherData => {
                        self.props.status =
                            Some(Status::Progress("Fetching weather data...".to_owned()));
                    }
                    WeatherDataStatus::FetchError(msg) => {
                        self.props.status = Some(Status::Error {
                            title: "Error fetching weather data:".to_owned(),
                            body: msg.to_owned(),
                        });
                    }
                    WeatherDataStatus::ParseError(msg) => {
                        self.props.status = Some(Status::Error {
                            title: "Received invalid weather data:".to_owned(),
                            body: msg.to_owned(),
                        });
                    }
                    WeatherDataStatus::WeatherDataRetrieved(_) => {
                        if let WeatherDataStatus::WaitingForWeatherData = self.props.weather {
                            self.props.status = None;
                        }
                    }
                }

                self.props.weather = data;
                true
            }
            Msg::Refresh => {
                // TODO
                debug!("Refreshing weather data...");
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let get_location = self.link.callback(move |_| Msg::RequestDeviceLocation);
        let geolocation_not_supported = !self.props.geolocation_supported;
        let refresh = self.link.callback(|_| Msg::Refresh);
        let weather = self.props.weather.clone();
        let status = self.props.status.clone();
        html! {
            <div class="app">
                <Frost weather={weather} />
                <StatusBar status={status} />
                <div class="controls">
                    <button disabled={geolocation_not_supported} onclick={get_location}>{"Use current location"}</button>
                    <button disabled=true>{"Select location"}</button>
                    <button onclick={refresh}>{"Refresh"}</button>
                </div>
            </div>
        }
    }
}

fn check_for_weather_update(
    app: &mut FrostApp,
    location: LocationStatus,
    link: ComponentLink<FrostApp>,
) {
    if let LocationStatus::LocationRetrieved(lat, lon) = location {
        match fetch_weather_data(lat, lon, link.clone()) {
            Ok(fetch_task) => {
                // prevent fetch task from being dropped / cancelled
                app.fetch_task = Some(fetch_task);
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
    link: ComponentLink<FrostApp>,
) -> Result<FetchTask, BackendError> {
    let callback = move |response: Response<Result<String, anyhow::Error>>| {
        let data = response.body();
        let status = match data {
            Ok(data) => {
                debug!("Response from backend: {}", data);
                match serde_json::from_str(&data) {
                    Ok(response) => WeatherDataStatus::WeatherDataRetrieved(response),
                    Err(e) => WeatherDataStatus::ParseError(e.to_string()),
                }
            }
            Err(e) => WeatherDataStatus::FetchError(e.to_string()),
        };
        Msg::WeatherUpdate(status)
    };

    let callback = link.callback(callback);

    debug!("Requesting weather data from backend...");
    let request = Request::get("/weather").body(Nothing)?;
    let fetch_task = convert_err(FetchService::fetch(request, callback));
    Ok(fetch_task?)
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

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    info!("WASM successfully loaded!");

    let geolocation_supported = js::is_geolocation_available();

    let location = if let Some(value) = js::get_cookie(LOCATION_COOKIE) {
        if let Ok((lat, lon)) = serde_json::from_str(&value) {
            LocationStatus::LocationRetrieved(lat, lon)
        } else {
            warn!("Location cookie invalid.");
            LocationStatus::WaitingForLocation
        }
    } else {
        debug!("Location cookie not set.");
        LocationStatus::WaitingForLocation
    };

    let thresholds = (5.0, 0.0);
    let value = serde_json::to_string(&thresholds).expect("can't fail");
    js::set_cookie(THRESHOLD_COOKIE, &value, 30);
    let weather = WeatherDataStatus::WaitingForWeatherData;
    let status = None;

    let props = Props {
        location,
        weather,
        status,
        geolocation_supported,
    };

    App::<FrostApp>::new().mount_to_body_with_props(props);

    Ok(())
}
