use bson::to_document;
use std::collections::HashMap;
use serde_json::Map;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use serde_tuple::*;

use crate::error::Error as RestError;

#[derive(Serialize, Deserialize)]
pub struct Doc(Vec<Value>);

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct AggregateWithOptions{
    pipeline: Doc,
    options: Option<Map<String, Value>>
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Aggregate {
    Doc(Doc),
    AggregateWithOptions(AggregateWithOptions),
}

impl Doc {
    pub fn pipeline(&self) -> Result<bson::Document, RestError> {
        let mut doc = to_document(self)?;
        let now = Utc::now();
        doc.insert("_time", now);
        Ok(doc)
    }

    pub fn options(&self) -> Option<Value> {
        None
    }
}

impl AggregateWithOptions {
    pub fn pipeline(&self) -> Result<bson::Document, RestError> {
        let mut doc = to_document(&self.pipeline)?;
        Ok(doc)
    }

    pub fn options(&self) -> Option<Value> {
        match &self.options {
            Some(f) => match serde_json::to_value(f) {
                Ok(t) => Some(t),
                Err(e)  => {
                    log::error!("Error converting options to document: {}", e);
                    None
                }
            },
            None => None
        }
    }
}

impl Aggregate {
    pub fn pipeline(&self) -> Result<bson::Document, RestError> {
        match self {
            Aggregate::Doc(s) => s.pipeline(),
            Aggregate::AggregateWithOptions(s) => s.pipeline()
        }
    }

    pub fn options(&self) -> Option<Value> {
        match self {
            Aggregate::Doc(s) => s.options(),
            Aggregate::AggregateWithOptions(s) => s.options()
        }
    }

    pub fn pipeline_and_options(&self) -> (Result<bson::Document, RestError>, Option<Value>) {
        (self.pipeline(), self.options())
    }
}
