use std::net::SocketAddr;
use axum::extract::State;
use crate::tokio::sync::mpsc::Sender;

use super::generics::{utils, structs::{ClientStore, WebsocketClient, WSPacket}};

/// Register a client into the ClientStore, so that they may recieve and send messages through WS.
/// 
/// ## Arguments
/// * [`account`][`ClientAccount`] - The account to register.
/// * [`store`][`ClientStore`] - Our ClientStore state
/// * [`tx`][`Sender<WSPacket>`] - Transmitter so we can send messages back to the client
/// 
pub async fn register(packet: &WSPacket, who: SocketAddr, State(store): State<ClientStore>, tx: &Sender<WSPacket>)
{

    let mut store = store.lock().await;
    if store.contains_key(&who)
    {
        tx.send(utils::info_packet("Client already registered.")).await.ok();
        return;
    }

    match utils::verify(&packet.sender, &packet.sid).await
    {
        Ok(true) => (),
        Ok(false) => { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return; },
        Err(e) => { tx.send(utils::info_packet(&utils::gen_err(&e))).await.ok(); return; }
    }

    // make a new channel
    store.insert(who, WebsocketClient { username: packet.sender.to_string(), session_id: packet.sid.to_string(), socket: tx.clone() });
    tx.send(utils::info_packet("Registered")).await.ok();
    return;
}