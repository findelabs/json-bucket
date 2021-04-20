use chrono::prelude::*;
use mongodb::bson::{document::Document};
//use mongodb::{options::ClientOptions, options::FindOptions, Client, Collection};
use crate::error::MyError;
use mongodb::{options::ClientOptions, Client};
//use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
    pub db: String
}

type Result<T> = std::result::Result<T, MyError>;

impl DB {
    pub async fn init(url: &str, db: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(url).await?;
        client_options.app_name = Some("json-bucket".to_string());
        Ok(Self {
            client: Client::with_options(client_options)?,
            db: db.to_owned()
        })
    }

    pub async fn insert(&self, collection: &str, mut mongodoc: Document) -> Result<String> {

        // Log which collection this is going into
        log::info!("Inserting doc into {}.{}", self.db, collection);

        let now = Utc::now();
        mongodoc.insert("_time", now);
        let collection = self.client.database(&self.db).collection(collection);
        match collection.insert_one(mongodoc, None).await {
            Ok(id) => Ok(id.inserted_id.to_string()),
            Err(e) => {
                log::error!("Error inserting into mongodb: {}", e);
                Err(MyError::MongodbError)
            }
        }
    }
}
