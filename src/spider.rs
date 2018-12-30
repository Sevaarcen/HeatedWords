// https://github.com/sebasmagri/rust-concurrency-patterns/tree/master/examples


use std::fmt;
use std::sync::{mpsc, Arc, Mutex};

use super::parser;

//a Spider is a producer
#[derive(Clone, Debug)]
struct Spider {
    id: String,
    url: String,
    master_rx: Arc<Mutex<mpsc::Receiver<usize>>>
}

impl fmt::Display for Spider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Spider {}", self.id, self.url)
    }
}

impl Spider {
    fn new(id: String, url: String, master_rx: Arc<Mutex<mpsc::Receiver<usize>>>) -> Spider {
        Spider {
            id,
            url,
            master_rx
        }
    }

    pub fn dispatch(&self) {
        println!("{} dispatched to retrieve {}", self, self.url);

        //query URL
        //get result
        //parse
        //feed to master
    }

    fn get_url(&self) -> Result<String, String> {
        match reqwest::get(url) {
            Ok(mut response) => {
                if response.status()  == reqwest::StatusCode::OK {
                    match response.text() {
                        Ok(text) => {
                            Ok(text)
                        },
                        Err(e) => Err(format!("{} encountered an error: {}", self, e))
                    }
                }
                else{
                    Err(format!("{} failed to GET \"{}\" and received status code {}", self, url, response.status()))
                }
            },
            Err(e) => Err(format!("{} failed to GET \"{}\": {}", self, url, e))
        }
    }
}