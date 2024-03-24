use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use futures::{future, pin_mut, SinkExt, StreamExt};
use tokio::{self, sync::mpsc};
use std::net::SocketAddr;

use crate::{generics::{structs::{ClientStore, WSAction, WSPacket}, utils}, routes::ws::{register_ws, send_ws}};
//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

pub async fn ws_handler(ws: WebSocketUpgrade, user_agent: Option<TypedHeader<headers::UserAgent>>, ConnectInfo(addr): ConnectInfo<SocketAddr>, State(store): State<ClientStore>) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");

    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, State(store)))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(socket: WebSocket, who: SocketAddr, State(store): State<ClientStore>) {

    let (tx, mut rx) = mpsc::channel::<WSPacket>(100);

    let (mut write, mut read) = socket.split();

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let msg = serde_json::to_string(&msg).unwrap();
            if write
                .send(Message::Text(msg.clone()))
                .await
                .is_err()
            {
                println!("Failed to send message");
                return;
            }
            println!("Sent message: {:?}", msg)
        }
    });

    let recv_task = tokio::spawn
    ({ let store = store.clone(); async move 
        {
            while let Some(Ok(msg)) = read.next().await 
            {
                let Ok(message) = serde_json::from_str::<WSPacket>(msg.to_text().unwrap())
                else { tx.send( utils::info_packet("Invalid WSPacket.")).await.ok(); continue; };

                match message.action
                {
                    WSAction::Register() => 
                    {
                        register_ws::register(message.clone(), who, State(store.clone()), &tx).await.ok();
                        println!("Client {} registered", &message.sender);
                        continue;
                    }
                    WSAction::Disconnect() => 
                    {
                        let mut store = store.lock().await;
                        store.remove(&who);
                        drop(store);
                    }
                    WSAction::SendMessage(d) => 
                    {
                        send_ws::send_msg(d, who, State(store.clone()), &tx).await.ok();
                        println!("Client {} sent a message", &message.sender);
                        continue;
                    }
                    WSAction::AddFriend(_) => { tx.send(utils::info_packet("Not implemented.")).await.ok(); continue; }
                    WSAction::RemoveFriend(_) => { tx.send(utils::info_packet("Not implemented.")).await.ok(); continue; }
                    WSAction::CreateConversation(_) => { tx.send(utils::info_packet("Not implemented.")).await.ok(); continue; } // planned
                    WSAction::DeleteConversation(_) => { tx.send(utils::info_packet("Not implemented.")).await.ok(); continue; } // planned
                    WSAction::Info(_) => { tx.send(utils::info_packet("Server does not accept info packets.")).await.ok(); continue; } // planned
                    WSAction::ReceiveMessage(_) => { tx.send(utils::info_packet("Server does not accept recieve message packets.")).await.ok(); continue; }
                }  
            }
        }
    });

    pin_mut!(send_task, recv_task);
    future::select(send_task, recv_task).await;

    // returning from the handler closes the websocket connection
    println!("Websocket context {who} destroyed");

    // remove the client from the store. This does the same thing as a Disconnect() packet, but is here in case the client disconnects without sending a Disconnect() packet.
    let mut store = store.lock().await;
    store.remove(&who);
    drop(store);
    
}
