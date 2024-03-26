use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

//----------------------------------------------//
//                                              //
//        File for commonly-used structs        //
//                                              //
//----------------------------------------------//
use super::{mongo, utils};
use mongodb::bson::{self, doc, Document};
use openssl::rsa::{Padding, Rsa};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

//----------------------------------------------//
//                                              //
//                User Accounts                 //
//                                              //
//----------------------------------------------//


//------------------------------//

#[derive(Deserialize, Serialize, Debug, Clone, Default)]

pub struct Account
{
    pub username: String,
    pub hash: String,
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

    /// Retrieves an account value from the database by username.
    /// 
    /// ## Arguments
    /// * [`username`][`String`] - The username of the account to retrieve.
    /// 
    /// ## Returns
    /// * [`Result<Option<Account>, String>`][`std::result::Result`] - A result containing an account option (None if no account is found) or an error string, if an internal error occurred.
    pub async fn get_account(username: &String) -> Result<Option<Account>, String>
    {
        let Ok(mut doc) = mongo::get_collection("accounts")
            .await
            .find(bson::doc! { "username": username }, None)
            .await
        else { return Err(utils::gen_err("An error occurred querying the database for an account by username.")) };

        if !doc.advance().await.unwrap() { return Ok(None) }

        let doc: Account = Account::from_document(doc.current().try_into().unwrap());
        Ok(Some(doc))
    }

    /// Retrieves an account value from the database by session ID.
    /// 
    /// ## Arguments
    /// * [`session_id`][`String`] - The session ID of the account to retrieve.
    /// 
    /// ## Returns
    /// * [`Result<Option<Account>, String>`][`std::result::Result`] - A result containing an account option (None if no account is found) or an error string, if an internal error occurred.
    /// 
    pub async fn get_account_by_sid(session_id: &String) -> Result<Option<Account>, String>
    {
        let Ok(mut doc) = mongo::get_collection("accounts")
            .await
            .find(bson::doc! { "session_id": session_id }, None)
            .await
        else { return Err(utils::gen_err("An error occurred querying the database for an account by SID.")) };

        if !doc.advance().await.unwrap() { return Ok(None) }

        let doc: Account = Account::from_document(doc.current().try_into().unwrap());
        Ok(Some(doc))
    }

    /// "Updates" an account value in the database. This is done by replacing the old account value with the new one.
    /// 
    /// ## Arguments
    /// * [`new`][`Account`] - The new account value to replace the old one with.
    /// 
    /// ## Returns
    /// * [`Result<(), String>`][`std::result::Result`] - Returns an error if one is present.
    /// 
    pub async fn update_account(new: &Account) -> Result<(), String>
    {
        if let Ok(_) = mongo::get_collection("accounts")
            .await
            .update_one(bson::doc! { "username": &new.username }, bson::doc! { "$set": bson::to_document(&new).unwrap() }, None)
            .await
        { return Ok(()) }
        else { return Err(utils::gen_err("An error occurred updating an account in the database.")) }
    }

    /// Deletes a given account from the database.
    /// 
    /// ## Arguments
    /// * [`username`][`String`] - The username of the account to delete.
    /// 
    /// ## Returns
    /// * [`Result<(), String>`][`std::result::Result`] - Returns an error if one is present.
    /// 
    pub async fn delete_account(username: &String) -> Result<(), String>
    {
        if let Ok(_) = mongo::get_collection("accounts")
            .await
            .delete_one(bson::doc! { "username": username }, None)
            .await  
        { return Ok(()) }
        else { return Err(utils::gen_err("An error occurred deleting an account from the database.")) }
    }

    /// Creates a new account entry from a given account value.
    /// 
    /// ## Arguments
    /// * [`new`][`Account`] - The account value to be created.
    /// 
    /// ## Returns
    /// * [`Result<(), String)>`][`std::result::Result`] - A result containing an error, if one is present.
    /// 
    pub async fn create_account(new: &Account) -> Result<(), String>
    {
        println!("Creating account...");
        if let Ok(_) = mongo::get_collection("accounts")
            .await
            .insert_one(bson::to_document(&new).unwrap(), None)
            .await
        { Ok(()) }
        else { return Err(utils::gen_err("An error occurred creating an account in the database.")) }
    }
}

//------------------------------//

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
/// A client-side version of the [`Account`] struct. Contains only necessary client-side info, and the SID for authentication.
///
///  This struct is NEVER stored in the database. They are first converted to [`Account`] structs before being stored.
/// 
/// ## Fields
/// * [`username`][`std::string::String`] - The username of the account.
/// * [`password`][`std::string::String`] - The password of the account.
/// * [`friends`][`std::vec::Vec`] - A vector of the usernames of the account's friends.
/// * [`conversations`][`std::vec::Vec`] - A vector of the account's conversations.
/// * [`session_id`][`std::string::String`] - The session ID of the account.
pub struct ClientAccount
{
    pub username: String,
    pub password: String,
    pub friends: Vec<String>,
    pub conversations: Vec<Conversation>,
    pub session_id: String
}

//------------------------------//

#[derive(Deserialize, Serialize, Debug, Default, Clone, Eq, PartialEq)]
/// An enum representing the different actions that can be taken when updating a user's data.
pub enum UpdateAction
{
    #[default]
    None,
    ChangeUsername,
    ChangePassword,
    AddFriend,
    RemoveFriend,
}

/// A unique user data struct, made specifically for updating one specific field of userdata.
/// 
/// ## Fields
/// * [`field`][`std::string::String`] - The field to be updated.
/// * [`data`][`std::string::String`] - The data to replace the old data of specified field with
/// * [`action`][`UpdateAction`] - The action to be taken with the data.
/// * [`session_id`][`std::string::String`] - The session ID of the user making the request.
/// 
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct UpdateUser
{
    pub data: String,
    pub action: UpdateAction,
    pub session_id: String
}

//------------------------------//

//----------------------------------------------//
//                                              //
//                  Messaging                   //
//                                              //
//----------------------------------------------//

#[derive(Serialize, Deserialize, Clone, Default, Debug)]


/// This contains a copy of the encrypted conversation key. The user who's name is attached to the `user` value is who's public key was used to encrypt it, and thus it can only be decrypted by the user with that name's attached.
pub struct UserKey
{
    owner: String,
    key: Vec<u8>
}

impl UserKey
{

    /// Parses a [`UserKey`] value into a BSON [`Document`]. Necessary to ensure the key bytes remain i32s.
    pub fn to_document(&self) -> Document
    {
        doc! {
            "owner": &self.owner,
            "key": &self.key.iter().map(|x| *x as i32).collect::<Vec<i32>>()
        }
    }

    /// Parses a BSON [`Document`] into a [`UserKey`] value
    pub fn from_document(doc: &Document) -> UserKey
    {
        let owner: String = doc.get_str("owner").unwrap().to_string();
        let key: Vec<u8> = doc
            .get_array("key")
            .unwrap()
            .iter()
            .map(|x| x.as_i32().unwrap() as u8)
            .collect();
        UserKey {
            owner,
            key
        }
    }

    /// Encrypts a key, intended to be the conversation key, with the public key of the provided user.
    /// 
    /// ## Arguments
    /// * [`key`][`std::vec::Vec`] - The key to be encrypted.
    /// 
    /// ## Returns
    /// * [`Result<UserKey, String>`][`std::result::Result`] - A result containing the encrypted key or an error string, if an internal error occurred.
    /// 
    pub async fn encrypt(key: &[u8], user: &String) -> Result<UserKey, String>
    {
        let Ok(Some(account)) = Account::get_account(user).await
        else { return Err(utils::gen_err("Error retrieving account from database.")) };

        let Ok(pub_key) = Rsa::public_key_from_pem(account.public_key.as_slice())
        else { return Err(utils::gen_err("Error retrieving public key from database.")) };

        let mut encrypted_key: Vec<u8> = vec![0; pub_key.size() as usize];
        pub_key
            .public_encrypt(key, &mut encrypted_key, Padding::PKCS1)
            .expect("failed to encrypt key");
        Ok(UserKey {
            owner: user.clone(),
            key: encrypted_key
        })
    }
}

//------------------------------//

/// Raw, unencrypted message value.
/// 
/// ## Fields
/// * [`message`][`std::vec::Vec`] - The message payload.
/// * [`time`][`std::string::String`] - The time the message was sent.
/// 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawMessage
{
    pub message: Vec<u8>,
    pub time: String
}

//------------------------------//

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
/// An encrypted message value.
/// 
/// ## Fields
/// * [`data`][`std::vec::Vec`] - The encrypted message payload. See [`UserKey`] to see how this data is encrypted.
/// * [`sender`][`std::string::String`] - The username of the user who sent the message.
/// * [`dest_convo_id`][`std::string::String`] - The ID of the conversation the message is being sent to (removed before upload.)
/// * [`sender_sid`][`std::string::String`] - The session ID of the user who sent the message (removed before upload.)
/// 
pub struct EncryptedMessage
{
    pub data: Vec<u8>,
    pub sender: String,
    pub dest_convo_id: String,
    pub sender_sid: String
}

impl EncryptedMessage
{
    /// Parses a BSON [`Document`] into a [`Conversation`] value
    fn from_document(doc: &Document) -> EncryptedMessage
    {
        let data: Vec<u8> = doc
            .get("data")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i32().unwrap() as u8)
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

//------------------------------//

#[derive(Deserialize, Serialize, Debug, Clone, Default)]

/// Contains information about a given conversation on the database.
/// 
/// ## Fields
/// * [`id`][`std::string::String`] - The ID of the conversation.
/// * [`users`][`std::vec::Vec`] - A vector of the usernames of the users in the conversation.
/// * [`keys`][`UserKey`] - A vector of the encrypted [`UserKey`]s for each user in the conversation.
/// * [`messages`][`EncryptedMessage`] - A vector of the [`EncryptedMessage`]s in the conversation.
/// 
pub struct Conversation
{
    pub id: String,
    pub users: Vec<String>,
    pub keys: Vec<UserKey>,
    pub messages: Vec<EncryptedMessage>
}

impl Conversation
{
    /// Parses a [`Conversation`] value into a BSON [`Document`].
    /// Most structs do not need a `to_document` because bson has a built-in method, but this is necessary to ensure the key bytes remain I32s; bson's `to_document()` will turn them into I64s.
    pub fn to_document(&self) -> Document
    {
        doc! {
            "id": &self.id,
            "users": &self.users.iter().map(|x| x.as_str()).collect::<Vec<&str>>(),
            "keys": &self.keys.iter().map(|x| x.to_document()).collect::<Vec<Document>>(),
            "messages": &self.messages.iter().map(|x| bson::to_document(&x).unwrap()).collect::<Vec<Document>>()
        }
    }

    /// Parses a BSON [`Document`] into a [`Conversation`] value
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

    /// Gets all conversations that a provided user is a part of.
    /// 
    /// ## Arguments
    /// * [`username`][`String`] - The username of the user to retrieve conversations for.
    /// 
    /// ## Returns
    /// * [`Result<Vec<Conversation>, String>`][`std::result::Result`] - A result containing a vector of conversations or an error string, if an internal error occurred.
    pub async fn get_all(username: &String) -> Result<Vec<Conversation>, String>
    {
        let mut convos: Vec<Conversation> = Vec::new();
        let Ok(mut doc) = mongo::get_collection("conversations")
            .await
            .find(Some(doc! {"users": username}), None)
            .await
        else { return Err(utils::gen_err("Failed to retrieve conversations from database.")) };

        while doc.advance().await.unwrap() == true
        {
            convos.push(Conversation::from_document(&doc.current().try_into().unwrap()));
        }

        Ok(convos)
    }

    /// Gets one conversation with the specified ID.
    /// 
    /// ## Arguments
    /// * [`id`][`String`] - The ID of the conversation to retrieve.
    /// 
    /// ## Returns
    /// * [`Result<Option<Conversation>, String>`][`std::result::Result`] - A result containing a conversation option (None if no conversation is found) or an error string, if an internal error occurred.
    /// 
    pub async fn get_one(id: &String) -> Result<Option<Conversation>, String>
    {
        let Ok(doc) = mongo::get_collection("conversations")
            .await
            .find_one(Some(doc! {"id": id}), None)
            .await
        else { return Err(utils::gen_err("There was an error trying to retrieve a conversation.")) };

        match doc
        {
            Some(doc) => Ok(Some(Conversation::from_document(&doc))),
            None => Ok(None)
        }
    }

    /// Sends a message to a conversation.
    /// 
    /// ## Arguments
    /// * [`message`][`EncryptedMessage`] - The message to be sent.
    /// 
    /// ## Returns
    /// * [`Result<(), (String)>`][`std::result::Result`] - A result containing either an empty value or an error string.
    /// 
    pub async fn send(&mut self, message: EncryptedMessage) -> Result<(), String>
    {
        // strip message of useless/private data; attaching SID means other member of convo would be able to access the other user's SID with some client-side manipulation.
        // TODO: pretty sure sender doesn't need to be on EncryptedMessage. Fix in client-side
        let message: EncryptedMessage = EncryptedMessage {
            data: message.data,
            sender: message.sender,
            dest_convo_id: String::new(),
            sender_sid: String::new()
        };
        self.messages.push(message);
        
        if let Ok(_) = mongo::get_collection("conversations")
            .await
            .find_one_and_update(doc! {"id": &self.id}, doc! {"$set": bson::to_document(&self).unwrap()}, None)
            .await
        { return Ok(()) } else { return Err(utils::gen_err("An error occurred pushing a new message to a conversation.")); }
    }
}

//----------------------------------------------//
//                                              //
//                   Websockets                 //
//                                              //
//----------------------------------------------//

#[derive(Debug)]
pub struct WebsocketClient
{
    pub username: String,
    pub session_id: String,
    pub socket: Sender<WSPacket>
}

pub type ClientStore = Arc<Mutex<HashMap<SocketAddr, WebsocketClient>>>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum WSAction
{
    SendMessage(EncryptedMessage),
    ReceiveMessage(EncryptedMessage),
    CreateConversation(Vec<String>),
    RecieveConversation(Conversation),
    DeleteConversation(String),
    AddFriend(String),
    RemoveFriend(String),
    Register(),
    Disconnect(),
    Info(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WSPacket
{
    pub sender: String,
    pub sid: String,
    pub action: WSAction
}