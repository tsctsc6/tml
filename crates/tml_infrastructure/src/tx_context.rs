use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait as _};
use tml_application::app_trait::tx_context::{Error, TxConnection, TxManager};

pub struct SeaOrmTxConnection {
    pub tx: DatabaseTransaction,
}

impl TxConnection for SeaOrmTxConnection {}

pub struct SeaOrmTxManager {
    db: DatabaseConnection,
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
