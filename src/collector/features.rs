use crate::model::windows_features::{
    DotNetStatus, IisComponentStatus, IisStatus, RdsStatus, WindowsFeaturesInfo,
    WindowsServiceStatus,
};
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;
use wmi::{COMLibrary, WMIConnection};

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_Service")]
#[allow(non_snake_case)]
struct WmiService {
    Name: Option<String>,
    StartMode: Option<String>,
}

/// 確認する主要Windowsサービス（サービス名, 表示名）
/// ※ IIS(w3svc) は専用セクションで表示するためここには含めない
const NOTABLE_SERVICES: &[(&str, &str)] = &[
    ("dns",           "DNS Server"),
    ("dhcpserver",    "DHCP Server"),
    ("ntds",          "Active Directory ドメインサービス (AD DS)"),
    ("adfs",          "AD フェデレーションサービス (ADFS)"),
    ("certsvc",       "Active Directory 証明書サービス (ADCS)"),
    ("vmms",          "Hyper-V"),
    ("winrm",         "Windows リモート管理 (WinRM)"),
    ("mssqlserver",   "SQL Server (MSSQLSERVER)"),
    ("wsusservice",   "WSUS"),
    ("clustersvc",    "フェールオーバークラスター"),
    ("dfsr",          "DFS レプリケーション"),
    ("smtpsvc",       "SMTP サーバー"),
    ("msftpsvc",      "FTP サービス (IIS)"),
    ("termservice",   "リモートデスクトップサービス"),
];

pub fn collect_windows_features() -> Result<WindowsFeaturesInfo> {
    // 全サービスを一括取得してHashMapに格納
    let service_map = collect_service_states()?;

    let iis = collect_iis_status(&service_map);
    let dotnet = collect_dotnet_status();
    let rds = collect_rds_status();
    let notable_services = build_notable_services(&service_map);

    Ok(WindowsFeaturesInfo {
        iis,
        dotnet,
        rds,
        notable_services,
    })
}

/// サービス名（小文字）→ enabled（StartMode が "Disabled" でなければ true）のマップを返す
fn collect_service_states() -> Result<HashMap<String, bool>> {
    let com = COMLibrary::new()?;
    let wmi = WMIConnection::new(com)?;
    let services: Vec<WmiService> = wmi.query()?;

    let map = services
        .into_iter()
        .filter_map(|s| s.Name.map(|name| (name, s.StartMode)))
        .map(|(name, start_mode)| {
            let enabled = start_mode
                .as_deref()
                .map(|m| !m.eq_ignore_ascii_case("Disabled"))
                .unwrap_or(true);
            (name.to_lowercase(), enabled)
        })
        .collect();

    Ok(map)
}

/// 確認する IIS ロールサービス（レジストリ値名, 表示名）
/// HKLM\SOFTWARE\Microsoft\InetStp\Components 配下の DWORD 値で有効/無効を判定する
const IIS_COMPONENTS: &[(&str, &str)] = &[
    ("WindowsAuthentication", "Windows 認証"),
    ("RequestMonitor",        "要求の監視"),
    ("ASPNET",                "ASP.NET 3.5"),
    ("Metabase",              "IIS 6 メタベース互換"),
    ("WMICompatibility",      "IIS 6 WMI 互換"),
];

fn collect_iis_status(_service_map: &HashMap<String, bool>) -> IisStatus {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let Ok(iis_key) = hklm.open_subkey_with_flags("SOFTWARE\\Microsoft\\InetStp", KEY_READ)
    else {
        return IisStatus::default();
    };

    let version = {
        let major: u32 = iis_key.get_value("MajorVersion").ok().unwrap_or(0);
        let minor: u32 = iis_key.get_value("MinorVersion").ok().unwrap_or(0);
        if major > 0 {
            Some(format!("{}.{}", major, minor))
        } else {
            None
        }
    };

    let components = collect_iis_components();

    IisStatus {
        installed: true,
        version,
        components,
    }
}

fn collect_iis_components() -> Vec<IisComponentStatus> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let components_key = hklm
        .open_subkey_with_flags("SOFTWARE\\Microsoft\\InetStp\\Components", KEY_READ)
        .ok();

    IIS_COMPONENTS
        .iter()
        .map(|(reg_name, display_name)| {
            let enabled = components_key
                .as_ref()
                .and_then(|k| k.get_value::<u32, _>(reg_name).ok())
                .map(|v| v != 0)
                .unwrap_or(false);
            IisComponentStatus {
                display_name: display_name.to_string(),
                enabled,
            }
        })
        .collect()
}

fn collect_dotnet_status() -> DotNetStatus {
    DotNetStatus {
        framework_versions: collect_dotnet_framework_versions(),
        core_versions: collect_dotnet_core_versions(),
    }
}

fn collect_dotnet_framework_versions() -> Vec<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let ndp = match hklm
        .open_subkey_with_flags("SOFTWARE\\Microsoft\\NET Framework Setup\\NDP", KEY_READ)
    {
        Ok(k) => k,
        Err(_) => return vec![],
    };

    let mut versions = Vec::new();

    // v2.0 / v3.0 / v3.5
    for (reg_key, display) in &[
        ("v2.0.50727", "2.0"),
        ("v3.0",       "3.0"),
        ("v3.5",       "3.5"),
    ] {
        if let Ok(key) = ndp.open_subkey_with_flags(reg_key, KEY_READ) {
            let installed: u32 = key.get_value("Install").unwrap_or(0);
            if installed == 1 {
                versions.push(display.to_string());
            }
        }
    }

    // v4.x はReleaseキーの値でバージョン判定
    if let Ok(key) = ndp.open_subkey_with_flags("v4\\Full", KEY_READ) {
        let release: u32 = key.get_value("Release").unwrap_or(0);
        if release > 0 {
            versions.push(release_to_dotnet4_version(release));
        }
    }

    versions
}

fn release_to_dotnet4_version(release: u32) -> String {
    match release {
        r if r >= 533320 => "4.8.1",
        r if r >= 528040 => "4.8",
        r if r >= 461808 => "4.7.2",
        r if r >= 461308 => "4.7.1",
        r if r >= 460798 => "4.7",
        r if r >= 394802 => "4.6.2",
        r if r >= 394254 => "4.6.1",
        r if r >= 393295 => "4.6",
        r if r >= 379893 => "4.5.2",
        r if r >= 378675 => "4.5.1",
        _               => "4.5",
    }
    .to_string()
}

fn collect_dotnet_core_versions() -> Vec<String> {
    // dotnetランタイムのインストールディレクトリを確認
    let dotnet_dir =
        Path::new(r"C:\Program Files\dotnet\shared\Microsoft.NETCore.App");
    if !dotnet_dir.exists() {
        return vec![];
    }

    let mut versions: Vec<String> = std::fs::read_dir(dotnet_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    versions.sort();
    versions
}

fn collect_rds_status() -> RdsStatus {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let ts_key = match hklm.open_subkey_with_flags(
        "SYSTEM\\CurrentControlSet\\Control\\Terminal Server",
        KEY_READ,
    ) {
        Ok(k) => k,
        Err(_) => return RdsStatus::default(),
    };

    let deny: u32 = ts_key.get_value("fDenyTSConnections").unwrap_or(1);
    let rdp_enabled = deny == 0;

    let rdp_tcp = hklm.open_subkey_with_flags(
        "SYSTEM\\CurrentControlSet\\Control\\Terminal Server\\WinStations\\RDP-Tcp",
        KEY_READ,
    );

    let port = rdp_tcp
        .as_ref()
        .ok()
        .and_then(|k| k.get_value::<u32, _>("PortNumber").ok())
        .unwrap_or(3389) as u16;

    let nla_enabled = rdp_tcp
        .as_ref()
        .ok()
        .and_then(|k| k.get_value::<u32, _>("UserAuthentication").ok())
        .map(|v| v != 0)
        .unwrap_or(false);

    RdsStatus {
        rdp_enabled,
        port,
        nla_enabled,
    }
}

fn build_notable_services(service_map: &HashMap<String, bool>) -> Vec<WindowsServiceStatus> {
    // サービスマップに存在するもの（インストール済み）のみ返す
    NOTABLE_SERVICES
        .iter()
        .filter_map(|(svc_name, display_name)| {
            service_map.get(*svc_name).map(|&enabled| WindowsServiceStatus {
                display_name: display_name.to_string(),
                enabled,
            })
        })
        .collect()
}
