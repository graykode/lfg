use lfg::core::{
    InstallOperation, InstallTarget, ManagerAdapterError, ManagerIntegrationAdapter, PackageManager,
};
use lfg::managers::npm::NpmManagerAdapter;

#[test]
fn npm_manager_adapter_implements_common_contract() {
    let adapter = NpmManagerAdapter;

    let request = adapter
        .parse_install(&["install".to_owned(), "left-pad".to_owned()])
        .expect("npm install should parse");

    assert_eq!(adapter.id(), "npm");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Npm);
    assert_eq!(request.operation, InstallOperation::Install);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "left-pad".to_owned()
        }]
    );
}

#[test]
fn npm_manager_adapter_uses_common_parse_errors() {
    let adapter = NpmManagerAdapter;

    assert_eq!(
        adapter.parse_install(&["run".to_owned(), "build".to_owned()]),
        Err(ManagerAdapterError::UnsupportedCommand("run".to_owned()))
    );
}
