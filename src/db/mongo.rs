use mongodb::{
    bson::doc, bson::Document, options::{ServerApi, ServerApiVersion}, Collection, Database
};
use mongodb::{options::ClientOptions, Client};

async fn init_mongo() -> mongodb::error::Result<Client>
{
    let mut client_options = ClientOptions::parse(dotenv::var("MONGO_URI").unwrap().as_str()).await?;
    // Set the server_api field of the client_options object to set the version of the Stable API on the client
    let server_api = ServerApi::builder()
        .version(ServerApiVersion::V1)
        .build();
    client_options.server_api = Some(server_api);
    // Get a handle to the cluster
    let client = Client::with_options(client_options)?;
    // Ping the server to see if you can connect to the cluster
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    Ok(client)
}

pub async fn ping() -> Result<(), mongodb::error::Error> { init_mongo().await.map(|_| ()).map_err(|e| e) }
pub async fn get_database(name: &str) -> Database { init_mongo().await.unwrap().database(name) }
pub async fn get_collection(name: &str) -> Collection<Document>
{
    get_database(dotenv::var("DB_NAME").unwrap().as_str())
        .await
        .collection::<Document>(name)
}
