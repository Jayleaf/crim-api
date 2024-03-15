use crate::db::mongo;
use super::structs::Account;


/// Verify a user's session
/// 
/// ## Parameters:
/// ```rust
/// username: String // The username of the user to verify
/// session_id: String // The session id of the user to verify
/// ```
/// 
/// ## Return Values:
/// ```rust
/// bool // Whether the session id is valid
/// ```
pub async fn verify(username: String, session_id: String) -> bool
{
    mongo::ping().await;
    let account: Account = Account::get_account(&username).await.unwrap();
    account.session_id == session_id
}