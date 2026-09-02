use super::{DomainInfo, DriveInfo, HardwareInfo, NetworkInfo, OsInfo, TaskInfo, WindowsFeaturesInfo};

pub type CollectResult<T> = Result<T, String>;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub collected_at: String,
    pub network: CollectResult<NetworkInfo>,
    pub os: CollectResult<OsInfo>,
    pub hardware: CollectResult<HardwareInfo>,
    pub drives: CollectResult<Vec<DriveInfo>>,
    pub windows_features: CollectResult<WindowsFeaturesInfo>,
    pub tasks: CollectResult<Vec<TaskInfo>>,
    pub domain: CollectResult<DomainInfo>,
}

impl SystemInfo {
    pub fn new(collected_at: String) -> Self {
        Self {
            collected_at,
            network: Err("未収集".to_string()),
            os: Err("未収集".to_string()),
            hardware: Err("未収集".to_string()),
            drives: Err("未収集".to_string()),
            windows_features: Err("未収集".to_string()),
            tasks: Err("未収集".to_string()),
            domain: Err("未収集".to_string()),
        }
    }

    pub fn hostname(&self) -> &str {
        match &self.network {
            Ok(n) => &n.hostname,
            Err(_) => "UNKNOWN",
        }
    }
}
