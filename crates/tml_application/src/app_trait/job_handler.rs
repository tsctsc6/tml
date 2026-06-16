use tml_domain::model::mgmt::job;

#[async_trait::async_trait]
pub trait Trait: Send + Sync + Clone + 'static {
    async fn handle(
        &self,
        job_id: i64,
        job_type: job::JobType,
        job_args: serde_json::Value,
        meilisearch_index_name: &str,
    );
}
