use axum::http::StatusCode;
use axum::response::IntoResponse;
use super::generics::structs::{Account, ClientAccount};
use argon2::Argon2;
use base64::{engine::general_purpose, Engine as _};
use getrandom::getrandom;
use super::mongo;

/// Updates a user's data.
pub async fn update(payload: String) -> impl IntoResponse {
    // Payload is a ClientAccount
    let mut account: ClientAccount = serde_json::from_str(&payload).unwrap();
    mongo::ping().await;

    // validate SID
    if let Some(server_account) = Account::get_account_by_sid(&account.session_id).await
    {
        let username = server_account.username;
        let password = server_account.hash;
        if account.username != server_account.username && account.username != ""
        {
            // opening for username change
        }
        Argon2::default()
            .hash_password_into(&account.password.clone().into_bytes(), &server_account.salt, &mut output)
            .expect("failed to hash password");
        let base64_encoded = general_purpose::STANDARD.encode(output);
        if base64_encoded == server_account.hash && account.password != ""
        {
            // opening for password change
        }
        if account.friends != server_account.friends && account.friends != vec![]
        {
            // opening for friends change
        }
        // update account in db
        let account = Account
        {
            username: account.username,
            hash: server_account.hash,
            salt: server_account.salt,
            session_id: account.session_id,
            friends: account.friends,
            public_key: server_account.public_key,
            priv_key_enc: server_account.priv_key_enc

        };
        match Account::update_account(&account).await
        {
            Ok(data) => return (StatusCode::OK, data.session_id),
            // Returned if account failed to update
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "".to_string())
        }


    }
    else
    {
        // Returned if the SID can't be found
        // + StatusCode::UNAUTHORIZED
        // - StatusCode::BAD_REQUEST
        return (StatusCode::UNAUTHORIZED, "".to_string())
        
    }
}