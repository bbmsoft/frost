use self::components::frost::Frost;
use self::components::header::Header;
use self::components::status::StatusBar;
use super::common::*;
use std::env;
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
    on_notification_permission: Closure<dyn Fn(JsValue)>,
    fetch_task: Option<FetchTask>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    RequestDeviceLocation,
    LocationUpdate(LocationStatus),
    WeatherUpdate(WeatherDataStatus),
    NotificationPermissionUpdate(NotificationPermissionStatus),
    Refresh,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub location: LocationStatus,
    pub weather: WeatherDataStatus,
    pub status: Option<Status>,
    pub notification_permission: NotificationPermissionStatus,
    pub geolocation_supported: bool,
    pub notifications_supported: bool,
    pub location_name: Option<String>,
}

impl Component for FrostApp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let link_success = link.clone();
        let link_error = link.clone();
        let link_notification = link.clone();
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
        let on_notification_permission = Closure::new(move |permission: JsValue| {
            let permission = permission.into();
            link_notification.send_message(Msg::NotificationPermissionUpdate(permission));
        });

        let mut app = FrostApp {
            link: link.clone(),
            props: props.clone(),
            on_location_success,
            on_location_error,
            on_notification_permission,
            fetch_task: None,
        };

        js::request_notification_permission(&app.on_notification_permission);
        app.check_for_weather_update();

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
                            "Waiting for location service...".to_owned(),
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
                        self.check_for_weather_update();
                    }
                    LocationStatus::LocationNotSet => {}
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
                    WeatherDataStatus::WeatherDataRetrieved(data) => {
                        if let WeatherDataStatus::WaitingForWeatherData = self.props.weather {
                            self.props.status = None;
                        }
                        if let &Ok(data) = &data {
                            self.try_send_weather_notification(data);
                            self.props.location_name = data.location.clone();
                        }
                    }
                }

                self.props.weather = data;

                true
            }
            Msg::Refresh => {
                self.check_for_weather_update();
                false
            }
            Msg::NotificationPermissionUpdate(notification_permission) => {
                self.props.notification_permission = notification_permission;
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
        let location = self.props.location_name.clone();
        html! {
            <div class="app">
                <Header location={location} />
                <Frost weather={weather} />
                <div class="footer">
                    <StatusBar status={status} />
                    <div class="controls">
                        <button disabled={geolocation_not_supported} onclick={get_location}><i class="fas fa-map-marker-alt"></i>{" Location"}</button>
                        <button disabled=true>{"Select Location"}</button>
                        <button onclick={refresh}>{"Refresh"}</button>
                    </div>
                </div>
            </div>
        }
    }
}

impl FrostApp {
    fn try_send_weather_notification(&self, data: &BackendResponse) {
        if data.cold_phases.is_empty() {
            return;
        }

        if self.props.notification_permission == NotificationPermissionStatus::Granted {
            self.send_weather_notification(&data.cold_phases);
        }
    }

    fn send_weather_notification(&self, data: &Vec<ColdPhase>) {
        let record_type = if data
            .iter()
            .find(|p| p.record_type == RecordType::Danger)
            .is_some()
        {
            RecordType::Danger
        } else {
            RecordType::Warning
        };
        let temp_min = data
            .iter()
            .map(|p| p.min_temp)
            .fold(9000f32, |a, b| a.min(b));
        let titel = record_type.to_string().to_uppercase();
        let text = format!("Temperatures as low as {} °C predicted.", temp_min);
        js::show_notification(&titel, &text, Some("/icon.png"), Some("frost"));
    }

    fn check_for_weather_update(self: &mut FrostApp) {
        if let LocationStatus::LocationRetrieved(_, _) = self.props.location {
            match self.fetch_weather_data() {
                Ok(fetch_task) => {
                    // prevent fetch task from being dropped / cancelled
                    self.fetch_task = Some(fetch_task);
                    self.link
                        .send_message(Msg::WeatherUpdate(WeatherDataStatus::WaitingForWeatherData))
                }
                Err(e) => {
                    self.link
                        .send_message(Msg::WeatherUpdate(WeatherDataStatus::FetchError(
                            e.to_string(),
                        )))
                }
            }
        }
    }

    fn fetch_weather_data(&self) -> Result<FetchTask, BackendError> {
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

        let callback = self.link.callback(callback);

        debug!("Requesting weather data from backend...");
        let request = Request::get("/weather").body(Nothing)?;
        let fetch_task = convert_err(FetchService::fetch(request, callback));
        Ok(fetch_task?)
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

impl From<JsValue> for NotificationPermissionStatus {
    fn from(value: JsValue) -> Self {
        if let Some(str) = value.as_string().as_deref() {
            match str {
                "granted" => NotificationPermissionStatus::Granted,
                "denied" => NotificationPermissionStatus::Denied,
                "default" => NotificationPermissionStatus::Default,
                _ => NotificationPermissionStatus::Unsupported,
            }
        } else {
            NotificationPermissionStatus::Unsupported
        }
    }
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    info!("WASM successfully loaded!");

    let geolocation_supported = js::is_geolocation_supported();
    let notifications_supported = js::are_notifications_supported();

    let location = if let Some(value) = js::get_cookie(LOCATION_COOKIE) {
        if let Ok((lat, lon)) = serde_json::from_str(&value) {
            debug!("Got location from local cookie.");
            LocationStatus::LocationRetrieved(lat, lon)
        } else {
            warn!("Location cookie invalid.");
            LocationStatus::LocationNotSet
        }
    } else {
        debug!("Location cookie not set.");
        LocationStatus::LocationNotSet
    };

    let warning_threshold = temp_from_env("FROST_WARNING_THRESHOLD", 5.0);
    let danger_threshold = temp_from_env("FROST_DANGER_THRESHOLD", 0.0);
    let thresholds = (warning_threshold, danger_threshold);

    let value = serde_json::to_string(&thresholds).expect("can't fail");
    js::set_cookie(THRESHOLD_COOKIE, &value, 30);
    let weather = WeatherDataStatus::WaitingForWeatherData;
    let notification_permission = NotificationPermissionStatus::Default;
    let status = match location {
        LocationStatus::LocationRetrieved(_, _) => {
            Some(Status::Info("Fetching weather data...".to_owned()))
        }
        _ => Some(Status::Info("Location not set.".to_owned())),
    };

    let props = Props {
        location,
        weather,
        status,
        notification_permission,
        geolocation_supported,
        notifications_supported,
        location_name: None,
    };

    App::<FrostApp>::new().mount_to_body_with_props(props);

    Ok(())
}

fn temp_from_env(key: &str, default: f32) -> f32 {
    env::var(key).map_or_else(
        |e| {
            warn!("Could not get value of {}: {}", key, e);
            default
        },
        |v| {
            v.parse().unwrap_or_else(|e| {
                warn!("Error parsing {}: {}", v, e);
                default
            })
        },
    )
}
