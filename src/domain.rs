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
}
