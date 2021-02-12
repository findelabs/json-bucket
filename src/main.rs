use chrono::Local;
use clap::{crate_version, App, Arg};
use env_logger::Builder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Server};
use log::LevelFilter;
use std::io::Write;

use db::DB;
use error::MyError;

mod db;
mod error;
mod server;
mod transform;

type Result<T> = std::result::Result<T, MyError>;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = App::new("mongodb_poster")
        .version(crate_version!())
        .author("Daniel F. <dan@findelabs.com>")
        .about("Main findereport site generator")
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .required(true)
                .value_name("URL")
                .env("MONGODB_URL")
                .help("MongoDB URL")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Set port to listen on")
                .required(false)
                .env("LISTEN_PORT")
                .default_value("8080")
                .takes_value(true),
        )
        .get_matches();

    // Initialize log Builder
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{{\"date\": \"{}\", \"level\": \"{}\", \"message\": \"{}\"}}",
                Local::now().format("%Y-%m-%dT%H:%M:%S:%f"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    // Read in config file
    let url = &opts.value_of("url").unwrap();
    let port: u16 = opts.value_of("port").unwrap().parse().unwrap_or_else(|_| {
        eprintln!("specified port isn't in a valid range, setting to 8080");
        8080
    });

    let db = DB::init(&url).await?;
    let addr = ([0, 0, 0, 0], port).into();
    let service = make_service_fn(move |_| {
        let db = db.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                server::echo(req, db.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

    println!(
        "Starting mongodb-poster:{} on http://{}",
        crate_version!(),
        addr
    );

    server.await?;
    //    if let Err(e) = server.await {
    //        eprintln!("server error: {}", e);
    //    }

    Ok(())
}
