use bson::to_document;
use serde_json::Map;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use serde_tuple::*;

use crate::error::Error as RestError;

//
// InsertMany Structs
//

#[derive(Serialize, Deserialize)]
pub struct InsertManyDocs(Vec<Value>);

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct InsertManyDocsWithOptions{
    pipeline: InsertManyDocs,
    options: Option<Map<String, Value>>
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum InsertManyDocsOptions {
    InsertManyDocs(InsertManyDocs),
    InsertManyDocsWithOptions(InsertManyDocsWithOptions),
}

//
// Aggregation Pipeline Structs
//

#[derive(Serialize, Deserialize)]
pub struct PipelineDocs(Vec<Value>);

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct PipelineDocsWithOptions{
    pipeline: PipelineDocs,
    options: Option<Map<String, Value>>
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Pipeline {
    PipelineDocs(PipelineDocs),
    PipelineDocsWithOptions(PipelineDocsWithOptions),
}

impl Docs {
    pub fn pipeline(&self) -> Result<Vec<bson::Document>, RestError> {
        let doc: Vec<bson::Document> = to_document(self)?;
        Ok(doc)
    }

    pub fn options(&self) -> Option<Value> {
        None
    }
}

impl DocsWithOptions {
    pub fn pipeline(&self) -> Result<bson::Document, RestError> {
        let doc = to_document(&self.pipeline)?;
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

impl DocsOptions {
    pub fn pipeline(&self) -> Result<Vec<bson::Document>, RestError> {
        match self {
            Self::Docs(s) => s.pipeline(),
            Self::DocsWithOptions(s) => s.pipeline()
        }
    }

    pub fn options(&self) -> Option<Value> {
        match self {
            Self::Docs(s) => s.options(),
            Self::DocsWithOptions(s) => s.options()
        }
    }

    pub fn pipeline_and_options(&self) -> (Result<Vec<bson::Document>, RestError>, Option<Value>) {
        (self.pipeline(), self.options())
    }
}
