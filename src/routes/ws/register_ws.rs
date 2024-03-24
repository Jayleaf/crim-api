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
/// ## Returns
/// * An `OK`, 100% of the time.
pub async fn register(packet: WSPacket, who: SocketAddr, State(store): State<ClientStore>, tx: &Sender<WSPacket>) -> Result<(), ()>
{
    // is this person already in our ClientStore?
    let mut store = store.lock().await;
    if store.contains_key(&who)
    {
        tx.send(utils::info_packet("Client already registered.")).await.ok();
        return Ok(());
    }

    // is this even a real person?
    match utils::verify(&packet.sender, &packet.sid).await
    {
        Ok(true) => (),
        Ok(false) => { tx.send(utils::info_packet("Invalid session ID.")).await.ok(); return Ok(()); },
        Err(e) => { tx.send(utils::info_packet(&utils::gen_err(&e))).await.ok(); return Ok(()); }
    }

    // make a new channel
    store.insert(who, WebsocketClient { username: packet.sender, session_id: packet.sid, socket: tx.clone() });
    tx.send(utils::info_packet("Registered")).await.ok();

    drop(store);
    Ok(())
}