use lfg::core::contracts::{ArchiveRef, ManagerIntegrationAdapter};
use lfg::core::install_request::{InstallOperation, InstallRequest, InstallTarget, PackageManager};
use lfg::core::outcome::{aggregate_verdicts, PackageOutcome};
use lfg::core::policy::SkipReason;
use lfg::core::registry::Registry;
use lfg::core::review_pipeline::{evaluate_install_request, ReleaseDecisionEvaluator};
use lfg::core::verdict::Verdict;
use lfg::evidence::archive::read_tgz_source_tree;
use lfg::evidence::archive_diff::{ArchiveFetcher, HttpArchiveFetcher};
use lfg::evidence::source_diff::{DiffEngine, SourceTree, UnifiedDiffEngine};
use lfg::managers::npm::adapter::NpmManagerAdapter;
use lfg::managers::npm::policy::NpmReleaseDecisionEvaluator;
use lfg::managers::npm::registry::NpmRegistryResolver;
use lfg::managers::npm::review::evaluate_npm_install_request;
use lfg::providers::output::parse_provider_output;

#[test]
fn public_modules_are_grouped_by_role() {
    let _archive = ArchiveRef {
        url: "memory://demo.tgz".to_owned(),
    };

    let request = InstallRequest {
        manager: PackageManager::Npm,
        operation: InstallOperation::Install,
        targets: vec![InstallTarget {
            spec: "left-pad".to_owned(),
        }],
        manager_args: vec!["install".to_owned(), "left-pad".to_owned()],
    };

    let adapter = NpmManagerAdapter;
    assert_eq!(adapter.id(), "npm");
    assert_eq!(
        aggregate_verdicts(&[PackageOutcome::Skipped(SkipReason::OlderThanThreshold)]),
        Verdict::Pass
    );

    let mut registry = Registry::new();
    registry.register("npm", adapter).expect("register adapter");
    assert_eq!(registry.available_ids(), vec!["npm"]);

    let previous = SourceTree::from_text_files([("package/index.js", "module.exports = 1;\n")]);
    let target = SourceTree::from_text_files([("package/index.js", "module.exports = 2;\n")]);
    let diff = UnifiedDiffEngine
        .diff(&previous, &target)
        .expect("source diff renders");
    assert!(diff.text.contains("module.exports = 2;"));

    let parsed = parse_provider_output("verdict: ask\nreason: demo\n");
    assert_eq!(parsed.verdict, Verdict::Ask);

    let _ = read_tgz_source_tree;
    let _ = HttpArchiveFetcher;
    let _ = <HttpArchiveFetcher as ArchiveFetcher>::fetch;
    let _ = NpmRegistryResolver::<NeverPackumentClient>::new;
    let _ = NpmReleaseDecisionEvaluator::new;
    let _ = evaluate_npm_install_request::<NpmRegistryResolver<NeverPackumentClient>>;
    let _ = evaluate_install_request::<NeverResolver, NeverDecisionEvaluator>;
    let _ = <NeverDecisionEvaluator as ReleaseDecisionEvaluator>::decide;
    let _ = request;
}

struct NeverPackumentClient;

impl lfg::managers::npm::registry::NpmPackumentClient for NeverPackumentClient {
    fn fetch_packument(
        &self,
        _package_name: &str,
    ) -> Result<String, lfg::managers::npm::registry::NpmFetchError> {
        unreachable!("module layout test does not fetch packuments")
    }
}

struct NeverResolver;

impl lfg::core::contracts::EcosystemReleaseResolver for NeverResolver {
    fn id(&self) -> &'static str {
        "never"
    }

    fn resolve(
        &self,
        _target: &InstallTarget,
    ) -> Result<lfg::core::contracts::ResolvedPackageReleases, lfg::core::contracts::ResolveError>
    {
        unreachable!("module layout test does not resolve packages")
    }
}

struct NeverDecisionEvaluator;

impl ReleaseDecisionEvaluator for NeverDecisionEvaluator {
    fn decide(
        &self,
        _releases: &lfg::core::contracts::ResolvedPackageReleases,
        _now: std::time::SystemTime,
    ) -> Result<lfg::core::policy::ReviewDecision, lfg::core::review_pipeline::ReleaseDecisionError>
    {
        unreachable!("module layout test does not evaluate releases")
    }
}
