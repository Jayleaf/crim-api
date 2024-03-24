use std::net::SocketAddr;

use axum::extract::State;
use crate::generics::structs::WSAction;
use crate::tokio::sync::mpsc::Sender;

use crate::generics::{structs::{Conversation, EncryptedMessage, WSPacket}, utils};
use super::super::message::send;

use super::generics::structs::ClientStore;
// websocket send message
pub async fn send_msg(data: EncryptedMessage, who: SocketAddr, State(store): State<ClientStore>, tx: &Sender<WSPacket>)
-> Result<(), ()>
{
    // the first thing we need to do is figure out who this client is
    let store = store.lock().await;

    let Some(client) = store.get(&who)
    else { tx.send(utils::info_packet("You are not registered with the server.")).await.ok(); return Ok(()); };

    // authenticate client
    if client.session_id != data.sender_sid
    {
        tx.send(utils::info_packet("Invalid session ID.")).await.ok();
        return Ok(());
    }

    // send message to db
    if let Err(e) = send::send(data.clone()).await { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); }

    // tell the client that the message was sent
    tx.send(utils::info_packet("Message sent.")).await.ok();
        

    // tell all the recipients there was a new message
    let conversation = match Conversation::get_one(&data.dest_convo_id).await
    {
        Ok(Some(convo)) => convo,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(());}
        Ok(None) => { tx.send(utils::info_packet("No such conversation.")).await.ok(); return Ok(()); }
    };

    for user in conversation.users
    {
        // find the client value by username
        let Some(client) = store.values().find(|c| c.username == user)
        else { continue; }; // user is not currently logged on
        if let Ok(x) = client.socket.send( WSPacket { sender: data.clone().sender, sid: String::from("0"), action: WSAction::ReceiveMessage(data.clone())}).await
        {
            println!("Sent message to client {}: {:?}", user, x);
            return Ok(())
        }
        else { println!("Failed to send message to client {}", user); return Ok(())}
    }
    Ok(())
}
