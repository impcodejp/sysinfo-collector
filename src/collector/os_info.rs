use crate::model::OsInfo;
use anyhow::Result;
use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_OperatingSystem")]
#[allow(non_snake_case)]
struct WmiOsInfo {
    Caption: Option<String>,
    BuildNumber: Option<String>,
    InstallDate: Option<String>,
    LastBootUpTime: Option<String>,
}

pub fn collect_os_info() -> Result<OsInfo> {
    let com = COMLibrary::new()?;
    let wmi = WMIConnection::new(com)?;

    let results: Vec<WmiOsInfo> = wmi.query()?;
    let wmi_os = results
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Win32_OperatingSystem が見つかりません"))?;

    let os_name = wmi_os.Caption.unwrap_or_else(|| "不明".to_string());
    let build_number = wmi_os.BuildNumber.unwrap_or_else(|| "不明".to_string());
    let install_date = parse_wmi_datetime(wmi_os.InstallDate.as_deref());
    let last_boot_raw = wmi_os.LastBootUpTime.as_deref().unwrap_or("").to_string();
    let last_boot_time = parse_wmi_datetime(wmi_os.LastBootUpTime.as_deref());
    let uptime = compute_uptime(&last_boot_raw);

    Ok(OsInfo {
        os_name,
        build_number,
        install_date,
        last_boot_time,
        uptime,
    })
}

/// WMI の日時文字列（"20250604143022.000000+540" 形式）を "YYYY-MM-DD HH:MM:SS" に変換する
fn parse_wmi_datetime(raw: Option<&str>) -> String {
    let raw = match raw {
        Some(s) if s.len() >= 14 => s,
        _ => return "不明".to_string(),
    };

    let year = &raw[0..4];
    let month = &raw[4..6];
    let day = &raw[6..8];
    let hour = &raw[8..10];
    let min = &raw[10..12];
    let sec = &raw[12..14];

    format!("{}-{}-{} {}:{}:{}", year, month, day, hour, min, sec)
}

/// 最終起動時刻から稼働時間を算出する
fn compute_uptime(last_boot_raw: &str) -> String {
    if last_boot_raw.len() < 14 {
        return "不明".to_string();
    }

    let parse = |s: &str| s.parse::<u64>().ok();

    let boot_year = match parse(&last_boot_raw[0..4]) {
        Some(v) => v,
        None => return "不明".to_string(),
    };
    let boot_month = match parse(&last_boot_raw[4..6]) {
        Some(v) => v,
        None => return "不明".to_string(),
    };
    let boot_day = match parse(&last_boot_raw[6..8]) {
        Some(v) => v,
        None => return "不明".to_string(),
    };
    let boot_hour = match parse(&last_boot_raw[8..10]) {
        Some(v) => v,
        None => return "不明".to_string(),
    };
    let boot_min = match parse(&last_boot_raw[10..12]) {
        Some(v) => v,
        None => return "不明".to_string(),
    };
    let boot_sec = match parse(&last_boot_raw[12..14]) {
        Some(v) => v,
        None => return "不明".to_string(),
    };

    use windows::Win32::System::SystemInformation::GetSystemTime;

    let now = unsafe { GetSystemTime() };

    let boot_days = days_from_ymd(boot_year, boot_month, boot_day);
    let now_days = days_from_ymd(now.wYear as u64, now.wMonth as u64, now.wDay as u64);

    let boot_secs = boot_days * 86400 + boot_hour * 3600 + boot_min * 60 + boot_sec;
    let now_secs = now_days * 86400
        + now.wHour as u64 * 3600
        + now.wMinute as u64 * 60
        + now.wSecond as u64;

    if now_secs < boot_secs {
        return "不明".to_string();
    }

    let elapsed = now_secs - boot_secs;
    let days = elapsed / 86400;
    let hours = (elapsed % 86400) / 3600;
    let mins = (elapsed % 3600) / 60;

    format!("{}日 {}時間 {}分", days, hours, mins)
}

/// 簡易ユリウス日相当（相対値として使用）
fn days_from_ymd(y: u64, m: u64, d: u64) -> u64 {
    let y = if m <= 2 { y - 1 } else { y };
    let m = if m <= 2 { m + 12 } else { m };
    365 * y + y / 4 - y / 100 + y / 400 + (306 * (m + 1)) / 10 + d - 428
}
