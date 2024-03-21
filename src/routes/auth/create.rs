use super::generics::structs::{Account, ClientAccount};
use crate::db::mongo;
use argon2::Argon2;
use axum::{http::StatusCode, response::IntoResponse};
use base64::{engine::general_purpose, Engine as _};
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
    let account: ClientAccount = serde_json::from_str(&payload).unwrap();
    if let Err(_) = mongo::ping().await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Error occurred connecting to database.".to_string());
    }

    if let None = Account::get_account(&account.username).await
    {
        return (StatusCode::BAD_REQUEST, "Duplicate username".to_string());
    }
    // create account

    // first, create pw hash
    let mut salt: [u8; 32] = [0u8; 32];
    getrandom(&mut salt).expect("Failed to generate a random salt");
    let mut output: [u8; 256] = [0u8; 256];
    Argon2::default()
        .hash_password_into(&account.password.as_bytes(), &salt, &mut output)
        .expect("failed to hash password");
    let base64_encoded = general_purpose::STANDARD.encode(output);
    // then, gen public and private keys

    let pkey: PKey<Private> = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let cipher: Cipher = Cipher::aes_256_cbc();
    let public_key: Vec<u8> = pkey.public_key_to_pem().unwrap();
    let private_key: Vec<u8> = pkey
        .private_key_to_pem_pkcs8_passphrase(cipher, &account.password.as_bytes())
        .unwrap();

    // build account value
    let account: Account = Account {
        username: account.username,
        hash: base64_encoded,
        salt: salt.to_vec(),
        public_key: public_key,
        priv_key_enc: private_key,
        friends: Vec::new(),
        session_id: "".to_string()
    };
    // write account value to database
    match Account::create_account(&account).await
    {
        Ok(_) => return (StatusCode::OK, "".to_string()),
        Err(_) => return (StatusCode::BAD_REQUEST, "Error occurred saving account to database.".to_string())
    }
}
