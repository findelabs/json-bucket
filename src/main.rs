use axum::{
    handler::Handler,
    routing::{get, post},
    Router,
    middleware,
    extract::Extension
};
use chrono::Local;
use clap::{crate_name, crate_version, Command, Arg};
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::future::ready;
use std::io::Write;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use mongodb::Client;
use mongodb::options::ClientOptions;

mod error;
mod handlers;
mod metrics;
mod mongo;
mod filters;
mod inserts;
mod request_body;

use crate::metrics::{setup_metrics_recorder, track_metrics};
use handlers::{echo, handler_404, health, help, root, find_one, find, insert, insert_many, collections, databases, aggregate};
use mongo::MongoClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let opts = Command::new(crate_name!())
        .version(crate_version!())
        .author("")
        .about(crate_name!())
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("Set port to listen on")
                .env("JSON_BUCKET_PORT")
                .default_value("8080")
                .takes_value(true),
        )
        .arg(
            Arg::new("mongo")
                .short('m')
                .long("mongo")
                .help("MongoDB connection url")
                .env("JSON_BUCKET_MONGO")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    // Initialize log Builder
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{{\"date\": \"{}\", \"level\": \"{}\", \"log\": {}}}",
                Local::now().format("%Y-%m-%dT%H:%M:%S:%f"),
                record.level(),
                record.args()
            )
        })
        .target(Target::Stdout)
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    // Set port
    let port: u16 = opts.value_of("port").unwrap().parse().unwrap_or_else(|_| {
        eprintln!("specified port isn't in a valid range, setting to 8080");
        8080
    });

    // Create mongo client
    let client_options = ClientOptions::parse(opts.value_of("mongo").unwrap()).await?;
    let client = Client::with_options(client_options)?;
    if let Err(e) = client.list_database_names(None, None).await {
        panic!("{}", e);
    };

    // Create mongo for axum
    let mongo = MongoClient::new(opts.clone(), client).await?;

    // Create prometheus handle
    let recorder_handle = setup_metrics_recorder();

    // These should be authenticated
    let base = Router::new()
        .route("/", get(root));

    // These should NOT be authenticated
    let standard = Router::new()
        .route("/health", get(health))
        .route("/echo", post(echo))
        .route("/:database/:collection/find_one", post(find_one))
        .route("/:database/:collection/find", post(find))
        .route("/:database/:collection/insert", post(insert))
        .route("/:database/:collection/insert_many", post(insert_many))
        .route("/:database/:collection/aggregate", post(aggregate))
        .route("/:database/_cat/collections", get(collections))
        .route("/_cat/databases", get(databases))
        .route("/help", get(help))
        .route("/metrics", get(move || ready(recorder_handle.render())));

    let app = Router::new()
        .merge(base)
        .merge(standard)
        .layer(TraceLayer::new_for_http())
        .route_layer(middleware::from_fn(track_metrics))
        .layer(Extension(mongo));

    // add a fallback service for handling routes to unknown paths
    let app = app.fallback(handler_404.into_service());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
