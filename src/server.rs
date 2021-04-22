use hyper::{Body, Method, Request, Response, StatusCode};
use std::str::from_utf8;
//use rust_tools::http::queries;
use rust_tools::strings::get_root_path;
use rust_tools::bson::to_doc;

use crate::db;
use crate::error;

// This is our service handler. It receives a Request, routes on its
// path, and returns a Future of a Response.
pub async fn echo(req: Request<Body>, db: db::DB) -> Result<Response<Body>, error::MyError> {
    
    // Match on method
    match req.method() {
        &Method::POST => {
            // Get last segment
            let last = &req.uri().path().split("/").last();
    
            // Filter on action
            match last {
                Some("create") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);
        
                    // Split apart request
                    let (parts,body) = req.into_parts();
        
                    // Get short root path
                    let collection = get_root_path(&parts);
        
                    // Create queriable hashmap from queries
                    // let _queries = queries(&parts).expect("Failed to generate hashmap of queries");
        
                    // Convert body to utf8
                    let whole_body = hyper::body::to_bytes(body).await?;
                    let whole_body_vec = whole_body.iter().cloned().collect::<Vec<u8>>();
                    let value = from_utf8(&whole_body_vec).to_owned()?;
        
                    // Convert string to bson
                    let data = match to_doc(value) {
                        Ok(d) => d,
                        Err(_) => {
                            return Err(error::MyError::JsonError)
                        }
                    };
        
                    // Print out converted bson doc
                    log::info!("Converted json into bson doc: {}", data);
        
                    match db.insert(&collection, data).await {
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
                Some("findone") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);
        
                    // Split apart request
                    let (parts,body) = req.into_parts();
        
                    // Get short root path
                    let collection = get_root_path(&parts);
        
                    // Create queriable hashmap from queries
                    // let _queries = queries(&parts).expect("Failed to generate hashmap of queries");
        
                    // Convert body to json Value
                    let whole_body = hyper::body::to_bytes(body).await?;
                    let whole_body_vec = whole_body.iter().cloned().collect::<Vec<u8>>();
                    let value = from_utf8(&whole_body_vec).to_owned()?;
        
                    // Convert string to bson
                    let data = match to_doc(value) {
                        Ok(d) => d,
                        Err(_) => {
                            return Err(error::MyError::JsonError)
                        }
                    };
        
                    // Print out converted bson doc
                    log::info!("Converted json into bson doc: {}", data);
        
                    match db.findone(&collection, data).await {
                        Ok(doc) => {
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
                Some("find") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);
        
                    // Split apart request
                    let (parts,body) = req.into_parts();
        
                    // Get short root path
                    let collection = get_root_path(&parts);
        
                    // Create queriable hashmap from queries
                    // let _queries = queries(&parts).expect("Failed to generate hashmap of queries");
        
                    // Convert body to json Value
                    let whole_body = hyper::body::to_bytes(body).await?;
                    let whole_body_vec = whole_body.iter().cloned().collect::<Vec<u8>>();
                    let value = from_utf8(&whole_body_vec).to_owned()?;
        
                    // Convert string to bson
                    let data = match to_doc(value) {
                        Ok(d) => d,
                        Err(_) => {
                            return Err(error::MyError::JsonError)
                        }
                    };
        
                    // Print out converted bson doc
                    log::info!("Converted json into bson doc: {}", data);
        
                    match db.find(&collection, data).await {
                        Ok(doc) => {
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
                _ => {
                    Ok(Response::new(Body::from("{ \"msg\" : \"Path currently not used\" }".to_string())))
                }
            }
        }

        // Match on Get
        &Method::GET => {

            // Get path
            let path = &req.uri().path();
        
            // Match on path
            match path {
                &"/_cat/collections" => {
                    match db.collections().await {
                        Ok(collections) => {
                            let json_doc = serde_json::to_string(&collections).expect("failed converting collection bson to json");
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
                _ => Ok(Response::new(Body::from("{ \"msg\" : \"Get method not currently used\" }".to_string()))),
            }
        }

        // Return the 404 Not Found for other routes.
        _ => {
            log::info!("Returning not found for {}", req.uri().path());
            let mut response = Response::new(Body::from(format!("{{\"msg\" : \"path or method not recognized\" }}")));
            *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
        }
    }
}
