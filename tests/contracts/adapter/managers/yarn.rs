use lfg::core::{InstallOperation, InstallTarget, ManagerIntegrationAdapter, PackageManager};
use lfg::managers::yarn::YarnManagerAdapter;

#[test]
fn yarn_manager_adapter_implements_common_contract() {
    let adapter = YarnManagerAdapter;

    let request = adapter
        .parse_install(&["add".to_owned(), "left-pad".to_owned()])
        .expect("yarn add should parse");

    assert_eq!(adapter.id(), "yarn");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Yarn);
    assert_eq!(request.operation, InstallOperation::Add);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "left-pad".to_owned()
        }]
    );
}
