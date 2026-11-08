/// A presence profile owned by the device account. One account may hold several
/// profiles; profiles live in the encrypted vault.
#[derive(Debug, Clone)]
pub struct Profile {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub enterprise: String,
    pub poste: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
