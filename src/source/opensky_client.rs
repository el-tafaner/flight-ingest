use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::HeaderMap;

use super::opensky::parse_response;
use super::{FlightSource, Snapshot};
use crate::error::IngestError;
use crate::geo::BoundingBox;

const DEFAULT_BASE_URL: &str = "https://opensky-network.org/api";
const USER_AGENT: &str = concat!("flight-ingest/", env!("CARGO_PKG_VERSION"));
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// An anonymous client for OpenSky's `/states/all` endpoint.
///
/// It always asks for a bounding box: a worldwide query costs four times
/// more credits than a small one, so there is no way to ask for "everything".
#[derive(Debug, Clone)]
pub struct OpenSkyClient {
    http: reqwest::Client,
    base_url: String,
    bbox: BoundingBox,
}

impl OpenSkyClient {
    pub fn new(bbox: BoundingBox) -> Result<Self, IngestError> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(network)?;

        Ok(Self {
            http,
            base_url: DEFAULT_BASE_URL.to_string(),
            bbox,
        })
    }

    /// Points the client at another server. Tests use a local one.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn credit_cost_per_request(&self) -> u32 {
        self.bbox.credit_cost()
    }
}

fn network(error: reqwest::Error) -> IngestError {
    IngestError::Network(error.to_string())
}

/// Reads a numeric header, returning `None` if it is missing or not a number.
fn header_number<T: std::str::FromStr>(headers: &HeaderMap, name: &str) -> Option<T> {
    headers.get(name)?.to_str().ok()?.trim().parse().ok()
}

impl FlightSource for OpenSkyClient {
    async fn fetch(&self) -> Result<Snapshot, IngestError> {
        let url = format!("{}/states/all", self.base_url);
        let response = self
            .http
            .get(url)
            .query(&self.bbox.query())
            .send()
            .await
            .map_err(network)?;

        let status = response.status();
        let credits_remaining = header_number(response.headers(), "x-rate-limit-remaining");

        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(IngestError::RateLimited {
                retry_after_secs: header_number(
                    response.headers(),
                    "x-rate-limit-retry-after-seconds",
                ),
            });
        }
        if !status.is_success() {
            return Err(IngestError::UnexpectedStatus(status.as_u16()));
        }

        let body = response.text().await.map_err(network)?;
        let mut snapshot = parse_response(&body)?;
        snapshot.credits_remaining = credits_remaining;
        Ok(snapshot)
    }
}
