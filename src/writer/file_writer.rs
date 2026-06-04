use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// レポートをファイルに書き出し、書き込んだファイルのパスを返す
pub fn write_report(
    content: &str,
    output_dir: &Path,
    hostname: &str,
    timestamp: &str,
) -> Result<PathBuf> {
    // "2025-06-04 14:30:22" → "20250604_143022"
    let safe_timestamp = timestamp
        .replace('-', "")
        .replace(' ', "_")
        .replace(':', "");

    let filename = format!("sysinfo_{}_{}.txt", hostname, safe_timestamp);
    let file_path = output_dir.join(&filename);

    fs::write(&file_path, content.as_bytes())?;

    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_report_creates_file() {
        let dir = TempDir::new().unwrap();
        let path =
            write_report("テスト内容", dir.path(), "TESTHOST", "2025-06-04 14:30:22").unwrap();

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "テスト内容");
    }

    #[test]
    fn test_write_report_filename_format() {
        let dir = TempDir::new().unwrap();
        let path =
            write_report("dummy", dir.path(), "MYSERVER01", "2025-06-04 14:30:22").unwrap();

        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(filename.starts_with("sysinfo_MYSERVER01_"));
        assert!(filename.ends_with(".txt"));
        assert!(filename.contains("20250604_143022"));
    }
}
