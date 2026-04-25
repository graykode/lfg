#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallOperation {
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
