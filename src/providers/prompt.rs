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

const DEFAULT_MAX_DIFF_BYTES: usize = 120_000;

impl DiffReviewPromptBuilder {
    pub fn build_with_max_diff_bytes(
        &self,
        releases: &ResolvedPackageReleases,
        diff: &SourceDiff,
        max_diff_bytes: usize,
    ) -> ReviewPrompt {
        let rendered_diff = render_diff_with_budget(&diff.text, max_diff_bytes);
        let truncation_notice = rendered_diff.truncation_notice();

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

{truncation_notice}Diff:
```diff
{diff}
```
",
                package_name = releases.package_name,
                previous_version = releases.previous.version,
                previous_published_at = releases.previous.published_at,
                target_version = releases.target.version,
                target_published_at = releases.target.published_at,
                truncation_notice = truncation_notice,
                diff = rendered_diff.text,
            ),
        }
    }
}

impl PromptBuilder for DiffReviewPromptBuilder {
    fn build(&self, releases: &ResolvedPackageReleases, diff: &SourceDiff) -> ReviewPrompt {
        self.build_with_max_diff_bytes(releases, diff, DEFAULT_MAX_DIFF_BYTES)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderedDiff {
    text: String,
    original_bytes: usize,
    shown_bytes: usize,
}

impl RenderedDiff {
    fn truncation_notice(&self) -> String {
        if self.shown_bytes == self.original_bytes {
            return String::new();
        }

        format!(
            "Diff was truncated by packvet. Showing first {} of {} bytes.\n\n",
            self.shown_bytes, self.original_bytes
        )
    }
}

fn render_diff_with_budget(diff: &str, max_bytes: usize) -> RenderedDiff {
    let original_bytes = diff.len();
    if original_bytes <= max_bytes {
        return RenderedDiff {
            text: diff.to_owned(),
            original_bytes,
            shown_bytes: original_bytes,
        };
    }

    let shown = truncate_to_char_boundary(diff, max_bytes);

    RenderedDiff {
        text: shown.to_owned(),
        original_bytes,
        shown_bytes: shown.len(),
    }
}

fn truncate_to_char_boundary(value: &str, max_bytes: usize) -> &str {
    let mut end = max_bytes.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }

    &value[..end]
}
