use super::generics::structs::{Account, ClientAccount};
use super::mongo;
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
    let account: ClientAccount = serde_json::from_str(&payload).unwrap();
    mongo::ping().await;
    if let Some(server_account) = Account::get_account(&account.username).await
    {
        // check if password is correct
        let mut output: [u8; 256] = [0u8; 256];
        Argon2::default()
            .hash_password_into(&account.password.clone().into_bytes(), &server_account.salt, &mut output)
            .expect("failed to hash password");
        let base64_encoded = general_purpose::STANDARD.encode(output);
        if base64_encoded == server_account.hash
        {
            // generate a new session id
            let mut session_id: [u8; 32] = [0u8; 32];
            getrandom(&mut session_id).expect("Failed to generate a random SID");
            let session_id = general_purpose::STANDARD.encode(session_id);
            // update the account with the new session id
            let mut server_account = server_account;
            server_account.session_id = session_id;
            match Account::update_account(&server_account).await
            {
                Ok(data) => return (StatusCode::OK, data.session_id + "|PRIVATEKEY:|" + &data.priv_key_enc.iter().map(|&x| x.to_string()).collect::<Vec<String>>().join(",")),
                // Returned if account failed to update
                Err(_) => return (StatusCode::BAD_REQUEST, "".to_string())
            }
        }
        else
        {
            // Returned if the password is invalid
            return (StatusCode::UNAUTHORIZED, "".to_string());
        }
    }
    else
    {
        // Returned if the account doesn't exist
        return (StatusCode::UNAUTHORIZED, "".to_string());
    }
}
