use super::generics::{
    structs::{Account, ClientAccount}, utils
};
use super::mongo;
use axum::http::StatusCode;

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
pub async fn delete_user(payload: String) -> StatusCode
{
    // parse the payload into a ClientAccount
    let account: ClientAccount = serde_json::from_str(&payload).unwrap();
    mongo::ping().await;

    // verify session
    if utils::verify(&account.username, &account.session_id).await
    {
        // delete the account
        match Account::delete_account(&account.username).await
        {
            Ok(_) => return StatusCode::OK,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR
        }
    }
    else
    {
        // Returned if the session id is invalid
        return StatusCode::UNAUTHORIZED;
    }
}
