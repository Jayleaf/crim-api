use super::generics::{utils, structs::{Account, ClientAccount}};
use argon2::Argon2;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use base64::{engine::general_purpose, Engine as _};
use getrandom::getrandom;
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

    let mut output: [u8; 256] = [0u8; 256];
    if let Err(_) = Argon2::default().hash_password_into(&client_account.password.clone().into_bytes(), &server_account.salt, &mut output)
    { return (StatusCode::INTERNAL_SERVER_ERROR, utils::gen_err("Error with password hashing.")) }

    let base64_encoded = general_purpose::STANDARD.encode(output);
    if base64_encoded != server_account.hash { return (StatusCode::UNAUTHORIZED, utils::gen_err("Invalid Username or Password.")) }

    let mut session_id: [u8; 32] = [0u8; 32];
    if let Err(_) = getrandom(&mut session_id)
    { return (StatusCode::INTERNAL_SERVER_ERROR, utils::gen_err("Error generating random session ID."))}
    let session_id = general_purpose::STANDARD.encode(session_id);
    server_account.session_id = session_id;

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
