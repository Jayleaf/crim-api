use axum::http::StatusCode;

use super::structs::Account;

/// Deletes entry in the database. If it was successful, returns 200 OK. If not, returns 400 Bad Request.
pub async fn delete_user(payload: String) -> StatusCode {
    // ok my idea for this is that when we switch to session-based authentication, we pass that in as well to make sure only an authenticated user can use an account.
    // session id would obv be stored in the account struct, would have to get changed
    // parse the string to an account value
    
    // also update this to use client account
    let account: Account = serde_json::from_str(&payload).unwrap();
    println!("Parsed!");
    // check if the account exists;
    if let Some(server_account) = Account::get_account(&account.username).await
    {
        // check if the provided session id is valid
        if server_account.session_id == account.session_id
        {
            // delete the account
            match Account::delete_account(&account.username).await
            {
                Ok(_) => return StatusCode::OK,
                Err(_) => return StatusCode::BAD_REQUEST
            }
        }
        else
        {
            // Returned if the session id is invalid
            return StatusCode::BAD_REQUEST
        }
    }
    else
    {
        // Returned if the account doesn't exist
        return StatusCode::BAD_REQUEST
        
    }
}