use axum::{http::StatusCode, response::IntoResponse};
use crate::db::mongo;
use super::generics::{utils, structs::{UserKey, MessageUser}};

struct EncryptedMessages
{
    messages: Vec<String>,
    key: UserKey
}

/// Retrieves the list of encrypted messages from the database, and returns them. Because the user (and only the user) knows their decrypted private key, the encrypted messages will be returned to them so they may decrypt it client side.
/// 
/// ## Parameters:
/// ```rust
/// payload: String // serialized MessageUser value
/// ```
/// 
/// ## Return Values:
/// ```rust
/// EncryptedMessageContainer // serialized
/// {
///    messages: Vec<EncryptedMessage> // Full list of encrypted messages
///    key: UserKey // The key belonging to the calling client (their username attached) containing the conversation key
/// }
/// ```
pub async fn recieve(payload: String) -> impl IntoResponse
{
    let user: MessageUser = serde_json::from_str(&payload).unwrap();
    mongo::ping().await;
    if utils::verify(user.username.clone(), user.session_id.clone()).await
    {
        let messages: Vec<String> = vec!["".to_string()];
        let key: UserKey = 
        return (StatusCode::OK, "".to_string());
    }
    (StatusCode::BAD_REQUEST, "Invalid session".to_string())
}