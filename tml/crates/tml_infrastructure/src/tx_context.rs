use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait as _};
use tml_application::app_trait::tx_context::{self, Error, TxConnection, TxManager};

pub struct SeaOrmTxConnection {
    pub tx: DatabaseTransaction,
}

impl TxConnection for SeaOrmTxConnection {}

pub struct SeaOrmTxManager {
    db: DatabaseConnection,
}

impl SeaOrmTxManager {
    pub fn new(db: DatabaseConnection) -> Self {
        SeaOrmTxManager { db }
    }
}

#[async_trait::async_trait]
impl TxManager for SeaOrmTxManager {
    type Tx = SeaOrmTxConnection;

    async fn begin(&self) -> Result<Self::Tx, Error> {
        let tx = self
            .db
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.to_string()))?;
        Ok(SeaOrmTxConnection { tx })
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<tx_context::IsolationLevel>,
        access_mode: Option<tx_context::AccessMode>,
    ) -> Result<Self::Tx, Error> {
        let isolation_level = match isolation_level {
            Some(i) => match i {
                tx_context::IsolationLevel::RepeatableRead => {
                    Some(sea_orm::IsolationLevel::RepeatableRead)
                }
                tx_context::IsolationLevel::ReadCommitted => {
                    Some(sea_orm::IsolationLevel::ReadCommitted)
                }
                tx_context::IsolationLevel::ReadUncommitted => {
                    Some(sea_orm::IsolationLevel::ReadUncommitted)
                }
                tx_context::IsolationLevel::Serializable => {
                    Some(sea_orm::IsolationLevel::Serializable)
                }
            },
            None => None,
        };
        let access_mode = match access_mode {
            Some(a) => match a {
                tx_context::AccessMode::ReadOnly => Some(sea_orm::AccessMode::ReadOnly),
                tx_context::AccessMode::ReadWrite => Some(sea_orm::AccessMode::ReadWrite),
            },
            None => None,
        };
        let tx = self
            .db
            .begin_with_config(isolation_level, access_mode)
            .await
            .map_err(|e| Error::Unknown(e.to_string()))?;
        Ok(SeaOrmTxConnection { tx })
    }

    async fn commit(&self, tx: Self::Tx) -> Result<(), Error> {
        tx.tx
            .commit()
            .await
            .map_err(|e| Error::Unknown(e.to_string()))
    }

    async fn rollback(&self, tx: Self::Tx) -> Result<(), Error> {
        tx.tx
            .rollback()
            .await
            .map_err(|e| Error::Unknown(e.to_string()))
    }
}
