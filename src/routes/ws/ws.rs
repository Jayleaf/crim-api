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
use tracing::{error, info};
use crate::{generics::{structs::{ClientStore, WSPacket}, utils}, routes::ws::recieve_ws};
use axum::extract::connect_info::ConnectInfo;

/// Handles incoming websocket connections.
pub async fn ws_handler(ws: WebSocketUpgrade, user_agent: Option<TypedHeader<headers::UserAgent>>, ConnectInfo(addr): ConnectInfo<SocketAddr>, State(store): State<ClientStore>) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, State(store)))
}

/// Websocket Statemachine
async fn handle_socket(socket: WebSocket, who: SocketAddr, State(store): State<ClientStore>) {
    let (tx, mut rx) = mpsc::channel::<WSPacket>(100);

    let (mut write, mut read) = socket.split();


    // Ran whenever `rx` recieves a message from `tx` through a `tx.send()` call
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let msg = serde_json::to_string(&msg).unwrap();
            if write
                .send(Message::Text(msg.clone()))
                .await
                .is_err()
            {
                error!("Failed to send message");
                return;
            }
        }
    });

    // Ran whenever the client sends messages to the websocket
    let recv_task = tokio::spawn
    ({ let store = store.clone(); async move 
        {
            while let Some(Ok(msg)) = read.next().await 
            {
                let Ok(message) = serde_json::from_str::<WSPacket>(msg.to_text().unwrap())
                else { tx.send( utils::info_packet("Invalid WSPacket.")).await.ok(); continue; };
                info!("Recieved message from {who}: {:#?}", message);
                recieve_ws::recieve_ws(message, who, State(store.clone()), tx.clone()).await;
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
    
}
