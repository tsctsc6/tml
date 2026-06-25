pub trait TxConnection: Send + Sync {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Isolation level
pub enum IsolationLevel {
    /// Consistent reads within the same transaction read the snapshot established by the first read.
    RepeatableRead,
    /// Each consistent read, even within the same transaction, sets and reads its own fresh snapshot.
    ReadCommitted,
    /// SELECT statements are performed in a nonlocking fashion, but a possible earlier version of a row might be used.
    ReadUncommitted,
    /// All statements of the current transaction can only see rows committed before the first query or data-modification statement was executed in this transaction.
    Serializable,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Access mode
pub enum AccessMode {
    /// Data can't be modified in this transaction
    ReadOnly,
    /// Data can be modified in this transaction (default)
    ReadWrite,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[async_trait::async_trait]
pub trait TxManager: Send + Sync {
    type Tx: TxConnection;

    async fn begin(&self) -> Result<Self::Tx, Error>;
    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<Self::Tx, Error>;
    async fn commit(&self, tx: Self::Tx) -> Result<(), Error>;
    async fn rollback(&self, tx: Self::Tx) -> Result<(), Error>;
}
