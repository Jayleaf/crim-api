use axum::{http::StatusCode, response::IntoResponse};
use getrandom::getrandom;
use crate::db::mongo;
use argon2::Argon2;
use base64::{engine::general_purpose, Engine as _};
use super::generics::structs::{ClientAccount, Account};
use openssl::{pkey::PKey, rsa::Rsa, symm::Cipher};

/// Creates a user entry in the database. If it was successful, returns 200 OK. If not, returns 400 Bad Request.
pub async fn create_user(payload: String) -> impl IntoResponse {
    // parse the string to an account value
    let account: ClientAccount = serde_json::from_str(&payload).unwrap();
    mongo::ping().await;
    if Account::get_account(&account.username).await.is_some() {
        // if an account with the username exists, return a 400 Bad Request with a reason.
        return (StatusCode::BAD_REQUEST, "Duplicate username".to_string())
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

    let pkey: PKey<openssl::pkey::Private> = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let cipher: Cipher = Cipher::aes_256_cbc();
    let public_key: Vec<u8> = pkey.public_key_to_pem().unwrap();
    let private_key: Vec<u8> = pkey
        .private_key_to_pem_pkcs8_passphrase(cipher, &account.password.as_bytes())
        .unwrap();

    // build account value
    let account = Account {
        username: account.username,
        hash: base64_encoded,
        salt: salt.to_vec(),
        public_key: public_key,
        priv_key_enc: private_key,
        friends: Vec::new(),
        session_id: "".to_string()
    };
    // write account value to database
    match Account::create_account(&account).await {
        Ok(_) => return (StatusCode::OK, "".to_string()),
        Err(_) => return (StatusCode::BAD_REQUEST, "Error occurred saving account to database.".to_string())
    }  
}