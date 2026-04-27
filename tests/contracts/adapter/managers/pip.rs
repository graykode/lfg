use lfg::core::{InstallOperation, InstallTarget, ManagerIntegrationAdapter, PackageManager};
use lfg::managers::pip::PipManagerAdapter;

#[test]
fn pip_manager_adapter_implements_common_contract() {
    let adapter = PipManagerAdapter;

    let request = adapter
        .parse_install(&["install".to_owned(), "requests".to_owned()])
        .expect("pip install should parse");

    assert_eq!(adapter.id(), "pip");
    assert_eq!(adapter.release_resolver_id(), "pypi-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "python-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Pip);
    assert_eq!(request.operation, InstallOperation::Install);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "requests".to_owned()
        }]
    );
}
