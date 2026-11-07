use serde::Deserialize;
use serde::de::IgnoredAny;
use serde_json::Value;

use super::Snapshot;
use crate::domain::{FlightState, Icao24, Position};
use crate::error::IngestError;

/// The envelope of a `/states/all` response.
#[derive(Debug, Deserialize)]
struct RawResponse {
    time: i64,
    /// `null` when there are no aircraft in the requested area.
    states: Option<Vec<Value>>,
}

/// One state vector: OpenSky sends each aircraft as a positional array.
///
/// The order of the fields is the contract, so do not reorder them.
#[derive(Debug, Deserialize)]
struct RawState(
    String,         // 0  icao24
    Option<String>, // 1  callsign
    IgnoredAny,     // 2  origin country
    IgnoredAny,     // 3  time of last position
    i64,            // 4  last contact
    Option<f64>,    // 5  longitude
    Option<f64>,    // 6  latitude
    Option<f64>,    // 7  barometric altitude (m)
    bool,           // 8  on ground
    Option<f64>,    // 9  velocity (m/s)
    Option<f64>,    // 10 true track (degrees)
    Option<f64>,    // 11 vertical rate (m/s)
    IgnoredAny,     // 12 sensors
    IgnoredAny,     // 13 geometric altitude (m)
    Option<String>, // 14 squawk
    IgnoredAny,     // 15 special purpose indicator
    IgnoredAny,     // 16 position source
);

impl TryFrom<RawState> for FlightState {
    type Error = IngestError;

    fn try_from(raw: RawState) -> Result<Self, Self::Error> {
        let RawState(
            icao24,
            callsign,
            _,
            _,
            last_contact,
            lon,
            lat,
            baro_altitude,
            on_ground,
            velocity,
            true_track,
            vertical_rate,
            _,
            _,
            squawk,
            _,
            _,
        ) = raw;

        let position = match (lat, lon) {
            (Some(lat), Some(lon)) => Some(Position::new(lat, lon)?),
            _ => None,
        };

        Ok(FlightState {
            icao24: Icao24::parse(&icao24)?,
            callsign,
            position,
            baro_altitude_m: baro_altitude,
            velocity_ms: velocity,
            heading_deg: true_track,
            vertical_rate_ms: vertical_rate,
            squawk,
            on_ground,
            last_contact_unix: last_contact,
        })
    }
}

fn malformed(error: serde_json::Error) -> IngestError {
    IngestError::MalformedJson(error.to_string())
}

/// Parses the body of a `/states/all` response.
///
/// A broken envelope is an error. A broken *record* is not: it goes to
/// `Snapshot::skipped` with its reason and the rest of the batch survives.
pub fn parse_response(json: &str) -> Result<Snapshot, IngestError> {
    let response: RawResponse = serde_json::from_str(json).map_err(malformed)?;

    let mut states = Vec::new();
    let mut skipped = Vec::new();

    for value in response.states.unwrap_or_default() {
        let parsed = serde_json::from_value::<RawState>(value)
            .map_err(malformed)
            .and_then(FlightState::try_from);

        match parsed {
            Ok(state) => states.push(state),
            Err(error) => skipped.push(error),
        }
    }

    Ok(Snapshot {
        time_unix: response.time,
        states,
        skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Icao24Error, PositionError};

    const NO_AIRCRAFT: &str = r#"{"time": 1762500000, "states": null}"#;

    #[test]
    fn null_states_means_no_aircraft() {
        let snapshot = parse_response(NO_AIRCRAFT).unwrap();
        assert_eq!(snapshot.time_unix, 1_762_500_000);
        assert!(snapshot.states.is_empty());
        assert!(snapshot.skipped.is_empty());
    }

    #[test]
    fn broken_envelope_is_an_error() {
        assert!(matches!(
            parse_response("not json"),
            Err(IngestError::MalformedJson(_))
        ));
        assert!(matches!(
            parse_response(r#"{"states": []}"#),
            Err(IngestError::MalformedJson(_))
        ));
    }

    #[test]
    fn a_bad_record_does_not_sink_the_batch() {
        let json = r#"{
            "time": 1762500000,
            "states": [
                ["4840d6","IBE123  ","Spain",1762499990,1762499995,2.1,41.3,10000.0,false,230.0,90.0,0.0,null,10100.0,"1000",false,0],
                ["zzzzzz","BAD1    ","Nowhere",1762499990,1762499995,2.1,41.3,10000.0,false,230.0,90.0,0.0,null,10100.0,"1000",false,0],
                ["4840d7","BAD2    ","Nowhere",1762499990,1762499995,2.1,95.0,10000.0,false,230.0,90.0,0.0,null,10100.0,"1000",false,0],
                ["4840d8", 12345]
            ]
        }"#;

        let snapshot = parse_response(json).unwrap();

        assert_eq!(snapshot.states.len(), 1);
        assert_eq!(snapshot.states[0].icao24.as_str(), "4840d6");
        assert_eq!(snapshot.skipped.len(), 3);
        assert_eq!(
            snapshot.skipped[0],
            IngestError::Icao24(Icao24Error::NotHex)
        );
        assert_eq!(
            snapshot.skipped[1],
            IngestError::Position(PositionError::LatitudeOutOfRange)
        );
        assert!(matches!(snapshot.skipped[2], IngestError::MalformedJson(_)));
    }

    #[test]
    fn nulls_become_none() {
        let json = r#"{
            "time": 1762500000,
            "states": [
                ["4840d6",null,"Spain",null,1762499995,null,null,null,true,null,null,null,null,null,null,false,0]
            ]
        }"#;

        let state = parse_response(json).unwrap().states.remove(0);

        assert_eq!(state.callsign, None);
        assert_eq!(state.position, None);
        assert_eq!(state.baro_altitude_m, None);
        assert_eq!(state.squawk, None);
        assert!(state.on_ground);
    }
}
