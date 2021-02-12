use chrono::prelude::*;
use mongodb::bson::{doc, document::Document};
use serde_json::Value;
use url::Url;

use crate::error::MyError;

pub fn transform_request(value: &str, column: &str) -> Result<Document, MyError> {
    let doc: serde_json::Value = serde_json::from_str(value)?;
    let now = Utc::now();

    let summary = doc["summary"]
        .as_str()
        .ok_or_else(|| MyError::MissingItem)?;
    let title = doc["title"].as_str().ok_or_else(|| MyError::MissingItem)?;
    let url = doc["url"].as_str().ok_or_else(|| MyError::MissingItem)?;
    let tags = doc["tags"].as_array().ok_or_else(|| MyError::MissingItem)?;

    if title.chars().count() > 75usize {
        return Err(MyError::TitleTooLong);
    } else {
        log::info!("Title is {} chars long", title.chars().count());
    }

    if summary.chars().count() > 230usize {
        return Err(MyError::SummaryTooLong);
    } else {
        log::info!("Summary is {} chars long", summary.chars().count());
    }

    // extract domain from url
    let full_url = Url::parse(url)?;
    let site = full_url.domain().ok_or_else(|| MyError::UrlParseError)?;

    let tag_array: Vec<String> = tags
        .iter()
        .filter_map(|entry| match entry {
            Value::String(v) => Some(v.to_owned()),
            _ => None,
        })
        .collect();

    let post = doc! {
        "site": site.to_owned(),
        "summary": summary.to_owned(),
        "title": title.to_owned(),
        "url": url.to_owned(),
        "time": now,
        "tags": tag_array,
        "column": column.to_owned()
    };
    Ok(post)
}
