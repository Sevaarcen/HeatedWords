use std::fmt;
use std::sync::{mpsc, Arc, Mutex};

use super::configuration;
use super::parser;

#[derive(Clone, Debug)]
pub struct Fetcher {
    url: String,
    transmitter: Arc<Mutex<mpsc::Sender<String>>>,
}

impl fmt::Display for Fetcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fetcher => \"{}\"", self.url)
    }
}

impl Fetcher {
    pub fn new(url: String, transmitter: Arc<Mutex<mpsc::Sender<String>>>) -> Fetcher {
        Fetcher {
            url,
            transmitter,
        }
    }

    pub fn dispatch(&mut self) {
        if configuration::read_debug() {
            println!("### - Dispatched {}", self);
        }

        match self.get_url() {
            Ok(response) => {
                let results = parser::parse(&mut response.clone());
                let unlocked_tx = self.transmitter.lock().unwrap();
                if configuration::read_debug() {
                    println!("{}: collected vectors of length {} & {}", self, results.0.len(), results.1.len());
                }
                for item in results.0 {
                    unlocked_tx.send(item).unwrap();
                }
                for item in results.1 {
                    unlocked_tx.send(item).unwrap();
                }
            }
            Err(e) => println!("!!! - {} encountered an error: {}", self, e)
        }

        println!("$$$ - {} has completed sucessfully", self);
    }

    fn get_url(&self) -> Result<String, String> {
        match reqwest::get(self.url.as_str()) {
            Ok(mut response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.text() {
                        Ok(text) => {
                            Ok(text)
                        }
                        Err(e) => Err(format!("!!! - {} encountered an error: {}", self, e))
                    }
                } else {
                    Err(format!("!!! - {} failed to GET \"{}\" and received status code {}", self, self.url, response.status()))
                }
            }
            Err(e) => Err(format!("!!! - {} failed to GET \"{}\": {}", self, self.url, e))
        }
    }
}