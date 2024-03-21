use super::generics::{
    structs::{Conversation, MessageUser}, utils
};
use crate::{
    db::mongo::{self, get_collection}, generics::structs::ClientAccount
};
use axum::{http::StatusCode, response::IntoResponse};
use mongodb::bson::doc;

/// Retrieves a user's conversations from the database and returns them. Because the user (and only the user) knows their decrypted private key, we send them the entire conversation object (containing all messages, encrypted).
///
/// ## Parameters:
/// * [`payload`][`std::string::String`] - A JSON string containing a serialized [`ClientAccount`] value.
///
/// ## Return Values:
/// * [`(StatusCode, String)`][axum::response::Response] - A tuple containing the [`StatusCode`] of the request and a serialized [`Conversation`] [`vector.`][`std::vec::Vec`]
///
///
pub async fn recieve(payload: String) -> impl IntoResponse
{
    let user: ClientAccount = serde_json::from_str(&payload).unwrap();
    mongo::ping().await;
    // validate sid
    if utils::verify(&user.username, &user.session_id).await
    {
        let doc: Option<Vec<Conversation>> = Conversation::get_all(&user.username).await;
        match doc
        {
            Some(convos) => return (StatusCode::OK, serde_json::to_string(&convos).unwrap()),
            None => return (StatusCode::BAD_REQUEST, "No conversations found".to_string())
        }
    }
    (StatusCode::BAD_REQUEST, "Invalid session".to_string())
}
