mod collector;
mod formatter;
mod model;
mod writer;

use anyhow::Result;
use chrono::Local;
use formatter::TextFormatter;
use model::SystemInfo;
use std::env;
use std::path::PathBuf;
use writer::write_report;

fn main() {
    if let Err(e) = run() {
        eprintln!("エラー: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    println!("[sysinfo-collector] 起動しました");

    check_admin_privilege()?;

    let output_dir = parse_output_dir();
    let collected_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut info = SystemInfo::new(collected_at.clone());

    print!("[収集中] OS情報 ...");
    std::io::Write::flush(&mut std::io::stdout())?;
    info.os = collector::collect_os_info().map_err(|e| e.to_string());
    println!(" 完了");

    print!("[収集中] ネットワーク情報 ...");
    std::io::Write::flush(&mut std::io::stdout())?;
    info.network = collector::collect_network().map_err(|e| e.to_string());
    println!(" 完了");

    print!("[収集中] ハードウェア情報 ...");
    std::io::Write::flush(&mut std::io::stdout())?;
    info.hardware = collector::collect_hardware().map_err(|e| e.to_string());
    println!(" 完了");

    print!("[収集中] ドライブ情報 ...");
    std::io::Write::flush(&mut std::io::stdout())?;
    info.drives = collector::collect_drives().map_err(|e| e.to_string());
    println!(" 完了");

    print!("[収集中] Windows機能情報 ...");
    std::io::Write::flush(&mut std::io::stdout())?;
    info.windows_features = collector::collect_windows_features().map_err(|e| e.to_string());
    println!(" 完了");

    print!("[収集中] タスクスケジューラ情報 ...");
    std::io::Write::flush(&mut std::io::stdout())?;
    info.tasks = collector::collect_tasks().map_err(|e| e.to_string());
    println!(" 完了");

    let text = TextFormatter::format(&info);
    let hostname = info.hostname().to_string();

    let file_path = write_report(&text, &output_dir, &hostname, &collected_at)?;
    println!("[書き込み] {}", file_path.display());
    println!("[完了] 収集が完了しました。");

    Ok(())
}

fn check_admin_privilege() -> Result<()> {
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows::Win32::Foundation::HANDLE;

    let mut token: HANDLE = HANDLE::default();
    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;
    }

    let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut return_length: u32 = 0;

    unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        )?;
    }

    if elevation.TokenIsElevated == 0 {
        anyhow::bail!("管理者権限が必要です。管理者として実行してください。");
    }

    println!("[権限確認] 管理者権限: OK");
    Ok(())
}

fn parse_output_dir() -> PathBuf {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--output" || args[i] == "-o") && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
        i += 1;
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
