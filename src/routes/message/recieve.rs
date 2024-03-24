use super::generics::{ structs::{Conversation, ClientAccount}, utils };
use axum::{http::StatusCode, response::IntoResponse};

/// Retrieves a user's conversations from the database and returns them. Because the user (and only the user) knows their decrypted private key, we send them the entire conversation object (containing all messages, encrypted).
///
/// ## Parameters:
/// * [`payload`][`std::string::String`] - A JSON string containing a serialized [`ClientAccount`] value.
///
/// ## Return Values:
/// * [`(StatusCode, String)`][axum::response::Response] - A tuple containing the [`StatusCode`] of the request and a serialized [`Conversation`] [`vector.`][`std::vec::Vec`]
///
pub async fn recieve(payload: String) -> impl IntoResponse
{
    let Ok(client_account) = serde_json::from_str::<ClientAccount>(&payload) 
    else { return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid Payload."))};

    match utils::verify(&client_account.username, &client_account.session_id).await
    {
        Ok(true) => (),
        Ok(false) => return (StatusCode::UNAUTHORIZED, utils::gen_err("Invalid session ID.")),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e)
    }

    let conversations: Vec<Conversation> = match Conversation::get_all(&client_account.username).await
    {
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e),
        Ok(convos) => convos
    };

    return (StatusCode::OK, serde_json::to_string(&conversations).unwrap());
}
