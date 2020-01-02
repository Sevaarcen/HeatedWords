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
use clap::{App, Arg, ArgGroup};
use config::Config;
use std::fs::File;
use std::io::{BufReader, BufRead};

fn main() {
    println!("-=<|[[[ HEATED WORDS STARTED ]]]|>=-");

    // gathering CLI arguments
    println!("###  Gathering arguments");
    let arguments =
        App::new("Heated Words")
            .arg(Arg::with_name("QUERY MODE")
                .long("query")
                .value_name("QUERY")
                .help("The phrase that you want to search online")
            )
            .arg(Arg::with_name("NO ENGINE MODE")
                .long("no-engines")
                .value_name("URL_LIST_FILE")
                .help("Rather than running a query, specify each URL you want to scrape. \
                Please note that all link quality checks will be ignored")
            )
            .group(ArgGroup::with_name("RUN MODES")
                .required(true)
                .args(&["QUERY MODE", "NO ENGINE MODE"])
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
                for it to be valid. Can be used multiple times!")
            )
            .arg(Arg::with_name("exclude words")
                .multiple(true)
                .short("e")
                .long("exclude")
                .value_name("WORD")
                .help("Specify a word which must NOT exist in the URL (minus domain) \
                for it to be valid. Can be used multiple times!")
            )
            .arg(Arg::with_name("number of links")
                .short("n")
                .long("link-number")
                .value_name("MAX")
                .help("Specify the maximum number of links each Engine will return. \
                -1 is unlimited.")
            )
            .arg(Arg::with_name("word bypass limit")
                .long("bypass-limit")
                .value_name("COUNT")
                .help("Specify the limit of words in the URL (minus domain) \
                that can cause a QA bypass")
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

    // loading configuration file
    let config_filename = arguments
        .value_of("configuration file")
        .unwrap_or("config.toml");
    println!("###  Loading configuration file: {}", config_filename);
    configuration::load_configuration_file(config_filename);

    // turning CLI arguments into "Config" object
    println!("###  Building configuration from arguments");
    let mut arg_config = Config::new();

    let search_query = arguments.value_of("QUERY MODE").unwrap_or("");
    arg_config.set("query", search_query).unwrap();

    let url_list_filename = arguments.value_of("NO ENGINE MODE").unwrap_or("");

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
                Ok(count) => {
                    arg_config.set("sensitivity.max_links", count).unwrap();
                }
                Err(e) => println!("!!! Match Ratio threshold is an invalid integer.\
                    Program will fall back to config file: {}", e)
            }
        }
        None => ()
    }
    match arguments.value_of("word bypass limit") {
        Some(value) => {
            match value.parse::<f64>() {
                Ok(count) => {
                    arg_config.set("sensitivity.word_bypass_limit", count).unwrap();
                }
                Err(e) => println!("!!! Match Ratio threshold is an invalid float.\
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
                Err(e) => println!("!!!  Match Ratio threshold is an invalid float.\
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

    // joining arguments "Config" object with the config file. The arguments override the file
    println!("###  Joining arguments to config file for global access");
    match configuration::CONFIGURATION.write() {
        Ok(mut config) => {
            match config.merge(arg_config) {
                Ok(_) => (),
                Err(e) => panic!("{}", e)
            }
        }
        Err(e) => panic!("Couldn't open configuration to write: {}", e)
    }


    // create Vector with thread safety to hold list of URLs to retrieve
    let complete_link_list = Arc::new(Mutex::new(Vec::new()));

    //run QUERY mode if specified
    if arguments.is_present("QUERY MODE") {
        // debug print query
        println!("###  Running program with query: \"{}\"", search_query);

        //build search engines from config file that we loaded earlier
        let engines = engine::build_engines();

        println!("###  Using the following search engines: ");
        for engine in &engines {
            println!("~~~   {}", &engine);
        }


        // create Vector of engine threads to join later
        let mut running_engines = Vec::new();

        // spawn new threads for each engine and give them reference to shared complete_link_list
        println!("###  Dispatching engines...");
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

        // wait for all engines to finish running
        for running in running_engines {
            match running.join() {
                Ok(_) => continue,
                Err(_) => panic!("!!!  Failed to join thread running an Engine")
            }
        }
    } else if arguments.is_present("NO ENGINE MODE") {  // If instead of QUERY mode just use link file...
        println!("###  Running in no-engine mode with list: {}", url_list_filename);

        // get the shared list
        let shared_list = Arc::clone(&complete_link_list);
        let mut link_vector = shared_list.lock().unwrap();

        // and copy the URLs from the file into the shared list
        let input_file = match File::open(url_list_filename) {
            Ok(file) => file,
            Err(e) => panic!("File cannot be opened: {}", e)
        };
        let url_reader = BufReader::new(input_file);
        for line in url_reader.lines() {
            let url = line.unwrap();
            link_vector.push(url);
        }
    } else {
        panic!("NO RUNNING MODE SPECIFIED"); //this should be unreachable, but just to make sure :)
    }


    //then gives the resulting list to the Spider (Fetcher manager)
    match complete_link_list.to_owned().lock() {
        Ok(list) => {
            println!("$$$  Collected {} links in total", list.len());
            if configuration::read_debug() {
                for link in list.iter() {
                    println!("L: {}", link);
                }
            }

            // create Spider with a list of URLs which it needs to retrieve
            let mut spider = spider::Spider::new(list.to_vec());
            println!("###  Dispatching spider...");
            // dispatch hands execution of program to spider which will eventually return a Vec<String> of each word the fetchers retrieved
            let mut results: Vec<String> = spider.dispatch();


            println!("###  Finalizing results");
            println!("~~~  finalizing link list of length {}", list.to_vec().len());
            finalizer::finish_link_vector(list.to_vec());
            println!("~~~  finalizing wordlist of length {}", results.len());
            finalizer::finish_wordlist(&mut results);

            println!("###  Executing post-processing");
            finalizer::run_post_processing();  // Run scripts/commands/etc as specified in configuration file

            println!("--=<{{[[[  HEATED WORDS COMPLETED  ]]]}}>=--");
        }
        Err(e) => panic!("{}", e)
    }
}
