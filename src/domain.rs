#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Icao24(String);

#[derive(Debug, PartialEq, Eq)]
pub enum Icao24Error {
    InvalidLength(usize),
    NotHex,
}

impl Icao24 {
    pub fn parse(s: &str) -> Result<Self, Icao24Error> {
        if s.len() != 6 {
            return Err(Icao24Error::InvalidLength(s.len()));
        }
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Icao24Error::NotHex);
        }
        Ok(Self(s.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    lat: f64,
    lon: f64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PositionError {
    LatitudeOutOfRange,
    LongitudeOutOfRange,
}

impl Position {
    pub fn new(lat: f64, lon: f64) -> Result<Self, PositionError> {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(PositionError::LatitudeOutOfRange);
        }
        if !(-180.0..=180.0).contains(&lon) {
            return Err(PositionError::LongitudeOutOfRange);
        }
        Ok(Self { lat, lon })
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn lon(&self) -> f64 {
        self.lon
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlightState {
    pub icao24: Icao24,
    pub callsign: Option<String>,
    pub position: Option<Position>,
    pub baro_altitude_m: Option<f64>,
    pub velocity_ms: Option<f64>,
    pub heading_deg: Option<f64>,
    pub vertical_rate_ms: Option<f64>,
    pub squawk: Option<String>,
    pub on_ground: bool,
    pub last_contact_unix: i64,
}

impl FlightState {
    pub fn callsign_trimmed(&self) -> Option<&str> {
        self.callsign
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icao24_accepts_six_hex_digits() {
        assert_eq!(Icao24::parse("4840D6").unwrap().as_str(), "4840d6");
    }

    #[test]
    fn icao24_rejects_wrong_length() {
        assert_eq!(Icao24::parse("4840"), Err(Icao24Error::InvalidLength(4)));
    }

    #[test]
    fn icao24_rejects_non_hex() {
        assert_eq!(Icao24::parse("zzzzzz"), Err(Icao24Error::NotHex));
    }

    fn sample_state(callsign: Option<&str>) -> FlightState {
        FlightState {
            icao24: Icao24::parse("4840d6").unwrap(),
            callsign: callsign.map(String::from),
            position: Some(Position::new(41.3, 2.1).unwrap()),
            baro_altitude_m: Some(10_000.0),
            velocity_ms: Some(230.0),
            heading_deg: Some(90.0),
            vertical_rate_ms: Some(0.0),
            squawk: Some("1000".to_string()),
            on_ground: false,
            last_contact_unix: 1_762_500_000,
        }
    }

    #[test]
    fn position_accepts_valid_coordinates() {
        let p = Position::new(41.3, 2.1).unwrap();
        assert_eq!((p.lat(), p.lon()), (41.3, 2.1));
    }

    #[test]
    fn position_rejects_out_of_range() {
        assert_eq!(
            Position::new(91.0, 0.0),
            Err(PositionError::LatitudeOutOfRange)
        );
        assert_eq!(
            Position::new(0.0, 181.0),
            Err(PositionError::LongitudeOutOfRange)
        );
    }

    #[test]
    fn position_rejects_nan() {
        assert!(Position::new(f64::NAN, 0.0).is_err());
    }

    #[test]
    fn callsign_trimmed_strips_padding() {
        let s = sample_state(Some("IBE123  "));
        assert_eq!(s.callsign_trimmed(), Some("IBE123"));
    }

    #[test]
    fn callsign_trimmed_is_none_when_blank_or_missing() {
        assert_eq!(sample_state(Some("   ")).callsign_trimmed(), None);
        assert_eq!(sample_state(None).callsign_trimmed(), None);
    }
}
