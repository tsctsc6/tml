use meilisearch_sdk::client::Client;

use crate::config::AppConfig;

pub async fn init(app_config: &AppConfig) -> Result<Client, meilisearch_sdk::errors::Error> {
    let meilisearch_client = Client::new(
        app_config.meilisearch.host.clone(),
        Some(app_config.meilisearch.api_key.clone()),
    )?;
    let task = meilisearch_client
        .index(app_config.meilisearch.index_name.clone())
        .set_filterable_attributes(&["artists"])
        .await?;
    task.wait_for_completion(&meilisearch_client, None, None)
        .await?;
    Ok(meilisearch_client)
}
