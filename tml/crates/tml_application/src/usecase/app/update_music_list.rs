use crate::app_trait;

pub mod repository {
    use tml_domain::model::app::music_list;

    use crate::app_trait::tx_context::TxConnection;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Name duplication")]
        NameDuplication,
        #[error("Music list not found")]
        MusicListNotFound,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        type Tx: TxConnection;

        async fn update_music_list(
            &self,
            tx_connection: &mut Self::Tx,
            id: i64,
            name: &str,
        ) -> Result<music_list::Model, Error>;
        async fn get_music_list_owner_id(
            &self,
            tx_connection: &mut Self::Tx,
            music_list_id: i64,
        ) -> Result<i64, Error>;
    }
}

pub mod validation {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Name is empty")]
        NameEmpty,
        #[error("Name too long error")]
        NameTooLong,
    }

    pub fn validate(request: &super::Request<'_>) -> Result<(), Error> {
        if request.name.is_empty() {
            return Err(Error::NameEmpty);
        }
        if request.name.chars().count() > 50 {
            return Err(Error::NameTooLong);
        }
        Ok(())
    }
}

pub struct Request<'a> {
    pub id: i64,
    pub name: &'a str,
    pub user_id: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Validation error: {0}")]
    ValidationError(#[from] validation::Error),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Transaction error: {0}")]
    TxError(#[from] app_trait::tx_context::Error),
}

pub async fn handle<R, M>(request: Request<'_>, repository: &R, tx_manager: &M) -> Result<(), Error>
where
    R: repository::Trait<Tx = M::Tx>,
    M: app_trait::tx_context::TxManager,
{
    validation::validate(&request)?;
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
    let _updated_music_list = repository
        .update_music_list(&mut tx, request.id, request.name)
        .await?;
    tx_manager.commit(tx).await?;
    Ok(())
}
