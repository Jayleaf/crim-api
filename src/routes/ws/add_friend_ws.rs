use std::net::SocketAddr;
use axum::extract::State;
use tracing::error;
use super::generics::structs::{Account, WSAction, Conversation};
use crate::routes::message::make;
use crate::tokio::sync::mpsc::Sender;
use crate::generics::{structs::WSPacket, utils};
use super::generics::structs::ClientStore;


pub async fn add_friend(packet: WSPacket, who: SocketAddr, State(store): State<ClientStore>, tx: &Sender<WSPacket>) -> Result<(), ()>
{

    let store = store.lock().await;

    let Some(client) = store.get(&who)
    else { tx.send(utils::info_packet("You are not registered with the server.")).await.ok(); return Ok(()); };

    if client.session_id != packet.sid
    { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return Ok(()); }

    let WSAction::AddFriend(x) = packet.action
    else { tx.send(utils::info_packet("Invalid action.")).await.ok(); return Ok(()); };

    let client: Account = match Account::get_account_by_sid(&client.session_id).await
    {
        Ok(Some(client)) => client,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); }
        Ok(None) => { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return Ok(()); }
    };

    if client.friends.iter().any(|user| user == &x)
    { tx.send(utils::info_packet("You are already friends with this user.")).await.ok(); return Ok(()); }

    let convo: Conversation = match make::create_conversation(vec![&client.username, &x]).await
    {
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); },
        Ok(convo) => convo
    };

    let Ok(Some(mut friend_acc)) = Account::get_account(&x).await
    else { tx.send(utils::info_packet("Failed to get friend account.")).await.ok(); return Ok(()); };
    friend_acc.friends.push(packet.sender);
    Account::update_account(&friend_acc).await.ok();

    // update this client side for all users, beginning with the client
    let c_packet: WSPacket = WSPacket { sender: String::from("API"), sid: String::from("0"), action: WSAction::RecieveArbitraryInfo(serde_json::to_string(&convo).unwrap(), 2) };
    if tx.send(c_packet).await.is_err() 
    { error!("Failed to send conversations to client {who}. Did they abruptly disconnect?") }

    let Some(friend_client) = store.values().find(|c| &c.username == &x)
    else { return Ok(()) }; // user is not online

    let f_packet = WSPacket { sender: String::from("API"), sid: String::from("0"), action: WSAction::RecieveArbitraryInfo(serde_json::to_string(&convo).unwrap(), 2) };
    if friend_client.socket.send(f_packet).await.is_err() 
    { error!("Failed to send conversations to client {x}. Did they abruptly disconnect?") }
    
    



    // tell the client that the message was sent (unnecessary in prod)
    tx.send(utils::info_packet("Conversation created.")).await.ok();
    Ok(())
}
