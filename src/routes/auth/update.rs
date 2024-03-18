use axum::http::StatusCode;
use axum::response::IntoResponse;
use super::generics::structs::{Account, ClientAccount};
use argon2::Argon2;
use base64::{engine::general_purpose, Engine as _};
use getrandom::getrandom;
use super::mongo;

/// Generic function for updating a user's data in the database.
/// 
/// ## Arguments
/// * [`payload`][`std::string::String`] - A JSON string containing a ```ClientAccount``` struct.
/// 
/// ## Returns
/// * [`(StatusCode, String)`][axum::response::Response] - A tuple containing the status code of the request and a serialized ClientAccount value.
/// 
pub async fn update(payload: String) -> impl IntoResponse
{

    let account: ClientAccount = serde_json::from_str(&payload).unwrap();
    mongo::ping().await;
    // validate SID
    if let Some(mut server_account) = Account::get_account_by_sid(&account.session_id).await
    {
        if &account.username != &server_account.username && &account.username != ""
        {
            // opening for username change
        }
        let mut output = [0u8; 32];

        // turn user-provided password into a hash
        Argon2::default()
            .hash_password_into(&account.password.clone().into_bytes(), &server_account.salt, &mut output)
            .expect("failed to hash password");
        let base64_encoded = general_purpose::STANDARD.encode(output);
        if base64_encoded != server_account.hash && account.password != ""
        {
           // hash the new provided password
           // a new salt will be generated
            let mut salt = [0u8; 16];
            getrandom(&mut salt).expect("failed to generate random salt");
            let mut output = [0u8; 32];
            Argon2::default()
                .hash_password_into(&account.password.clone().into_bytes(), &salt, &mut output)
                .expect("failed to hash password");
            let base64_encoded = general_purpose::STANDARD.encode(output);
            server_account.hash = base64_encoded;
            server_account.salt = salt.to_vec();

            // also reset the session ID
            let mut session_id = [0u8; 32];
            getrandom(&mut session_id).expect("failed to generate random session ID");
            let session_id = general_purpose::STANDARD.encode(session_id);
            server_account.session_id = session_id;

        }
        if account.friends != server_account.friends && !account.friends.is_empty()
        {
            let target: String = 
            {
                let mut target: String = String::new();
                for friend in &account.friends
                {
                    if friend.starts_with("T_")
                    {
                        target = friend.to_string().trim_start_matches("T_").to_string();
                    }
                }
                target
            };
            
            //check if target user exists in db

            if Account::get_account(&target).await.is_none()
            {
                return (StatusCode::NOT_FOUND, "".to_string());
            }
            // replace the target user in account.friends with the same user, removing the "T_" because it was only a tag to specify which friend was the target of the action.
            server_account.friends.push(target);
            server_account.friends = account.friends.into_iter().map(|x| x.trim_start_matches("T_").to_string()).collect();
            // update account in db
            match Account::update_account(&server_account).await
            {
                Ok(data) =>
                {
                    let returndata = ClientAccount 
                    {
                    username: data.username,
                    password: "".to_string(), // we don't really need to return this. client will be forced to relogin after password change anyway
                    friends: data.friends,
                    conversations: account.conversations,
                    session_id: data.session_id
                    };
                    return (StatusCode::OK, serde_json::to_string(&returndata).unwrap());
                }
                // Returned if account failed to update
                Err(_) => {return (StatusCode::INTERNAL_SERVER_ERROR, "".to_string());}
            }
        }
    }
    else
    {
        // Returned if the SID can't be found
        // + StatusCode::UNAUTHORIZED
        // - StatusCode::BAD_REQUEST
        return (StatusCode::UNAUTHORIZED, "".to_string());
        
    }
    return (StatusCode::UNAUTHORIZED, "".to_string());
}