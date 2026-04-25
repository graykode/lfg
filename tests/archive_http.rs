use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use lfg::core::contracts::ArchiveRef;
use lfg::evidence::archive_diff::{ArchiveFetcher, HttpArchiveFetcher};

fn serve_bytes_once(body: &'static [u8]) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
    let address = listener.local_addr().expect("read local server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept one request");
        let mut buffer = [0; 2048];
        let read = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..read]).to_string();
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        stream.write_all(header.as_bytes()).expect("write header");
        stream.write_all(body).expect("write body");
        request
    });

    (format!("http://{address}"), handle)
}

#[test]
fn http_archive_fetcher_downloads_archive_bytes() {
    let (base_url, server) = serve_bytes_once(b"tgz-bytes");
    let archive = ArchiveRef {
        url: format!("{base_url}/demo-1.1.0.tgz"),
    };

    let bytes = HttpArchiveFetcher
        .fetch(&archive)
        .expect("archive bytes download");

    assert_eq!(bytes, b"tgz-bytes");

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /demo-1.1.0.tgz HTTP/1.1\r\n"));
}
