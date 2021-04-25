use chrono::prelude::*;
use mongodb::bson::{doc, document::Document};
//use mongodb::{options::ClientOptions, options::FindOptions, Client, Collection};
use crate::error::MyError;
use mongodb::{options::ClientOptions, options::FindOneOptions, options::FindOptions, Client};
//use serde::{Deserialize, Serialize};
use futures::StreamExt;
use clap::ArgMatches;

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
    pub db: String,
}

type Result<T> = std::result::Result<T, MyError>;

impl DB {
    pub async fn init(url: &str, db: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(url).await?;
        client_options.app_name = Some("json-bucket".to_string());
        Ok(Self {
            client: Client::with_options(client_options)?,
            db: db.to_owned(),
        })
    }

    pub async fn findone(&self, collection: &str, query: Document) -> Result<Document> {
        // Log which collection this is going into
        log::debug!("Searching {}.{}", self.db, collection);

        let find_one_options = FindOneOptions::builder()
            .sort(doc! { "_id": -1 })
            .projection(doc! { "_id" : 0 })
            .build();

        let collection = self.client.database(&self.db).collection(collection);

        match collection.find_one(query, find_one_options).await {
            Ok(result) => match result {
                Some(doc) => {
                    log::debug!("Found a result");
                    Ok(doc)
                }
                None => {
                    log::debug!("No results found");
                    Ok(doc! { "msg": "no results found" })
                }
            },
            Err(e) => {
                log::error!("Error searching mongodb: {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn find(&self, collection: &str, query: Document) -> Result<Vec<Document>> {
        // Log which collection this is going into
        log::debug!("Searching {}.{}", self.db, collection);

        let find_options = FindOptions::builder()
            .sort(doc! { "_id": -1 })
            .projection(doc! { "_id" : 0 })
            .limit(100)
            .build();

        let collection = self.client.database(&self.db).collection(collection);
        let mut cursor = collection.find(query, find_options).await?;

        let mut result: Vec<Document> = Vec::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(converted) => result.push(converted),
                Err(e) => {
                    log::error!("Caught error, skipping: {}", e);
                    continue;
                }
            }
        }
        Ok(result)
    }

    pub async fn insert(&self, opts: ArgMatches<'_>, collection: &str, mut mongodoc: Document) -> Result<String> {
        match opts.is_present("readonly") {
            true => {
                log::error!("Rejecting post, as we are in readonly mode");
                return Err(MyError::ReadOnly)
            }
            _ => {
                // Log which collection this is going into
                log::debug!("Inserting doc into {}.{}", self.db, collection);
            }
        };

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

    pub async fn collections(&self) -> Result<Vec<String>> {
        // Log that we are trying to list collections
        log::debug!("Getting collections in {}", self.db);

        match self
            .client
            .database(&self.db)
            .list_collection_names(None)
            .await
        {
            Ok(collections) => {
                log::debug!("Success listing collections in {}", self.db);
                Ok(collections)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn count(&self, collection: &str) -> Result<Document> {
        // Log that we are trying to list collections
        log::debug!("Getting document count in {}", self.db);

        let collection = self.client.database(&self.db).collection(collection);

        match collection.estimated_document_count(None).await {
            Ok(count) => {
                log::debug!("Successfully counted docs in {}", self.db);
                let result = doc! {"docs" : count};
                Ok(result)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }
}
