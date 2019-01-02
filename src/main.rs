#[macro_use]
//#[allow(dead_code)]
//#[allow(unused_variables)]

extern crate lazy_static;
extern crate config;
extern crate regex;
extern crate reqwest;
extern crate url;

pub mod configuration;
mod engine;
mod spider;
mod fetcher;
mod parser;
mod finalizer;

use std::env;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    //read configuration file into the Config
    configuration::load_configuration_file();

    //handle arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("You must specify a search query as an argument (e.g. \"College football teams\").");
    }

    // handle CLI arguments
    let query = &args[1];
    println!("### - Running program with query: \"{}\"", query);

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
        let qry = query.clone();
        let shared_list = Arc::clone(&complete_link_list);
        running_engines.push(thread::spawn(move || {
            match engine.dispatch(qry.as_str()) {
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
            finalizer::finish_link_vector(list.to_vec()); //handles how the link vector should be handled
            finalizer::finish_wordlist(&mut results); //handles cleanup, final parsing, and post-processing

            println!("### - Executing post-processing...");
            finalizer::run_post_processing();

            println!("-=<|[[[ HEATED WORDS COMPLETED ]]]|>=-");
        },
        Err(e) => panic!("{}", e)
    }


}
