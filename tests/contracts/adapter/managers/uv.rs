use lfg::core::{InstallOperation, InstallTarget, ManagerIntegrationAdapter, PackageManager};
use lfg::managers::uv::UvManagerAdapter;

#[test]
fn uv_manager_adapter_implements_common_contract() {
    let adapter = UvManagerAdapter;

    let request = adapter
        .parse_install(&["add".to_owned(), "requests".to_owned()])
        .expect("uv add should parse");

    assert_eq!(adapter.id(), "uv");
    assert_eq!(adapter.release_resolver_id(), "pypi-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "python-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Uv);
    assert_eq!(request.operation, InstallOperation::Add);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "requests".to_owned()
        }]
    );
}
