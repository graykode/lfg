use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::core::{ResolvedPackageReleases, Verdict};
use crate::providers::{ProviderReview, ReviewPrompt};

pub fn write_provider_review_log(
    releases: &ResolvedPackageReleases,
    provider_id: &str,
    prompt: &ReviewPrompt,
    provider_output: &str,
    review: &ProviderReview,
) -> Result<(), std::io::Error> {
    let Some(log_dir) = review_log_dir() else {
        return Ok(());
    };
    fs::create_dir_all(&log_dir)?;

    let record = json!({
        "timestamp_unix_seconds": current_unix_seconds(),
        "provider_id": provider_id,
        "package": &releases.package_name,
        "previous": {
            "version": &releases.previous.version,
            "published_at": &releases.previous.published_at,
            "archive_url": &releases.previous.archive.url,
        },
        "target": {
            "version": &releases.target.version,
            "published_at": &releases.target.published_at,
            "archive_url": &releases.target.archive.url,
        },
        "prompt": &prompt.text,
        "provider_output": provider_output,
        "verdict": verdict_label(review.verdict),
        "reason": &review.reason,
        "evidence": review.evidence.iter().map(|item| {
            json!({
                "path": &item.path,
                "signal": &item.signal,
            })
        }).collect::<Vec<_>>(),
    });

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("reviews.jsonl"))?;
    writeln!(file, "{record}")
}

fn review_log_dir() -> Option<PathBuf> {
    if let Ok(value) = env::var("PACKVET_REVIEW_LOG_DIR") {
        return Some(PathBuf::from(value));
    }

    #[cfg(test)]
    {
        return None;
    }

    #[cfg(not(test))]
    {
        env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".packvet").join("reviews"))
    }
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn verdict_label(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Pass => "pass",
        Verdict::Ask => "ask",
        Verdict::Block => "block",
    }
}
