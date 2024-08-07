use std::net::SocketAddr;
use axum::extract::State;
use tracing::error;
use crate::generics::structs::{Account, WSAction};
use crate::routes::message::make;
use crate::tokio::sync::mpsc::Sender;
use crate::generics::{structs::WSPacket, utils};
use super::generics::structs::ClientStore;
use tracing::info;

pub async fn make_convo(packet: WSPacket, who: SocketAddr, State(store): State<ClientStore>, tx: &Sender<WSPacket>)
{

    let store = store.lock().await;

    let Some(client) = store.get(&who)
    else { tx.send(utils::info_packet("You are not registered with the server.")).await.ok(); return; };

    if client.session_id != packet.sid || client.username != packet.sender
    { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return; }

    let WSAction::CreateConversation(mut x) = packet.action
    else { tx.send(utils::info_packet("Invalid action.")).await.ok(); return; };

    let client: Account = match Account::get_account_by_sid(&client.session_id).await
    {
        Ok(Some(client)) => client,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return; }
        Ok(None) => { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return; }
    };

    if x.iter().any(|user| !client.friends.contains(user) || user == &client.username)
    { tx.send(utils::info_packet("You are not friends with all the users you are trying to create a conversation with.")).await.ok(); return; }

    x.push(client.username.clone());
    let Ok(convo) = make::create_conversation(x.iter().map(|x| x).collect()).await
    else { tx.send(utils::info_packet("Failed to create conversation.")).await.ok(); return; };


    for user in x.iter()
    {
        // then, see if user is online to live-update their conversation list
        let Some(client) = store.values().find(|c| &c.username == user)
        else { continue };
        let s_packet: WSPacket = WSPacket { sender: String::from("API"), sid: String::from("0"), action: WSAction::ReceiveArbitraryInfo(serde_json::to_string(&convo).unwrap(), 2) };
        if client.socket.send(s_packet).await.is_ok() 
        { info!("Sent conversation to client {user} from {x}", x = client.username) } 
        else { error!("Failed to send conversation to client {user}. Did they abruptly disconnect?") }
    }


    // tell the client that the message was sent (unnecessary in prod)
    tx.send(utils::info_packet("Conversation created.")).await.ok();
}
