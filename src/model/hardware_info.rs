#[derive(Debug, Clone, Default)]
pub struct HardwareInfo {
    pub cpu_name: String,
    pub cpu_clock_mhz: u32,
    pub cpu_cores: u32,
    pub cpu_logical_processors: u32,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub memory_free_bytes: u64,
}
