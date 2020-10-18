use chrono::prelude::*;
use std::fmt;

pub type BackendResult = Result<Vec<ColdPhase>, BackendError>;

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
    WeatherDataRetrieved(BackendResult),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ColdPhase {
    pub location: String,
    pub min_temp: f32,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub record_type: RecordType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackendError {
    BrightskyError(BrightskyApiError),
    NetworkError(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::BrightskyError(e) => e.fmt(f),
            BackendError::NetworkError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for BackendError {}

impl From<BrightskyApiError> for BackendError {
    fn from(e: BrightskyApiError) -> Self {
        BackendError::BrightskyError(e)
    }
}

impl From<Box<dyn std::error::Error>> for BackendError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        BackendError::NetworkError(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrightskyApiError {
    pub title: String,
    pub description: String,
}

impl fmt::Display for BrightskyApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            serde_json::to_string_pretty(self).expect(
                "BrightskyApiError must not contain fields that could actually cause this to fail"
            )
        )
    }
}

impl std::error::Error for BrightskyApiError {}
