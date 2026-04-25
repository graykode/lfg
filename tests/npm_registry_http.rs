use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use lfg::npm_registry::{NpmHttpPackumentClient, NpmPackumentClient};

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
fn fetches_packument_from_configured_registry() {
    let (base_url, handle) = serve_once(r#"{"name":"left-pad"}"#);
    let client = NpmHttpPackumentClient::new(base_url);

    assert_eq!(
        client.fetch_packument("left-pad"),
        Ok(r#"{"name":"left-pad"}"#.to_owned())
    );

    let request = handle.join().expect("server thread completes");
    assert!(request.starts_with("GET /left-pad HTTP/1.1\r\n"));
    assert!(request
        .to_ascii_lowercase()
        .contains("accept: application/vnd.npm.install-v1+json, application/json\r\n"));
}

#[test]
fn encodes_scoped_package_slash_in_registry_url() {
    let (base_url, handle) = serve_once(r#"{"name":"@scope/pkg"}"#);
    let client = NpmHttpPackumentClient::new(base_url);

    assert_eq!(
        client.fetch_packument("@scope/pkg"),
        Ok(r#"{"name":"@scope/pkg"}"#.to_owned())
    );

    let request = handle.join().expect("server thread completes");
    assert!(request.starts_with("GET /@scope%2Fpkg HTTP/1.1\r\n"));
}
