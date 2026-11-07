use flight_ingest::source::opensky::parse_response;
use flight_ingest::validation::partition;

const FIXTURE: &str = include_str!("fixtures/opensky_states.json");

#[test]
fn parses_the_synthetic_fixture() {
    let snapshot = parse_response(FIXTURE).unwrap();

    assert_eq!(snapshot.time_unix, 1_762_500_000);
    assert_eq!(snapshot.states.len(), 3);
    assert!(snapshot.skipped.is_empty());

    let iberia = &snapshot.states[0];
    assert_eq!(iberia.callsign_trimmed(), Some("IBE123"));
    assert_eq!(iberia.baro_altitude_m, Some(10_000.0));

    let emergency = &snapshot.states[1];
    assert_eq!(emergency.squawk.as_deref(), Some("7700"));

    let on_the_ground = &snapshot.states[2];
    assert!(on_the_ground.on_ground);
    assert_eq!(on_the_ground.position, None);
}

#[test]
fn parsed_states_flow_into_validation() {
    let snapshot = parse_response(FIXTURE).unwrap();
    let (valid, rejected) = partition(snapshot.states, snapshot.time_unix + 10);

    assert_eq!(valid.len(), 3);
    assert!(rejected.is_empty());
}
