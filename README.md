
# CRIM: A Rust IM 🦀✉️ 
You are reading the __API__, and thus this repo only covers server-side functions. To view the front-end/client side and it's functionality, click [here](https://github.com/Jayleaf/crim-tauri).
## Introduction
Originally, CRIM was entirely terminal-based, had no dedicated API, and was rather insecure in terms of user authentication-- however, user-to-user communication was still secure due to end-to-end encryption. Now, however, I have built a backend to further facilitate proper user authentication and security.





## Features / Roadmap

- [x]   Session-Based User Authentication
- [x]   Websocket Integration for Messaging (Live Updating)
- [x]   Add/Remove Friends
- [x]   [End-to-End Message Encryption]()
- [x]   ID-Based Conversations between users
- [x]   Group Chats
- [x]   Change Username/Password `🚧 Completed, but not implemented front-end`
- [ ]   Preferences `🟡 Medium Priority`
- [ ]   Pinned Messages `🟡 Medium Priority`
- [ ]   Compatibility with other DBs `🟢 Low Priority`
- [ ]   User Blocklist `🟢 Low Priority`

## API Reference

This API was explicitly designed to be used with the `serde_json` crate, and thus all POST payloads are serialized structs of the given `Payload Struct`.

--------------------
#### Create a user/register an account `🟢 Functional`

```http
POST api/auth/create
```

| Parameter |    Payload Struct   |    Utilized Fields   | Returns |
| :--------:| :-----------------: |:---------------------|:-------:|
| `payload` |   `ClientAccount`   |`username`, `password`|   N/A   |

-------------
#### Delete a user from the database. `🟡 Functional, but Unsafe`

```http
POST api/auth/delete
```

| Parameter | Payload Struct  |          Utilized Fields             | Returns |
| :-------: | :--------------:| :--------------------------------    |:--------: 
| `payload` | `ClientAccount` | `username`, `password`, `session_id` |   N/A   |

--------------
#### Authenticate a user `🟢 Functional`
```http
POST api/auth/login
```

| Parameter | Payload Struct  |      Utilized Fields     |   Returns  |
| :-------: | :--------------:| :-----------------------:|:----------:| 
| `payload` | `ClientAccount` |  `username`, `password`  |`session_id`|

--------------
#### Change a user's password `🟢 Functional`
```http
POST api/auth/change-password
```

| Parameter | Payload Struct  |              Utilized Fields           |   Returns  |
| :-------: | :--------------:| :-------------------------------------:|:----------:| 
| `payload` | `ClientAccount` |  `username`, `password`, `session_id`  |`StatusCode`|


--------------
#### Get all of a user's client-side data `🟢 Functional`
```http
GET api/auth/get
```

| Parameter | Payload Struct  |      Utilized Fields     |    Returns    |
| :-------: | :--------------:| :-----------------------:|:-------------:| 
|   `sid`   |     `String`    |        `session_id`      |`ClientAccount`|

--------------
#### Establish a websocket connection `🟢 Functional`
```http
GET api/ws
```
For more info as to how websocket communication works, see [`src/routes/ws/ws.rs`](https://github.com/Jayleaf/crim-api/blob/main/src/routes/ws/ws.rs) and [`/src/generics/structs.rs`](https://github.com/Jayleaf/crim-api/blob/main/src/generics/structs.rs)



## E2EE Protocols

Messaging in CRIM operates with a RSA shared-key style protocol. When a conversation is created, one overarching conversation key is created.

* [`from /src/routes/message/make.rs`](https://github.com/Jayleaf/crim-api/blob/main/src/routes/message/make.rs)
```rust
let mut raw_conversation_key: [u8; 32] = [0; 32];
getrandom(&mut raw_conversation_key).expect("Failed to generate random conversation key ");
// getrandom() can sometimes give a 0, which will fuck everything up.
while raw_conversation_key.iter().any(|x| *x == 0_u8)
{
    getrandom(&mut raw_conversation_key).expect("Failed to generate random conversation key.");
}
```
The key is then encrypted with the public keys of all users in the conversation, and stored in the `keys` array on the DB.
* [`from /src/routes/message/make.rs`](https://github.com/Jayleaf/crim-api/blob/main/src/routes/message/make.rs)
```rust
 keys: {
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
```
* [`from src/generics/structs.rs on impl UserKey`](https://github.com/Jayleaf/crim-api/blob/main/src/routes/message/make.rs)
```rust
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
```

For information and code references about **decryption**, please view the [front-end README.](https://github.com/Jayleaf/crim-tauri)




## Appendix
CRIM is a passion project, and also my first ever full-fledged project in Rust. Nothing will be perfect, but I aim for it to be as secure and structurally sound as possible, continuing to improve it and add features in time. This README is in no way final, and some code snapshots may not accurately reflect the source at a current point in time, though I will try to keep them as up-to-date as possible. Thank you to [Julius 🥭](https://github.com/juliuskreutz) for inspiring me to pick up Rust.
