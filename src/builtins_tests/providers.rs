use std::env;

use crate::builtins::{
    built_in_review_provider, built_in_review_provider_with_preference, ProgramDetector,
    ReviewProviderPreference,
};
use crate::providers::ReviewProvider;

#[derive(Debug, Clone, Copy)]
struct StaticProgramDetector {
    claude: bool,
    codex: bool,
}

impl ProgramDetector for StaticProgramDetector {
    fn is_available(&self, program: &str) -> bool {
        match program {
            "claude" => self.claude,
            "codex" => self.codex,
            _ => false,
        }
    }
}

#[test]
fn built_in_review_provider_prefers_claude_before_codex() {
    let provider = built_in_review_provider_with_preference(
        ReviewProviderPreference::Auto,
        &StaticProgramDetector {
            claude: true,
            codex: true,
        },
    );

    assert_eq!(provider.id(), "claude-cli");
}

#[test]
fn built_in_review_provider_uses_codex_when_claude_is_missing() {
    let provider = built_in_review_provider_with_preference(
        ReviewProviderPreference::Auto,
        &StaticProgramDetector {
            claude: false,
            codex: true,
        },
    );

    assert_eq!(provider.id(), "codex-cli");
}

#[test]
fn built_in_review_provider_returns_unavailable_when_no_local_provider_exists() {
    let provider = built_in_review_provider_with_preference(
        ReviewProviderPreference::Auto,
        &StaticProgramDetector {
            claude: false,
            codex: false,
        },
    );

    assert_eq!(provider.id(), "unavailable");
}

#[test]
fn configured_review_provider_returns_unavailable_when_missing() {
    let provider = built_in_review_provider_with_preference(
        ReviewProviderPreference::ClaudeCli,
        &StaticProgramDetector {
            claude: false,
            codex: true,
        },
    );

    assert_eq!(provider.id(), "unavailable");
}

#[test]
fn built_in_review_provider_can_be_disabled_by_env() {
    let previous = env::var_os("LFG_REVIEW_PROVIDER");
    env::set_var("LFG_REVIEW_PROVIDER", "none");

    let provider = built_in_review_provider(&StaticProgramDetector {
        claude: true,
        codex: true,
    });

    match previous {
        Some(value) => env::set_var("LFG_REVIEW_PROVIDER", value),
        None => env::remove_var("LFG_REVIEW_PROVIDER"),
    }

    assert_eq!(provider.id(), "unavailable");
}
