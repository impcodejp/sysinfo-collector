#[derive(Debug, Clone)]
pub struct IisComponentStatus {
    pub display_name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct IisStatus {
    pub installed: bool,
    pub version: Option<String>,
    /// IIS ロールサービスごとの有効/無効状態
    pub components: Vec<IisComponentStatus>,
}

impl Default for IisStatus {
    fn default() -> Self {
        Self {
            installed: false,
            version: None,
            components: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DotNetStatus {
    /// .NET Framework のインストール済みバージョン（例: "3.5", "4.8"）
    pub framework_versions: Vec<String>,
    /// .NET (Core/5+) のインストール済みバージョン（例: "6.0.25", "8.0.10"）
    pub core_versions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RdsStatus {
    pub rdp_enabled: bool,
    pub port: u16,
    pub nla_enabled: bool,
}

impl Default for RdsStatus {
    fn default() -> Self {
        Self {
            rdp_enabled: false,
            port: 3389,
            nla_enabled: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowsServiceStatus {
    pub display_name: String,
    /// true = 有効（StartMode が Disabled 以外）、false = 無効（StartMode が Disabled）
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WindowsFeaturesInfo {
    pub iis: IisStatus,
    pub dotnet: DotNetStatus,
    pub rds: RdsStatus,
    pub notable_services: Vec<WindowsServiceStatus>,
}
