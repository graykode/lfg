use serde::{Deserialize, Serialize};

use crate::core::{EcosystemReleaseResolver, ManagerIntegrationAdapter, Verdict};

pub const ADAPTER_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterCapability {
    pub kind: AdapterCapabilityKind,
    pub id: String,
}

impl AdapterCapability {
    pub fn manager_integration(id: impl Into<String>) -> Self {
        Self {
            kind: AdapterCapabilityKind::ManagerIntegration,
            id: id.into(),
        }
    }

    pub fn ecosystem_release_resolver(id: impl Into<String>) -> Self {
        Self {
            kind: AdapterCapabilityKind::EcosystemReleaseResolver,
            id: id.into(),
        }
    }

    pub fn llm_adapter(id: impl Into<String>) -> Self {
        Self {
            kind: AdapterCapabilityKind::LlmAdapter,
            id: id.into(),
        }
    }

    pub fn from_manager_adapter(adapter: &dyn ManagerIntegrationAdapter) -> Self {
        Self::manager_integration(adapter.id())
    }

    pub fn from_release_resolver(resolver: &dyn EcosystemReleaseResolver) -> Self {
        Self::ecosystem_release_resolver(resolver.id())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterCapabilityKind {
    ManagerIntegration,
    EcosystemReleaseResolver,
    LlmAdapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AdapterProtocolRequest {
    Handshake {
        protocol_version: u16,
        packvet_version: String,
    },
    Capabilities {
        protocol_version: u16,
    },
    ParseInstall {
        protocol_version: u16,
        manager_id: String,
        args: Vec<String>,
    },
    ResolveRelease {
        protocol_version: u16,
        resolver_id: String,
        target: AdapterProtocolInstallTarget,
    },
    Review {
        protocol_version: u16,
        provider_id: String,
        prompt: String,
        timeout_seconds: u64,
    },
}

impl AdapterProtocolRequest {
    pub fn handshake(packvet_version: impl Into<String>) -> Self {
        Self::Handshake {
            protocol_version: ADAPTER_PROTOCOL_VERSION,
            packvet_version: packvet_version.into(),
        }
    }

    pub const fn capabilities() -> Self {
        Self::Capabilities {
            protocol_version: ADAPTER_PROTOCOL_VERSION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AdapterProtocolResponse {
    HandshakeAccepted {
        protocol_version: u16,
        adapter_id: String,
        capabilities: Vec<AdapterCapability>,
    },
    Capabilities {
        protocol_version: u16,
        capabilities: Vec<AdapterCapability>,
    },
    InstallParsed {
        protocol_version: u16,
        request: AdapterProtocolInstallRequest,
        real_command: AdapterProtocolRealCommand,
    },
    ReleaseResolved {
        protocol_version: u16,
        releases: AdapterProtocolResolvedPackageReleases,
    },
    ReviewCompleted {
        protocol_version: u16,
        raw_output: String,
    },
    Error {
        protocol_version: u16,
        code: AdapterProtocolErrorCode,
        message: String,
        ask: bool,
    },
}

impl AdapterProtocolResponse {
    pub fn handshake_accepted(
        adapter_id: impl Into<String>,
        capabilities: Vec<AdapterCapability>,
    ) -> Self {
        Self::HandshakeAccepted {
            protocol_version: ADAPTER_PROTOCOL_VERSION,
            adapter_id: adapter_id.into(),
            capabilities,
        }
    }

    pub fn capabilities(capabilities: Vec<AdapterCapability>) -> Self {
        Self::Capabilities {
            protocol_version: ADAPTER_PROTOCOL_VERSION,
            capabilities,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterProtocolInstallRequest {
    pub manager_id: String,
    pub operation: AdapterProtocolInstallOperation,
    pub targets: Vec<AdapterProtocolInstallTarget>,
    pub manager_args: Vec<String>,
    pub release_resolver_id: String,
    pub release_decision_evaluator_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterProtocolInstallOperation {
    Add,
    Install,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterProtocolInstallTarget {
    pub spec: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterProtocolRealCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterProtocolArchiveRef {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterProtocolResolvedPackageRelease {
    pub version: String,
    pub published_at: String,
    pub archive: AdapterProtocolArchiveRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterProtocolResolvedPackageReleases {
    pub package_name: String,
    pub target: AdapterProtocolResolvedPackageRelease,
    pub previous: AdapterProtocolResolvedPackageRelease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterProtocolError {
    code: AdapterProtocolErrorCode,
    message: String,
}

impl AdapterProtocolError {
    pub fn new(code: AdapterProtocolErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub const fn verdict(&self) -> Verdict {
        Verdict::Ask
    }

    pub fn into_response(self) -> AdapterProtocolResponse {
        AdapterProtocolResponse::Error {
            protocol_version: ADAPTER_PROTOCOL_VERSION,
            code: self.code,
            message: self.message,
            ask: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterProtocolErrorCode {
    UnsupportedProtocolVersion,
    InvalidRequest,
    InvalidResponse,
    UnsupportedCommand,
    UnsupportedOption,
    Unavailable,
    Timeout,
    Failed,
}
