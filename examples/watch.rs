//! Fetches one real snapshot from OpenSky and prints a summary.
//!
//! Run it with `cargo run --example watch`. It costs one API credit.

use flight_ingest::geo::BoundingBox;
use flight_ingest::source::FlightSource;
use flight_ingest::source::opensky_client::OpenSkyClient;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // The example box from OpenSky's documentation (about 9 square degrees).
    let bbox = BoundingBox::new(45.8389, 5.9962, 47.8229, 10.5226)?;
    let client = OpenSkyClient::new(bbox)?;

    println!(
        "Asking OpenSky (this costs {} credit)...",
        client.credit_cost_per_request()
    );
    let snapshot = client.fetch().await?;

    println!(
        "{} aircraft parsed, {} records skipped, credits remaining: {}",
        snapshot.states.len(),
        snapshot.skipped.len(),
        snapshot
            .credits_remaining
            .map_or("unknown".to_string(), |c| c.to_string())
    );

    for state in snapshot.states.iter().take(5) {
        println!(
            "  {}  {:<8}  altitude: {}",
            state.icao24.as_str(),
            state.callsign_trimmed().unwrap_or("-"),
            state
                .baro_altitude_m
                .map_or("unknown".to_string(), |a| format!("{a:.0} m"))
        );
    }

    Ok(())
}
