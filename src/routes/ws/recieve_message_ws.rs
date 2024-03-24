use axum::extract::{ws::WebSocket, State};
use std::net::SocketAddr;

use super::generics::structs::{ClientStore, WebsocketClient};
// websocket send message
pub async fn recieve_msg(mut socket: WebSocket, who: SocketAddr, State(store): State<ClientStore>)
{
    // the first thing we need to do is figure out who this client is
    let store = store.lock().unwrap();
    let client: &WebsocketClient = store.get(&who).unwrap();

    // debug
    println!("Client {} tried to send a message with SID {}", who, client.session_id);
}