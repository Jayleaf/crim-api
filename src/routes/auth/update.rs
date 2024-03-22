use crate::routes::message::make;

use super::generics::{utils, structs::{Account, ClientAccount, UpdateUser, UpdateAction}};
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Generic function for updating a user's data in the database.
///
/// ## Arguments
/// * [`payload`][`UpdateUser`] - A JSON string containing a serialized [`UpdateUser`] value.
///
/// ## Returns the
/// * [`(StatusCode, String)`][axum::response::Response] - A tuple containing the status code of the request and a serialized ClientAccount value.
///     * 200 OK if action was successful
///     * 401 UNAUTHORIZED if the session ID was invalid
///     * 500 INTERNAL_SERVER_ERROR if an error occurred updating the account
///
pub async fn update(payload: String) -> impl IntoResponse
{
    let Ok(update_val) = serde_json::from_str::<UpdateUser>(&payload) 
    else { return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid Payload."))};

    let mut server_account = match Account::get_account_by_sid(&update_val.session_id).await
    {
        Ok(Some(account)) => account,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e),
        Ok(None) => return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid SID."))
    };

    match update_val.action
    {
        UpdateAction::None => return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid action.")),

        UpdateAction::AddFriend =>
        {
            let target = &update_val.data;
            let mut friend = match Account::get_account(target).await
            {
                Ok(Some(account)) => account,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e),
                Ok(None) => return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid friend username."))
            };
            server_account.friends.push(friend.username.clone());
            friend.friends.push(server_account.username.clone());
            if let Err(e) = Account::update_account(&friend).await 
            { return (StatusCode::INTERNAL_SERVER_ERROR, e) };

            if let Err(e) = make::create_conversation(vec![&server_account.username, &friend.username]).await 
            { return (StatusCode::INTERNAL_SERVER_ERROR, e) };
            
        },

        UpdateAction::RemoveFriend =>
        {
            let target = &update_val.data;
            let mut friend = match Account::get_account(target).await
            {
                Ok(Some(account)) => account,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e),
                Ok(None) => return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid friend username."))
            };
            server_account.friends.retain(|x| x != &friend.username);
            friend.friends.retain(|x| x != &server_account.username);
            if let Err(e) = Account::update_account(&friend).await { return (StatusCode::INTERNAL_SERVER_ERROR, e) }
        },

        UpdateAction::ChangePassword =>
        {
            return (StatusCode::NOT_IMPLEMENTED, utils::gen_err("Not implemented yet."))
        },

        UpdateAction::ChangeUsername =>
        {
            return (StatusCode::NOT_IMPLEMENTED, utils::gen_err("Not implemented yet."))
        }
    }

    if let Err(e) = Account::update_account(&server_account).await 
    { return (StatusCode::INTERNAL_SERVER_ERROR, e) }

    let client_account: ClientAccount = ClientAccount
    {
        username: server_account.username,
        password: if update_val.action == UpdateAction::ChangePassword { update_val.data } else { String::new() },
        friends: server_account.friends,
        conversations: Vec::new(),
        session_id: update_val.session_id
    };

    return (StatusCode::OK, serde_json::to_string(&client_account).unwrap()) 
  
}
