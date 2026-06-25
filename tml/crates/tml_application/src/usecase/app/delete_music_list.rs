use crate::app_trait;

pub mod repository {
    use crate::app_trait::tx_context::TxConnection;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Music list not found")]
        MusicListNotFound,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        type Tx: TxConnection;

        async fn delete_music_list(
            &self,
            tx_connection: &mut Self::Tx,
            id: i64,
        ) -> Result<(), Error>;
        async fn get_music_list_owner_id(
            &self,
            tx_connection: &mut Self::Tx,
            music_list_id: i64,
        ) -> Result<i64, Error>;
    }
}

pub struct Request {
    pub id: i64,
    pub user_id: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Transaction error: {0}")]
    TxError(#[from] app_trait::tx_context::Error),
}

pub async fn handle<R, M>(request: Request, repository: &R, tx_manager: &M) -> Result<(), Error>
where
    R: repository::Trait<Tx = M::Tx>,
    M: app_trait::tx_context::TxManager,
{
    let mut tx = tx_manager
        .begin_with_config(
            Some(app_trait::tx_context::IsolationLevel::ReadCommitted),
            None,
        )
        .await?;
    let owner_id = repository
        .get_music_list_owner_id(&mut tx, request.id)
        .await?;
    if request.user_id != owner_id {
        return Err(Error::PermissionDenied);
    }
    repository.delete_music_list(&mut tx, request.id).await?;
    tx_manager.commit(tx).await?;
    Ok(())
}
