use self::components::frost::Frost;
use self::components::header::Header;
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
    on_notification_permission: Closure<dyn Fn(JsValue)>,
    fetch_task: Option<FetchTask>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    WeatherUpdate(WeatherDataStatus),
    NotificationPermissionUpdate(NotificationPermissionStatus),
    Refresh,
    LocationUpdate(LocationStatus),
    PlaceUpdate(PlaceStatus),
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct Props {
    pub location: Option<LocationStatus>,
    pub selected_place: PlaceStatus,
    pub weather: WeatherDataStatus,
    pub status: Option<Status>,
    pub notification_permission: NotificationPermissionStatus,
    pub geolocation_supported: bool,
    pub notifications_supported: bool,
    pub thresholds: Thresholds,
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
            Msg::LocationUpdate(status) => {
                self.props.location = Some(status.clone());
                match status {
                    LocationStatus::LocationFailed(_code, msg) => {
                        self.props.status = Some(Status::Error {
                            title: "Error getting location:".to_owned(),
                            body: msg.to_owned(),
                        });
                    }
                    LocationStatus::LocationRetrieved(lat, lon) => {
                        if let Some(LocationStatus::RequestDeviceLocation) = self.props.location {
                            self.props.status = None;
                        }
                        self.props.location = Some(LocationStatus::LocationRetrieved(lat, lon));
                        self.check_for_weather_update();
                    }
                    LocationStatus::RequestDeviceLocation => {
                        self.props.status = Some(Status::Progress(
                            "Requesting location from device...".to_owned(),
                        ));
                        js::get_location(&self.on_location_success, &self.on_location_error);
                    }
                }

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
                        if let Ok(data) = data {
                            self.try_send_weather_notification(data);
                        }
                    }
                }

                self.props.weather = data;

                true
            }
            Msg::PlaceUpdate(place) => {
                if let PlaceStatus::PlacePicked(Some(place)) = &place {
                    self.props.location = None;
                    if place.geometry.is_some() {
                        self.props.selected_place = PlaceStatus::PlacePicked(Some(place.clone()));
                        debug!("Storing location: {:?}", place);
                        let json = serde_json::to_string(&place).expect("can't fail");
                        js::store(LOCATION_KEY, &json);
                        self.check_for_weather_update();
                    } else {
                        self.props.status = Some(Status::Error {
                            title: "Invalid location".to_owned(),
                            body: "Please select a suggestion from the dropdown list!".to_owned(),
                        });
                    }
                }
                true
            }
            Msg::Refresh => {
                self.props.location = None;
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
        let geolocation_supported = self.props.geolocation_supported;
        let weather = self.props.weather.clone();
        let status = self.props.status.clone();

        let location = if let (
            Some(LocationStatus::LocationRetrieved(_, _)),
            WeatherDataStatus::WeatherDataRetrieved(Ok(data)),
        ) = (&self.props.location, &weather)
        {
            data.location.clone()
        } else if let PlaceStatus::PlacePicked(Some(place)) = &self.props.selected_place {
            Some(place.name.clone())
        } else {
            None
        };
        let app_link = self.link.clone();
        html! {
            <div class="app">
                <Header location={location} app_link={app_link} notifications_on={false} geolocation_supported={geolocation_supported} />
                <Frost weather={weather} />
                <div class="footer">
                    <StatusBar status={status} />
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
        let coordinates =
            if let Some(LocationStatus::LocationRetrieved(lat, lon)) = self.props.location {
                Some((lat, lon))
            } else if let PlaceStatus::PlacePicked(Some(place)) = &self.props.selected_place {
                let location = place
                    .geometry
                    .as_ref()
                    .expect("must be set when stored in props")
                    .location
                    .clone();
                Some((location.lat, location.lng))
            } else {
                None
            };

        if let Some((lat, lon)) = coordinates {
            match self.fetch_weather_data(lat, lon) {
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

    fn fetch_weather_data(&self, lat: f32, lon: f32) -> Result<FetchTask, BackendError> {
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

        let warning_threshold = self.props.thresholds.0;
        let danger_threshold = self.props.thresholds.1;

        let uri = format!(
            "/weather?lat={}&lon={}&warning_threshold={}&danger_threshold={}",
            lat, lon, warning_threshold, danger_threshold
        );
        debug!("Requesting weather data from backend...");
        let request = Request::get(&uri).body(Nothing)?;
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

    let place: Option<Place> =
        js::get_stored(LOCATION_KEY).map_or(None, |val| serde_json::from_str(&val).unwrap_or(None));

    let thresholds = if let Some(value) = js::get_stored(THRESHOLD_KEY) {
        if let Ok(thresholds) = serde_json::from_str(&value) {
            debug!("Got thresholds from web storage.");
            Some(thresholds)
        } else {
            warn!("Stored thresholds invalid.");
            None
        }
    } else {
        debug!("No thresholds stored.");
        None
    };

    let thresholds = if let Some(thresholds) = thresholds {
        thresholds
    } else {
        let thresholds = (5.0, 0.0);
        let value = serde_json::to_string(&thresholds).expect("can't fail");
        js::store(THRESHOLD_KEY, &value);
        thresholds
    };

    let weather = WeatherDataStatus::WaitingForWeatherData;
    let notification_permission = NotificationPermissionStatus::Default;
    let status = None;
    let location = None;

    let props = Props {
        location,
        weather,
        status,
        notification_permission,
        geolocation_supported,
        notifications_supported,
        selected_place: PlaceStatus::PlacePicked(place),
        thresholds,
    };

    App::<FrostApp>::new().mount_to_body_with_props(props);

    Ok(())
}
