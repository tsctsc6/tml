pub trait TxConnection: Send + Sync {}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[async_trait::async_trait]
pub trait TxManager: Send + Sync {
    type Tx: TxConnection;

    async fn begin(&self) -> Result<Self::Tx, Error>;
    async fn commit(&self, tx: Self::Tx) -> Result<(), Error>;
    async fn rollback(&self, tx: Self::Tx) -> Result<(), Error>;
}
