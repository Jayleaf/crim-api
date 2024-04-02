use std::net::SocketAddr;
use axum::extract::State;
use tracing::{error, info};
use tracing_subscriber::field::debug;
use super::generics::structs::{Account, WSAction, Conversation};
use crate::tokio::sync::mpsc::Sender;
use crate::generics::{structs::WSPacket, utils};
use super::generics::structs::ClientStore;


pub async fn remove_friend(packet: WSPacket, who: SocketAddr, State(store): State<ClientStore>, tx: &Sender<WSPacket>) -> Result<(), ()>
{
    info!("Recieved remove friend request from {who}: {:#?}", packet);

    let store = store.lock().await;

    let Some(client) = store.get(&who)
    else { tx.send(utils::info_packet("You are not registered with the server.")).await.ok(); return Ok(()); };

    if client.session_id != packet.sid
    { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return Ok(()); }

    let WSAction::RemoveFriend(x) = packet.action
    else { tx.send(utils::info_packet("Invalid action.")).await.ok(); return Ok(()); };

    let mut client: Account = match Account::get_account_by_sid(&client.session_id).await
    {
        Ok(Some(client)) => client,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); }
        Ok(None) => { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return Ok(()); }
    };

    let mut friend: Account = match Account::get_account(&x).await
    {
        Ok(Some(user)) => user,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); }
        Ok(None) => { tx.send(utils::info_packet("Friend does not exist.")).await.ok(); return Ok(()); }
    };



    if !client.friends.iter().any(|user| user == &x)
    { tx.send(utils::info_packet("You are not friends with this user.")).await.ok(); return Ok(()); }


    friend.friends.retain(|u| u != &client.username);
    client.friends.retain(|u| u != &x);
    match Account::update_account(&friend).await
    {
        Ok(_) => (),
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); }
    }
    match Account::update_account(&client).await
    {
        Ok(_) => (),
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); }
    }


    // update this client side for all users, beginning with the client
    let c_packet = WSPacket { sender: String::from("API"), sid: String::from("0"), action: WSAction::RecieveArbitraryInfo(x.clone(), 4) };
    if tx.send(c_packet).await.is_err() 
    { error!("Failed to send conversations to client {who}. Did they abruptly disconnect?") }


    let Some(friend_client) = store.values().find(|c| &c.username == &x)
    else { return Ok(()) }; // user is not online

    let f_packet = WSPacket { sender: String::from("API"), sid: String::from("0"), action: WSAction::RecieveArbitraryInfo(client.username, 4) };
    if friend_client.socket.send(f_packet).await.is_err() 
    { error!("Failed to send conversations to client {x}. Did they abruptly disconnect?") }
    
    



    // tell the client that the message was sent (unnecessary in prod)
    tx.send(utils::info_packet("Conversation created.")).await.ok();
    Ok(())
}
