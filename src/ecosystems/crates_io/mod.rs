mod policy;
mod registry;

pub use policy::RustReleaseDecisionEvaluator;
pub use registry::{
    CratesIoCrateClient, CratesIoFetchError, CratesIoHttpCrateClient, CratesIoRegistryResolver,
};

#[cfg(test)]
mod http_tests;
#[cfg(test)]
mod registry_tests;
