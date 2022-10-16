use clap::ArgMatches;
use std::error::Error;

//use crate::https::{HttpsClient, ClientBuilder};
//use crate::error::Error as RestError;

type BoxResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone, Debug)]
pub struct State {
    pub client: mongodb::Client,
}

impl State {
    pub async fn new(opts: ArgMatches, client: mongodb::Client) -> BoxResult<Self> {
        Ok(State {
            client,
        })
    }

}
