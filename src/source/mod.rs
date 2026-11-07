use std::future::Future;

use crate::domain::FlightState;
use crate::error::IngestError;

pub mod opensky;
pub mod opensky_client;

/// One reading of the sky: when it was taken and which states were parsed.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub time_unix: i64,
    pub states: Vec<FlightState>,
    /// Records that could not be parsed, kept instead of dropped silently.
    pub skipped: Vec<IngestError>,
    /// API credits left after this reading, when the server reports them.
    pub credits_remaining: Option<u32>,
}

/// Anything that can produce snapshots of the sky.
///
/// The `+ Send` bound lets callers use the future from multi-threaded
/// runtimes such as `tokio`.
pub trait FlightSource {
    fn fetch(&self) -> impl Future<Output = Result<Snapshot, IngestError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeSource {
        time_unix: i64,
    }

    impl FlightSource for FakeSource {
        async fn fetch(&self) -> Result<Snapshot, IngestError> {
            Ok(Snapshot {
                time_unix: self.time_unix,
                states: Vec::new(),
                skipped: Vec::new(),
                credits_remaining: None,
            })
        }
    }

    async fn latest_time(source: &impl FlightSource) -> i64 {
        source.fetch().await.unwrap().time_unix
    }

    #[tokio::test]
    async fn a_fake_source_can_stand_in_for_a_real_one() {
        let source = FakeSource { time_unix: 42 };
        assert_eq!(latest_time(&source).await, 42);
    }
}
