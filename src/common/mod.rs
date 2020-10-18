use chrono::prelude::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RecordType {
    Warning,
    Danger,
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordType::Warning => write!(f, "warning"),
            RecordType::Danger => write!(f, "danger"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocationStatus {
    WaitingForLocation,
    LocationFailed(u16, String),
    LocationRetrieved(f32, f32),
    LocationDisabled,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WeatherDataStatus {
    WaitingForWeatherData,
    FetchError(String),
    ParseError(String),
    WeatherDataRetrieved(brtsky::Response),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColdPhase {
    pub location: String,
    pub min_temp: f32,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub record_type: RecordType,
}

pub fn accumulate_cold_phases(
    warning_threshold: f32,
    danger_threshold: f32,
    data: &brtsky::Response,
) -> Vec<ColdPhase> {
    let mut phases: Vec<ColdPhase> = Vec::new();

    let mut current_phase: Option<ColdPhase> = None;

    for data in data.weather_data_sets() {
        let temp = data.weather_data().temperature;
        if temp > warning_threshold {
            // end current phase if there is on
            if let Some(phase) = current_phase.as_mut() {
                phases.push(phase.clone());
                current_phase = None;
            }
        } else if let Some(phase) = current_phase.as_mut() {
            // update current phase if there is one
            if temp < phase.min_temp {
                phase.min_temp = temp;
            }
            if temp <= danger_threshold {
                phase.record_type = RecordType::Danger;
            }
            phase.end =
                data.weather_data().timestamp.with_timezone(&Local) + chrono::Duration::hours(1);
        } else {
            // start new phase
            let phase = ColdPhase {
                location: data.source().station_name.to_owned(),
                min_temp: temp,
                start: data.weather_data().timestamp.with_timezone(&Local),
                end: data.weather_data().timestamp.with_timezone(&Local)
                    + chrono::Duration::hours(1),
                record_type: if temp <= danger_threshold {
                    RecordType::Danger
                } else {
                    RecordType::Warning
                },
            };
            current_phase = Some(phase);
        }
    }

    if let Some(phase) = current_phase.as_mut() {
        phases.push(phase.clone());
    }

    phases
}
