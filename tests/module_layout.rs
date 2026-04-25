use lfg::core::{
    aggregate_verdicts, evaluate_install_request, evaluate_install_request_with_reviewer,
    ArchiveRef, InstallOperation, InstallRequest, InstallTarget, ManagerIntegrationAdapter,
    PackageManager, PackageOutcome, Registry, ReleaseDecisionEvaluator, ReleaseReviewer,
    SkipReason, UnavailableReleaseReviewer, Verdict,
};
use lfg::evidence::{
    read_tgz_source_tree, ArchiveFetcher, DiffEngine, HttpArchiveFetcher, SourceTree,
    UnifiedDiffEngine,
};
use lfg::managers::npm::{
    evaluate_npm_install_request, NpmManagerAdapter, NpmPackumentClient, NpmRegistryResolver,
    NpmReleaseDecisionEvaluator,
};
use lfg::providers::{
    parse_provider_output, ArchiveDiffReviewer, CommandReviewProvider, DiffReviewPromptBuilder,
    ProviderError, ReviewPrompt, ReviewProvider, UnavailableReviewProvider,
};

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
    let _ = ArchiveDiffReviewer::<HttpArchiveFetcher, UnifiedDiffEngine>::new;
    let _ = CommandReviewProvider::new("demo", "true", Vec::<String>::new());
    let _ = DiffReviewPromptBuilder;
    let _ = UnavailableReviewProvider;
    let _ = ReviewPrompt {
        text: String::new(),
    };
    let _ = ProviderError::Timeout;
    let _ = evaluate_npm_install_request::<NpmRegistryResolver<NeverPackumentClient>>;
    let _ = evaluate_install_request::<NeverResolver, NeverDecisionEvaluator>;
    let _ = evaluate_install_request_with_reviewer::<
        NeverResolver,
        NeverDecisionEvaluator,
        UnavailableReleaseReviewer,
    >;
    let _ = <NeverDecisionEvaluator as ReleaseDecisionEvaluator>::decide;
    let _ = <UnavailableReleaseReviewer as ReleaseReviewer>::review;
    let _ = <UnavailableReviewProvider as ReviewProvider>::review;
    let _ = request;
}

struct NeverPackumentClient;

impl NpmPackumentClient for NeverPackumentClient {
    fn fetch_packument(
        &self,
        _package_name: &str,
    ) -> Result<String, lfg::managers::npm::NpmFetchError> {
        unreachable!("module layout test does not fetch packuments")
    }
}

struct NeverResolver;

impl lfg::core::EcosystemReleaseResolver for NeverResolver {
    fn id(&self) -> &'static str {
        "never"
    }

    fn resolve(
        &self,
        _target: &InstallTarget,
    ) -> Result<lfg::core::ResolvedPackageReleases, lfg::core::ResolveError> {
        unreachable!("module layout test does not resolve packages")
    }
}

struct NeverDecisionEvaluator;

impl ReleaseDecisionEvaluator for NeverDecisionEvaluator {
    fn id(&self) -> &'static str {
        "never"
    }

    fn decide(
        &self,
        _releases: &lfg::core::ResolvedPackageReleases,
        _now: std::time::SystemTime,
    ) -> Result<lfg::core::ReviewDecision, lfg::core::ReleaseDecisionError> {
        unreachable!("module layout test does not evaluate releases")
    }
}
