#[derive(Debug, Clone, Default)]
pub struct TaskInfo {
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub last_run_time: String,
    pub next_run_time: String,
    pub run_as_user: String,
}
