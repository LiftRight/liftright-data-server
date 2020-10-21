use dotenv::dotenv;
use mongodb::{options::ClientOptions, Client, Collection};
use std::env;
use url::Url;
use warp::reject::Reject;

pub mod imurecords;
pub mod json_api;
pub mod query_selector;
pub mod repetition;
pub mod session;
pub mod set;
pub mod survey;
pub mod traits;
pub mod user;

#[derive(Debug)]
pub enum LrdsError {
    ConversionError,
    UnimplementedError,
    UrlParseError(url::ParseError),
    DbError(mongodb::error::Error),
    ObjectIdError(mongodb::bson::oid::Error),
    DbSerializationError(mongodb::bson::ser::Error),
    DbDeserializationError(mongodb::bson::de::Error),
}

impl Reject for LrdsError {}
type LrdsResult<T> = Result<T, LrdsError>;

/// Establishes a connection to mongo db.
/// Records are stored as a tree in the form
/// TODO
pub async fn establish_db_connection() -> Result<Collection, LrdsError> {
    dotenv().ok();

    let connection_handle =
        env::var("MONGO_DATABASE_CONN").expect("MONGO_DATABASE_CONN must be set!");

    let mongodb_url = Url::parse(&connection_handle).map_err(LrdsError::UrlParseError)?;
    let database_name = &mongodb_url.path()[1..];

    let client_options = ClientOptions::parse(&connection_handle)
        .await
        .map_err(LrdsError::DbError)?;

    let client = Client::with_options(client_options).map_err(LrdsError::DbError)?;
    let db = client.database(database_name);
    let collection = db.collection("session_data");

    Ok(collection)
}
