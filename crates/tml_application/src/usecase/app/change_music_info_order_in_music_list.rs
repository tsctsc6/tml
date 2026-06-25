use fractional_index::FractionalIndex;

use crate::app_trait;

pub mod repository {
    use crate::app_trait::tx_context::TxConnection;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Music list not found")]
        MusicListNotFound,
        #[error("Music info not in music list")]
        MusicInfoNotInMusicList,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        type Tx: TxConnection;

        async fn get_music_list_owner_id(
            &self,
            tx_connection: &mut Self::Tx,
            music_list_id: i64,
        ) -> Result<i64, Error>;
        async fn get_prev_and_next_order(
            &self,
            tx_connection: &mut Self::Tx,
            music_list_id: i64,
            prev_music_info_id: Option<&[u8]>,
        ) -> Result<(Option<Vec<u8>>, Option<Vec<u8>>), Error>;
        async fn update_order_of_music_info_in_music_list(
            &self,
            tx_connection: &mut Self::Tx,
            music_list_id: i64,
            music_info_id: &[u8],
            new_order: &[u8],
        ) -> Result<(), Error>;
    }
}

pub struct Request<'a> {
    pub music_list_id: i64,
    pub music_info_id: &'a [u8],
    /// Move music_info_id after prev_music_info_id, None means move to the top
    pub prev_music_info_id: Option<&'a [u8]>,
    pub user_id: i64,
}

pub struct Response {
    pub new_order: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Decode error: {0}")]
    DecodeError(String),
    #[error("Cannot reorder")]
    InvalidReorder,
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
            Some(app_trait::tx_context::IsolationLevel::ReadCommitted),
            None,
        )
        .await?;
    let owner_id = repository
        .get_music_list_owner_id(&mut tx, request.music_list_id)
        .await?;
    if request.user_id != owner_id {
        return Err(Error::PermissionDenied);
    }

    // get adj order
    let (prev_order, next_order) = repository
        .get_prev_and_next_order(&mut tx, request.music_list_id, request.prev_music_info_id)
        .await?;

    // 4. generate fractional index
    let new_order = match (prev_order, next_order) {
        (None, None) => FractionalIndex::default().as_bytes().to_vec(),
        (None, Some(next)) => {
            // the top, before next
            let next_index =
                FractionalIndex::from_bytes(next).map_err(|e| Error::DecodeError(e.to_string()))?;
            FractionalIndex::new_before(&next_index).as_bytes().to_vec()
        }
        (Some(prev), None) => {
            // the last, after prev
            let prev_index =
                FractionalIndex::from_bytes(prev).map_err(|e| Error::DecodeError(e.to_string()))?;
            FractionalIndex::new_after(&prev_index).as_bytes().to_vec()
        }
        (Some(prev), Some(next)) => {
            // between prev and next
            let prev_index =
                FractionalIndex::from_bytes(prev).map_err(|e| Error::DecodeError(e.to_string()))?;
            let next_index =
                FractionalIndex::from_bytes(next).map_err(|e| Error::DecodeError(e.to_string()))?;
            FractionalIndex::new_between(&prev_index, &next_index)
                .ok_or(Error::InvalidReorder)?
                .as_bytes()
                .to_vec()
        }
    };

    // Update the order
    repository
        .update_order_of_music_info_in_music_list(
            &mut tx,
            request.music_list_id,
            request.music_info_id,
            &new_order,
        )
        .await?;
    tx_manager.commit(tx).await?;

    Ok(Response { new_order })
}
