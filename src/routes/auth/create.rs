use super::generics::{utils, structs::{Account, ClientAccount}};
use crate::db::mongo;
use argon2::{self, Config};
use axum::{http::StatusCode, response::IntoResponse};
use getrandom::getrandom;
use openssl::{pkey::{PKey, Private}, rsa::Rsa, symm::Cipher};

/// Creates a user entry in the database. If it was successful, returns 200 OK. If not, returns 400 Bad Request.
///
/// ## Arguments
/// * [`payload`][`std::string::String`] - A JSON string containing a serialized ClientAccount of the account to create.
///
/// ## Returns
/// * [`StatusCode`][`axum::http::StatusCode`] - The status code of the operation:
///    * 200 OK if the account was created successfully
///    * 400 BAD REQUEST if the account already exists
///
pub async fn create_user(payload: String) -> impl IntoResponse
{
    // parse the string to an account value
    let Ok(account) = serde_json::from_str::<ClientAccount>(&payload) else { return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid Payload."))};
    if let Err(_) = mongo::ping().await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Error occurred connecting to database.".to_string());
    }

    if let Err(e) = Account::get_account(&account.username).await
    {
        return (StatusCode::BAD_REQUEST, e);
    }
    // create account

    // first, create pw hash
    let mut salt: [u8; 32] = [0u8; 32];
    getrandom(&mut salt).expect("Failed to generate a random salt");
    let config = Config::default();
    let hash: String = argon2::hash_encoded(account.password.as_bytes(), &salt, &config).unwrap();

    let pkey: PKey<Private> = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let cipher: Cipher = Cipher::aes_256_cbc();
    let public_key: Vec<u8> = pkey.public_key_to_pem().unwrap();
    let private_key: Vec<u8> = pkey
        .private_key_to_pem_pkcs8_passphrase(cipher, &account.password.as_bytes())
        .unwrap();

    let account: Account = Account {
        username: account.username,
        hash,
        public_key,
        priv_key_enc: private_key,
        friends: Vec::new(),
        session_id: "".to_string()
    };
    
    match Account::create_account(&account).await
    {
        Ok(_) => return (StatusCode::OK, "".to_string()),
        Err(_) => return (StatusCode::BAD_REQUEST, utils::gen_err("Error creating account."))
    }
}
