/// Public projection of an account — safe to expose to the presentation layer.
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub created_at: i64,
    pub updated_at: i64,
}
