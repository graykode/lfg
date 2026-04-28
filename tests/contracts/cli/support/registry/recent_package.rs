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

pub(crate) fn serve_recent_python_project_with_archives(
) -> (String, thread::JoinHandle<Vec<String>>) {
    serve_recent_metadata_with_archives(|registry_base_url| {
        let project = format!(
            r#"{{
      "info": {{ "name": "recent-python-package", "version": "1.1.0" }},
      "releases": {{
        "1.0.0": [
          {{
            "packagetype": "sdist",
            "url": "{registry_base_url}/recent-python-package-1.0.0.tar.gz",
            "upload_time_iso_8601": "1970-01-01T00:00:00.000000Z"
          }}
        ],
        "1.1.0": [
          {{
            "packagetype": "sdist",
            "url": "{registry_base_url}/recent-python-package-1.1.0.tar.gz",
            "upload_time_iso_8601": "1970-01-02T00:00:00.000000Z"
          }}
        ]
      }}
    }}"#
        );

        BTreeMap::from([
            (
                "/pypi/recent-python-package/json".to_owned(),
                ("application/json", project.into_bytes()),
            ),
            (
                "/recent-python-package-1.0.0.tar.gz".to_owned(),
                (
                    "application/octet-stream",
                    tgz(&[("package/index.py", "VALUE = 1\n")]),
                ),
            ),
            (
                "/recent-python-package-1.1.0.tar.gz".to_owned(),
                (
                    "application/octet-stream",
                    tgz(&[("package/index.py", "VALUE = 2\n")]),
                ),
            ),
        ])
    })
}

pub(crate) fn serve_recent_crate_with_archives() -> (String, thread::JoinHandle<Vec<String>>) {
    serve_recent_metadata_with_archives(|_registry_base_url| {
        let metadata = r#"{
      "crate": { "id": "recent-crate", "max_version": "1.1.0" },
      "versions": [
        {
          "num": "1.1.0",
          "created_at": "1970-01-02T00:00:00+00:00",
          "dl_path": "/api/v1/crates/recent-crate/1.1.0/download"
        },
        {
          "num": "1.0.0",
          "created_at": "1970-01-01T00:00:00+00:00",
          "dl_path": "/api/v1/crates/recent-crate/1.0.0/download"
        }
      ]
    }"#;

        BTreeMap::from([
            (
                "/api/v1/crates/recent-crate".to_owned(),
                ("application/json", metadata.as_bytes().to_vec()),
            ),
            (
                "/api/v1/crates/recent-crate/1.0.0/download".to_owned(),
                (
                    "application/octet-stream",
                    tgz(&[("package/src/lib.rs", "pub const VALUE: u8 = 1;\n")]),
                ),
            ),
            (
                "/api/v1/crates/recent-crate/1.1.0/download".to_owned(),
                (
                    "application/octet-stream",
                    tgz(&[("package/src/lib.rs", "pub const VALUE: u8 = 2;\n")]),
                ),
            ),
        ])
    })
}

pub(crate) fn serve_recent_gem_with_archives() -> (String, thread::JoinHandle<Vec<String>>) {
    serve_recent_metadata_with_archives(|_registry_base_url| {
        let versions = r#"[
      {
        "number": "1.1.0",
        "created_at": "1970-01-02T00:00:00.000Z"
      },
      {
        "number": "1.0.0",
        "created_at": "1970-01-01T00:00:00.000Z"
      }
    ]"#;

        BTreeMap::from([
            (
                "/api/v1/versions/recent-gem.json".to_owned(),
                ("application/json", versions.as_bytes().to_vec()),
            ),
            (
                "/gems/recent-gem-1.0.0.gem".to_owned(),
                (
                    "application/octet-stream",
                    gem(&[("lib/recent_gem.rb", "VALUE = 1\n")]),
                ),
            ),
            (
                "/gems/recent-gem-1.1.0.gem".to_owned(),
                (
                    "application/octet-stream",
                    gem(&[("lib/recent_gem.rb", "VALUE = 2\n")]),
                ),
            ),
        ])
    })
}

fn serve_recent_metadata_with_archives(
    responses: impl FnOnce(&str) -> BTreeMap<String, (&'static str, Vec<u8>)>,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
    listener
        .set_nonblocking(true)
        .expect("configure nonblocking listener");
    let address = listener.local_addr().expect("read local server address");
    let registry_base_url = format!("http://{address}");
    let responses = responses(&registry_base_url);

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

fn gem(entries: &[(&str, &str)]) -> Vec<u8> {
    let data_archive = tgz(entries);
    let mut archive = Vec::new();
    {
        let mut builder = Builder::new(&mut archive);
        let mut header = Header::new_gnu();
        header.set_path("data.tar.gz").expect("set gem data path");
        header.set_size(data_archive.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append(&header, data_archive.as_slice())
            .expect("append gem data");
        builder.finish().expect("finish gem archive");
    }

    archive
}
