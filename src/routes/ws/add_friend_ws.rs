use std::net::SocketAddr;
use axum::extract::State;
use tracing::{error, info};
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

    let mut client: Account = match Account::get_account_by_sid(&client.session_id).await
    {
        Ok(Some(client)) => client,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); }
        Ok(None) => { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return Ok(()); }
    };

    let mut friend: Account = match Account::get_account(if client.username == x.receiver { &x.sender } else { &x.receiver} ).await
    {
        Ok(Some(user)) => user,
        Err(e) => { tx.send(utils::info_packet(&e)).await.ok(); return Ok(()); }
        Ok(None) => { tx.send(utils::info_packet("Friend does not exist.")).await.ok(); return Ok(()); }
    };

    if client.friends.iter().any(|user| user == &x.receiver)
    { tx.send(utils::info_packet("You are already friends with this user.")).await.ok(); return Ok(()); }

    let mut info_code: u8 = 0;

    match x.status.as_str() {
        "PENDING" => {
            // ensure no existing friend request exists
            if client.friend_requests.iter().any(|req| req.sender == x.sender && req.receiver == x.receiver)
            { tx.send(utils::info_packet("You have already sent a friend request to this user.")).await.ok(); return Ok(()); }
            
            friend.friend_requests.push(x.clone());
            Account::update_account(&friend).await.ok();

            client.friend_requests.push(x.clone());
            Account::update_account(&client).await.ok();
            tx.send(utils::info_packet(&format!("Sent friend request to {}!", &x.receiver))).await.ok();
            info_code = 5;

        },
        "REJECTED" => {
            client.friend_requests.retain(|req| req.sender != x.sender && req.receiver != x.receiver);
            Account::update_account(&client).await.ok();

            println!("{:#?} || {:#?}", friend.friend_requests, &x);
            friend.friend_requests.retain(|req| req.sender != x.sender && req.receiver != x.receiver);
            Account::update_account(&friend).await.ok();

            tx.send(utils::info_packet("Friend request cancelled.")).await.ok();

            info_code = 6;
            
        },
        _ => { tx.send(utils::info_packet("Invalid friend request status.")).await.ok(); return Ok(()); }
    }

        let c_packet: WSPacket = WSPacket { sender: String::from("API"), sid: String::from("0"), action: WSAction::ReceiveArbitraryInfo(serde_json::to_string(&x).unwrap(), info_code) };
        if tx.send(c_packet).await.is_err() 
        { error!("Failed to send conversations to client {who}. Did they abruptly disconnect?") }

        let Some(friend_client) = store.values().find(|c| &c.username == &x.receiver)
        else { return Ok(()) }; // user is not online

        let f_packet = WSPacket { sender: String::from("API"), sid: String::from("0"), action: WSAction::ReceiveArbitraryInfo(serde_json::to_string(&x).unwrap(), info_code) };
        if friend_client.socket.send(f_packet).await.is_err() 
        { error!("Failed to send conversations to reciever. Did they abruptly disconnect?") }
        Ok(())
    }

    

