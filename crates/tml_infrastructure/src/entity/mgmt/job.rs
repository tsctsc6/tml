use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "mgmt", table_name = "job")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub job_type: JobType,
    pub job_args: serde_json::Value,
    pub status: JobStatus,
    pub description: String,
    pub error_message: String,
    pub success: bool,
    pub created_by_id: i64,
    #[sea_orm(belongs_to, from = "created_by_id", to = "id")]
    pub created_by: HasOne<super::super::auth::user::Entity>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "job_type")]
pub enum JobType {
    #[sea_orm(string_value = "undefined")]
    Undefined,
    #[sea_orm(string_value = "scan_incremental")]
    ScanIncremental,
    #[sea_orm(string_value = "build_index")]
    BuildIndex,
    #[sea_orm(string_value = "update_index")]
    UpdateIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "job_status")]
pub enum JobStatus {
    #[sea_orm(string_value = "undefined")]
    Undefined,
    #[sea_orm(string_value = "waiting_start")]
    WaitingStart,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "completed")]
    Completed,
}
