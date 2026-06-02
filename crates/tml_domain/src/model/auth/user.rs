pub struct Model {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub security_stamp: uuid::Uuid,
}
