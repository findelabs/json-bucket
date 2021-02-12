use chrono::prelude::*;
use mongodb::bson::{doc, document::Document};
//use mongodb::{options::ClientOptions, options::FindOptions, Client, Collection};
use crate::error::MyError;
use mongodb::Collection;
use mongodb::{options::ClientOptions, options::FindOneOptions, Client};
use serde::{Deserialize, Serialize};

// Articles location: articles.published
const DB: &str = "dds_posts";

// Users location: vault.users
const DB_USERS: &str = "vault";
const COLL_USERS: &str = "users";

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub email: String,
    pub token: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub site: String,
    pub summary: String,
    pub title: String,
    pub url: String,
    pub time: DateTime<Utc>,
    pub tags: Vec<String>,
    pub column: String,
}

type Result<T> = std::result::Result<T, MyError>;

impl DB {
    pub async fn init(url: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(url).await?;
        client_options.app_name = Some("findereport-poster".to_string());
        Ok(Self {
            client: Client::with_options(client_options)?,
        })
    }

    pub async fn insert(&self, collection: &str, mut mongodoc: Document) -> Result<String> {
        let now = Utc::now();
        mongodoc.insert("_time", now);
        let collection = self.client.database(DB).collection(collection);
        match collection.insert_one(mongodoc, None).await {
            Ok(id) => Ok(id.inserted_id.to_string()),
            Err(e) => {
                log::error!("Error inserting into mongodb: {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    // This function takes a token , and returns an email if the user exists, as long as the user is a publisher
    pub async fn get_user_email(&self, token: &str) -> Option<String> {
        let search = Some(doc!{ "token" : token});
        let collection = self.client.database(DB_USERS).collection(COLL_USERS);
        let doc = collection.find_one(search, None).await.ok();
        match doc {
            Some(d) => {
                match d {
                    Some(user) => match user.get_str("email") {
                        Ok(email) => {
                            log::info!("Matched {} to {}", &token, &email);
                            match user.get_str("role") {
                                Ok(role) => {
                                    match role {
                                        "publisher" => {
                                            Some(email.to_owned())
                                        },
                                        _ => {
                                            log::info!("{} is not a publisher", &email);
                                            None
                                        }
                                    }
                                },
                                Err(_) => {
                                    log::info!("Located {}, but there is no role assigned", &token);
                                    None
                                }
                            }
                        },
                        Err(_) => {
                            log::info!("Located {}, but there is no email listed", &token);
                            None
                        }
                    },
                    None => {
                        log::info!("{} not found in users database", &token);
                        None
                    }
                }
            },
            None => {
                log::info!("Error connecting to db");
                None
            }
        }
    }
}
