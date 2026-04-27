pub mod adapter_protocol;
pub mod contracts;
pub mod execution;
pub mod install_assessment;
pub mod install_request;
pub mod outcome;
pub mod policy;
pub mod registry;
pub mod release_age;
pub mod verdict;

pub use adapter_protocol::*;
pub use contracts::*;
pub use execution::*;
pub use install_assessment::*;
pub use install_request::*;
pub use outcome::*;
pub use policy::*;
pub use registry::*;
pub use release_age::*;
pub use verdict::*;

#[cfg(test)]
mod install_assessment_tests;
