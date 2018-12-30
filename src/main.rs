#[macro_use]
#[allow(dead_code)]
#[allow(unused_variables)]

extern crate lazy_static;
extern crate config;
extern crate regex;
extern crate reqwest;
extern crate url;


pub mod configuration;
mod fetcher;
mod parser;
mod engine;

use std::env;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

fn main() {
    //read configuration file into the Config
    configuration::load_configuration_file();

    //handle arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("You must specify a search query as an argument (e.g. \"College football teams\").");
    }

    //build search engines from config file that we loaded earlier
    let engines = engine::build_engines();

    println!("Using the following search engines: ");
    for engine in &engines {
        println!("{}", engine);
    }

    // handle CLI arguments
    let search_string = args[1].as_str();
    let query = utf8_percent_encode(search_string, DEFAULT_ENCODE_SET).to_string();

    println!("Dispatching...");
    for engine in &engines {
        engine.dispatch(&query.as_str());
    }



//    let search_results = fetcher::search(&engines, &search_string);
//    for result in search_results {
//        match result {
//            Ok((url, mut response)) => {
//                println!("Suceeded in GET request to: {}", url);
//
//                let word_list = parser::parse(&mut response);
//
//                println!("Wordlist contains {} critical words", word_list.0.len());
//                println!("Wordlist contains {} individual words", word_list.1.len());
//                for word in word_list.0 {
//                    println!("<|[{}]|> => {}", url, word);
//                }
//                for word in word_list.1 {
//                    if word.len() > 4 {
//                        println!("[{}] -> {}", url, word);
//                    }
//                }
//            }
//            Err(e) => eprintln!("Failed GET request: {}", e)
//        }
//    }
}
