use std::fmt;
use std::sync::{mpsc, Arc, Mutex};
use std::fs::File;
use std::io::Write;
use regex::Regex;

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
            println!("###  Dispatched {}", self);
        }

        // retrieve the URL
        match self.get_url() {
            Ok(response) => {

                if configuration::read_debug() {
                    println!("~~~  {} got the URL with an OK", self);
                }
                
                // parse the webpage and get the list of words
                let results = parser::parse(&mut response.clone());

                // grab a lock on the transmitter to the Spider
                let unlocked_tx = self.transmitter.lock().unwrap();

                if configuration::read_debug() {
                    println!("~~~  {} collected vectors of length {} & {}", self, results.0.len(), results.1.len());
                }

                // transmit the list of 'critical words' first
                for item in results.0 {
                    unlocked_tx.send(item).unwrap();
                }

                // then transmit the list of normal words for the wordlist
                for item in results.1 {
                    unlocked_tx.send(item).unwrap();
                }

                if configuration::read_debug() {
                    println!("~~~  {} transmitted all to spider", self);
                }
                
            }
            Err(e) => println!("!!!  {} encountered an error: {}", self, e)
        }

        println!("$$$  {} has completed sucessfully", self);
    }

    fn get_url(&self) -> Result<String, String> {

        if configuration::read_debug() {
            println!("~~~ {} started to get URL", self);
        }

        // turn the URL into a str object for ease of reference
        let target_url = self.url.as_str();

        // grab user-agent to impersonate from configuration file
        let user_agent_string = match configuration::CONFIGURATION.read().unwrap().get_str("user-agent") {
            Ok(result) => result.clone(),
            Err(e) => panic!("Could not read user-agent from configuration: {}", e)
        };

        // build request client that uses blocking IO and the specified user-agent
        let request_client = match reqwest::blocking::Client::builder()
            .user_agent(user_agent_string)
            .build() {
                Ok(client) => {
                    client
                },
                Err(e) => panic!("Could not create Fetcher Client: {}", e)
            };
        
        if configuration::read_debug() {
            println!("~~~  {} has finished building the request client", self);
        }

        // actually make the request
        let response = match request_client.get(target_url).send() {
            Ok(resp) => resp,
            Err(e) => return Err(format!("!!!  {} failed to make request due to error: {}", self, e))
        };

        // check if the status code is a 2XX
        if response.status() != reqwest::StatusCode::OK {
            return Err(format!("!!!  {} made request but received status code '{}'", self, response.status()))
        }

        if configuration::read_debug() {
            println!("~~~  {} has finished making the request to '{}'", self, target_url);
        }

        // get the response text (html webpage)
        let response_text = match response.text() {
            Ok(text) => text,
            Err(e) => return Err(format!("!!!  {} could not retrieve text from response due to error: {}", self, e))
        };

        if configuration::read_debug() {
            println!("~~~  {} has successfully retrieved the response text", self);
        }

        // if debug write the webpage to file
        if configuration::read_debug() {
            println!("~~~  fetcher is running in debug mode and will write everything to a special file");
            let rere = Regex::new(r"[\W]").unwrap();
            let debug_filename = rere.replace_all(self.url.as_str(), "_");
            println!("~~~~~ to file: {}", debug_filename);
            match File::create(format!("{}.html", debug_filename)) {
                Ok(mut file) => {
                    match write!(file, "{}\n", response_text) {
                        Ok(_) => (),
                        Err(e) => println!("!!!  Spider debug could not write special debug wordlist: {}", e)
                    }
                },
                Err(e) => println!("!!!  Spider debug could not create special debug wordlist: {}", e)
            }
        }

        // return the content of the webpage in an Ok
        Ok(response_text)
    }
}