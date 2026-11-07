use thiserror::Error;

use crate::domain::{Icao24Error, PositionError};

/// Errors raised while turning raw data into domain types.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IngestError {
    #[error("invalid ICAO24: {0}")]
    Icao24(#[from] Icao24Error),
    #[error("invalid position: {0}")]
    Position(#[from] PositionError),
    #[error("malformed JSON: {0}")]
    MalformedJson(String),
}
