use crate::entity::mgmt::job;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use tml_application::usecase::mgmt::create_job;

#[derive(Clone)]
pub struct Repository {
    db: sea_orm::DatabaseConnection,
}

impl Repository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Repository { db }
    }
}

#[async_trait::async_trait]
impl create_job::repository::Trait for Repository {
    async fn create_job(
        &self,
        job_type: &tml_domain::model::mgmt::job::JobType,
        job_args: &serde_json::Value,
        description: &str,
        created_by_id: i64,
    ) -> Result<tml_domain::model::mgmt::job::Model, create_job::repository::Error> {
        let job_to_create = job::ActiveModel {
            job_type: Set(match job_type {
                tml_domain::model::mgmt::job::JobType::Undefined => job::JobType::Undefined,
                tml_domain::model::mgmt::job::JobType::ScanIncremental => {
                    job::JobType::ScanIncremental
                }
                tml_domain::model::mgmt::job::JobType::BuildIndex => job::JobType::BuildIndex,
                tml_domain::model::mgmt::job::JobType::UpdateIndex => job::JobType::UpdateIndex,
                tml_domain::model::mgmt::job::JobType::DeleteIndex => job::JobType::DeleteIndex,
                tml_domain::model::mgmt::job::JobType::RebuildIndex => job::JobType::RebuildIndex,
            }),
            job_args: Set(job_args.clone()),
            status: Set(job::JobStatus::Running),
            description: Set(description.into()),
            error_message: Set(String::new()),
            success: Set(false),
            created_by_id: Set(created_by_id),
            created_at: Set(chrono::Utc::now()),
            completed_at: Set(None),
            ..Default::default()
        };
        let new_job = match job_to_create.insert(&self.db).await {
            Ok(job) => job,
            Err(e) => {
                return Err(create_job::repository::Error::Unknown(e.to_string()));
            }
        };
        Ok(new_job.into())
    }
}
