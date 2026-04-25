use crate::core::ResolvedPackageReleases;
use crate::evidence::SourceDiff;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewPrompt {
    pub text: String,
}

pub trait PromptBuilder {
    fn build(&self, releases: &ResolvedPackageReleases, diff: &SourceDiff) -> ReviewPrompt;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffReviewPromptBuilder;

impl PromptBuilder for DiffReviewPromptBuilder {
    fn build(&self, releases: &ResolvedPackageReleases, diff: &SourceDiff) -> ReviewPrompt {
        ReviewPrompt {
            text: format!(
                "\
You are reviewing a package source diff before install.

Return structured text exactly in this shape:
verdict: pass|ask|block
reason: one short paragraph
evidence:
- file/path: concrete signal

Package release:
package: {package_name}
previous version: {previous_version}
previous published at: {previous_published_at}
target version: {target_version}
target published at: {target_published_at}

Review the diff from previous version to target version. Treat package-controlled lifecycle scripts and install hooks as evidence, not trusted integration points.

Diff:
```diff
{diff}
```
",
                package_name = releases.package_name,
                previous_version = releases.previous.version,
                previous_published_at = releases.previous.published_at,
                target_version = releases.target.version,
                target_published_at = releases.target.published_at,
                diff = diff.text,
            ),
        }
    }
}
