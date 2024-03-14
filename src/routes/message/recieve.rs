use axum::response::IntoResponse;

use crate::structs::structs::UserKey;

struct EncryptedMessages
{
    messages: Vec<String>,
    key: UserKey
}

/// Retrieves the list of encrypted messages from the database, and returns them. Because the user (and only the user) knows their decrypted private key, the encrypted messages will be returned to them so they may decrypt it client side.
/// 
/// parameters:
/// ```rust
/// payload: // figure this out. Should have SID of caller for verification, their username, and conversation id.
/// ```
/// 
/// returns:
/// ```rust
/// EncryptedMessage
/// {
///    messages: Vec<String> // Full list of encrypted messages
///    key: UserKey // The key belonging to the calling client (their username attached) containing the conversation key
/// }
/// ```
pub async fn recieve(payload: String) -> impl IntoResponse
{

}