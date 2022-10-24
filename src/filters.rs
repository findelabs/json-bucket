use bson::to_document;
use serde_json::Value;
use serde_json::Map;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Filter(Map<String, Value>);

#[derive(Serialize, Deserialize)]
pub struct FilterOptions([Filter; 2]);

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Filters{
    Filter(Filter),
    FilterOptions(FilterOptions),
}

impl Filter {
    pub fn filter(&self) -> Option<bson::Document> {
        match to_document(self) {
            Ok(c) => Some(c),
            Err(e) => {
                log::error!("Error converting filter to document: {}", e);
                None
            }
        }
    }

    pub fn options(&self) -> Option<bson::Document> {
        None
    }
}

impl FilterOptions {
    pub fn filter(&self) -> Option<bson::Document> {
        match to_document(&self.0[0]) {
            Ok(c) => Some(c),
            Err(e) => {
                log::error!("Error converting filter to document: {}", e);
                None
            }
        }
    }

    pub fn options(&self) -> Option<bson::Document> {
        match to_document(&self.0[1]) {
            Ok(mut c) => {
                if let None = c.get("maxTime") {
                    c.insert("maxTime", 60000);
                }
                Some(c)
            },
            Err(e) => {
                log::error!("Error converting options to document: {}", e);
                None
            }
        }
    }
}

impl Filters {
    pub fn filter(&self) -> Option<bson::Document> {
        match self {
            Filters::Filter(s) => s.filter(),
            Filters::FilterOptions(s) => s.filter()
        }
    }

    pub fn options(&self) -> Option<bson::Document> {
        match self {
            Filters::Filter(s) => s.options(),
            Filters::FilterOptions(s) => s.options()
        }
    }

    pub fn filter_and_options(&self) -> (Option<bson::Document>, Option<bson::Document>) {
        (self.filter(), self.options())
    }
}

