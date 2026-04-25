use crate::core::ManagerIntegrationAdapter;
use crate::managers::cargo::CargoManagerAdapter;
use crate::managers::gem::GemManagerAdapter;
use crate::managers::npm::NpmManagerAdapter;
use crate::managers::pip::PipManagerAdapter;
use crate::managers::pnpm::PnpmManagerAdapter;
use crate::managers::uv::UvManagerAdapter;
use crate::managers::yarn::YarnManagerAdapter;

macro_rules! manager_adapter_catalog {
    ($($adapter:expr),+ $(,)?) => {
        pub fn built_in_manager_adapter_catalog() -> Vec<Box<dyn ManagerIntegrationAdapter>> {
            vec![
                $(Box::new($adapter) as Box<dyn ManagerIntegrationAdapter>),+
            ]
        }
    };
}

manager_adapter_catalog![
    CargoManagerAdapter,
    GemManagerAdapter,
    NpmManagerAdapter,
    PipManagerAdapter,
    PnpmManagerAdapter,
    UvManagerAdapter,
    YarnManagerAdapter,
];
