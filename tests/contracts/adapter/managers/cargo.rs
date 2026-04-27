use lfg::core::{InstallOperation, InstallTarget, ManagerIntegrationAdapter, PackageManager};
use lfg::managers::cargo::CargoManagerAdapter;

#[test]
fn cargo_manager_adapter_implements_common_contract() {
    let adapter = CargoManagerAdapter;

    let request = adapter
        .parse_install(&["add".to_owned(), "serde".to_owned()])
        .expect("cargo add should parse");

    assert_eq!(adapter.id(), "cargo");
    assert_eq!(adapter.release_resolver_id(), "crates-io-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "rust-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Cargo);
    assert_eq!(request.operation, InstallOperation::Add);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "serde".to_owned()
        }]
    );
}
