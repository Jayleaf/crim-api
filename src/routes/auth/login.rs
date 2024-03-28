use super::generics::{utils, structs::{Account, ClientAccount}};
use argon2;
use axum::http::StatusCode;
use axum::response::IntoResponse;
/// "Logs" a user in. Generates a session ID and spits it back if the login was successful.
///
/// ## Arguments
/// * [`payload`][`std::string::String`] - A JSON string containing a serialized ClientAccount of the account to log into.
///     * Utilized Fields:
///         * `username`
///         * `password`
///
/// ## Returns
/// * [`(StatusCode, String)`][axum::response::Response] - A tuple containing the [`StatusCode`] of the request and a [`String`] containing the newly minted session ID and the encrypted private key, separated by the signifier "|PRIVATEKEY:|"
/// 
pub async fn login_user(payload: String) -> impl IntoResponse
{
    let Ok(client_account) = serde_json::from_str::<ClientAccount>(&payload) 
    else { return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid Payload."))};
    
    let mut server_account: Account = match Account::get_account(&client_account.username).await
    {
        Ok(Some(account)) => account,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e),
        Ok(None) => return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid Username or Password."))
    };

    let Ok(true) = argon2::verify_encoded(&server_account.hash, client_account.password.as_bytes()) // doesn't check for an Argon2 error
    else { return (StatusCode::UNAUTHORIZED, utils::gen_err("Invalid Username or Password.")) };

    server_account.session_id = utils::rand_hex(32);

    if let Err(e) = Account::update_account(&server_account).await 
    { return (StatusCode::INTERNAL_SERVER_ERROR, e) }
    else 
    { 
        return (
        StatusCode::OK, 
        server_account.session_id + 
        "|PRIVATEKEY:|" 
        + &server_account.priv_key_enc
            .iter()
            .map(|&x| x.to_string()).
            collect::<Vec<String>>()
            .join(",")
        ) 
    }
}
