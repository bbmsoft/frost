use super::common::*;
use chrono::prelude::*;

pub fn accumulate_cold_phases(
    warning_threshold: f32,
    danger_threshold: f32,
    data: &brtsky::Response,
) -> BackendResponse {
    let mut phases: Vec<ColdPhase> = Vec::new();

    let mut current_phase: Option<ColdPhase> = None;

    let mut location = None;

    for data in data.weather_data_sets() {
        location = Some(data.source().station_name.to_owned());

        if let Some(temp) = data.weather_data().temperature {
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
                phase.end = data.weather_data().timestamp.with_timezone(&Local)
                    + chrono::Duration::hours(1);
            } else {
                // start new phase
                let phase = ColdPhase {
                    min_temp: temp,
                    start: data.weather_data().timestamp.with_timezone(&Local),
                    end: data.weather_data().timestamp.with_timezone(&Local)
                        + chrono::Duration::hours(1),
                    record_type: if temp <= danger_threshold {
                        RecordType::Danger
                    } else {
                        RecordType::Warning
                    },
                    warning_threshold,
                    danger_threshold,
                };
                current_phase = Some(phase);
            }
        }
    }

    if let Some(phase) = current_phase.as_mut() {
        phases.push(phase.clone());
    }

    BackendResponse {
        location,
        cold_phases: phases,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let data = std::fs::read("test/test.json").unwrap();
        let data: brtsky::Response = serde_json::from_slice(&data).unwrap();
        let cold_phases = accumulate_cold_phases(10.0, 7.0, &data).cold_phases;

        assert_eq!(cold_phases.len(), 1);

        let json = serde_json::to_string(&cold_phases).unwrap();

        let expected_json = r#"[{"min_temp":6.7,"start":"2020-04-21T04:00:00+02:00","end":"2020-04-21T09:00:00+02:00","record_type":"Danger","warning_threshold":10.0,"danger_threshold":7.0}]"#;

        assert_eq!(&json, expected_json);

        let roundtrip: Vec<ColdPhase> = serde_json::from_str(&json).unwrap();

        assert_eq!(cold_phases, roundtrip);
    }
}
