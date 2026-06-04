#[derive(Debug, Clone, Default)]
pub struct AdapterInfo {
    pub name: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub subnet_mask: Option<String>,
    pub gateway: Option<String>,
    pub mac_address: Option<String>,
    pub dns_primary: Option<String>,
    pub dns_secondary: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkInfo {
    pub hostname: String,
    pub adapters: Vec<AdapterInfo>,
}
