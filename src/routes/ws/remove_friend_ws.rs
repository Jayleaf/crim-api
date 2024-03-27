use std::net::SocketAddr;
use axum::extract::State;
use tracing::error;
use super::generics::structs::{Account, WSAction, Conversation};
use crate::tokio::sync::mpsc::Sender;
use crate::generics::{structs::WSPacket, utils};
use super::generics::structs::ClientStore;


pub async fn remove_friend(packet: WSPacket, who: SocketAddr, State(store): State<ClientStore>, tx: &Sender<WSPacket>) -> Result<(), ()>
{

    let store = store.lock().await;

    let Some(client) = store.get(&who)
    else { tx.send(utils::info_packet("You are not registered with the server.")).await.ok(); return Ok(()); };

    if client.session_id != packet.sid
    { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return Ok(()); }

    let WSAction::RemoveFriend(x) = packet.action
    else { tx.send(utils::info_packet("Invalid action.")).await.ok(); return Ok(()); };

    let client: Account = match Account::get_account_by_sid(&client.session_id).await
    {
        Ok(Some(client)) => client,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); }
        Ok(None) => { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return Ok(()); }
    };

    if !client.friends.iter().any(|user| user == &x)
    { tx.send(utils::info_packet("You are not friends with this user.")).await.ok(); return Ok(()); }

    let Ok(mut conversations) = Conversation::get_all(&client.username).await
    else { tx.send(utils::info_packet("Failed to get conversations.")).await.ok(); return Ok(()); };

    conversations = conversations.iter().filter(|c| !c.users.contains(&x)).cloned().collect();
    for c in &mut conversations 
    { 
        c.users.retain(|u| u != &x);
        if let Err(x) = Conversation::modify(&c).await 
        { tx.send(utils::info_packet(&format!("Failed to get conversations: {x}"))).await.ok(); return Ok(());} 
    }

    // there is no bulk remove friend operation, so this isn't in a loop.
    let Ok(Some(mut friend_acc)) = Account::get_account(&x).await
    else { tx.send(utils::info_packet("Failed to get friend account.")).await.ok(); return Ok(()); };
    friend_acc.friends.retain(|u| u != &client.username);
    Account::update_account(&friend_acc).await.ok();

    // update this client side for all users, beginning with the client
    let c_packet = WSPacket { sender: String::from("API"), sid: String::from("0"), action: WSAction::RecieveArbitraryInfo(serde_json::to_string(&conversations).unwrap(), 1) };
    if tx.send(c_packet).await.is_err() 
    { error!("Failed to send conversations to client {who}. Did they abruptly disconnect?") }

    // then the affected friend. this really pains me to make two get_all calls, but it's gonna be done either way.
    // this is already updated server-side, so we just need to basically tell the client to update what they see. We won't be removing them from their conversations.
    let Ok(friend_conversations) = Conversation::get_all(&x).await
    else { tx.send(utils::info_packet("Failed to get conversations.")).await.ok(); return Ok(()); };

    let Some(friend_client) = store.values().find(|c| &c.username == &x)
    else { return Ok(()) }; // user is not online

    let f_packet = WSPacket { sender: String::from("API"), sid: String::from("0"), action: WSAction::RecieveArbitraryInfo(serde_json::to_string(&friend_conversations).unwrap(), 1) };
    if friend_client.socket.send(f_packet).await.is_err() 
    { error!("Failed to send conversations to client {x}. Did they abruptly disconnect?") }
    
    



    // tell the client that the message was sent (unnecessary in prod)
    tx.send(utils::info_packet("Conversation created.")).await.ok();
    Ok(())
}
