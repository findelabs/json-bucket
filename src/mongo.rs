use clap::ArgMatches;
use bson::Document;
//use axum::response::IntoResponse;
use mongodb::Collection;
use bson::from_document;
use futures::stream::{StreamExt};
use serde_json::from_value;
use chrono::Utc;

use crate::error::Error as RestError;
use crate::filters::Filters;
use crate::inserts::InsertOne;
use crate::request_body::DocsOptions;


#[derive(Clone, Debug)]
pub struct MongoClient {
    pub client: mongodb::Client,
}

#[derive(Clone, Debug)]
pub struct MongoDoc(Document);

//impl IntoResponse for MongoDoc {
//    fn into_response(self) -> Response {
//        let payload = self.to_string();
//        let body = body::boxed(body::Full::from(payload));
//        let mut res = Response::builder();
//        let headers = res
//            .headers_mut()
//            .expect("Failed to get headers from response");
//
//        res.status(StatusCode::Ok).body(body).unwrap()
//    }
//}

impl MongoClient {
    pub async fn new(opts: ArgMatches, client: mongodb::Client) -> Result<Self, RestError> {
        Ok(MongoClient {
            client,
        })
    }

    pub fn collection(&self, database: &str, collection: &str) -> Collection<Document> {
        self.client
            .database(database)
            .collection(collection)
    }

    pub async fn find_one(&self, database: &str, collection: &str, filters: Filters) -> Result<String, RestError> {
        let coll_handle = self.collection(database, collection);
        let (filter, options) = filters.filter_and_options();

        let options = match options {
            Some(t) => Some(from_document(t)?),
            None => None
        };

        match coll_handle.find_one(filter, options).await {
            Ok(v) => match v {
                Some(v) => {
                    Ok(serde_json::to_string(&v)?)
                },
                None => Err(RestError::NotFound),
            },
            Err(e) => {
                log::error!("Error searching for docs: {}", e);
                Err(RestError::NotFound)
            }
        }
    }

    pub async fn find(&self, database: &str, collection: &str, filters: Filters) -> Result<String, RestError> {
        let coll_handle = self.collection(database, collection);
        let (filter, options) = filters.filter_and_options();

        let options = match options {
            Some(t) => Some(from_document(t)?),
            None => None
        };

        let mut cursor = coll_handle.find(filter, options).await?;
        let mut result: Vec<String> = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(k) => result.push(serde_json::to_string(&k)?),
                Err(e) => {
                    log::error!("Caught error, skipping: {}", e);
                    continue;
                }
            }
        }

        let result = result.into_iter().rev().collect();
        Ok(result)
    }

    pub async fn aggregate(&self, database: &str, collection: &str, body: DocsOptions) -> Result<String, RestError> {
        let coll_handle = self.collection(database, collection);
        let (pipeline, options) = body.pipeline_and_options();

        let options = match options {
            Some(t) => Some(from_value(t)?),
            None => None
        };

        let mut cursor = coll_handle.aggregate(pipeline, options).await?;
        let mut result: Vec<String> = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(k) => result.push(serde_json::to_string(&k)?),
                Err(e) => {
                    log::error!("Caught error, skipping: {}", e);
                    continue;
                }
            }
        }

        let result = result.into_iter().rev().collect();
        Ok(result)
    }

    pub async fn insert_many(&self, database: &str, collection: &str, body: DocsOptions) -> Result<String, RestError> {
        let coll_handle = self.collection(database, collection);
        let (pipeline, options) = body.pipeline_and_options();

        let options = match options {
            Some(t) => Some(from_value(t)?),
            None => None
        };

        let now = Utc::now();
        for mongodoc in pipeline?.iter_mut() {
            mongodoc.insert("_time", now);
        };

        match coll_handle.insert_many(pipeline, options).await {
            Ok(v) => {
                Ok(serde_json::to_string(&v)?)
            },
            Err(e) => {
                log::error!("Error inserting doc: {}", e);
                Err(RestError::BadInsert)
            }
        }
    }

    pub async fn insert(&self, database: &str, collection: &str, doc: InsertOne) -> Result<String, RestError> {
        let coll_handle = self.collection(database, collection);
        let (doc, options) = doc.doc_and_options();

        let options = match options {
            Some(t) => Some(from_document(t)?),
            None => None
        };

        match coll_handle.insert_one(doc?, options).await {
            Ok(v) => {
                Ok(serde_json::to_string(&v)?)
            },
            Err(e) => {
                log::error!("Error inserting doc: {}", e);
                Err(RestError::BadInsert)
            }
        }
    }

    pub async fn databases(&self) -> Result<String, RestError> {
        match self
            .client
            .list_database_names(None, None)
            .await
        {
            Ok(dbs) => {
                log::debug!("Success listing databases");
                Ok(serde_json::to_string(&dbs)?)
            }
            Err(e) => {
                log::error!("Got error listing databases: {}", e);
                Err(RestError::Databases)
            }
        }
    }

    pub async fn collections(&self, database: &str) -> Result<String, RestError> {
        match self
            .client
            .database(database)
            .list_collection_names(None)
            .await
        {
            Ok(collections) => {
                log::debug!("Success listing collections in {}", database);
                Ok(serde_json::to_string(&collections)?)
            }
            Err(e) => {
                log::error!("Got error listing collections: {}", e);
                Err(RestError::Collections)
            }
        }

    }
}
