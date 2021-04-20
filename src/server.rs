use hyper::{Body, Method, Request, Response, StatusCode};
use std::str::from_utf8;
use http::request::Parts;
use url::form_urlencoded;
use std::collections::HashMap;
use serde_json::{Map, Value};
use bson::Document;
use std::convert::TryFrom;

use crate::db;
use crate::error;
//use crate::transform;

pub type Queries = HashMap<String, String>;

// This is our service handler. It receives a Request, routes on its
// path, and returns a Future of a Response.
pub async fn echo(req: Request<Body>, db: db::DB) -> Result<Response<Body>, error::MyError> {
    match req.method() {
        // Alert transformed card with received variables
        &Method::POST => {
            let path = req.uri().path();
            log::info!("Received POST to {}", &path);

            // Split apart request
            let (parts,body) = req.into_parts();

            let path = match parts.uri.path() {
                "/" => "default".to_owned(),
                _ => {
                    let transform = parts.uri.path().replace("/","_");  // Get path without forward slashes
                    let new = &transform[1..transform.len()];  // Chop off first char
                    new.to_string()
                }
            };

            // Create queriable hashmap from queries
            let _queries = queries(&parts).expect("Failed to generate hashmap of queries");

            // Convert body to json Value
            let whole_body = hyper::body::to_bytes(body).await?;
            let whole_body_vec = whole_body.iter().cloned().collect::<Vec<u8>>();
            let value = from_utf8(&whole_body_vec).to_owned()?;

            // Convert json to bson
            let v: Map<String, Value> = match serde_json::from_str(value) {
                Ok(val) => {
                    log::info!("Successfully transformed data into hashmap");
                    val
                },
                Err(e) => {
                    log::info!("Got error converting data to json {}", e);
                    return Err(error::MyError::JsonError)
                }
            };

            // Convert to bson Document
            let data = Document::try_from(v).expect("failed to convert to Doc");
            
            // Print out converted bson doc
            log::info!("Converted json into bson doc: {}", data);

// This will work with bson 1.2.0, accourding to: https://github.com/mongodb/bson-rust/issues/189
// The issue currently is that bson::to_bson does not handle unsigned integers properly currently
//
//            let data = match bson::to_bson(&v) {
//                Ok(d) => d,
//                Err(e) => {
//                    log::info!("Got error converting {} to bson: {}", value, e);
//                    return Err(error::MyError::JsonError)
//                }
//            };

            match db.insert(&path, data).await {
                Ok(_) => {
                    let mut response = Response::default();
                    *response.status_mut() = StatusCode::OK;
                    Ok(response)
                },
                Err(e) => {
                    log::info!("Got error {}", e);
                    let mut response = Response::default();
                    *response.status_mut() = StatusCode::NOT_FOUND;
                    Ok(response)
                }
            }
        }

        // echo transformed card with received variables
        &Method::GET => Ok(Response::new(Body::from("ok".to_string()))),

        // Return the 404 Not Found for other routes.
        _ => {
            log::info!("Returning not found for {}", req.uri().path());
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn queries(parts: &Parts) -> Option<Queries> {
    let queries: HashMap<String, String> = parts
        .uri
        .query()
        .map(|v| {
            form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);

    Some(queries)
}
