use crate::model::{DomainInfo, DomainRole};
use anyhow::Result;
use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ComputerSystem")]
#[allow(non_snake_case)]
struct WmiComputerSystem {
    Domain: Option<String>,
    DomainRole: Option<u32>,
}

pub fn collect_domain() -> Result<DomainInfo> {
    let com = COMLibrary::new()?;
    let wmi = WMIConnection::new(com)?;

    let results: Vec<WmiComputerSystem> = wmi.query()?;
    let cs = results
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Win32_ComputerSystem が見つかりません"))?;

    let name = cs.Domain.unwrap_or_else(|| "不明".to_string());
    let role = cs
        .DomainRole
        .and_then(DomainRole::from_u32)
        .unwrap_or(DomainRole::StandaloneWorkstation);

    Ok(DomainInfo { name, role })
}
