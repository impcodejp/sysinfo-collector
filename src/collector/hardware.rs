use crate::model::HardwareInfo;
use anyhow::Result;
use serde::Deserialize;
use sysinfo::System;
use wmi::{COMLibrary, WMIConnection};

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_Processor")]
#[allow(non_snake_case)]
struct WmiProcessor {
    Name: Option<String>,
    MaxClockSpeed: Option<u32>,
    NumberOfCores: Option<u32>,
    NumberOfLogicalProcessors: Option<u32>,
}

pub fn collect_hardware() -> Result<HardwareInfo> {
    let (cpu_name, cpu_clock_mhz, cpu_cores, cpu_logical_processors) = collect_cpu_via_wmi()?;
    let (memory_total_bytes, memory_used_bytes, memory_free_bytes) = collect_memory();

    Ok(HardwareInfo {
        cpu_name,
        cpu_clock_mhz,
        cpu_cores,
        cpu_logical_processors,
        memory_total_bytes,
        memory_used_bytes,
        memory_free_bytes,
    })
}

fn collect_cpu_via_wmi() -> Result<(String, u32, u32, u32)> {
    let com = COMLibrary::new()?;
    let wmi = WMIConnection::new(com)?;

    let results: Vec<WmiProcessor> = wmi.query()?;
    let proc = results
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Win32_Processor が見つかりません"))?;

    Ok((
        proc.Name.unwrap_or_else(|| "不明".to_string()),
        proc.MaxClockSpeed.unwrap_or(0),
        proc.NumberOfCores.unwrap_or(0),
        proc.NumberOfLogicalProcessors.unwrap_or(0),
    ))
}

fn collect_memory() -> (u64, u64, u64) {
    let mut sys = System::new();
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let free = sys.available_memory();

    (total, used, free)
}
