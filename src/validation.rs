use thiserror::Error;

use crate::domain::FlightState;

const MIN_ALTITUDE_M: f64 = -500.0;
const MAX_ALTITUDE_M: f64 = 25_000.0;
const MAX_VELOCITY_MS: f64 = 1_000.0;

/// A reason why a state is not plausible.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ValidationIssue {
    #[error("barometric altitude {0} m is outside the plausible range")]
    AltitudeImplausible(f64),
    #[error("velocity {0} m/s is outside the plausible range")]
    VelocityImplausible(f64),
    #[error("heading {0} degrees is outside [0, 360)")]
    HeadingOutOfRange(f64),
    #[error("squawk {0:?} is not four octal digits")]
    InvalidSquawk(String),
    #[error("last contact {last_contact} is later than now ({now})")]
    TimestampInFuture { last_contact: i64, now: i64 },
}

/// A state that failed validation, together with the reasons.
#[derive(Debug, Clone, PartialEq)]
pub struct Rejected {
    pub state: FlightState,
    pub issues: Vec<ValidationIssue>,
}

/// Returns every plausibility problem found in `state`.
///
/// `now_unix` is passed in, not read from the clock, so tests are deterministic.
pub fn validate(state: &FlightState, now_unix: i64) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Some(alt) = state.baro_altitude_m
        && !(MIN_ALTITUDE_M..=MAX_ALTITUDE_M).contains(&alt)
    {
        issues.push(ValidationIssue::AltitudeImplausible(alt));
    }
    if let Some(v) = state.velocity_ms
        && !(0.0..=MAX_VELOCITY_MS).contains(&v)
    {
        issues.push(ValidationIssue::VelocityImplausible(v));
    }
    if let Some(h) = state.heading_deg
        && !(0.0..360.0).contains(&h)
    {
        issues.push(ValidationIssue::HeadingOutOfRange(h));
    }
    if let Some(sq) = &state.squawk {
        let ok = sq.len() == 4 && sq.chars().all(|c| ('0'..='7').contains(&c));
        if !ok {
            issues.push(ValidationIssue::InvalidSquawk(sq.clone()));
        }
    }
    if state.last_contact_unix > now_unix {
        issues.push(ValidationIssue::TimestampInFuture {
            last_contact: state.last_contact_unix,
            now: now_unix,
        });
    }

    issues
}

/// Splits `states` into valid ones and rejected ones (with their reasons).
///
/// Takes ownership of the vector: each state is moved into exactly one side.
pub fn partition(states: Vec<FlightState>, now_unix: i64) -> (Vec<FlightState>, Vec<Rejected>) {
    let mut valid = Vec::new();
    let mut rejected = Vec::new();

    for state in states {
        let issues = validate(&state, now_unix);
        if issues.is_empty() {
            valid.push(state);
        } else {
            rejected.push(Rejected { state, issues });
        }
    }

    (valid, rejected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Icao24, Position};

    const NOW: i64 = 1_762_500_100;

    fn good_state() -> FlightState {
        FlightState {
            icao24: Icao24::parse("4840d6").unwrap(),
            callsign: Some("IBE123".to_string()),
            position: Some(Position::new(41.3, 2.1).unwrap()),
            baro_altitude_m: Some(10_000.0),
            velocity_ms: Some(230.0),
            heading_deg: Some(90.0),
            vertical_rate_ms: Some(0.0),
            squawk: Some("1000".to_string()),
            on_ground: false,
            last_contact_unix: NOW - 10,
        }
    }

    #[test]
    fn good_state_has_no_issues() {
        assert!(validate(&good_state(), NOW).is_empty());
    }

    #[test]
    fn missing_optional_fields_are_not_issues() {
        let mut s = good_state();
        s.baro_altitude_m = None;
        s.velocity_ms = None;
        s.heading_deg = None;
        s.squawk = None;
        assert!(validate(&s, NOW).is_empty());
    }

    #[test]
    fn detects_implausible_altitude_and_velocity() {
        let mut s = good_state();
        s.baro_altitude_m = Some(90_000.0);
        s.velocity_ms = Some(-5.0);
        assert_eq!(
            validate(&s, NOW),
            vec![
                ValidationIssue::AltitudeImplausible(90_000.0),
                ValidationIssue::VelocityImplausible(-5.0),
            ]
        );
    }

    #[test]
    fn heading_360_is_out_of_range() {
        let mut s = good_state();
        s.heading_deg = Some(360.0);
        assert_eq!(
            validate(&s, NOW),
            vec![ValidationIssue::HeadingOutOfRange(360.0)]
        );
    }

    #[test]
    fn squawk_must_be_four_octal_digits() {
        for bad in ["7708", "770", "77000", "abcd"] {
            let mut s = good_state();
            s.squawk = Some(bad.to_string());
            assert_eq!(
                validate(&s, NOW),
                vec![ValidationIssue::InvalidSquawk(bad.to_string())],
                "squawk {bad:?}"
            );
        }
        let mut s = good_state();
        s.squawk = Some("7700".to_string());
        assert!(validate(&s, NOW).is_empty());
    }

    #[test]
    fn detects_timestamp_in_the_future() {
        let mut s = good_state();
        s.last_contact_unix = NOW + 1;
        assert_eq!(
            validate(&s, NOW),
            vec![ValidationIssue::TimestampInFuture {
                last_contact: NOW + 1,
                now: NOW
            }]
        );
    }

    #[test]
    fn partition_separates_valid_and_rejected() {
        let good = good_state();
        let mut bad = good_state();
        bad.heading_deg = Some(400.0);

        let (valid, rejected) = partition(vec![good.clone(), bad.clone()], NOW);

        assert_eq!(valid, vec![good]);
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].state, bad);
        assert_eq!(
            rejected[0].issues,
            vec![ValidationIssue::HeadingOutOfRange(400.0)]
        );
    }

    #[test]
    fn issues_have_readable_messages() {
        assert_eq!(
            ValidationIssue::InvalidSquawk("7708".into()).to_string(),
            "squawk \"7708\" is not four octal digits"
        );
    }
}
