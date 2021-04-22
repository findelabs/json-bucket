use chrono::prelude::*;
use mongodb::bson::{doc, document::Document};
//use mongodb::{options::ClientOptions, options::FindOptions, Client, Collection};
use crate::error::MyError;
use mongodb::{options::ClientOptions, Client, options::FindOneOptions};
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

    pub async fn query(&self, collection: &str, query: Document) -> Result<Document> {

        // Log which collection this is going into
        log::info!("Searching {}.{}", self.db, collection);

        let find_one_options = FindOneOptions::builder()
            .sort(doc! { "time": -1 })
            .projection( doc! { "_id" : 0, "_time" : 0 })
            .build();

        let collection = self.client.database(&self.db).collection(collection);

        match collection.find_one(query, find_one_options).await {
            Ok(result) => {
                match result {
                    Some(doc) => {
                        log::info!("Found a result");
                        Ok(doc)
                    },
                    None => {
                        log::info!("No results found");
                        Ok(doc! { "msg": "no results found" })
                    }
                }
            },
            Err(e) => {
                log::error!("Error searching mongodb: {}", e);
                Err(MyError::MongodbError)
            }
        }
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

    pub async fn collections(&self) -> Result<Document> {
        // Log that we are trying to list collections
        log::info!("Getting collections in {}", self.db);

        let command = doc! { "listCollections": 1.0, "authorizedCollections": true, "truenameOnly": true };
        match self.client.database(&self.db).run_command(command,None).await {
            Ok(collections) => {
                log::info!("Success listing collections in {}", self.db);
                Ok(collections)
            },
            Err(e) => {
                log::info!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }
}
