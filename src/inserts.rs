use bson::to_document;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use chrono::Utc;

use crate::error::Error as RestError;

#[derive(Serialize, Deserialize)]
pub struct Doc(Value);

#[derive(Serialize, Deserialize)]
pub struct DocWithOptions([Value; 2]);

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum InsertOne{
    Doc(Doc),
    DocWithOptions(DocWithOptions),
}

impl Doc {
    pub fn doc(&self) -> Result<bson::Document, RestError> {
        let mut doc = to_document(self)?;
        let now = Utc::now();
        doc.insert("_time", now);
        Ok(doc)
    }

    pub fn options(&self) -> Option<bson::Document> {
        None
    }
}

impl DocWithOptions {
    pub fn doc(&self) -> Result<bson::Document, RestError> {
        let mut doc = to_document(&self.0[0])?;
        let now = Utc::now();
        doc.insert("_time", now);
        Ok(doc)
    }

    pub fn options(&self) -> Option<bson::Document> {
        match to_document(&self.0[1]) {
            Ok(c) => {
                Some(c)
            },
            Err(e) => {
                log::error!("Error converting options to document: {}", e);
                None
            }
        }
    }
}

impl InsertOne {
    pub fn doc(&self) -> Result<bson::Document, RestError> {
        match self {
            InsertOne::Doc(s) => s.doc(),
            InsertOne::DocWithOptions(s) => s.doc()
        }
    }

    pub fn options(&self) -> Option<bson::Document> {
        match self {
            InsertOne::Doc(s) => s.options(),
            InsertOne::DocWithOptions(s) => s.options()
        }
    }

    pub fn doc_and_options(&self) -> (Result<bson::Document, RestError>, Option<bson::Document>) {
        (self.doc(), self.options())
    }
}

