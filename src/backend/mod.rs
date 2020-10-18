use super::common::*;
use chrono::prelude::*;

pub fn accumulate_cold_phases<Tz: TimeZone>(
    warning_threshold: f32,
    danger_threshold: f32,
    data: &brtsky::Response,
    start: &DateTime<Tz>,
) -> Vec<ColdPhase> {
    let mut phases: Vec<ColdPhase> = Vec::new();

    let mut current_phase: Option<ColdPhase> = None;

    for data in data
        .weather_data_sets()
        .filter(|ds| &ds.weather_data().timestamp >= start)
    {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let start: DateTime<Utc> = "2020-04-21T02:00:00+00:00".parse().unwrap();
        let data = std::fs::read("test/test.json").unwrap();
        let data: brtsky::Response = serde_json::from_slice(&data).unwrap();
        let cold_phases = accumulate_cold_phases(10.0, 7.0, &data, &start);

        assert_eq!(cold_phases.len(), 1);

        let json = serde_json::to_string(&cold_phases).unwrap();

        let expected_json = r#"[{"location":"Münster/Osnabrück","min_temp":6.7,"start":"2020-04-21T04:00:00+02:00","end":"2020-04-21T09:00:00+02:00","record_type":"Danger"}]"#;

        assert_eq!(&json, expected_json);

        let roundtrip: Vec<ColdPhase> = serde_json::from_str(&json).unwrap();

        assert_eq!(cold_phases, roundtrip);
    }
}
