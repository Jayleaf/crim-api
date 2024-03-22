use super::generics::{
    structs::{Account, ClientAccount}, utils
};
use axum::{http::StatusCode, response::IntoResponse};

/// Deletes a user entry in the database.
///
/// ## Arguments
/// * [`payload`][`super::generics::structs::ClientAccount`] - A JSON string containing the a serialized of the user to be deleted.
///     * Utilized Fields:
///         * `username`
///         * `session_id`
///
/// ## Returns
/// * [`StatusCode`][axum::http::StatusCode] - The status code of the operation:
///     * 200 OK if deletion was successful
///     * 500 INTERNAL_SERVER_ERROR if an error occurred deleting the account
///     * 401 UNAUTHORIZED if the session is invalid.
///
pub async fn delete_user(payload: String) -> impl IntoResponse
{

    let Ok(account) = serde_json::from_str::<ClientAccount>(&payload) 
    else { return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid Payload."))};

    match utils::verify(&account.username, &account.session_id).await
    {
        Ok(true) => (),
        Ok(false) => return (StatusCode::UNAUTHORIZED, utils::gen_err("Invalid session ID.")),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e)
    }
    
    if let Err(e) = Account::delete_account(&account.username).await { return (StatusCode::INTERNAL_SERVER_ERROR, e) }
    else { return (StatusCode::OK, String::new()); }

}
