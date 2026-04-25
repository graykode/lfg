use crate::core::contracts::{EcosystemReleaseResolver, ManagerIntegrationAdapter};
use crate::core::registry::{Registry, RegistryError};
use crate::managers::npm::adapter::NpmManagerAdapter;
use crate::managers::npm::registry::{NpmHttpPackumentClient, NpmRegistryResolver};

pub type ManagerAdapterRegistry = Registry<Box<dyn ManagerIntegrationAdapter>>;
pub type ReleaseResolverRegistry = Registry<Box<dyn EcosystemReleaseResolver>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterConfig {
    pub npm_registry_base_url: String,
}

pub fn built_in_manager_adapters() -> Result<ManagerAdapterRegistry, RegistryError> {
    let mut registry = Registry::new();
    let adapter: Box<dyn ManagerIntegrationAdapter> = Box::new(NpmManagerAdapter);
    let id = adapter.id();

    registry.register(id, adapter)?;

    Ok(registry)
}

pub fn built_in_release_resolvers(
    config: AdapterConfig,
) -> Result<ReleaseResolverRegistry, RegistryError> {
    let mut registry = Registry::new();
    let resolver: Box<dyn EcosystemReleaseResolver> = Box::new(NpmRegistryResolver::new(
        NpmHttpPackumentClient::new(config.npm_registry_base_url),
    ));
    let id = resolver.id();

    registry.register(id, resolver)?;

    Ok(registry)
}
