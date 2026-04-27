use lfg::core::{AdapterCapability, AdapterCapabilityKind};
use lfg::ecosystems::npm::{NpmPackumentClient, NpmRegistryResolver};
use lfg::managers::npm::NpmManagerAdapter;

#[test]
fn built_in_contracts_can_be_described_as_protocol_capabilities() {
    let manager = NpmManagerAdapter;
    let resolver = NpmRegistryResolver::new(NeverPackumentClient);

    assert_eq!(
        AdapterCapability::from_manager_adapter(&manager),
        AdapterCapability {
            kind: AdapterCapabilityKind::ManagerIntegration,
            id: "npm".to_owned(),
        }
    );
    assert_eq!(
        AdapterCapability::from_release_resolver(&resolver),
        AdapterCapability {
            kind: AdapterCapabilityKind::EcosystemReleaseResolver,
            id: "npm-registry".to_owned(),
        }
    );
}

struct NeverPackumentClient;

impl NpmPackumentClient for NeverPackumentClient {
    fn fetch_packument(
        &self,
        _package_name: &str,
    ) -> Result<String, lfg::ecosystems::npm::NpmFetchError> {
        unreachable!("adapter protocol test does not fetch packuments")
    }
}
