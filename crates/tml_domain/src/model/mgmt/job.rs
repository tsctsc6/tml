pub struct Model {
    pub id: i64,
    pub job_type: JobType,
    pub job_args: serde_json::Value,
    pub status: JobStatus,
    pub description: String,
    pub error_message: String,
    pub success: bool,
    pub created_by_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobType {
    Undefined,
    ScanIncremental,
    BuildIndex,
    UpdateIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Undefined,
    WaitingStart,
    Running,
    Completed,
}
