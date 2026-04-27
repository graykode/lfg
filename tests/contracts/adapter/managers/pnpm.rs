use lfg::core::{InstallOperation, InstallTarget, ManagerIntegrationAdapter, PackageManager};
use lfg::managers::pnpm::PnpmManagerAdapter;

#[test]
fn pnpm_manager_adapter_implements_common_contract() {
    let adapter = PnpmManagerAdapter;

    let request = adapter
        .parse_install(&["add".to_owned(), "left-pad".to_owned()])
        .expect("pnpm add should parse");

    assert_eq!(adapter.id(), "pnpm");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Pnpm);
    assert_eq!(request.operation, InstallOperation::Add);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "left-pad".to_owned()
        }]
    );
}
