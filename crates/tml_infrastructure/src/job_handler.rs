use crate::entity::{app::music_info, mgmt::job};
use futures::stream::StreamExt as _;
use meilisearch_sdk::client::Client;
use sea_orm::{
    ActiveModelTrait as _, ActiveValue::Set, ConnectionTrait, DbErr, EntityTrait,
    sea_query::OnConflict,
};
use tml_application::app_trait::music_info_provider::{MusicInfoMeiliSearch, Trait as _};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Clone)]
pub struct JobHandler {
    repository: Repository,
    music_info_provider: crate::music_info_provider::MusicInfoProvider,
    pub meilisearch_client: Client,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Storage id not found error")]
    StorageIdNotFoundError,
    #[error("Meilisearch error: {0}")]
    MeilisearchError(#[from] meilisearch_sdk::errors::Error),
}

impl JobHandler {
    pub fn new(
        repository: Repository,
        music_info_provider: crate::music_info_provider::MusicInfoProvider,
        meilisearch_client: Client,
    ) -> Self {
        JobHandler {
            repository,
            music_info_provider,
            meilisearch_client,
        }
    }

    async fn handle_inner(
        &self,
        job_type: tml_domain::model::mgmt::job::JobType,
        job_args: &serde_json::Value,
        meilisearch_index_name: &str,
    ) -> Result<(), Error> {
        match job_type {
            tml_domain::model::mgmt::job::JobType::Undefined => (),
            tml_domain::model::mgmt::job::JobType::ScanIncremental => {
                let storage_id = job_args["storage_id"]
                    .as_i64()
                    .ok_or(Error::StorageIdNotFoundError)?;
                self.handle_scan_incremental_job(storage_id, meilisearch_index_name)
                    .await?;
            }
            tml_domain::model::mgmt::job::JobType::BuildIndex => (),
            tml_domain::model::mgmt::job::JobType::UpdateIndex => (),
            tml_domain::model::mgmt::job::JobType::DeleteIndex => (),
            tml_domain::model::mgmt::job::JobType::RebuildIndex => (),
        };
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
        meilisearch_index_name: &str,
    ) {
        match self
            .handle_inner(job_type, &job_args, meilisearch_index_name)
            .await
        {
            Ok(_) => {
                match self.repository.finish_job(job_id, true, "").await {
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
                match self
                    .repository
                    .finish_job(job_id, false, &e.to_string())
                    .await
                {
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

// scan_incremental
impl JobHandler {
    async fn handle_scan_incremental_job(
        &self,
        storage_id: i64,
        meilisearch_index_name: &str,
    ) -> Result<(), Error> {
        let path = self.repository.get_storage_path(storage_id).await?;
        let mut music_info_chunk_stream = self.get_music_info_chunk_stream(path).await;
        while let Some(chunk) = music_info_chunk_stream.next().await {
            self.repository
                .create_or_update_music_info(storage_id, chunk.clone().into_iter())
                .await?;
            let meilisearch_models: Vec<_> = chunk
                .into_iter()
                .map(|x| MusicInfoMeiliSearch {
                    id: hex::encode(x.0),
                    artists: x.1.artists,
                    album_title: x.1.album_title,
                    title: x.1.title,
                })
                .collect();
            let _task = self
                .meilisearch_client
                .index(meilisearch_index_name.to_string())
                .add_documents(&meilisearch_models, Some("id"))
                .await?;
        }
        self.repository
            .reindex_concurrently("app.music_info_pkey")
            .await?;
        Ok(())
    }

    /// Return chunk with 500 items
    pub async fn get_music_info_chunk_stream(
        &self,
        path: String,
    ) -> impl tokio_stream::Stream<
        Item = Vec<(
            Vec<u8>,
            tml_application::app_trait::music_info_provider::MusicInfo,
        )>,
    > {
        let (tx, rx) = mpsc::unbounded_channel();

        let music_info_provider = self.music_info_provider.clone();

        tokio::task::spawn_blocking(move || {
            let iter = music_info_provider.scan(&path);

            for item in iter {
                if tx.send(item).is_err() {
                    break;
                }
            }
        });

        UnboundedReceiverStream::new(rx).chunks(500)
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

    async fn finish_job(
        &self,
        job_id: i64,
        success: bool,
        error_message: &str,
    ) -> Result<(), RepositoryError> {
        let job_to_update = crate::entity::mgmt::job::Entity::find_by_id(job_id)
            .one(&self.db)
            .await?
            .ok_or(RepositoryError::JobNotFound)?;
        let mut job_to_update: job::ActiveModel = job_to_update.into();
        job_to_update.status = Set(job::JobStatus::Completed);
        job_to_update.success = Set(success);
        job_to_update.error_message = Set(error_message.into());
        job_to_update.completed_at = Set(Some(chrono::Utc::now()));
        let _updated_job = job_to_update.update(&self.db).await?;
        Ok(())
    }

    async fn get_storage_path(&self, storage_id: i64) -> Result<String, RepositoryError> {
        let storage = crate::entity::mgmt::storage::Entity::find_by_id(storage_id)
            .one(&self.db)
            .await?
            .ok_or(RepositoryError::StorageNotFound)?;
        Ok(storage.path)
    }

    async fn create_or_update_music_info(
        &self,
        storage_id: i64,
        music_info: impl IntoIterator<
            Item = (
                Vec<u8>,
                tml_application::app_trait::music_info_provider::MusicInfo,
            ),
        > + Send,
    ) -> Result<(), RepositoryError> {
        let on_conflict = OnConflict::column(music_info::Column::Id)
            .update_columns([music_info::Column::FilePath])
            .to_owned();
        let music_info_collection = music_info.into_iter().map(|x| music_info::ActiveModel {
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

    async fn reindex_concurrently(&self, index: &str) -> Result<(), RepositoryError> {
        self.db
            .execute_raw(sea_orm::Statement::from_string(
                self.db.get_database_backend(),
                format!("REINDEX INDEX CONCURRENTLY {}", index),
            ))
            .await?;
        Ok(())
    }
}
