use thiserror::Error;

use crate::domain::{Icao24Error, PositionError};
use crate::geo::BoundingBoxError;

/// Errors raised while turning raw data into domain types or fetching it.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IngestError {
    #[error("invalid ICAO24: {0}")]
    Icao24(#[from] Icao24Error),
    #[error("invalid position: {0}")]
    Position(#[from] PositionError),
    #[error("invalid bounding box: {0}")]
    BoundingBox(#[from] BoundingBoxError),
    #[error("malformed JSON: {0}")]
    MalformedJson(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("rate limited by the server (retry after {retry_after_secs:?} seconds)")]
    RateLimited { retry_after_secs: Option<u64> },
    #[error("unexpected HTTP status {0}")]
    UnexpectedStatus(u16),
}
