use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;

use super::fetcher::Fetcher;

#[derive(Debug)]
pub struct Spider {
    link_vector: Vec<String>
}

impl Spider {
    pub fn new(link_vector: Vec<String>) -> Spider {
        Spider {
            link_vector
        }
    }

    pub fn dispatch(&mut self) -> Vec<String> {
        println!("### - Spider dispatched");

        println!("### - Spider is now dispatching fetchers...");
        //create workers and the master receiver
        let (spider_rx, fetchers) = self.build_fetchers();
        //start workers
        for mut fetcher in fetchers {
            thread::spawn(move || {
                fetcher.dispatch();
            });
        }

        println!("### - Spider is now receiving results from the Fetchers");
        let mut results = Vec::new();
        for received in spider_rx {
            results.push(received);
        }
        println!("$$$ - Spider has retrieved all values from workers");

        results
    }

    fn build_fetchers(&mut self) -> (Receiver<String>, Vec<Fetcher>) {
        let mut fetchers = Vec::new();
        let (tx, rx) = mpsc::channel();
        let master_rx = rx;
        let slave_tx = Arc::new(Mutex::new(tx));
        for link in &self.link_vector {
            fetchers.push(Fetcher::new(link.clone(),
                                       slave_tx.clone())
            );
        }
        (master_rx, fetchers)
    }
}