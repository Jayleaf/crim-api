use std::net::SocketAddr;
use axum::extract::State;
use tracing::{debug, error};
use crate::generics::structs::WSAction;
use crate::tokio::sync::mpsc::Sender;
use crate::generics::{structs::{Conversation, EncryptedMessage, WSPacket, Account}, utils};
use super::super::message::send;
use super::generics::structs::ClientStore;
use tracing::info;

/// Client interface for message sending through the websocket.
/// 
/// ## Arguments
/// * [`data`][`EncryptedMessage`] - The message to send.
/// * [`who`][`SocketAddr`] - The address of the client.
/// * [`State<ClientStore>`][`State`] - The global client store.
/// * [`tx`][`Sender<WSPacket>`] - Transmitter, so we can relay info back to the sender of this message if needed
/// 
pub async fn send_msg(data: EncryptedMessage, who: SocketAddr, State(store): State<ClientStore>, tx: &Sender<WSPacket>)
{

    let store = store.lock().await;

    let Some(client) = store.get(&who)
    else { tx.send(utils::info_packet("You are not registered with the server.")).await.ok(); return; };

    let account = match Account::get_account_by_sid(&client.session_id).await
    {
        Ok(Some(account)) => account,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return; }
        Ok(None) => { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return; }
    };
        

    let conversation = match Conversation::get_one(&data.dest_convo_id).await
    {
        Ok(Some(convo)) => convo,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return;}
        Ok(None) => { tx.send(utils::info_packet("No such conversation.")).await.ok(); return; }
    };

    // ensure the sender is friends with all users in the conversation
    if !conversation.users.iter().all(|user| account.friends.contains(user))
    { info!("User is not friends with all users."); tx.send(utils::info_packet("You are not friends with all users in this conversation, so you may not send messages to it.")).await.ok(); return; }

    // send message to db
    if let Err(e) = send::send(data.clone()).await 
    { tx.send(utils::info_packet(&e)).await.ok(); return; }

    
    // forward message to all online recipients
    for user in conversation.users {
        let Some(client) = store.values().find(|c| c.username == user)
        else { continue }; // user is not currently logged on

        if client.socket.send(WSPacket { sender: data.clone().sender, sid: String::from("0"), action: WSAction::ReceiveMessage(data.clone())}).await.is_ok() 
        { info!("Sent message to client {user} from {x}", x = data.sender) } 
        else { error!("Failed to send message to client {user}. Did they abruptly disconnect?") }
    }
}
