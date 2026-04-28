use crate::core::{ArchiveRef, ResolvedPackageRelease, ResolvedPackageReleases};
use crate::evidence::SourceDiff;
use crate::providers::{DiffReviewPromptBuilder, PromptBuilder};

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
fn prompt_matches_release_baseline_diff_and_structured_output_contract() {
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

    assert_eq!(
        prompt.text,
        "\
You are reviewing a package source diff before install.

Return structured text exactly in this shape:
verdict: pass|ask|block
reason: one short paragraph
evidence:
- file/path: concrete signal

Package release:
package: demo
previous version: 1.0.0
previous published at: 1970-01-01T00:00:00.000Z
target version: 1.1.0
target published at: 1970-01-02T00:00:00.000Z

Review the diff from previous version to target version. Treat package-controlled lifecycle scripts and install hooks as evidence, not trusted integration points.

Diff:
```diff
diff --git a/package/index.js b/package/index.js
--- a/package/index.js
+++ b/package/index.js
@@
-module.exports = 1;
+module.exports = 2;

```
"
    );
}

#[test]
fn prompt_discloses_truncated_diff_when_context_budget_is_exceeded() {
    let prompt = DiffReviewPromptBuilder.build_with_max_diff_bytes(
        &releases(),
        &SourceDiff {
            text: "0123456789abcdef".to_owned(),
        },
        8,
    );

    assert!(prompt.text.contains("Diff was truncated by packvet."));
    assert!(prompt.text.contains("Showing first 8 of 16 bytes"));
    assert!(prompt.text.contains("01234567"));
    assert!(!prompt.text.contains("89abcdef"));
}
