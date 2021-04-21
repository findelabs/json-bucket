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
    
    // Get last segment
    let last = &req.uri().path().split("/").last();
    
    match (req.method(), last) {
        // Alert transformed card with received variables
        (&Method::POST, Some("create")) => {
            let path = req.uri().path();
            log::info!("Received POST to {}", &path);

            // Split apart request
            let (parts,body) = req.into_parts();

            let path = match parts.uri.path() {
                "/" => "default".to_owned(),
                _ => {
                    let stage_one: Vec<&str> = parts.uri.path().split("/").collect();    // Convert to array
                    let stage_two = &stage_one[1..stage_one.len() - 1];                  // Remove the last path
                    let stage_three = stage_two.join("_");                               // Join back with underscores
                    stage_three
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
                    log::info!("Successfully transformed data into json");
                    val
                },
                Err(e) => {
                    log::info!("Got error converting data to json {}", e);
                    return Err(error::MyError::JsonError)
                }
            };

            // Convert to bson Document
            let data = Document::try_from(v).expect("failed to convert to doc");
            
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
                    let mut response = Response::new(Body::from(format!("{{\"msg\" : \"Successfully save\" }}")));
                    *response.status_mut() = StatusCode::OK;
                    Ok(response)
                },
                Err(e) => {
                    log::info!("Got error {}", e);
                    let mut response = Response::new(Body::from(format!("{{\"error\" : \"{}\" }}", e)));
                    *response.status_mut() = StatusCode::NOT_FOUND;
                    Ok(response)
                }
            }
        }

        (&Method::POST, Some("query")) => {
            let path = req.uri().path();
            log::info!("Received POST to {}", &path);

            // Split apart request
            let (parts,body) = req.into_parts();

            let path = match parts.uri.path() {
                "/" => "default".to_owned(),
                _ => {
                    let stage_one: Vec<&str> = parts.uri.path().split("/").collect();    // Convert to array
                    let stage_two = &stage_one[1..stage_one.len() - 1];                  // Remove the last path
                    let stage_three = stage_two.join("_");                               // Join back with underscores
                    stage_three
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
                    log::info!("Successfully transformed data into json");
                    val
                },
                Err(e) => {
                    log::info!("Got error converting data to json {}", e);
                    return Err(error::MyError::JsonError)
                }
            };

            // Convert to bson Document
            let data = Document::try_from(v).expect("failed to convert to doc");
            
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

            match db.query(&path, data).await {
                Ok(doc) => {
//                    let json_doc = stringify(doc.to_string());
                    let json_doc = serde_json::to_string(&doc).expect("failed converting bson to json");
                    let mut response = Response::new(Body::from(json_doc));
                    *response.status_mut() = StatusCode::OK;
                    Ok(response)
                },
                Err(e) => {
                    log::info!("Got error {}", e);
                    let mut response = Response::new(Body::from(format!("{{\"error\" : \"{}\" }}", e)));
                    *response.status_mut() = StatusCode::NOT_FOUND;
                    Ok(response)
                }
            }
        }
        // echo transformed card with received variables
        (&Method::GET, _) => Ok(Response::new(Body::from("{ \"msg\" : \"Get method not currently used\" }".to_string()))),

        // Return the 404 Not Found for other routes.
        (_, _) => {
            log::info!("Returning not found for {}", req.uri().path());
            let mut response = Response::new(Body::from(format!("{{\"msg\" : \"path or method not recognized\" }}")));
            *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
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
