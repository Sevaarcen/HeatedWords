extern crate regex;
extern crate url;
extern crate reqwest;

use std::fmt;
use regex::Regex;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use url::Url;

use super::configuration;

const LINK_REX: &str = r#"(?P<link>[\S\s]+?)"#;

#[derive(Clone, Debug)]
pub struct Engine {
    base_url: Url,
    link_regex: Regex,
}
impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Engine \"{}\"", self.base_url)
    }
}
impl Engine {
    fn new(base_url: String, contents_pattern: String) -> Engine {
        Engine {
            base_url: Url::parse(base_url.as_str()).unwrap(),
            link_regex: Regex::new(
                contents_pattern.replace("{}", LINK_REX).as_str()
                ).unwrap(),
        }
    }

    pub fn dispatch(&self, query: &str) -> Result<Vec<String>, String> {
        println!("### - {} dispatched with query: {}", self, query);
        let encoded_query = utf8_percent_encode(query, DEFAULT_ENCODE_SET).to_string();
        let url = self.base_url.as_str().replace("{}", encoded_query.as_str());

        match self.generate_links(url.as_str()) {
            Ok(links) => {
                if configuration::read_debug() {
                    println!("{} full url = {:?}", self, url);
                    println!("{} extracted the following links with the regex \"{}\":", self, self.link_regex);
                    for link in &links {
                        println!("{}", link);
                    }
                }
                Ok(links)
            },
            Err(e) => Err(e)
        }
    }

    //runs actual search via search engine
    fn generate_links(&self, url: &str) -> Result<Vec<String>, String> {

        match reqwest::get(url) {
            Ok(mut response) => {
                if response.status()  == reqwest::StatusCode::OK {
                    match response.text() {
                        Ok(text) => {
                            if configuration::read_debug() {
                                println!("\n\n\n{}\n\n\n", text);
                            }
                            Ok(self.extract_links(text))
                        },
                        Err(e) => Err(format!("!!! - {} encountered an error: {}", self, e))
                    }
                }
                else{
                    Err(format!("!!! - Engine failed to GET \"{}\" due to receiving status code {}", url, response.status()))
                }
            },
            Err(e) => Err(format!("!!! - {} failed to GET its URL \"{}\": {}", self, url, e))
        }
    }

    //returns a vector of the URLs
    fn extract_links(&self, search_results: String) -> Vec<String> {
        let links = self.link_regex
            .captures_iter(search_results.as_str())
            .map(|cap| match self.base_url.join(cap.name("link").unwrap().as_str()) { //try joining the link with the base_url
                Ok(full_url) => {
                    if configuration::read_debug() {
                        println!("RL: {}", cap.name("link").unwrap().as_str());
                    }
                    full_url.to_string()
                }, //if sucessful, the link was relative
                Err(_) => cap.name("link").unwrap().as_str().to_string(), //otherwise its absolute and can be returned
            })
            .collect();

        links
    }
}

pub fn build_engines() -> Vec<Engine> {
    let mut engines: Vec<Engine> = Vec::new();

    match configuration::CONFIGURATION.read() {
        Ok(config) => {
            match config.get_table("engines") {
                Ok(table_list) => {
                    for table in table_list
                        .values()
                        .map(|table| table.clone().into_table().unwrap())
                        {
                            engines.push(
                                Engine::new(
                                    table.get("url").unwrap().clone().into_str().unwrap(),
                                    table.get("regex").unwrap().clone().into_str().unwrap())
                            );
                        }
                }
                Err(_) => panic!("!!! - The \"engines\" table and its relevant sub-tables are missing from the configuration.")
            }
        },
        Err(e) => panic!("!!! - Configuration could not be read : {}", e)
    }

    println!("$$$ - Sucessfully built {} search Engines", engines.len());

    engines
}