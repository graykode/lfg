use lfg::builtins::built_in_release_decision_evaluators;
use lfg::core::ReviewPolicy;

#[test]
fn built_in_release_decision_evaluator_registry_contains_release_policies() {
    let policy = ReviewPolicy::default();
    let registry =
        built_in_release_decision_evaluators(&policy).expect("built-in evaluators register");

    assert_eq!(
        registry.available_ids(),
        vec![
            "npm-release-policy",
            "python-release-policy",
            "ruby-release-policy",
            "rust-release-policy"
        ]
    );

    let evaluator = registry
        .get("npm-release-policy")
        .expect("npm release decision evaluator");
    assert_eq!(evaluator.id(), "npm-release-policy");

    let evaluator = registry
        .get("python-release-policy")
        .expect("python release decision evaluator");
    assert_eq!(evaluator.id(), "python-release-policy");

    let evaluator = registry
        .get("rust-release-policy")
        .expect("rust release decision evaluator");
    assert_eq!(evaluator.id(), "rust-release-policy");

    let evaluator = registry
        .get("ruby-release-policy")
        .expect("ruby release decision evaluator");
    assert_eq!(evaluator.id(), "ruby-release-policy");
}
