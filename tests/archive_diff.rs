use std::collections::BTreeMap;
use std::io::Write;

use flate2::write::GzEncoder;
use flate2::Compression;
use lfg::core::{ArchiveRef, ResolvedPackageRelease, ResolvedPackageReleases};
use lfg::evidence::UnifiedDiffEngine;
use lfg::evidence::{ArchiveDiffBuilder, ArchiveDiffError, ArchiveFetchError, ArchiveFetcher};
use tar::{Builder, Header};

#[derive(Debug, Clone)]
struct StaticArchiveFetcher {
    archives: BTreeMap<String, Vec<u8>>,
}

impl ArchiveFetcher for StaticArchiveFetcher {
    fn fetch(&self, archive: &ArchiveRef) -> Result<Vec<u8>, ArchiveFetchError> {
        self.archives
            .get(&archive.url)
            .cloned()
            .ok_or_else(|| ArchiveFetchError::Unavailable(archive.url.clone()))
    }
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

fn releases() -> ResolvedPackageReleases {
    ResolvedPackageReleases {
        package_name: "demo".to_owned(),
        previous: ResolvedPackageRelease {
            version: "1.0.0".to_owned(),
            published_at: "1970-01-01T00:00:00.000Z".to_owned(),
            archive: ArchiveRef {
                url: "memory://demo-1.0.0.tgz".to_owned(),
            },
        },
        target: ResolvedPackageRelease {
            version: "1.1.0".to_owned(),
            published_at: "1970-01-02T00:00:00.000Z".to_owned(),
            archive: ArchiveRef {
                url: "memory://demo-1.1.0.tgz".to_owned(),
            },
        },
    }
}

#[test]
fn builds_source_diff_between_previous_and_target_archives() {
    let fetcher = StaticArchiveFetcher {
        archives: BTreeMap::from([
            (
                "memory://demo-1.0.0.tgz".to_owned(),
                tgz(&[("package/index.js", "module.exports = 1;\n")]),
            ),
            (
                "memory://demo-1.1.0.tgz".to_owned(),
                tgz(&[("package/index.js", "module.exports = 2;\n")]),
            ),
        ]),
    };
    let builder = ArchiveDiffBuilder::new(fetcher, UnifiedDiffEngine);

    let diff = builder
        .build(&releases())
        .expect("archive diff should build");

    assert_eq!(
        diff.text,
        "\
diff --git a/package/index.js b/package/index.js
--- a/package/index.js
+++ b/package/index.js
@@
-module.exports = 1;
+module.exports = 2;
"
    );
}

#[test]
fn returns_typed_error_when_archive_fetch_fails() {
    let builder = ArchiveDiffBuilder::new(
        StaticArchiveFetcher {
            archives: BTreeMap::new(),
        },
        UnifiedDiffEngine,
    );

    assert_eq!(
        builder.build(&releases()),
        Err(ArchiveDiffError::Fetch(ArchiveFetchError::Unavailable(
            "memory://demo-1.0.0.tgz".to_owned()
        )))
    );
}
