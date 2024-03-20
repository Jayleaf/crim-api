//----------------------------------------------//
//                                              //
//        File for commonly-used structs        //
//                                              //
//----------------------------------------------//

use super::mongo;
use mongodb::{bson::{self, doc, Document}, Cursor};
use openssl::{
    pkey::{Private, Public}, rsa::{Padding, Rsa}
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

//----------------------------------------------//
//                                              //
//                User Accounts                 //
//                                              //
//----------------------------------------------//

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Account
{
    pub username: String,
    pub hash: String,
    pub salt: Vec<u8>,
    pub public_key: Vec<u8>,
    pub priv_key_enc: Vec<u8>,
    pub friends: Vec<String>,
    pub session_id: String
}

impl Account
{
    /// Parses a BSON Document into an account value
    pub fn from_document(doc: bson::Document) -> Account
    {
        Account {
            username: doc.get_str("username").unwrap().to_string(),
            hash: doc.get_str("hash").unwrap().to_string(),
            salt: doc
                .get_array("salt")
                .unwrap()
                .iter()
                .map(|x| x.as_i32().unwrap() as u8)
                .collect::<Vec<u8>>(),
            public_key: doc
                .get_array("public_key")
                .unwrap()
                .iter()
                .map(|x| x.as_i32().unwrap() as u8)
                .collect::<Vec<u8>>(),
            priv_key_enc: doc
                .get_array("priv_key_enc")
                .unwrap()
                .iter()
                .map(|x| x.as_i32().unwrap() as u8)
                .collect::<Vec<u8>>(),
            friends: doc
                .get_array("friends")
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect(),
            session_id: doc.get_str("session_id").unwrap().to_string()
        }
    }

    /// Takes in a string, finds the matching account in the database, and returns it. Will return none if no account is found, or will panic if it fails to access a database.
    pub async fn get_account(username: &String) -> Option<Account>
    {
        let doc = mongo::get_collection("accounts")
            .await
            .find(bson::doc! { "username": username }, None)
            .await;
        match doc
        {
            Err(_) => panic!("An error occurred querying the database for an account."),
            Ok(mut doc) =>
            {
                if doc.advance().await.unwrap() == false
                {
                    return None;
                }
                let doc = Account::from_document(doc.current().try_into().unwrap());
                return Some(doc);
            }
        }
    }

    pub async fn get_account_by_sid(session_id: &String) -> Option<Account>
    {
        let doc = mongo::get_collection("accounts")
            .await
            .find(bson::doc! { "session_id": session_id }, None)
            .await;
        match doc
        {
            Err(_) => panic!("An error occurred querying the database for an account."),
            Ok(mut doc) =>
            {
                if doc.advance().await.unwrap() == false
                {
                    return None;
                }
                let doc = Account::from_document(doc.current().try_into().unwrap());
                return Some(doc);
            }
        }
    }

    /// Takes in an account value reference, and updates the first database entry with the same username. If the update is successful, it will return the account. If not, it will return an error. Most errors from this will likely be from trying to update a non-existent account.
    pub async fn update_account(new: &Account) -> Result<Account, mongodb::error::Error>
    {
        let result = mongo::get_collection("accounts")
            .await
            .update_one(bson::doc! { "username": &new.username }, bson::doc! { "$set": bson::to_document(&new).unwrap() }, None)
            .await;
        match result
        {
            Ok(_) => Ok(Account::from_document(
                mongo::get_collection("accounts")
                    .await
                    .find_one(bson::doc! { "username": &new.username }, None)
                    .await
                    .unwrap()
                    .unwrap()
            )),
            Err(result) => Err(result)
        }
    }

    /// Finds the first instance of a database account entry with a given username, and removes it. Returns an empty result.
    pub async fn delete_account(username: &String) -> Result<(), mongodb::error::Error>
    {
        match mongo::get_collection("accounts")
            .await
            .delete_one(bson::doc! { "username": username }, None)
            .await
        {
            Ok(_) => Ok(()),
            Err(result) => Err(result)
        }
    }

    /// Creates a new account entry from a given account value ref. Returns the account if successful, or an error if not. Most errors from this will be from faults in database setup.
    pub async fn create_account(new: &Account) -> Result<Account, mongodb::error::Error>
    {
        println!("Creating account...");
        let result = mongo::get_collection("accounts")
            .await
            .insert_one(bson::to_document(&new).unwrap(), None)
            .await;
        match result
        {
            Ok(_) => Ok(Account::from_document(
                mongo::get_collection("accounts")
                    .await
                    .find_one(bson::doc! { "username": &new.username }, None)
                    .await
                    .unwrap()
                    .unwrap()
            )),
            Err(result) => Err(result)
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ClientAccount
{
    pub username: String,
    pub password: String,
    pub friends: Vec<String>,
    /// This could get really thick if the conversations are too big. Will load test eventually.
    pub conversations: Vec<Conversation>,
    pub session_id: String
}

//----------------------------------------------//
//                                              //
//                   Messages                   //
//                                              //
//----------------------------------------------//

#[derive(Serialize, Deserialize, Clone, Default, Debug)]

/// ## Client, API
///
/// This contains a copy of the encrypted conversation key. The user who's name is attached to the `user` value is who's public key was used to encrypt it, and thus it can only be decrypted by the user with that name's attached.
pub struct UserKey
{
    owner: String,
    key: Vec<u8>
}

impl UserKey
{
    pub fn from_document(doc: &Document) -> UserKey
    {
        let owner: String = doc.get_str("owner").unwrap().to_string();
        let key: Vec<u8> = doc
            .get_array("key")
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap() as u8)
            .collect();
        UserKey {
            owner,
            key
        }
    }
    pub async fn encrypt(key: &[u8], user: &String) -> UserKey
    {
        let pub_key: Vec<u8> = Account::get_account(user)
            .await
            .unwrap()
            .public_key;
        let pub_key: Rsa<Public> = Rsa::public_key_from_pem(pub_key.as_slice()).expect("Failed to retrieve a public key from database.");
        let mut encrypted_key: Vec<u8> = vec![0; pub_key.size() as usize];
        pub_key
            .public_encrypt(key, &mut encrypted_key, Padding::PKCS1)
            .expect("failed to encrypt key");
        UserKey {
            owner: user.clone(),
            key: encrypted_key
        }
    }
}

/// ## Client, API
///
/// Raw, unencrypted message value.
///
/// ```rust
///   sender: String, // The username of the user who sent this message
///   message: Vec<u8>, // The message payload
///   time: String // The time the message was sent.
/// ```
#[derive(Serialize, Deserialize, Clone, Debug)]

//TODO: updated this. Please update the respective struct client-side.
pub struct RawMessage
{
    pub message: Vec<u8>,
    pub time: String
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
///  ##Client, API
///
/// This is pretty stupid, but necessary. This is used in the message modules, where we need both the user and session id but not the password (for security reasons), and the convo id.
///
/// ```rust
///    username: String, // The username of the user who sent this payload
///    conversation_id: String, // The conversation id of the conversation this payload is referring to
///    session_id: String, // The session id of the user who sent this payload
///    message: Option<RawMessage> // The message payload, if it exists. Not necessary for reception but necessary for sending.
/// ```
pub struct MessageUser
{
    pub username: String,
    pub conversation_id: String,
    pub session_id: String,
    pub message: Option<RawMessage>
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct EncryptedMessage
{
    pub data: Vec<u8>,
    pub sender: String,
    pub dest_convo_id: String,
    pub sender_sid: String,
}

impl EncryptedMessage
{
    fn from_document(doc: &Document) -> EncryptedMessage
    {
        let data: Vec<u8> = doc
            .get("data")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap() as u8)
            .collect();
        let sender: String = doc.get_str("sender").unwrap().to_string();
        EncryptedMessage {
            data,
            sender,
            dest_convo_id: String::new(),
            sender_sid: String::new()
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Conversation
{
    pub id: String,
    pub users: Vec<String>,
    pub keys: Vec<UserKey>,
    pub messages: Vec<EncryptedMessage>
}

impl Conversation
{
    pub fn from_document(doc: &Document) -> Conversation
    {
        let id: String = doc.get_str("id").unwrap().to_string();
        let users: Vec<String> = doc
            .get("users")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect();
        let messages: Vec<EncryptedMessage> = doc
            .get("messages")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| EncryptedMessage::from_document(x.as_document().unwrap()))
            .collect();
        let keys: Vec<UserKey> = doc
            .get("keys")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| UserKey::from_document(x.as_document().unwrap()))
            .collect();
        Conversation {
            id,
            users,
            messages,
            keys
        }
    }

    pub async fn get_all(username: &String) -> Option<Vec<Conversation>>
    {
        let mut convos: Vec<Conversation> = Vec::new();
        let mut doc: Cursor<Document> = mongo::get_collection("conversations")
            .await
            .find(Some(doc! {"users": username}), None)
            .await
            .unwrap();
        while doc.advance().await.unwrap() == true
        {
            // can't iterate over an async cursor so this is ugly
            convos.push(Conversation::from_document(&doc.current().try_into().unwrap()));
        }
        if convos.is_empty()
        {
            return None;
        }
        Some(convos)
    }

    pub async fn get_one(id: &String) -> Option<Conversation>
    {
        let doc: Option<Document> = mongo::get_collection("conversations")
            .await
            .find_one(Some(doc! {"id": id}), None)
            .await
            .unwrap();
        match doc
        {
            Some(doc) => Some(Conversation::from_document(&doc)),
            None => None
        }
    }

    pub async fn send(&mut self, message: EncryptedMessage) -> StatusCode
    {
        // strip message of all identifying data (except sender)
        let message: EncryptedMessage = EncryptedMessage
        {
            data: message.data,
            sender: message.sender,
            dest_convo_id: String::new(),
            sender_sid: String::new()
        };
        self.messages.push(message);
        // update conversation in database
        match mongo::get_collection("conversations").await.find_one_and_update(
            doc! {"id": &self.id},
            doc! {"$set": bson::to_document(&self).unwrap()},
            None
        ).await
        {
            Ok(_) => return StatusCode::OK,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR
        }
    }      
}

