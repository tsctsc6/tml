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

impl From<Model> for tml_domain::model::mgmt::job::Model {
    fn from(model: Model) -> Self {
        tml_domain::model::mgmt::job::Model {
            id: model.id,
            job_type: model.job_type.into(),
            job_args: model.job_args,
            status: model.status.into(),
            description: model.description,
            error_message: model.error_message,
            success: model.success,
            created_by_id: model.created_by_id,
            created_at: model.created_at,
            completed_at: model.completed_at,
        }
    }
}

impl From<JobType> for tml_domain::model::mgmt::job::JobType {
    fn from(job_type: JobType) -> Self {
        match job_type {
            JobType::Undefined => tml_domain::model::mgmt::job::JobType::Undefined,
            JobType::ScanIncremental => tml_domain::model::mgmt::job::JobType::ScanIncremental,
            JobType::BuildIndex => tml_domain::model::mgmt::job::JobType::BuildIndex,
            JobType::UpdateIndex => tml_domain::model::mgmt::job::JobType::UpdateIndex,
        }
    }
}

impl From<JobStatus> for tml_domain::model::mgmt::job::JobStatus {
    fn from(status: JobStatus) -> Self {
        match status {
            JobStatus::Undefined => tml_domain::model::mgmt::job::JobStatus::Undefined,
            JobStatus::WaitingStart => tml_domain::model::mgmt::job::JobStatus::WaitingStart,
            JobStatus::Running => tml_domain::model::mgmt::job::JobStatus::Running,
            JobStatus::Completed => tml_domain::model::mgmt::job::JobStatus::Completed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum JobType {
    Undefined = 0,
    ScanIncremental = 1,
    BuildIndex = 2,
    UpdateIndex = 3,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum JobStatus {
    Undefined = 0,
    WaitingStart = 1,
    Running = 2,
    Completed = 3,
}
