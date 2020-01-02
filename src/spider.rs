use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::fs::File;
use std::io::Write;

use super::configuration;
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
        println!("###  Spider dispatched");

        println!("###  Spider is now dispatching fetchers...");
        //create workers and the master receiver
        let (spider_rx, fetchers) = self.build_fetchers();
        //start workers
        for mut fetcher in fetchers {
            thread::spawn(move || {
                fetcher.dispatch();
            });
        }

        println!("###  Spider is now receiving results from the Fetchers");
        if configuration::read_debug() {
            println!("~~~  Spider is running in debug mode, each fetcher will write its own file");

        }
        // create Vector to store the results
        let mut results = Vec::new();
        // as long as it is open, read the contents of the receiver into the results Vector
        for received in spider_rx {
            results.push(received);
        }
        println!("$$$  Spider has retrieved all values from workers");

        if configuration::read_debug() {
            println!("~~~  Spider is running in debug mode and will write everything to a special file");
            match File::create("debug_wordlist.txt") {
                Ok(mut file) => for result in results.clone() {
                    match write!(file, "{}\n", result) {
                        Ok(_) => (),
                        Err(e) => println!("!!!  Spider debug could not write special debug wordlist: {}", e)
                    }
                },
                Err(e) => println!("!!!  Spider debug could not create special debug wordlist: {}", e)
            }
            
        }

        results
    }

    fn build_fetchers(&mut self) -> (Receiver<String>, Vec<Fetcher>) {
        // create list of fetchers so it can be managed
        let mut fetchers = Vec::new();
        // create a channel transmitter and receiver for interprocesses communication
        let (tx, rx) = mpsc::channel();
        // rereference the variable so it doesn't conflict
        let master_rx = rx;
        // clone the transmitter so it can be shared with the fetcher
        let slave_tx = Arc::new(Mutex::new(tx));
        // for every URL managed by the Spider, create a fetcher and give it a copy of the transmitter
        for link in &self.link_vector {
            fetchers.push(Fetcher::new(link.clone(),
                                       slave_tx.clone())
            );
        }
        (master_rx, fetchers)
    }
}