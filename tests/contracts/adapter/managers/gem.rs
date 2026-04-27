use lfg::core::{InstallOperation, InstallTarget, ManagerIntegrationAdapter, PackageManager};
use lfg::managers::gem::GemManagerAdapter;

#[test]
fn gem_manager_adapter_implements_common_contract() {
    let adapter = GemManagerAdapter;

    let request = adapter
        .parse_install(&["install".to_owned(), "rack".to_owned()])
        .expect("gem install should parse");

    assert_eq!(adapter.id(), "gem");
    assert_eq!(adapter.release_resolver_id(), "rubygems-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "ruby-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Gem);
    assert_eq!(request.operation, InstallOperation::Install);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "rack".to_owned()
        }]
    );
}
