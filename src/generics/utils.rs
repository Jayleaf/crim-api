use rand::RngCore;
use super::structs::Account;

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
pub async fn verify(username: &String, session_id: &String) -> Result<bool, String>
{
    Account::get_account(username)
    .await
    .map_err(|e| e)?
    .map(|a| &a.session_id == session_id)
    .ok_or_else(|| String::from("Tried to validate with a non-existent account."))
}

pub fn rand_hex(len: usize) -> String
{
    let mut bytes: Vec<u8> = vec![0; len];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn gen_err(msg: &str) -> String
{
    return format!("{} ({})", msg, std::env::current_dir().unwrap().to_str().unwrap().to_string());
}
