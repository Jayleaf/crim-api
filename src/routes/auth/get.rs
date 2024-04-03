use crate::generics::{structs::Conversation, utils};
use axum::{extract::Path, http::StatusCode};
use axum::response::IntoResponse;
use super::generics::structs::{Account, ClientAccount};


/// Gets a users data (conversations included) from the database.
///
/// ## Arguments
/// * [`payload`][`std::string::String`] - A JSON string containing a serialized SID.
///
/// ## Returns
/// * [`(StatusCode, String)`][axum::response::Response] - A tuple containing the [`StatusCode`] of the request and a serialized [`ClientAccount`] value.
///
pub async fn get( Path(sid): Path<String>,) -> impl IntoResponse
{
    println!("GET");
    let server_account: Account = match Account::get_account_by_sid(&sid).await
    {
        Ok(Some(account)) => account,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e),
        Ok(None) => return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid SID."))
    };
    
    let convos: Vec<Conversation> = match Conversation::get_all(&server_account.username).await
    {
        Ok(convos) => convos,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e)
    };
    
    let result: ClientAccount = ClientAccount 
    {
        username: server_account.username,
        password: String::new(),
        friends: server_account.friends,
        friend_requests: server_account.friend_requests,
        conversations: convos,
        session_id: String::new(),
    };

    return (StatusCode::OK, serde_json::to_string(&result).unwrap());
}
