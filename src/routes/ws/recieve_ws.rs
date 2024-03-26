use super::{generics::{
    structs::{ClientStore, WSAction, WSPacket},
    utils,
}, make_convo_ws, send_ws, register_ws};
use crate::tokio::sync::mpsc::Sender;
use axum::extract::State;
use std::net::SocketAddr;

// Handles incoming websocket packets.
//
// ## Parameters:
// * [`socket`][`axum::extract::ws::WebSocket`] - The websocket connection.
// * [`who`][`std::net::SocketAddr`] - The address of the client.
// * [`State<ClientStore>`][`axum::extract::State`] - The global client store.
//
pub async fn recieve_ws(packet: WSPacket, who: SocketAddr, State(store): State<ClientStore>, tx: Sender<WSPacket>) {
    match packet.action {
        WSAction::Register() => 
        {
            register_ws::register(&packet, who, State(store.clone()), &tx).await;
            println!("Client {} registered", &packet.sender);
        }
        WSAction::Disconnect() => 
        {
            let mut store = store.lock().await;
            store.remove(&who);
        }
        WSAction::SendMessage(d) => 
        {
            send_ws::send_msg(d, who, State(store.clone()), &tx).await;
        }
        WSAction::AddFriend(_) => 
        {
            tx.send(utils::info_packet("Not implemented."))
                .await
                .ok();
        }
        WSAction::RemoveFriend(_) => 
        {
            tx.send(utils::info_packet("Not implemented."))
                .await
                .ok();
        }
        WSAction::CreateConversation(_) => 
        {
            make_convo_ws::make_convo(packet, who, State(store.clone()), &tx).await;
        }
        WSAction::DeleteConversation(_) => 
        {
            tx.send(utils::info_packet("Not implemented."))
                .await
                .ok();
        } // planned
        WSAction::Info(_) => 
        {
            tx.send(utils::info_packet("Server does not accept info packets."))
                .await
                .ok();
        } // planned
        WSAction::ReceiveMessage(_) => 
        {
            tx.send(utils::info_packet("Server does not accept recieve message packets."))
                .await
                .ok();
        }
        WSAction::RecieveConversation(_) => 
        {
            tx.send(utils::info_packet("Server does not accept a recieve conversation packet."))
                .await
                .ok();
        }
    }
}
