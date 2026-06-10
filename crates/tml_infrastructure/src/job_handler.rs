use crate::entity::{app::music_info, mgmt::job};
use sea_orm::{ActiveModelTrait as _, ActiveValue::Set, DbErr, EntityTrait, sea_query::OnConflict};
use tml_application::app_trait::music_info_provider::Trait as _;

#[derive(Clone)]
pub struct JobHandler {
    repository: Repository,
    music_info_provider: crate::music_info_provider::MusicInfoProvider,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Storage id not found error")]
    StorageIdNotFoundError,
}

impl JobHandler {
    pub fn new(
        repository: Repository,
        music_info_provider: crate::music_info_provider::MusicInfoProvider,
    ) -> Self {
        JobHandler {
            repository,
            music_info_provider,
        }
    }

    async fn handle_inner(
        &self,
        job_type: tml_domain::model::mgmt::job::JobType,
        job_args: &serde_json::Value,
    ) -> Result<(), Error> {
        match job_type {
            tml_domain::model::mgmt::job::JobType::Undefined => (),
            tml_domain::model::mgmt::job::JobType::ScanIncremental => {
                let storage_id = job_args["storage_id"]
                    .as_i64()
                    .ok_or(Error::StorageIdNotFoundError)?;
                self.handle_scan_incremental_job(storage_id).await?;
            }
            tml_domain::model::mgmt::job::JobType::BuildIndex => (),
            tml_domain::model::mgmt::job::JobType::UpdateIndex => (),
        };
        Ok(())
    }

    async fn handle_scan_incremental_job(&self, storage_id: i64) -> Result<(), Error> {
        let path = self.repository.get_storage_path(storage_id).await?;
        let iter = self.music_info_provider.scan(&path);
        let mut iter = iter.peekable();
        while iter.peek().is_some() {
            let chunk = iter.by_ref().take(500);
            self.repository
                .create_or_update_music_info(storage_id, chunk)
                .await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl tml_application::app_trait::job_handler::Trait for JobHandler {
    async fn handle(
        &self,
        job_id: i64,
        job_type: tml_domain::model::mgmt::job::JobType,
        job_args: serde_json::Value,
    ) {
        match self.handle_inner(job_type, &job_args).await {
            Ok(_) => {
                match self.repository.finish_job(job_id, true).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("{}", e);
                        return;
                    }
                };
                tracing::info!("job {} done", job_id);
            }
            Err(e) => {
                tracing::error!("{}", e);
                match self.repository.finish_job(job_id, false).await {
                    Ok(_) => {}
                    Err(_) => {
                        tracing::error!("{}", e);
                        return;
                    }
                };
            }
        }
    }
}

#[derive(Clone)]
pub struct Repository {
    db: sea_orm::DatabaseConnection,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("Job not found")]
    JobNotFound,
    #[error("Storage not found")]
    StorageNotFound,
}

impl Repository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Repository {
        Repository { db }
    }

    async fn finish_job(&self, job_id: i64, success: bool) -> Result<(), RepositoryError> {
        let job_to_update = crate::entity::mgmt::job::Entity::find_by_id(job_id)
            .one(&self.db)
            .await?
            .ok_or(RepositoryError::JobNotFound)?;
        let mut job_to_update: job::ActiveModel = job_to_update.into();
        job_to_update.status = Set(job::JobStatus::Completed);
        job_to_update.success = Set(success);
        job_to_update.completed_at = Set(Some(chrono::Utc::now()));
        let _updated_job = job_to_update.update(&self.db).await?;
        Ok(())
    }

    async fn get_storage_path(&self, storage_id: i64) -> Result<String, RepositoryError> {
        let storage = crate::entity::mgmt::storage::Entity::find_by_id(storage_id)
            .one(&self.db)
            .await?
            .ok_or(RepositoryError::JobNotFound)?;
        Ok(storage.path)
    }

    async fn create_or_update_music_info(
        &self,
        storage_id: i64,
        music_info: impl Iterator<
            Item = (
                Vec<u8>,
                tml_application::app_trait::music_info_provider::MusicInfo,
            ),
        > + Send,
    ) -> Result<(), RepositoryError> {
        let on_conflict = OnConflict::column(music_info::Column::Id)
            .update_columns([music_info::Column::FilePath])
            .to_owned();
        let music_info_collection = music_info.map(|x| music_info::ActiveModel {
            id: Set(x.0),
            artists: Set(x.1.artists),
            album_title: Set(x.1.album_title),
            title: Set(x.1.title),
            track_number: Set(x.1.track_number),
            audio_bitrate: Set(x.1.audio_bitrate),
            sample_rate: Set(x.1.sample_rate),
            channels: Set(x.1.channels),
            bit_depth: Set(x.1.bit_depth),
            storage_id: Set(storage_id),
            file_path: Set(x.1.file_path),
        });
        let _reslut = music_info::Entity::insert_many(music_info_collection)
            .on_conflict(on_conflict)
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
