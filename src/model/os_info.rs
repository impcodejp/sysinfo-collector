#[derive(Debug, Clone, Default)]
pub struct OsInfo {
    pub os_name: String,
    pub build_number: String,
    pub install_date: String,
    pub last_boot_time: String,
    pub uptime: String,
}
