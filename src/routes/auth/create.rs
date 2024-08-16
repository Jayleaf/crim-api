use super::generics::{utils, structs::{Account, ClientAccount}};
use crate::db::mongo;
use argon2::{self, Config};
use axum::{debug_handler, http::StatusCode, response::IntoResponse};
use rsa::{pkcs8::{EncodePrivateKey, EncodePublicKey}, RsaPrivateKey, RsaPublicKey};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, generic_array},
    Aes256Gcm, Nonce, Key // Or `Aes128Gcm`
    
};

/// Creates a user entry in the database.
///
/// ## Arguments
/// * [`payload`][`std::string::String`] - A JSON string containing a serialized ClientAccount of the account to create.
///
/// ## Returns
/// * [`StatusCode`][`axum::http::StatusCode`] - The status code of the operation:
///    * 200 OK if the account was created successfully
///    * 400 BAD REQUEST if the account already exists
///
#[debug_handler]
pub async fn create_user(payload: String) -> impl IntoResponse
{
    // parse the string to an account value
    let Ok(account) = serde_json::from_str::<ClientAccount>(&payload) 
    else { return (StatusCode::BAD_REQUEST, utils::gen_err("Invalid Payload."))};
    
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
    let salt = utils::rand_hex(32);
    let config = Config::default();
    let hash: String = argon2::hash_encoded(account.password.as_bytes(), &salt.as_bytes(), &config).unwrap();

    let priv_key: RsaPrivateKey = {
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
        RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key")
    };
    let pub_key: RsaPublicKey = RsaPublicKey::from(&priv_key);
    let public_key = pub_key.to_public_key_pem(rsa::pkcs8::LineEnding::CRLF).unwrap().as_bytes().to_vec();
    let private_key = priv_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::CRLF).unwrap();
    let pvkeyhash: Vec<u8> = argon2::hash_raw(account.password.as_bytes(), b"00000000", &config).unwrap();
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng).to_vec();
    let key = Key::<Aes256Gcm>::from_slice(&pvkeyhash);
    println!("{:#?}", key);
    let private_key = Aes256Gcm::new(&key).encrypt(&generic_array::GenericArray::clone_from_slice(nonce.as_slice()), private_key.as_bytes().as_ref()).unwrap();    
    let account: Account = Account {
        username: account.username,
        hash,
        public_key,
        priv_key_enc: private_key,
        nonce,
        friends: Vec::new(),
        friend_requests: Vec::new(),
        session_id: "".to_string()
        
    };
    
    match Account::create_account(&account).await
    {
        Ok(_) => return (StatusCode::OK, "".to_string()),
        Err(_) => return (StatusCode::BAD_REQUEST, utils::gen_err("Error creating account."))
    }
}
