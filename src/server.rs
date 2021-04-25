use hyper::{Body, Method, Request, Response, StatusCode};
use std::str::from_utf8;
//use rust_tools::http::queries;
use rust_tools::bson::to_doc;
use rust_tools::strings::get_root_path;
use std::error::Error;
use clap::ArgMatches;
use bson::Document;
use crate::db;

type BoxResult<T> = Result<T,Box<dyn Error + Send + Sync>>;

// This is the main handler, to catch any failures in the echo fn
pub async fn main_handler(
    opts: ArgMatches<'_>,
    req: Request<Body>,
    db: db::DB,
) -> BoxResult<Response<Body>> {
    match echo(opts, req, db).await {
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
async fn echo(opts: ArgMatches<'_>, req: Request<Body>, db: db::DB) -> BoxResult<Response<Body>> {

    // Check if first folder in path is _cat
    // Get first segment in uri path, looking for _cat (for now)
    let chunks: Vec<&str> = req.uri().path().split("/").collect();
    let first = chunks.get(1).unwrap_or_else(|| &"na");

    // Get path
    let path = &req.uri().path();

    // Match on first folder in path. Currently we just are looking for _cat, but there will be more in the future.
    match first {
        &"_cat" => {
            match (req.method(), path) {
                (&Method::GET, &"/_cat/collections") => {
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
        // Here, since _cat has been skipped, match Method and Last folder as action
        _ => {
            // Get last segment in uri path
            let last = &req.uri().path().split("/").last().unwrap_or_else(|| "na");

            match (req.method(), last) {
                (&Method::POST, &"create") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, data) = data_to_bson(req).await?;

                    match db.insert(opts, &collection, data).await {
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
                (&Method::POST, &"findone") => {
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
                (&Method::POST, &"find") => {
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
                (&Method::GET, &"_count") => {
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
                _ => Ok(Response::new(Body::from(format!(
                    "{{ \"msg\" : \"{} is not a recognized action\" }}",
                    last)
                ))),
            }
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
