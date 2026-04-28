use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::providers::{CommandReviewProvider, ProviderError, ReviewPrompt, ReviewProvider};

fn temp_prompt_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("packvet-provider-prompt-{nanos}.txt"))
}

#[test]
fn command_provider_sends_prompt_to_stdin_and_returns_stdout() {
    let prompt_path = temp_prompt_path();
    let provider = CommandReviewProvider::new(
        "fixture",
        "sh",
        [
            "-c".to_owned(),
            "cat > \"$1\"; printf 'verdict: block\nreason: fixture blocked\n\nevidence:\n- package.json: fixture signal\n'"
                .to_owned(),
            "sh".to_owned(),
            prompt_path.to_string_lossy().into_owned(),
        ],
    );

    let output = provider
        .review(&ReviewPrompt {
            text: "review this diff".to_owned(),
        })
        .expect("provider command should run");

    assert_eq!(
        output,
        "verdict: block\nreason: fixture blocked\n\nevidence:\n- package.json: fixture signal\n"
    );
    assert_eq!(
        fs::read_to_string(&prompt_path).expect("prompt should be captured"),
        "review this diff"
    );

    fs::remove_file(prompt_path).expect("remove captured prompt");
}

#[test]
fn command_provider_maps_missing_command_to_unavailable() {
    let provider = CommandReviewProvider::new(
        "missing",
        "definitely-missing-packvet-provider-command",
        Vec::<String>::new(),
    );

    assert!(matches!(
        provider.review(&ReviewPrompt {
            text: "review this diff".to_owned(),
        }),
        Err(ProviderError::Unavailable(_))
    ));
}

#[test]
fn command_provider_maps_nonzero_exit_to_failure() {
    let provider = CommandReviewProvider::new(
        "failing",
        "sh",
        ["-c".to_owned(), "printf 'bad' >&2; exit 7".to_owned()],
    );

    assert!(matches!(
        provider.review(&ReviewPrompt {
            text: "review this diff".to_owned(),
        }),
        Err(ProviderError::Failure(_))
    ));
}

#[test]
fn command_provider_maps_timeout_to_timeout_error() {
    let provider = CommandReviewProvider::with_timeout(
        "slow",
        "sh",
        ["-c".to_owned(), "sleep 1".to_owned()],
        Duration::from_millis(20),
    );

    assert_eq!(
        provider.review(&ReviewPrompt {
            text: "review this diff".to_owned(),
        }),
        Err(ProviderError::Timeout)
    );
}
