use super::generics::{structs::{Conversation, EncryptedMessage}, utils };
use axum::{http::StatusCode, response::IntoResponse};

/// Uploads a message to a conversation in the database. The encrypted message value is recieved, and uploaded to the database, stripped of any identifying info.
///
/// ## Arguments:
/// * [`payload`] - A JSON string containing a serialized EncryptedMessage struct, paired with a conversation ID.
///
/// ## Returns:
/// * [`(StatusCode, String)`][axum::response::Response] - The status code of the request call. The string is not useful, except for error identification.
///     * 200 OK if message sending was successful
///     * 401 UNAUTHORIZED if the user's session is invalid
///     * 403 FORBIDDEN if the user is not a part of the provided conversation ID
///     * 404 NOT FOUND if the provided conversation ID does not match a conversation
///     * 500 INTERNAL_SERVER_ERROR if an error occurred sending the message
///
pub async fn send(payload: String) -> impl IntoResponse
{
    let Ok(message) = serde_json::from_str::<EncryptedMessage>(&payload)
    else { return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid payload.")) };

    match utils::verify(&message.sender, &message.sender_sid).await
    {
        Ok(true) => (),
        Ok(false) => return (StatusCode::UNAUTHORIZED, utils::gen_err("Invalid SID.")),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e)
    }

    let mut convo = match Conversation::get_one(&message.dest_convo_id).await
    {
        Ok(Some(convo)) => convo,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, utils::gen_err("Error retrieving conversation.")),
        Ok(None) => return (StatusCode::NOT_FOUND, utils::gen_err("Attempted to send message to non-existent conversation.")),
    };

    if !convo.users.contains(&message.sender) 
    { return (StatusCode::FORBIDDEN, utils::gen_err("User is not a part of the conversation they're trying to send to.")) };

    if let Err(e) = convo.send(message).await 
    { return (StatusCode::INTERNAL_SERVER_ERROR, e) }
    else { return (StatusCode::OK, String::new()) };

}
