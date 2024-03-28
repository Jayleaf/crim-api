use super::generics::{structs::{Conversation, EncryptedMessage}, utils };

/// Uploads a message to a conversation in the database.
///
/// ## Arguments:
/// * [`message`][`super::generics::structs::EncryptedMessage`] - The message to be sent.
///
/// ## Returns:
/// * [`Result<(), String>`] - A result containing an error message, if any.
/// 
pub async fn send(message: EncryptedMessage) -> Result<(), String>
{

    match utils::verify(&message.sender, &message.sender_sid).await
    {
        Ok(true) => (),
        Ok(false) => return Err(utils::gen_err("Invalid SID.")),
        Err(e) => return Err(e)
    }

    let mut convo = match Conversation::get_one(&message.dest_convo_id).await
    {
        Ok(Some(convo)) => convo,
        Err(_) => return Err(utils::gen_err("Error retrieving conversation.")),
        Ok(None) => return Err(utils::gen_err("Attempted to send message to non-existent conversation.")),
    };

    if !convo.users.contains(&message.sender) 
    { return Err(utils::gen_err("User is not a part of the conversation they're trying to send to.")) };

    convo.send(message).await

}
