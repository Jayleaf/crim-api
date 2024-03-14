use axum::http::StatusCode;

use super::structs::{ClientAccount, Account};

/// Creates a user entry in the database. If it was successful, returns 200 OK. If not, returns 400 Bad Request.
pub async fn create_user(payload: String) -> StatusCode {
    // parse the string to an account value
    let account: ClientAccount = serde_json::from_str(&payload).unwrap();
    // handle account creation logic here. no time to do it now
    println!("Parsed!");
    // check for username uniquity
    println!("Queried Database!");
    if Account::get_account(&account.username).await.is_some() {
        // return an error
        return StatusCode::BAD_REQUEST;
    }
    else
    {
        // create the account
        let success = Account::create_account(&account).await;
        if success.is_err()
        {
            println!("{:?}", success.err().unwrap().to_string());
            return StatusCode::BAD_REQUEST;
        }
        return StatusCode::OK;
    }
    
}