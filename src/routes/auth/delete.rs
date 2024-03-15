use axum::http::StatusCode;
use super::mongo;
use super::generics::structs::{ClientAccount, Account};

/// Deletes entry in the database. If it was successful, returns 200 OK. If not, returns 400 Bad Request.
pub async fn delete_user(payload: String) -> StatusCode {
    // parse the string to an account value
    //TODO: use utils::verify in this
    let account: ClientAccount = serde_json::from_str(&payload).unwrap();
    println!("Parsed!");
    mongo::ping().await;
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