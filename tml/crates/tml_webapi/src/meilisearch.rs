use meilisearch_sdk::client::Client;

use crate::config::AppConfig;

pub async fn init(app_config: &AppConfig) -> Result<Client, meilisearch_sdk::errors::Error> {
    let meilisearch_client = Client::new(
        app_config.meilisearch.host.clone(),
        Some(app_config.meilisearch.api_key.clone()),
    )?;
    Ok(meilisearch_client)
}
