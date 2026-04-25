use lfg::core::{ArchiveRef, ResolvedPackageRelease, ResolvedPackageReleases};
use lfg::evidence::SourceDiff;
use lfg::providers::{DiffReviewPromptBuilder, PromptBuilder};

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
fn prompt_contains_release_baseline_diff_and_structured_output_contract() {
    let prompt = DiffReviewPromptBuilder.build(
        &releases(),
        &SourceDiff {
            text: "\
diff --git a/package/index.js b/package/index.js
--- a/package/index.js
+++ b/package/index.js
@@
-module.exports = 1;
+module.exports = 2;
"
            .to_owned(),
        },
    );

    assert!(prompt.text.contains("package: demo"));
    assert!(prompt.text.contains("previous version: 1.0.0"));
    assert!(prompt.text.contains("target version: 1.1.0"));
    assert!(prompt.text.contains("verdict: pass|ask|block"));
    assert!(prompt.text.contains("reason: one short paragraph"));
    assert!(prompt.text.contains("evidence:"));
    assert!(prompt
        .text
        .contains("diff --git a/package/index.js b/package/index.js"));
    assert!(prompt.text.contains("+module.exports = 2;"));
}
