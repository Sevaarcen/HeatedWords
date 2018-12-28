extern crate config;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::sync::RwLock;
use config::Config;

mod fetcher;
mod parser;

// example websites to test this prototype
// "https://en.wikipedia.org/wiki/List_of_professional_sports_teams_in_the_United_States_and_Canada"
// "https://www.cbssports.com/college-football/teams/"

lazy_static! {
    static ref CONFIGURATION: RwLock<Config> = RwLock::new(Config::default());
}

fn main() {
    //read configuration file into the Config
    match CONFIGURATION.write() {
        Ok(mut config_file) => {
            config_file.merge(config::File::with_name("config.toml")).unwrap();
            ()
        }
        Err(e) => panic!("Error loading configuration file in main: {}", e)
    }

//    match CONFIGURATION.read() {
//        Ok(config_file) => {
//            println!("{}", config_file.get::<String>("test_table.greeting").unwrap());
//            println!("{}", config_file.get::<String>("test_table.response").unwrap());
//
//            let cmap = config_file.get_table("test_table").unwrap();
//            println!("{}", cmap.get("greeting").unwrap());
//            println!("{}", cmap.get("response").unwrap());
//        }
//        Err(e) => panic!("Error reading configuration file in main: {}", e)
//    }

    //handle arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("You must specify a search query. You only specified {} arguments", args.len());
    }

    let engines: Vec<String> = match CONFIGURATION.read() {
        Ok(config_file) => {
            config_file
                .get_table("search_engines")
                .unwrap()
                .into_iter()
                .map(|(_, value)| value.into_str().unwrap().clone())
                .collect()
        }
        Err(e) => panic!("Error reading the search base URLs from configuration file : {}", e)
    };

    let search_string = args[1].clone();

    let search_results = fetcher::search(&engines, &search_string);
    for result in search_results {
        match result {
            Ok((url, mut response)) => {
                println!("Suceeded in GET request to: {}", url);

                let word_list = parser::parse(&mut response);

                println!("Wordlist contains {} critical words", word_list.0.len());
                println!("Wordlist contains {} individual words", word_list.1.len());
                for word in word_list.0 {
                    println!("<|[{}]|> => {}", url, word);
                }
                for word in word_list.1 {
                    if word.len() > 4 {
                        println!("[{}] -> {}", url, word);
                    }
                }
            }
            Err(e) => eprintln!("Failed GET request: {}", e)
        }
    }
}
