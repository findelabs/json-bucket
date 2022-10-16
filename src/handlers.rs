use axum::{
    extract::{OriginalUri},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Serialize, Deserialize};
use clap::{crate_description, crate_name, crate_version};
use serde_json::json;
use serde_json::Value;
use axum::extract::Path;
use axum::response::Response;
use axum::Extension;
use std::collections::HashMap;
use serde_json::Map;
use bson::to_document;

use crate::error::Error as RestError;
use crate::State;

// This is required in order to get the method from the request
#[derive(Debug)]
pub struct RequestMethod(pub hyper::Method);

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
                log::error!("Error converting filter to document");
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
                log::error!("Error converting filter to document");
                None
            }
        }
    }

    pub fn options(&self) -> Option<bson::Document> {
        match to_document(&self.0[1]) {
            Ok(c) => Some(c),
            Err(e) => {
                log::error!("Error converting options to document");
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
}

pub async fn find_one(
    Extension(mut state): Extension<State>,
    Path((database, collection)): Path<(String, String)>,
    Json(body): Json<Filters>,
) -> Result<Response, RestError> {

    Ok((StatusCode::OK, "Response").into_response())
}

pub async fn health() -> Json<Value> {
    log::info!("{{\"fn\": \"health\", \"method\":\"get\"}}");
    Json(json!({ "msg": "Healthy"}))
}

pub async fn root() -> Json<Value> {
    log::info!("{{\"fn\": \"root\", \"method\":\"get\"}}");
    Json(
        json!({ "version": crate_version!(), "name": crate_name!(), "description": crate_description!()}),
    )
}

pub async fn echo(Json(payload): Json<Value>) -> Json<Value> {
    log::info!("{{\"fn\": \"echo\", \"method\":\"post\"}}");
    Json(payload)
}

pub async fn help() -> Json<Value> {
    log::info!("{{\"fn\": \"help\", \"method\":\"get\"}}");
    let payload = json!({"paths": {
            "/health": "Get the health of the api",
            "/config": "Get config of api",
            "/reload": "Reload the api's config",
            "/echo": "Echo back json payload (debugging)",
            "/help": "Show this help message",
            "/:endpoint": "Show config for specific endpoint",
            "/:endpoint/*path": "Pass through any request to specified endpoint"
        }
    });
    Json(payload)
}

pub async fn handler_404(OriginalUri(original_uri): OriginalUri) -> impl IntoResponse {
    let parts = original_uri.into_parts();
    let path_and_query = parts.path_and_query.expect("Missing post path and query");
    log::info!(
        "{{\"fn\": \"handler_404\", \"method\":\"get\", \"path\":\"{}\"}}",
        path_and_query
    );
    (
        StatusCode::NOT_FOUND,
        "{\"error_code\": 404, \"message\": \"HTTP 404 Not Found\"}",
    )
}
