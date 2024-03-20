use axum::{http::StatusCode, response::IntoResponse};
use super::generics::{
    structs::{Conversation, EncryptedMessage, MessageUser}, utils
};

/// Uploads a message to a conversation in the database. The encrypted message value is recieved, and uploaded to the database, stripped of any identifying info.
/// 
/// ## Arguments:
/// * [`payload`] - A JSON string containing a serialized EncryptedMessage struct, paired with a conversation ID.
/// 
/// ## Returns:
/// * [`StatusCode`][axum::response::Response] - The status code of the request call.
///     * 200 OK if message sending was successful
///     * 401 UNAUTHORIZED if the user's session is invalid
///     * 403 FORBIDDEN if the user is not a part of the provided conversation ID
///     * 404 NOT FOUND if the provided conversation ID does not match a conversation
///     * 500 INTERNAL_SERVER_ERROR if an error occurred sending the message
/// 
pub async fn send(payload: String) -> impl IntoResponse 
{

    let message: EncryptedMessage = serde_json::from_str(&payload).unwrap();

    // validate user
    if utils::verify(&message.sender, &message.sender_sid).await
    {
        // retrieve conversation
        match Conversation::get_one(&message.dest_convo_id).await
        {
            Some(mut convo) =>
            {
                // check if user is in conversation
                if convo.users.contains(&message.sender)
                {
                    return convo.send(message).await;
                }
                else
                {
                    return StatusCode::FORBIDDEN // user is not a member of the conversation they're trying to upload to
                }
            }
            None => return StatusCode::NOT_FOUND // can't find conversation
        }
    }
    else
    {
        return StatusCode::UNAUTHORIZED; // invalid session
    }

}
