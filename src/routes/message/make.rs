use super::{
    db::mongo, generics::{
        structs::{Conversation, UserKey}, utils
    }
};
use getrandom::getrandom;
use mongodb::bson::Document;

pub async fn create_conversation(users: Vec<&String>) -> Result<Conversation, String>
{
    let mut raw_conversation_key: [u8; 32] = [0; 32];
    getrandom(&mut raw_conversation_key).expect("Failed to generate random conversation key.");
    while raw_conversation_key.iter().any(|x| *x == 0_u8)
    {
        getrandom(&mut raw_conversation_key).expect("Failed to generate random conversation key.");
    } // getrandom() can sometimes give a 0, which will fuck everything up.

    let conversation: Conversation = Conversation {
        id: utils::rand_hex(4),     
        users: users.iter().map(|x| x.to_owned().to_owned()).collect(), // double to_owned()? really?
        keys: 
        {
            let mut k: Vec<UserKey> = Vec::new();
            for user in users {
                k.push(
                    UserKey::encrypt(&raw_conversation_key, &user)
                        .await
                        .map_err(|x|  utils::gen_err(&x))?
                );
            }
            k
        },
        messages: vec![]
    };

    let doc: Document = conversation.to_document();
    if let Err(e) = mongo::get_collection("conversations")
        .await
        .insert_one(doc, None)
        .await
    { return Err(utils::gen_err(&format!("An error occurred generating a conversation: {}", e)))}

    return Ok(conversation);
}
