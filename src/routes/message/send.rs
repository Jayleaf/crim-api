use axum::response::IntoResponse;

/// Uploads a message to a conversation in the database. The raw message is received, encrypted with the conversation key, and uploaded to the database.
pub async fn send(payload: String) -> impl IntoResponse {}
