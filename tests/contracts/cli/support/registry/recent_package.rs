use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};

pub(crate) fn serve_recent_package_with_archives() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
    listener
        .set_nonblocking(true)
        .expect("configure nonblocking listener");
    let address = listener.local_addr().expect("read local server address");
    let registry_base_url = format!("http://{address}");
    let packument = format!(
        r#"{{
      "name": "recent-package",
      "dist-tags": {{ "latest": "1.1.0" }},
      "time": {{
        "1.0.0": "1970-01-01T00:00:00.000Z",
        "1.1.0": "1970-01-02T00:00:00.000Z"
      }},
      "versions": {{
        "1.0.0": {{
          "dist": {{ "tarball": "{registry_base_url}/recent-package-1.0.0.tgz" }}
        }},
        "1.1.0": {{
          "dist": {{ "tarball": "{registry_base_url}/recent-package-1.1.0.tgz" }}
        }}
      }}
    }}"#
    );
    let responses = BTreeMap::from([
        (
            "/recent-package".to_owned(),
            ("application/json", packument.into_bytes()),
        ),
        (
            "/recent-package-1.0.0.tgz".to_owned(),
            (
                "application/octet-stream",
                tgz(&[("package/index.js", "module.exports = 1;\n")]),
            ),
        ),
        (
            "/recent-package-1.1.0.tgz".to_owned(),
            (
                "application/octet-stream",
                tgz(&[("package/index.js", "module.exports = 2;\n")]),
            ),
        ),
    ]);

    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        let started_at = Instant::now();
        let mut last_request_at = Instant::now();

        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_nonblocking(false)
                        .expect("configure blocking stream");
                    let mut buffer = [0; 2048];
                    let read = stream.read(&mut buffer).expect("read request");
                    let request = String::from_utf8_lossy(&buffer[..read]).to_string();
                    let path = request
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .expect("request path")
                        .to_owned();
                    let (content_type, body) = responses
                        .get(&path)
                        .unwrap_or_else(|| panic!("unexpected request path: {path}"));
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n",
                        body.len()
                    );
                    stream
                        .write_all(response.as_bytes())
                        .expect("write response header");
                    stream.write_all(body).expect("write response body");
                    requests.push(request);
                    last_request_at = Instant::now();
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if requests.len() >= 3
                        || (!requests.is_empty()
                            && last_request_at.elapsed() > Duration::from_millis(100))
                        || started_at.elapsed() > Duration::from_secs(2)
                    {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept request: {error}"),
            }
        }

        requests
    });

    (registry_base_url, handle)
}

fn tgz(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        for (path, content) in entries {
            let mut header = Header::new_gnu();
            header.set_path(path).expect("set tar path");
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append(&header, content.as_bytes())
                .expect("append tar entry");
        }
        builder.finish().expect("finish tar");
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes).expect("write gzip body");
    encoder.finish().expect("finish gzip")
}
