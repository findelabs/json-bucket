use clap::ArgMatches;
use crate::error::Error as RestError;
use crate::filters::Filters;
use bson::Document;
//use axum::response::IntoResponse;
use mongodb::Collection;
use bson::from_document;
use futures::stream::{StreamExt, TryStreamExt};

//type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

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
}
