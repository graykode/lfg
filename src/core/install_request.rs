#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Cargo,
    Gem,
    Npm,
    Pip,
    Pnpm,
    Uv,
    Yarn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallOperation {
    Add,
    Install,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallTarget {
    pub spec: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallRequest {
    pub manager: PackageManager,
    pub operation: InstallOperation,
    pub targets: Vec<InstallTarget>,
    pub manager_args: Vec<String>,
}
