use super::generics::{utils, structs::{Account, ClientAccount}};
use crate::db::mongo;
use argon2::{self, Config};
use axum::{http::StatusCode, response::IntoResponse};

/// Changes a user's password.
///
/// ## Arguments
/// * [`payload`][`std::string::String`] - A JSON string containing a serialized ClientAccount of the account to create.
///
/// ## Returns
/// * [`StatusCode`][`axum::http::StatusCode`] - The status code of the operation:
///    * 200 OK if the password changed successfully
///    * 400 BAD REQUEST if the account doesn't exist, or if the payload is invalid
///    * 401 UNAUTHORIZED if the password is incorrect or the SID is incorrect
///    * 500 INTERNAL SERVER ERROR if there was an error connecting to the database at any point
///
pub async fn change_password(payload: String) -> impl IntoResponse
{
    // parse the string to an account value
    let Ok(account) = serde_json::from_str::<ClientAccount>(&payload) 
    else { return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid Payload."))};
    
    if let Err(_) = mongo::ping().await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Error occurred connecting to database.".to_string());
    }

    let server_account = match Account::get_account(&account.username).await
    {
        Ok(Some(account)) => account,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e),
        Ok(None) => return (StatusCode::BAD_REQUEST, utils::gen_err("Tried changing the password of a non-existent account. Confirm the username is correct."))
    };
    
    // requires extra layer of security, will be asked for password to confirm

    let Ok(true) = argon2::verify_encoded(&server_account.hash, &account.password.as_bytes()) // doesn't check for an Argon2 error
    else { return (StatusCode::UNAUTHORIZED, utils::gen_err("Invalid password.")) };

    match utils::verify(&account.username, &account.session_id).await
    {
        Ok(false) => { return (StatusCode::UNAUTHORIZED, utils::gen_err("Invalid session ID.")) },
        Err(e) =>  { return (StatusCode::INTERNAL_SERVER_ERROR, e) } ,
        Ok(true) => {()}
    }

    let salt = utils::rand_hex(32);
    let config = Config::default();
    let hash: String = argon2::hash_encoded(account.password.as_bytes(), &salt.as_bytes(), &config).unwrap();

    let account: Account = Account {
        username: server_account.username,
        hash,
        public_key: server_account.public_key,
        priv_key_enc: server_account.priv_key_enc,
        friends: server_account.friends,
        friend_requests: server_account.friend_requests,
        session_id: utils::rand_hex(32) // invalidate session on password change
    };
    
    if let Err(e) = Account::update_account(&account).await
    { return (StatusCode::INTERNAL_SERVER_ERROR, utils::gen_err(&e)) }
    else { return (StatusCode::OK, "Password changed successfully.".to_string())}
}
