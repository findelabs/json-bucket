use hyper::{Body, Method, Request, Response, StatusCode};
use std::str::from_utf8;
//use rust_tools::http::queries;
use rust_tools::bson::to_doc;
use rust_tools::strings::get_root_path;
use std::error::Error;
use bson::Document;
use crate::db;

type BoxResult<T> = Result<T,Box<dyn Error + Send + Sync>>;

// This is the main handler, to catch any failures in the echo fn
pub async fn main_handler(
    req: Request<Body>,
    db: db::DB,
) -> BoxResult<Response<Body>> {
    match echo(req, db).await {
        Ok(s) => {
            log::debug!("Handler got success");
            Ok(s)
        }
        Err(e) => {
            log::debug!("Handler caught error: {}", e);
            let mut response = Response::new(Body::from(format!("{{\"error\" : \"{}\"}}", e)));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(response)
        }
    }
}

// This is our service handler. It receives a Request, routes on its
// path, and returns a Future of a Response.
async fn echo(req: Request<Body>, db: db::DB) -> BoxResult<Response<Body>> {
    // Match on method
    match req.method() {
        &Method::POST => {
            // Get last segment in uri path
            let last = &req.uri().path().split("/").last();

            // Filter on action
            match last {
                Some("create") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, data) = data_to_bson(req).await?;

                    match db.insert(&collection, data).await {
                        Ok(_) => {
                            let mut response = Response::new(Body::from(format!(
                                "{{\"msg\" : \"Successfully saved\" }}"
                            )));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::debug!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                Some("findone") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, data) = data_to_bson(req).await?;

                    match db.findone(&collection, data).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                Some("find") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, data) = data_to_bson(req).await?;

                    match db.find(&collection, data).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                _ => Ok(Response::new(Body::from(format!(
                    "{{ \"msg\" : \"{} is not a recognized action\" }}",
                    last.unwrap_or_else(|| "missing")
                )))),
            }
        }

        // Match on Get
        &Method::GET => {
            // Get first segment in uri path, looking for _cat (for now)
            let chunks: Vec<&str> = req.uri().path().split("/").collect();
            let first = chunks.get(1).unwrap_or_else(|| &"na");

            // Get path
            let path = &req.uri().path();

            // Match on first in path
            match first {
                &"_cat" => {
                    match path {
                        &"/_cat/collections" => {
                            let path = req.uri().path();
                            log::info!("Received GET to {}", &path);
        
                            match db.collections().await {
                                Ok(collections) => {
                                    let json_doc = serde_json::to_string(&collections)
                                        .expect("failed converting collection bson to json");
                                    let mut response = Response::new(Body::from(json_doc));
                                    *response.status_mut() = StatusCode::OK;
                                    Ok(response)
                                }
                                Err(e) => {
                                    log::error!("Got error {}", e);
                                    Err(Box::new(e))
                                }
                            }
                        }
                        _ => Ok(Response::new(Body::from(format!(
                            "{{ \"msg\" : \"{} is not a known path under /_cat\" }}",
                            path
                        )))),
                    }
                },
                // Here, we are going to look for ending paths for collections, such as _count
                _ => {
                     // Get last segment in uri path
                    let last = &req.uri().path().split("/").last().unwrap_or_else(|| "na");
                    
                    match last {
                        &"_count" => {
                            log::info!("Received GET to {}", req.uri().path());

                            // Get short root path (the collection name)
                            let (parts, _body) = req.into_parts();
                            let collection = get_root_path(&parts);

                            match db.count(&collection).await {
                                Ok(doc) => {
                                    let json_doc = serde_json::to_string(&doc)
                                        .expect("failed converting bson to json");
                                    let mut response = Response::new(Body::from(json_doc));
                                    *response.status_mut() = StatusCode::OK;
                                    Ok(response)
                                }
                                Err(e) => {
                                    log::error!("Got error {}", e);
                                    Err(Box::new(e))
                                }
                            }
                        },
                        _ => {
                            log::info!("Received GET to {}", req.uri().path());

                            Ok(Response::new(Body::from(format!(
                                "{{ \"msg\" : \"Unknown path for collection GET\" }}"
                            ))))
                        }
                    }
                }
            }
        }

        // Return the 404 for unknown meth
        _ => {
            log::info!("Method not recognized {}", req.method());
            let mut response = Response::new(Body::from(format!(
                "{{\"msg\" : \"method not recognized\" }}"
            )));
            *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
        }
    }
}

pub async fn data_to_bson(req: Request<Body>) -> BoxResult<(String, Document)> {
    // Split apart request
    let (parts, body) = req.into_parts();

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
        Err(e) => return Err(e),
    };

    // Print out converted bson doc
    log::debug!("Converted json into bson doc: {}", data);

    Ok((collection, data))
}
