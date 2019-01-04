#[macro_use]
//#[allow(dead_code)]
//#[allow(unused_variables)]
extern crate lazy_static;
extern crate config;
extern crate regex;
extern crate reqwest;
extern crate url;
extern crate clap;

pub mod configuration;
mod engine;
mod spider;
mod fetcher;
mod parser;
mod finalizer;

use std::thread;
use std::sync::{Arc, Mutex};
use clap::{App, Arg};
use config::Config;

fn main() {
    println!("-=<|[[[ HEATED WORDS STARTED ]]]|>=-");

    println!("### - Gathering arguments");
    let arguments =
        App::new("Heated Words")
            .arg(Arg::with_name("QUERY")
                .required(true)
                .help("A phrase that you would search on something like Google")
            )
            .arg(Arg::with_name("configuration file")
                .short("c")
                .long("config")
                .value_name("CONFIG_FILE.toml")
                .help("Sets a custom config file rather. Defaults to \"config.toml\".")
            )
            .arg(Arg::with_name("debug")
                .long("debug")
            )
            .arg(Arg::with_name("wordlist filename")
                .long("wordlist-output")
                .value_name("FILENAME")
                .help("Sets a custom name for the wordlist output file.")
            )
            .arg(Arg::with_name("link list filename")
                .long("links-output")
                .value_name("FILENAME")
                .help("Sets a custom name for the link list output file.")
            )
            .arg(Arg::with_name("required words")
                .multiple(true)
                .short("r")
                .long("require")
                .value_name("WORD")
                .help("Specify a word which must exist in the URL (minus domain) \
                for it to be valid")
            )
            .arg(Arg::with_name("exclude words")
                .multiple(true)
                .short("e")
                .long("exclude")
                .value_name("WORD")
                .help("Specify a word which must NOT exist in the URL (minus domain) \
                for it to be valid")
            )
            .arg(Arg::with_name("number of links")
                .short("n")
                .long("link-number")
                .value_name("MAX")
                .help("Specify the maximum number of links each Engine will return. \
                -1 is unlimited.")
            )
            .arg(Arg::with_name("match ratio")
                .long("match-ratio")
                .value_name("RATIO")
                .help("Specify the minimum ratio of words in the query \
                that must match the URL (minus domain)")
            )
            .arg(Arg::with_name("extra ratio")
                .long("extra-ratio")
                .value_name("RATIO")
                .help("Specify the maximum ratio of words in the URL (minus domain) \
                that don't match any word in the query")
            )
            .get_matches();

    let config_filename = arguments
        .value_of("configuration file")
        .unwrap_or("config.toml");
    println!("### - Loading configuration file: {}", config_filename);
    configuration::load_configuration_file(config_filename);
    println!("Done");

    print!("### - Building configuration from arguments");
    let mut arg_config = Config::new();

    let search_query = arguments.value_of("QUERY").unwrap();
    arg_config.set("query", search_query).unwrap();

    if arguments.is_present("debug") {
        arg_config.set("debug", true).unwrap();
    }

    match arguments.value_of("wordlist filename") {
        Some(filename) => {
            arg_config.set("filenames.wordlist", filename).unwrap();
        }
        None => ()
    }

    match arguments.value_of("link list filename") {
        Some(filename) => {
            arg_config.set("filenames.links", filename).unwrap();
        }
        None => ()
    }

    match arguments.values_of("required words") {
        Some(required_words) => {
            arg_config.set("required_words", required_words.collect::<Vec<&str>>()).unwrap();
        }
        None => ()
    }

    match arguments.values_of("exclude words") {
        Some(required_words) => {
            arg_config.set("exclude words", required_words.collect::<Vec<&str>>()).unwrap();
        }
        None => ()
    }

    match arguments.value_of("number of links") {
        Some(value) => {
            match value.parse::<i64>() {
                Ok(threshold) => {
                    arg_config.set("sensitivity.max_links", threshold).unwrap();
                }
                Err(e) => println!("!!! Match Ratio threshold is an invalid integer.\
                Program will fall back to config file: {}", e)
            }
        }
        None => ()
    }

    match arguments.value_of("match ratio") {
        Some(value) => {
            match value.parse::<f64>() {
                Ok(threshold) => {
                    arg_config.set("sensitivity.match_threshold", threshold).unwrap();
                }
                Err(e) => println!("!!! Match Ratio threshold is an invalid float.\
                Program will fall back to config file: {}", e)
            }
        }
        None => ()
    }

    match arguments.value_of("extra ratio") {
        Some(value) => {
            match value.parse::<f64>() {
                Ok(threshold) => {
                    arg_config.set("sensitivity.extra_threshold", threshold).unwrap();
                }
                Err(e) => println!("!!! Extra Ratio threshold is an invalid float.\
                Program will fall back to config file: {}", e)
            }
        }
        None => ()
    }
    println!("Done");

    println!("### - Joining arguments to config file for global access");
    println!("Debug is {} before set", configuration::read_debug());
    match configuration::CONFIGURATION.write() {
        Ok(mut config) => {
            match config.merge(arg_config) {
                Ok(_) => println!("Done"),
                Err(e) => panic!("{}", e)
            }
        }
        Err(e) => panic!("Couldn't open configuration to write: {}", e)
    }
    println!("Debug is {} after set", configuration::read_debug());
    println!("Done");

    // handle CLI arguments
    println!("### - Running program with query: \"{}\"", search_query);

    //build search engines from config file that we loaded earlier
    let engines = engine::build_engines();

    println!("### - Using the following search engines: ");
    for engine in &engines {
        println!("{}", &engine);
    }

    let complete_link_list = Arc::new(Mutex::new(Vec::new()));
    let mut running_engines = Vec::new();

    println!("### - Dispatching engines...");
    for engine in engines {
        let shared_list = Arc::clone(&complete_link_list);
        running_engines.push(thread::spawn(move || {
            match engine.dispatch() {
                Ok(links) => {
                    let mut link_vector = shared_list.lock().unwrap();
                    for link in links {
                        link_vector.push(link);
                    }
                }
                Err(e) => println!("{}", e)
            }
        }));
    }

    //waits for them to complete
    for running in running_engines {
        running.join().unwrap();
    }

    //then gives the resulting list to the Spider
    match complete_link_list.to_owned().lock() {
        Ok(list) => {
            println!("$$$ - Collected {} links in total", list.len());
            if configuration::read_debug() {
                for link in list.iter() {
                    println!("L: {}", link);
                }
            }

            let mut spider = spider::Spider::new(list.to_vec());
            println!("### - Dispatching spider...");
            let mut results: Vec<String> = spider.dispatch();


            println!("### - Finalizing results...");
            finalizer::finish_link_vector(list.to_vec());
            finalizer::finish_wordlist(&mut results);

            println!("### - Executing post-processing...");
            finalizer::run_post_processing();

            println!("-=<|[[[ HEATED WORDS COMPLETED ]]]|>=-");
        }
        Err(e) => panic!("{}", e)
    }
}
