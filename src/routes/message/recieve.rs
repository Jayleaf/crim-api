use super::generics::{
    structs::{Conversation, MessageUser}, utils
};
use crate::db::mongo::{self, get_collection};
use axum::{http::StatusCode, response::IntoResponse};
use mongodb::bson::doc;

/// Retrieves a conversation object from the database and returns it. Because the user (and only the user) knows their decrypted private key, we send them the entire conversation object (containing all messages, encrypted).
/// Sending the entire object rather than just the messages allows convenience, because we can get the list of users as well, helping build the client-side conversation page with less work.
///
/// ## Parameters:
/// ```rust
/// payload: String // serialized MessageUser value
/// ```
///
/// ## Return Values:
/// ```rust
/// Conversation // serialized
/// ```
pub async fn recieve(payload: String) -> impl IntoResponse
{
    let user: MessageUser = serde_json::from_str(&payload).unwrap();
    mongo::ping().await;
    // validate sid
    if utils::verify(&user.username, &user.session_id).await
    {
        let doc = Conversation::get(&user.conversation_id)
            .await
            .unwrap();
        return (StatusCode::OK, serde_json::to_string(&doc).unwrap());
    }
    (StatusCode::BAD_REQUEST, "Invalid session".to_string())
}
