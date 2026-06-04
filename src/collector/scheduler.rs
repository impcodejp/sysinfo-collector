use crate::model::TaskInfo;
use anyhow::Result;
use windows::Win32::Foundation::VARIANT_BOOL;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::TaskScheduler::{ITaskFolder, ITaskService, TaskScheduler};
use windows::core::{BSTR, VARIANT};

/// `\Microsoft\Windows\` 配下のシステムタスクを除外するパスプレフィックス
const SYSTEM_TASK_PREFIX: &str = r"\Microsoft\Windows\";

pub fn collect_tasks() -> Result<Vec<TaskInfo>> {
    unsafe { collect_tasks_unsafe() }
}

unsafe fn collect_tasks_unsafe() -> Result<Vec<TaskInfo>> {
    CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    let result = collect_all_tasks();
    CoUninitialize();
    result
}

unsafe fn collect_all_tasks() -> Result<Vec<TaskInfo>> {
    let service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_ALL)?;

    service.Connect(
        &VARIANT::default(),
        &VARIANT::default(),
        &VARIANT::default(),
        &VARIANT::default(),
    )?;

    let root_folder = service.GetFolder(&BSTR::from("\\"))?;

    let mut tasks = Vec::new();
    collect_folder_tasks(&root_folder, &mut tasks)?;

    Ok(tasks)
}

unsafe fn collect_folder_tasks(folder: &ITaskFolder, tasks: &mut Vec<TaskInfo>) -> Result<()> {
    // Path() は戻り値スタイル
    let folder_path_str = folder.Path().map(|b| b.to_string()).unwrap_or_default();

    // サブフォルダを再帰的に探索
    let subfolders = folder.GetFolders(0)?;
    let count = subfolders.Count()?;
    for i in 1..=count {
        let subfolder: ITaskFolder = subfolders.get_Item(&VARIANT::from(i))?;
        collect_folder_tasks(&subfolder, tasks)?;
    }

    // \Microsoft\Windows\ 配下はスキップ
    if folder_path_str.starts_with(SYSTEM_TASK_PREFIX) {
        return Ok(());
    }

    // 非表示タスクも含めて取得 (flags = 1)
    let task_collection = folder.GetTasks(1)?;
    let task_count = task_collection.Count()?;

    for i in 1..=task_count {
        let task = task_collection.get_Item(&VARIANT::from(i))?;

        // Name / Path は戻り値スタイル
        let name = task.Name().map(|b| b.to_string()).unwrap_or_default();
        let path = task.Path().map(|b| b.to_string()).unwrap_or_default();

        if path.starts_with(SYSTEM_TASK_PREFIX) {
            continue;
        }

        let definition = task.Definition()?;
        let settings = definition.Settings()?;

        // Enabled は out-pointer スタイル
        let mut enabled_vb = VARIANT_BOOL::default();
        let enabled = if settings.Enabled(&mut enabled_vb).is_ok() {
            enabled_vb.as_bool()
        } else {
            false
        };

        // LastRunTime / NextRunTime は戻り値スタイル
        let last_run_str = match task.LastRunTime() {
            Ok(dt) => format_ole_date(dt),
            Err(_) => "未実行".to_string(),
        };
        let next_run_str = match task.NextRunTime() {
            Ok(dt) => format_ole_date(dt),
            Err(_) => "未スケジュール".to_string(),
        };

        let principal = definition.Principal()?;

        // UserId は out-pointer スタイル
        let mut user_id = BSTR::default();
        let run_as_user = if principal.UserId(&mut user_id).is_ok() && !user_id.is_empty() {
            user_id.to_string()
        } else {
            "不明".to_string()
        };

        tasks.push(TaskInfo {
            name,
            path,
            enabled,
            last_run_time: last_run_str,
            next_run_time: next_run_str,
            run_as_user,
        });
    }

    Ok(())
}

/// OLE Automation Date（1899-12-30基点のf64日数）を "YYYY-MM-DD HH:MM:SS" 形式に変換する
fn format_ole_date(date: f64) -> String {
    if date <= 0.0 {
        return "未実行".to_string();
    }

    let days_since_1899_12_30 = date as i64;
    let fraction = date - date.floor();

    // 1899-12-30 から Unix エポック(1970-01-01) まで: 25569日
    let unix_days = days_since_1899_12_30 - 25569;
    let unix_secs = unix_days * 86400 + (fraction * 86400.0) as i64;

    secs_to_datetime_str(unix_secs)
}

fn secs_to_datetime_str(unix_secs: i64) -> String {
    if unix_secs < 0 {
        return "未実行".to_string();
    }

    let days = unix_secs / 86400;
    let time_secs = unix_secs % 86400;
    let hour = time_secs / 3600;
    let min = (time_secs % 3600) / 60;
    let sec = time_secs % 60;

    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, min, sec
    )
}

fn days_to_ymd(mut days: i64) -> (i64, i64, i64) {
    let mut year = 1970i64;
    loop {
        let dy = if is_leap(year) { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [i64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1i64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}
