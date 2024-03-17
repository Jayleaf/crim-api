use axum::http::StatusCode;
use axum::response::IntoResponse;
use super::generics::structs::{Account, ClientAccount};
use argon2::Argon2;
use base64::{engine::general_purpose, Engine as _};
use getrandom::getrandom;
use super::mongo;

/// "Logs" a user in. Generates a session ID and spits it back if the login was successful.
pub async fn login_user(payload: String) -> impl IntoResponse {
    // the payload is going to be a clientaccount, containing plaintext username, password, and SID. SID doesn't matter because it's gonna get regenerated anyway.
    let account: ClientAccount = serde_json::from_str(&payload).unwrap();
    mongo::ping().await;
    if let Some(server_account) = Account::get_account(&account.username).await
    {
        // check if the provided password is valid by hashing the plaintext password and comparing it to the db entry
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
                        Ok(data) => return (StatusCode::OK, data.session_id),
                        // Returned if account failed to update
                        Err(_) => return (StatusCode::BAD_REQUEST, "".to_string())
                    }
                }
                else
                {
                    // Returned if the password is invalid
                    return (StatusCode::UNAUTHORIZED, "".to_string())
                }

    }
    else
    {
        // Returned if the account doesn't exist
        // + StatusCode::UNAUTHORIZED
        // - StatusCode::BAD_REQUEST
        return (StatusCode::UNAUTHORIZED, "".to_string())
        
    }
}