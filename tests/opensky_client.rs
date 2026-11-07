use flight_ingest::error::IngestError;
use flight_ingest::geo::BoundingBox;
use flight_ingest::source::FlightSource;
use flight_ingest::source::opensky_client::OpenSkyClient;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

const FIXTURE: &str = include_str!("fixtures/opensky_states.json");

fn http_response(status: &str, headers: &[(&str, &str)], body: &str) -> String {
    let mut response = format!("HTTP/1.1 {status}\r\n");
    for (name, value) in headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    ));
    response
}

/// Starts a one-shot HTTP server on a free local port.
///
/// Returns its base URL and a handle that yields the raw request it received.
async fn serve_once(response: String) -> (String, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = vec![0u8; 8192];
        let read = socket.read(&mut buffer).await.unwrap();
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.shutdown().await.ok();
        String::from_utf8_lossy(&buffer[..read]).to_string()
    });

    (format!("http://{address}"), handle)
}

fn switzerland() -> BoundingBox {
    BoundingBox::new(45.8389, 5.9962, 47.8229, 10.5226).unwrap()
}

fn client_for(base_url: String) -> OpenSkyClient {
    OpenSkyClient::new(switzerland())
        .unwrap()
        .with_base_url(base_url)
}

#[tokio::test]
async fn fetch_parses_the_body_and_reads_the_credits() {
    let response = http_response(
        "200 OK",
        &[
            ("Content-Type", "application/json"),
            ("X-Rate-Limit-Remaining", "399"),
        ],
        FIXTURE,
    );
    let (base_url, request) = serve_once(response).await;

    let snapshot = client_for(base_url).fetch().await.unwrap();

    assert_eq!(snapshot.states.len(), 3);
    assert_eq!(snapshot.credits_remaining, Some(399));

    let request = request.await.unwrap().to_lowercase();
    assert!(
        request
            .starts_with("get /states/all?lamin=45.8389&lomin=5.9962&lamax=47.8229&lomax=10.5226 ")
    );
    assert!(request.contains("user-agent: flight-ingest/"));
}

#[tokio::test]
async fn a_missing_credits_header_is_not_an_error() {
    let response = http_response("200 OK", &[], FIXTURE);
    let (base_url, _) = serve_once(response).await;

    let snapshot = client_for(base_url).fetch().await.unwrap();

    assert_eq!(snapshot.credits_remaining, None);
}

#[tokio::test]
async fn too_many_requests_reports_how_long_to_wait() {
    let response = http_response(
        "429 Too Many Requests",
        &[("X-Rate-Limit-Retry-After-Seconds", "3600")],
        "",
    );
    let (base_url, _) = serve_once(response).await;

    let result = client_for(base_url).fetch().await;

    assert_eq!(
        result,
        Err(IngestError::RateLimited {
            retry_after_secs: Some(3600)
        })
    );
}

#[tokio::test]
async fn other_error_statuses_are_reported_with_their_code() {
    let response = http_response("500 Internal Server Error", &[], "oops");
    let (base_url, _) = serve_once(response).await;

    let result = client_for(base_url).fetch().await;

    assert_eq!(result, Err(IngestError::UnexpectedStatus(500)));
}

#[tokio::test]
async fn a_garbage_body_is_a_parse_error_not_a_panic() {
    let response = http_response("200 OK", &[], "<html>maintenance</html>");
    let (base_url, _) = serve_once(response).await;

    let result = client_for(base_url).fetch().await;

    assert!(matches!(result, Err(IngestError::MalformedJson(_))));
}

#[tokio::test]
async fn nobody_listening_is_a_network_error() {
    // Bind a port, note it, and release it so that nothing listens there.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);

    let result = client_for(format!("http://{address}")).fetch().await;

    assert!(matches!(result, Err(IngestError::Network(_))));
}

#[test]
fn the_cost_of_a_request_is_known_before_sending_it() {
    let client = OpenSkyClient::new(switzerland()).unwrap();
    assert_eq!(client.credit_cost_per_request(), 1);
}
