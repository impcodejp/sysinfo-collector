use crate::model::{DriveInfo, HardwareInfo, NetworkInfo, OsInfo, SystemInfo, TaskInfo};
use crate::model::windows_features::WindowsFeaturesInfo;

const SEPARATOR: &str = "================================================================";
const INNER_SEP: &str = "----------------------------------------------------------------";

pub struct TextFormatter;

impl TextFormatter {
    pub fn format(info: &SystemInfo) -> String {
        let mut out = String::with_capacity(8192);

        write_header(&mut out, info);
        write_os_section(&mut out, &info.os);
        write_network_section(&mut out, &info.network);
        write_hardware_section(&mut out, &info.hardware);
        write_drive_section(&mut out, &info.drives);
        write_windows_features_section(&mut out, &info.windows_features);
        write_task_section(&mut out, &info.tasks);
        write_footer(&mut out);

        out
    }
}

fn write_header(out: &mut String, info: &SystemInfo) {
    out.push_str(SEPARATOR);
    out.push('\n');
    out.push_str("  システム情報収集レポート\n");
    out.push_str(&format!("  収集日時: {}\n", info.collected_at));
    out.push_str(&format!("  ホスト名 : {}\n", info.hostname()));
    out.push_str(SEPARATOR);
    out.push('\n');
    out.push('\n');
}

fn write_os_section(out: &mut String, os: &Result<OsInfo, String>) {
    out.push_str("[OS情報]\n");
    match os {
        Ok(o) => {
            out.push_str(&format!("  OS名称     : {}\n", o.os_name));
            out.push_str(&format!("  ビルド番号  : {}\n", o.build_number));
            out.push_str(&format!("  インストール: {}\n", o.install_date));
            out.push_str(&format!("  起動日時   : {} (稼働 {})\n", o.last_boot_time, o.uptime));
        }
        Err(e) => {
            out.push_str(&format!("  取得失敗: {}\n", e));
        }
    }
    out.push('\n');
}

fn write_network_section(out: &mut String, network: &Result<NetworkInfo, String>) {
    out.push_str("[ネットワーク情報]\n");
    match network {
        Ok(n) => {
            out.push_str(&format!("  ホスト名: {}\n", n.hostname));
            for adapter in &n.adapters {
                out.push_str(&format!("\n  アダプター: {}\n", adapter.name));
                if let Some(v) = &adapter.ipv4 {
                    out.push_str(&format!("    IPアドレス (IPv4) : {}\n", v));
                }
                if let Some(v) = &adapter.ipv6 {
                    out.push_str(&format!("    IPアドレス (IPv6) : {}\n", v));
                }
                if let Some(v) = &adapter.subnet_mask {
                    out.push_str(&format!("    サブネットマスク   : {}\n", v));
                }
                if let Some(v) = &adapter.gateway {
                    out.push_str(&format!("    ゲートウェイ      : {}\n", v));
                }
                if let Some(v) = &adapter.mac_address {
                    out.push_str(&format!("    MACアドレス       : {}\n", v));
                }
                match (&adapter.dns_primary, &adapter.dns_secondary) {
                    (Some(p), Some(s)) => {
                        out.push_str(&format!("    DNSサーバー       : {} / {}\n", p, s));
                    }
                    (Some(p), None) => {
                        out.push_str(&format!("    DNSサーバー       : {}\n", p));
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            out.push_str(&format!("  取得失敗: {}\n", e));
        }
    }
    out.push('\n');
}

fn write_hardware_section(out: &mut String, hardware: &Result<HardwareInfo, String>) {
    out.push_str("[ハードウェア情報]\n");
    match hardware {
        Ok(h) => {
            out.push_str(&format!(
                "  CPU : {} ({:.0} MHz)\n",
                h.cpu_name, h.cpu_clock_mhz
            ));
            out.push_str(&format!(
                "        コア数: {} / 論理プロセッサ数: {}\n",
                h.cpu_cores, h.cpu_logical_processors
            ));
            out.push_str(&format!(
                "  メモリ : 合計 {} / 使用中 {} / 空き {}\n",
                format_bytes(h.memory_total_bytes),
                format_bytes(h.memory_used_bytes),
                format_bytes(h.memory_free_bytes),
            ));
        }
        Err(e) => {
            out.push_str(&format!("  取得失敗: {}\n", e));
        }
    }
    out.push('\n');
}

fn write_drive_section(out: &mut String, drives: &Result<Vec<DriveInfo>, String>) {
    out.push_str("[ドライブ情報]\n");
    match drives {
        Ok(list) => {
            for d in list {
                out.push_str(&format!(
                    "  {} 合計 {} / 使用済 {} / 空き {} (使用率 {}%)\n",
                    d.letter,
                    format_bytes(d.total_bytes),
                    format_bytes(d.used_bytes),
                    format_bytes(d.free_bytes),
                    d.usage_percent().round() as u64,
                ));
            }
        }
        Err(e) => {
            out.push_str(&format!("  取得失敗: {}\n", e));
        }
    }
    out.push('\n');
}

fn write_windows_features_section(
    out: &mut String,
    features: &Result<WindowsFeaturesInfo, String>,
) {
    out.push_str("[Windows機能情報]\n");
    match features {
        Err(e) => {
            out.push_str(&format!("  取得失敗: {}\n", e));
            out.push('\n');
            return;
        }
        Ok(f) => {
            // IIS
            out.push_str("  ■ IIS (Internet Information Services)\n");
            if f.iis.installed {
                let ver = f.iis.version.as_deref().unwrap_or("不明");
                out.push_str(&format!("    状態     : 有効 (バージョン {})\n", ver));
                if !f.iis.components.is_empty() {
                    let max_w = f
                        .iis
                        .components
                        .iter()
                        .map(|c| display_width(&c.display_name))
                        .max()
                        .unwrap_or(0);
                    for comp in &f.iis.components {
                        let pad = max_w - display_width(&comp.display_name);
                        out.push_str(&format!(
                            "      {}{} : {}\n",
                            comp.display_name,
                            " ".repeat(pad),
                            if comp.enabled { "有効" } else { "無効" }
                        ));
                    }
                }
            } else {
                out.push_str("    状態     : 無効\n");
            }
            out.push('\n');

            // .NET Framework
            out.push_str("  ■ .NET インストール状況\n");
            if f.dotnet.framework_versions.is_empty() {
                out.push_str("    .NET Framework: 未インストール\n");
            } else {
                out.push_str(&format!(
                    "    .NET Framework: {}\n",
                    f.dotnet.framework_versions.join(", ")
                ));
            }
            if f.dotnet.core_versions.is_empty() {
                out.push_str("    .NET (Core/5+): 未インストール\n");
            } else {
                // メジャーバージョン単位にまとめて表示
                out.push_str(&format!(
                    "    .NET (Core/5+): {}\n",
                    f.dotnet.core_versions.join(", ")
                ));
            }
            out.push('\n');

            // RDS / RDP
            out.push_str("  ■ リモートデスクトップ (RDP/RDS)\n");
            out.push_str(&format!(
                "    RDP        : {}\n",
                if f.rds.rdp_enabled { "有効" } else { "無効" }
            ));
            out.push_str(&format!("    ポート番号 : {}\n", f.rds.port));
            out.push_str(&format!(
                "    NLA認証    : {}\n",
                if f.rds.nla_enabled { "必須" } else { "不要" }
            ));
            out.push('\n');

            // 有効化されている主要Windowsサービス / 機能（未インストールは非表示）
            out.push_str("  ■ 主要Windowsサービス / 機能（インストール済みのみ）\n");
            if f.notable_services.is_empty() {
                out.push_str("    検出されませんでした\n");
            } else {
                // 全角文字を考慮した最大表示幅を算出してコロン位置を揃える
                let max_width = f
                    .notable_services
                    .iter()
                    .map(|s| display_width(&s.display_name))
                    .max()
                    .unwrap_or(0);
                for svc in &f.notable_services {
                    let padding = max_width - display_width(&svc.display_name);
                    out.push_str(&format!(
                        "    {}{} : {}\n",
                        svc.display_name,
                        " ".repeat(padding),
                        if svc.enabled { "有効" } else { "無効" }
                    ));
                }
            }
        }
    }
    out.push('\n');
}

fn write_task_section(out: &mut String, tasks: &Result<Vec<TaskInfo>, String>) {
    out.push_str("[タスクスケジューラ]\n");
    match tasks {
        Ok(list) if list.is_empty() => {
            out.push_str("  ユーザー定義タスクはありません\n");
        }
        Ok(list) => {
            for (i, task) in list.iter().enumerate() {
                if i > 0 {
                    out.push_str(&format!("  {}\n", INNER_SEP));
                }
                out.push_str(&format!("  タスク名           : {}\n", task.name));
                out.push_str(&format!("  パス               : {}\n", task.path));
                out.push_str(&format!(
                    "  状態               : {}\n",
                    if task.enabled { "有効" } else { "無効" }
                ));
                out.push_str(&format!("  最終実行日時       : {}\n", task.last_run_time));
                out.push_str(&format!("  次回実行予定       : {}\n", task.next_run_time));
                out.push_str(&format!("  実行ユーザー       : {}\n", task.run_as_user));
            }
        }
        Err(e) => {
            out.push_str(&format!("  取得失敗: {}\n", e));
        }
    }
    out.push('\n');
}

fn write_footer(out: &mut String) {
    out.push_str(SEPARATOR);
    out.push('\n');
    out.push_str("  収集完了\n");
    out.push_str(SEPARATOR);
    out.push('\n');
}

/// 文字列の端末表示幅を返す（全角文字=2、半角文字=1）
fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| if (c as u32) > 0x7F { 2 } else { 1 })
        .sum()
}

fn format_bytes(bytes: u64) -> String {
    const GB: f64 = 1_073_741_824.0;
    const MB: f64 = 1_048_576.0;

    if bytes >= GB as u64 {
        format!("{:.1} GB", bytes as f64 / GB)
    } else if bytes >= MB as u64 {
        format!("{:.1} MB", bytes as f64 / MB)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdapterInfo, DriveInfo, HardwareInfo, NetworkInfo, OsInfo, SystemInfo, TaskInfo};

    fn make_test_info() -> SystemInfo {
        let mut info = SystemInfo::new("2025-06-04 14:30:22".to_string());
        info.os = Ok(OsInfo {
            os_name: "Windows Server 2022 Standard".to_string(),
            build_number: "20348.2340".to_string(),
            install_date: "2024-01-15 10:00:00".to_string(),
            last_boot_time: "2025-06-01 09:15:00".to_string(),
            uptime: "3日 5時間 15分".to_string(),
        });
        info.network = Ok(NetworkInfo {
            hostname: "MYSERVER01".to_string(),
            adapters: vec![AdapterInfo {
                name: "イーサネット".to_string(),
                ipv4: Some("192.168.1.10".to_string()),
                ipv6: Some("fe80::1".to_string()),
                subnet_mask: Some("255.255.255.0".to_string()),
                gateway: Some("192.168.1.1".to_string()),
                mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
                dns_primary: Some("192.168.1.2".to_string()),
                dns_secondary: Some("8.8.8.8".to_string()),
            }],
        });
        info.hardware = Ok(HardwareInfo {
            cpu_name: "Intel(R) Xeon(R) E-2314 @ 2.80GHz".to_string(),
            cpu_clock_mhz: 2800,
            cpu_cores: 4,
            cpu_logical_processors: 4,
            memory_total_bytes: 32 * 1024 * 1024 * 1024,
            memory_used_bytes: 8 * 1024 * 1024 * 1024,
            memory_free_bytes: 24 * 1024 * 1024 * 1024,
        });
        info.drives = Ok(vec![DriveInfo {
            letter: "C:".to_string(),
            total_bytes: 254_926_323_712,
            used_bytes: 105_888_727_040,
            free_bytes: 149_037_596_672,
        }]);
        info.tasks = Ok(vec![TaskInfo {
            name: "MyBackupTask".to_string(),
            path: "\\MyBackupTask".to_string(),
            enabled: true,
            last_run_time: "2025-06-04 10:00:00".to_string(),
            next_run_time: "2025-06-05 10:00:00".to_string(),
            run_as_user: "SYSTEM".to_string(),
        }]);
        info
    }

    #[test]
    fn test_format_contains_header() {
        let info = make_test_info();
        let text = TextFormatter::format(&info);
        assert!(text.contains("システム情報収集レポート"));
        assert!(text.contains("MYSERVER01"));
        assert!(text.contains("2025-06-04 14:30:22"));
    }

    #[test]
    fn test_format_contains_os_info() {
        let info = make_test_info();
        let text = TextFormatter::format(&info);
        assert!(text.contains("Windows Server 2022 Standard"));
        assert!(text.contains("20348.2340"));
        assert!(text.contains("3日 5時間 15分"));
    }

    #[test]
    fn test_format_contains_network_info() {
        let info = make_test_info();
        let text = TextFormatter::format(&info);
        assert!(text.contains("192.168.1.10"));
        assert!(text.contains("AA:BB:CC:DD:EE:FF"));
        assert!(text.contains("192.168.1.2 / 8.8.8.8"));
    }

    #[test]
    fn test_format_contains_hardware_info() {
        let info = make_test_info();
        let text = TextFormatter::format(&info);
        assert!(text.contains("Intel(R) Xeon(R) E-2314"));
        assert!(text.contains("コア数: 4"));
        assert!(text.contains("32.0 GB"));
    }

    #[test]
    fn test_format_contains_drive_info() {
        let info = make_test_info();
        let text = TextFormatter::format(&info);
        assert!(text.contains("C:"));
        assert!(text.contains("237.4 GB") || text.contains("使用済"));
    }

    #[test]
    fn test_format_contains_task_info() {
        let info = make_test_info();
        let text = TextFormatter::format(&info);
        assert!(text.contains("MyBackupTask"));
        assert!(text.contains("有効"));
        assert!(text.contains("SYSTEM"));
    }

    #[test]
    fn test_format_error_sections() {
        let mut info = SystemInfo::new("2025-06-04 00:00:00".to_string());
        info.os = Err("WMI接続失敗".to_string());
        let text = TextFormatter::format(&info);
        assert!(text.contains("取得失敗"));
        assert!(text.contains("WMI接続失敗"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(512), "512 B");
    }

    #[test]
    fn test_empty_task_list() {
        let mut info = SystemInfo::new("2025-06-04 00:00:00".to_string());
        info.tasks = Ok(vec![]);
        let text = TextFormatter::format(&info);
        assert!(text.contains("ユーザー定義タスクはありません"));
    }
}
