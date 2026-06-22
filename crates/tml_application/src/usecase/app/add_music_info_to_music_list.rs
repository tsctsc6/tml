use fractional_index::FractionalIndex;

use crate::app_trait;

pub mod repository {
    use tml_domain::model::app::music_info_music_list;

    use crate::app_trait::tx_context::TxConnection;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Music list not found")]
        MusicListNotFound,
        #[error("Music info not found")]
        MusicInfoNotFound,
        #[error("Music info already in music list")]
        MusicInfoAlreadyInMusicList,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        type Tx: TxConnection;

        async fn get_last_order(
            &self,
            tx_connection: &mut Self::Tx,
            music_list_id: i64,
        ) -> Result<Vec<u8>, Error>;
        async fn add_music_info_to_music_list(
            &self,
            tx_connection: &mut Self::Tx,
            music_list_id: i64,
            music_info_id: &[u8],
            order: &[u8],
        ) -> Result<music_info_music_list::Model, Error>;
        async fn get_music_list_owner_id(
            &self,
            tx_connection: &mut Self::Tx,
            music_list_id: i64,
        ) -> Result<i64, Error>;
    }
}

pub struct Request<'a> {
    pub music_list_id: i64,
    pub music_info_id: &'a [u8],
    pub user_id: i64,
}

pub struct Response {
    pub order: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Decode error: {0}")]
    DecodeError(String),
    #[error("Transaction error: {0}")]
    TxError(#[from] app_trait::tx_context::Error),
}

pub async fn handle<R, M>(
    request: Request<'_>,
    repository: &R,
    tx_manager: &M,
) -> Result<Response, Error>
where
    R: repository::Trait<Tx = M::Tx>,
    M: app_trait::tx_context::TxManager,
{
    let mut tx = tx_manager
        .begin_with_config(
            Some(app_trait::tx_context::IsolationLevel::Serializable),
            None,
        )
        .await?;
    let owner_id = repository
        .get_music_list_owner_id(&mut tx, request.music_list_id)
        .await?;
    if request.user_id != owner_id {
        return Err(Error::PermissionDenied);
    }
    let last_order = repository
        .get_last_order(&mut tx, request.music_list_id)
        .await?;

    let new_order = if last_order.is_empty() {
        let default_index = FractionalIndex::default();
        default_index.as_bytes().to_vec()
    } else {
        let last_index = match FractionalIndex::from_bytes(last_order) {
            Ok(o) => o,
            Err(e) => Err(Error::DecodeError(e.to_string()))?,
        };
        let new_index = FractionalIndex::new_after(&last_index);
        new_index.as_bytes().to_vec()
    };

    let result = repository
        .add_music_info_to_music_list(
            &mut tx,
            request.music_list_id,
            request.music_info_id,
            &new_order,
        )
        .await?;
    tx_manager.commit(tx).await?;
    Ok(Response {
        order: result.order,
    })
}
