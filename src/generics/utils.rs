use rand::RngCore;

use super::structs::Account;
use crate::db::mongo;

/// Verify a user's session
///
/// ## Parameters:
/// * username: [`&String`][`std::string::String`] // The username of the user to verify
/// * session_id: [`&String`][`std::string::String`] // The session id of the user to verify
///
///
/// ## Return Values:
/// * [`bool`][`std::primitive::bool`] // True if the session id is valid, false if it is not
///
///
pub async fn verify(username: &String, session_id: &String) -> bool
{
    mongo::ping().await;
    println!("username: {}", username);
    let account: Account = Account::get_account(&username).await.expect("Failed to get account");
    &account.session_id == session_id
}

pub fn rand_hex() -> String
{
    let mut bytes = [0; 4];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
