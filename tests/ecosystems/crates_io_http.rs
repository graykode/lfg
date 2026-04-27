use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use lfg::ecosystems::crates_io::{CratesIoCrateClient, CratesIoHttpCrateClient};

fn serve_once(response_body: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
    let address = listener.local_addr().expect("read local server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept one request");
        let mut buffer = [0; 2048];
        let read = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..read]).to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        request
    });

    (format!("http://{address}"), handle)
}

#[test]
fn fetches_crate_metadata_from_configured_registry() {
    let (base_url, handle) = serve_once(r#"{"crate":{"id":"serde"}}"#);
    let client = CratesIoHttpCrateClient::new(base_url);

    assert_eq!(
        client.fetch_crate("serde"),
        Ok(r#"{"crate":{"id":"serde"}}"#.to_owned())
    );

    let request = handle.join().expect("server thread completes");
    assert!(request.starts_with("GET /api/v1/crates/serde HTTP/1.1\r\n"));
    assert!(request
        .to_ascii_lowercase()
        .contains("accept: application/json\r\n"));
}

#[test]
fn encodes_crate_name_slash_in_registry_url() {
    let (base_url, handle) = serve_once(r#"{"crate":{"id":"owner/name"}}"#);
    let client = CratesIoHttpCrateClient::new(base_url);

    assert_eq!(
        client.fetch_crate("owner/name"),
        Ok(r#"{"crate":{"id":"owner/name"}}"#.to_owned())
    );

    let request = handle.join().expect("server thread completes");
    assert!(request.starts_with("GET /api/v1/crates/owner%2Fname HTTP/1.1\r\n"));
}
