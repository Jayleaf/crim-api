use axum::http::StatusCode;
use axum::response::IntoResponse;
use mongodb::bson::{self, doc, Document};
use super::{mongo, generics::structs::{Account, ClientAccount}};
use argon2::Argon2;
use base64::{engine::general_purpose, Engine as _};
use getrandom::getrandom;

///Local to get.rs, no need to put it in structs.rs

#[derive(serde::Serialize, serde::Deserialize)]
struct GetResult
{
    username: String,
    friends: Vec<String>,
    convos: Vec<Document>
}

/// Gets a users data from the database.
pub async fn get(payload: String) -> impl IntoResponse 
{
    // payload is gonna be a session ID
    mongo::ping().await;
    if let Some(server_account) = Account::get_account_by_sid(&payload).await
    {
        // the only way this conditional is true is if the session ID is valid, so no additional verification is needed.
        let username = server_account.username;
        let friends = server_account.friends;
        // now, get all current conversations. Guess who didn't make a way to retrieve all a user's conversations? This guy.
        let convos = 
        {
            match mongo::get_collection("conversations").await.find(doc! {"users": &username}, None).await
            {
                Ok(mut cursor) => 
                {
                    let mut convos: Vec<Document> = Vec::new();
                    if cursor.advance().await.unwrap() == true
                    {
                        convos.push(cursor.current().try_into().unwrap());
                    }
                    convos
                },
                // user has no convos
                Err(_) => return (StatusCode::BAD_REQUEST, "".to_string())
            }
        };
        let result = GetResult {username, friends, convos};
        return (StatusCode::OK, serde_json::to_string(&result).unwrap());
    }
    else
    {
            // Returned if SID can't be found.
            // + StatusCode::UNAUTHORIZED
            // - StatusCode::BAD_REQUEST
            return (StatusCode::UNAUTHORIZED, "".to_string())
    }
}