use crate::builtins::built_in_manager_adapters;

#[test]
fn built_in_manager_registry_contains_manager_adapters() {
    let registry = built_in_manager_adapters().expect("built-in manager adapters register");

    assert_eq!(
        registry.available_ids(),
        vec!["bun", "cargo", "gem", "npm", "pip", "pnpm", "uv", "yarn"]
    );

    let adapter = registry.get("bun").expect("bun manager adapter");
    assert_eq!(adapter.id(), "bun");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );

    let request = adapter
        .parse_install(&["add".to_owned(), "left-pad".to_owned()])
        .expect("parse bun add");
    assert_eq!(request.targets[0].spec, "left-pad");

    let adapter = registry.get("cargo").expect("cargo manager adapter");
    assert_eq!(adapter.id(), "cargo");
    assert_eq!(adapter.release_resolver_id(), "crates-io-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "rust-release-policy"
    );

    let request = adapter
        .parse_install(&["add".to_owned(), "serde".to_owned()])
        .expect("parse cargo add");
    assert_eq!(request.targets[0].spec, "serde");

    let adapter = registry.get("gem").expect("gem manager adapter");
    assert_eq!(adapter.id(), "gem");
    assert_eq!(adapter.release_resolver_id(), "rubygems-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "ruby-release-policy"
    );

    let request = adapter
        .parse_install(&["install".to_owned(), "rack".to_owned()])
        .expect("parse gem install");
    assert_eq!(request.targets[0].spec, "rack");

    let adapter = registry.get("npm").expect("npm manager adapter");
    assert_eq!(adapter.id(), "npm");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );

    let request = adapter
        .parse_install(&["install".to_owned(), "left-pad".to_owned()])
        .expect("parse npm install");
    assert_eq!(request.targets[0].spec, "left-pad");

    let adapter = registry.get("pip").expect("pip manager adapter");
    assert_eq!(adapter.id(), "pip");
    assert_eq!(adapter.release_resolver_id(), "pypi-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "python-release-policy"
    );

    let request = adapter
        .parse_install(&["install".to_owned(), "requests".to_owned()])
        .expect("parse pip install");
    assert_eq!(request.targets[0].spec, "requests");

    let adapter = registry.get("pnpm").expect("pnpm manager adapter");
    assert_eq!(adapter.id(), "pnpm");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );

    let request = adapter
        .parse_install(&["add".to_owned(), "left-pad".to_owned()])
        .expect("parse pnpm add");
    assert_eq!(request.targets[0].spec, "left-pad");

    let adapter = registry.get("uv").expect("uv manager adapter");
    assert_eq!(adapter.id(), "uv");
    assert_eq!(adapter.release_resolver_id(), "pypi-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "python-release-policy"
    );

    let request = adapter
        .parse_install(&["add".to_owned(), "requests".to_owned()])
        .expect("parse uv add");
    assert_eq!(request.targets[0].spec, "requests");

    let adapter = registry.get("yarn").expect("yarn manager adapter");
    assert_eq!(adapter.id(), "yarn");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );

    let request = adapter
        .parse_install(&["add".to_owned(), "left-pad".to_owned()])
        .expect("parse yarn add");
    assert_eq!(request.targets[0].spec, "left-pad");
}
