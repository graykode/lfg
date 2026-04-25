use lfg::core::{
    aggregate_verdicts, evaluate_install_request, evaluate_install_request_with_reviewer,
    ArchiveRef, InstallOperation, InstallRequest, InstallTarget, ManagerIntegrationAdapter,
    PackageManager, PackageOutcome, Registry, ReleaseDecisionEvaluator, ReleaseReviewer,
    SkipReason, UnavailableReleaseReviewer, Verdict,
};
use lfg::ecosystems::crates_io::{
    CratesIoCrateClient, CratesIoHttpCrateClient, CratesIoRegistryResolver,
    RustReleaseDecisionEvaluator,
};
use lfg::ecosystems::npm::{
    evaluate_npm_install_request, NpmPackumentClient, NpmRegistryResolver,
    NpmReleaseDecisionEvaluator,
};
use lfg::ecosystems::pypi::{
    PypiHttpProjectClient, PypiProjectClient, PypiRegistryResolver, PythonReleaseDecisionEvaluator,
};
use lfg::ecosystems::rubygems::{
    RubyGemsHttpVersionsClient, RubyGemsRegistryResolver, RubyGemsVersionsClient,
    RubyReleaseDecisionEvaluator,
};
use lfg::evidence::{
    read_tgz_source_tree, ArchiveFetcher, DiffEngine, HttpArchiveFetcher, SourceTree,
    UnifiedDiffEngine,
};
use lfg::managers::cargo::CargoManagerAdapter;
use lfg::managers::gem::GemManagerAdapter;
use lfg::managers::npm::NpmManagerAdapter;
use lfg::managers::pip::PipManagerAdapter;
use lfg::managers::pnpm::PnpmManagerAdapter;
use lfg::managers::uv::UvManagerAdapter;
use lfg::managers::yarn::YarnManagerAdapter;
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
    let _ = CargoManagerAdapter;
    let _ = CratesIoRegistryResolver::new(NeverCrateClient, "https://crates.io");
    let _ = CratesIoHttpCrateClient::new("https://crates.io");
    let _ = RustReleaseDecisionEvaluator::new;
    let _ = GemManagerAdapter;
    let _ = RubyGemsRegistryResolver::new(NeverVersionsClient, "https://rubygems.org");
    let _ = RubyGemsHttpVersionsClient::new("https://rubygems.org");
    let _ = RubyReleaseDecisionEvaluator::new;
    let _ = NpmRegistryResolver::<NeverPackumentClient>::new;
    let _ = NpmReleaseDecisionEvaluator::new;
    let _ = PnpmManagerAdapter;
    let _ = PipManagerAdapter;
    let _ = UvManagerAdapter;
    let _ = YarnManagerAdapter;
    let _ = PypiRegistryResolver::<NeverProjectClient>::new;
    let _ = PypiHttpProjectClient::new("https://pypi.org");
    let _ = PythonReleaseDecisionEvaluator::new;
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

struct NeverCrateClient;

impl CratesIoCrateClient for NeverCrateClient {
    fn fetch_crate(
        &self,
        _crate_name: &str,
    ) -> Result<String, lfg::ecosystems::crates_io::CratesIoFetchError> {
        unreachable!("module layout test does not fetch crates")
    }
}

impl NpmPackumentClient for NeverPackumentClient {
    fn fetch_packument(
        &self,
        _package_name: &str,
    ) -> Result<String, lfg::ecosystems::npm::NpmFetchError> {
        unreachable!("module layout test does not fetch packuments")
    }
}

struct NeverProjectClient;

struct NeverVersionsClient;

impl RubyGemsVersionsClient for NeverVersionsClient {
    fn fetch_versions(
        &self,
        _gem_name: &str,
    ) -> Result<String, lfg::ecosystems::rubygems::RubyGemsFetchError> {
        unreachable!("module layout test does not fetch RubyGems versions")
    }
}

impl PypiProjectClient for NeverProjectClient {
    fn fetch_project(
        &self,
        _package_name: &str,
    ) -> Result<String, lfg::ecosystems::pypi::PypiFetchError> {
        unreachable!("module layout test does not fetch PyPI projects")
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
