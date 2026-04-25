use std::collections::BTreeMap;
use std::io::Write;

use flate2::write::GzEncoder;
use flate2::Compression;
use lfg::core::{
    ArchiveRef, PackageOutcome, ReleaseReviewer, ResolvedPackageRelease, ResolvedPackageReleases,
    ReviewUnavailableReason, Verdict,
};
use lfg::evidence::{ArchiveFetchError, ArchiveFetcher, UnifiedDiffEngine};
use lfg::providers::{ArchiveDiffReviewer, ProviderError, ReviewPrompt, ReviewProvider};
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

#[derive(Debug, Clone)]
struct StaticProvider {
    output: Result<String, ProviderError>,
}

impl ReviewProvider for StaticProvider {
    fn id(&self) -> &'static str {
        "static"
    }

    fn review(&self, prompt: &ReviewPrompt) -> Result<String, ProviderError> {
        assert!(prompt.text.contains("package: demo"));
        assert!(prompt.text.contains("target version: 1.1.0"));
        assert!(prompt.text.contains("+module.exports = 2;"));

        self.output.clone()
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
fn successful_archive_diff_waits_for_provider_review() {
    let reviewer = ArchiveDiffReviewer::new(
        StaticArchiveFetcher {
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
        },
        UnifiedDiffEngine,
    );

    assert_eq!(
        reviewer.review(&releases()),
        PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::ProviderFailure)
    );
}

#[test]
fn successful_archive_diff_returns_provider_verdict() {
    let reviewer = ArchiveDiffReviewer::with_provider(
        StaticArchiveFetcher {
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
        },
        UnifiedDiffEngine,
        StaticProvider {
            output: Ok(
                "verdict: block\nreason: added risky code\n\nevidence:\n- package/index.js: changed runtime export\n"
                    .to_owned(),
            ),
        },
    );

    assert_eq!(
        reviewer.review(&releases()),
        PackageOutcome::ProviderVerdict(Verdict::Block)
    );
}

#[test]
fn provider_failure_returns_provider_failure() {
    let reviewer = ArchiveDiffReviewer::with_provider(
        StaticArchiveFetcher {
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
        },
        UnifiedDiffEngine,
        StaticProvider {
            output: Err(ProviderError::Failure("provider failed".to_owned())),
        },
    );

    assert_eq!(
        reviewer.review(&releases()),
        PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::ProviderFailure)
    );
}

#[test]
fn archive_diff_failure_stays_a_diff_failure() {
    let reviewer = ArchiveDiffReviewer::new(
        StaticArchiveFetcher {
            archives: BTreeMap::new(),
        },
        UnifiedDiffEngine,
    );

    assert_eq!(
        reviewer.review(&releases()),
        PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::DiffFailure)
    );
}
