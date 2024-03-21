use super::{
    db::mongo, generics::{
        structs::{Conversation, UserKey}, utils
    }
};
use getrandom::getrandom;
use mongodb::bson::{self, Document};

pub async fn create_conversation(users: &Vec<String>) -> Option<Conversation>
{
    let mut raw_conversation_key: [u8; 32] = [0; 32];
    getrandom(&mut raw_conversation_key).expect("Failed to generate random conversation key.");
    while raw_conversation_key.iter().any(|x| *x == 0_u8)
    {
        getrandom(&mut raw_conversation_key).expect("Failed to generate random conversation key.");
    } // getrandom() can sometimes give a 0, which will fuck everything up.

    let conversation = Conversation {
        id: utils::rand_hex(),
        users: users.clone(),
        keys: {
            // this could be prettier with a stream
            let mut k: Vec<UserKey> = Vec::new();
            for user in users
            {
                k.push(UserKey::encrypt(&raw_conversation_key, &user).await);
            }
            k
        },
        messages: vec![]
    };
    let doc: Document = conversation.to_document();
    match mongo::get_collection("conversations")
        .await
        .insert_one(doc, None)
        .await
    {
        Ok(_) => return Some(conversation),
        Err(_) => return None
    }
}
