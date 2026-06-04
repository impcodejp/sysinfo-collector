use crate::model::DriveInfo;
use anyhow::Result;
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives,
};
use windows::core::PCWSTR;

// GetDriveTypeW の戻り値: 3 = DRIVE_FIXED
const DRIVE_FIXED: u32 = 3;

pub fn collect_drives() -> Result<Vec<DriveInfo>> {
    let drive_mask = unsafe { GetLogicalDrives() };

    let mut drives = Vec::new();

    for i in 0..26u32 {
        if drive_mask & (1 << i) == 0 {
            continue;
        }

        let letter = (b'A' + i as u8) as char;
        let path_str = format!("{}:\\", letter);
        let path_wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

        let drive_type = unsafe { GetDriveTypeW(PCWSTR(path_wide.as_ptr())) };
        if drive_type != DRIVE_FIXED {
            continue;
        }

        let mut total = 0u64;
        let mut free = 0u64;
        let mut free_available = 0u64;

        let ok = unsafe {
            GetDiskFreeSpaceExW(
                PCWSTR(path_wide.as_ptr()),
                Some(&mut free_available),
                Some(&mut total),
                Some(&mut free),
            )
        };

        if ok.is_err() {
            continue;
        }

        let used = total.saturating_sub(free);

        drives.push(DriveInfo {
            letter: format!("{}:", letter),
            total_bytes: total,
            used_bytes: used,
            free_bytes: free,
        });
    }

    Ok(drives)
}
