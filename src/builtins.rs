use std::env;

use crate::core::{EcosystemReleaseResolver, ManagerIntegrationAdapter};
use crate::core::{Registry, RegistryError};
use crate::core::{ReleaseDecisionEvaluator, ReviewPolicy};
use crate::managers::npm::NpmManagerAdapter;
use crate::managers::npm::NpmReleaseDecisionEvaluator;
use crate::managers::npm::{NpmHttpPackumentClient, NpmRegistryResolver};

pub type ManagerAdapterRegistry = Registry<Box<dyn ManagerIntegrationAdapter>>;
pub type ReleaseResolverRegistry = Registry<Box<dyn EcosystemReleaseResolver>>;
pub type ReleaseDecisionEvaluatorRegistry<'a> = Registry<Box<dyn ReleaseDecisionEvaluator + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterConfig {
    pub npm_registry_base_url: String,
}

impl AdapterConfig {
    pub fn from_env() -> Self {
        Self {
            npm_registry_base_url: env::var("LFG_NPM_REGISTRY_URL")
                .unwrap_or_else(|_| "https://registry.npmjs.org".to_owned()),
        }
    }
}

pub fn built_in_manager_adapters() -> Result<ManagerAdapterRegistry, RegistryError> {
    let mut registry = Registry::new();
    let adapter: Box<dyn ManagerIntegrationAdapter> = Box::new(NpmManagerAdapter);
    let id = adapter.id();

    registry.register(id, adapter)?;

    Ok(registry)
}

pub fn built_in_release_decision_evaluators<'a>(
    policy: &'a ReviewPolicy,
) -> Result<ReleaseDecisionEvaluatorRegistry<'a>, RegistryError> {
    let mut registry = Registry::new();
    let evaluator: Box<dyn ReleaseDecisionEvaluator + 'a> =
        Box::new(NpmReleaseDecisionEvaluator::new(policy));
    let id = evaluator.id();

    registry.register(id, evaluator)?;

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
