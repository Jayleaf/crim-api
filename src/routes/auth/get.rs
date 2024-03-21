use std::f32::consts::E;

use crate::generics::structs::Conversation;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use mongodb::bson::{doc, Document};

use super::{
    generics::structs::{Account, ClientAccount}, mongo
};

/// Gets a users data from the database.
///
/// ## Arguments
/// * [`payload`][`std::string::String`] - A JSON string containing a serialized SID.
///
/// ## Returns
/// * [`(StatusCode, String)`][axum::response::Response] - A tuple containing the [`StatusCode`] of the request and a serialized [`ClientAccount`] value.
///
pub async fn get(payload: String) -> impl IntoResponse
{
    mongo::ping().await;
    if let Some(server_account) = Account::get_account_by_sid(&payload).await
    {
        // the only way this conditional is true is if the session ID is valid, so no additional verification is needed.
        let username = server_account.username;
        let friends = server_account.friends;
        // now, get all current conversations. Guess who didn't make a way to retrieve all a user's conversations? This guy.
        let convos = {
            match mongo::get_collection("conversations")
                .await
                .find(doc! {"users": &username}, None)
                .await
            {
                Ok(mut cursor) =>
                {
                    let mut convos: Vec<Document> = Vec::new();
                    // i <3 asynchronous rust
                    if cursor.advance().await.unwrap() == true
                    {
                        convos.push(cursor.current().try_into().unwrap());
                    }
                    convos
                }
                // user has no convos
                Err(_) => Vec::new()
            }
        };
        let result = ClientAccount {
            username,
            password: "".to_string(),
            friends,
            conversations: convos
                .into_iter()
                .map(|x| Conversation::from_document(&x))
                .collect(),
            session_id: "".to_string()
        };
        return (StatusCode::OK, serde_json::to_string(&result).unwrap());
    }
    else
    {
        // Returned if SID can't be found.
        // + StatusCode::UNAUTHORIZED
        // - StatusCode::BAD_REQUEST
        return (StatusCode::UNAUTHORIZED, "".to_string());
    }
}
