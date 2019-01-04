use super::configuration;

use config::Value;

use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;

const CHARACTER_LIMIT_MAX: usize = 12;//inclusive
const CHARACTER_LIMIT_MIN: usize = 6; //inclusive

pub fn finish_link_vector(link_vector: Vec<String>) {
    println!("!!! - finalizing link list of length {}", link_vector.len());
    match configuration::CONFIGURATION.read() {
        Ok(config) => {
            match config.get_str("filenames.links") {
                Ok(value) => {
                    match File::create(&value) {
                        Ok(mut file) => {
                            for link in link_vector {
                                match write!(file, "{}\n", link) {
                                    Ok(_) => (),
                                    Err(e) => println!("!!! - Could not write to file: {}", e)
                                }
                            }
                        }
                        Err(e) => println!("!!! - Could not create/write to file \"{}\": {}", value, e)
                    }
                }
                Err(_) => println!("!!! - The \"links\" key is missing from the \"filenames\" table in config.toml.\
                The link vector won't be processed into a file.")
            }
        }
        Err(e) => println!("!!! - Could not read config and finalize link vector handling: {}", e)
    }
}

pub fn finish_wordlist(end_list: &mut Vec<String>) {
    println!("!!! - finalizing wordlist of length {}", end_list.len());
    end_list.retain(|s| s.len() <= CHARACTER_LIMIT_MAX && s.len() >= CHARACTER_LIMIT_MIN);

    // dedup
    for start in 0..end_list.len() {
        let mut end = start;
        while end < end_list.len() { //use a while loop rather than for since removing changes index
            if start != end && end_list[start] == end_list[end] {
                end_list.remove(end);
            } else {
                end += 1;
            }
        }
    }

    println!("$$$ - wordlist was finalized to length {}", end_list.len());

    //then write to file
    match configuration::CONFIGURATION.read() {
        Ok(config) => {
            match config.get_str("filenames.wordlist") {
                Ok(value) => {
                    match File::create(&value) {
                        Ok(mut file) => {
                            for word in end_list {
                                match write!(file, "{}\n", word) {
                                    Ok(_) => (),
                                    Err(e) => println!("!!! - Could not write to file: {}", e)
                                }
                            }
                        }
                        Err(e) => println!("!!! - Could not create/write to file \"{}\": {}", value, e)
                    }
                }
                Err(_) => println!("The \"wordlist\" key is missing from the \"filenames\" table in config.toml.\
                The word list won't be processed into a file... which kinda defeats the purpose of running this program")
            }
        }
        Err(e) => println!("Could not read config and finalize link vector handling: {}", e)
    }
}

pub fn run_post_processing() {
    match configuration::CONFIGURATION.read() {
        Ok(config) => {
            match config.get_table("post-processing") {
                Ok(table_list) => {
                    for table
                        in
                        table_list
                            .values()
                            .map(|table| table.clone().into_table().unwrap()) {
                        if configuration::read_debug() {
                            println!("{:?}", table);
                        }

                        match run_process(table) {
                            Ok(output) => {
                                println!("{}", output.trim())
                            }
                            Err(e) => println!("{}", e)
                        }
                    }
                }
                Err(_) => println!("!!! - Config Table \"post-processing\" is missing/malformed and no post-processing will occur")
            }
        }
        Err(e) => println!("!!! - Could not read config for post-processing: {}", e)
    }
}

fn run_process(table: HashMap<String, Value, RandomState>) -> Result<String, String> {
    match table.get("description") {
        Some(desc) => println!("### - Running post-process: {}", desc),
        None => println!("!!! - Error finding \"description\" in config.toml")
    }

    match table.get("command") {
        Some(value) => match value.to_owned().into_str() {
            Ok(cmd) => {
                //build command
                let mut command = Command::new(cmd);

                //add any args (if found)
                match table.get("args") {
                    Some(value) => match value.to_owned().into_array() {
                        Ok(args) => {
                            for arg in args {
                                match arg.to_owned().into_str() {
                                    Ok(arg_str) => command.arg(arg_str),
                                    Err(e) => return Err(format!("!!! - arg {} could not be converted into a string: {}", arg, e))
                                };
                            }
                        }
                        Err(e) => return Err(format!("!!! - \"args\" exists but its value is malformed (it should be an array): {}", e))
                    }
                    None => return Err(format!("!!! - \"args\" key not found in table"))
                }

                //and run
                match command.output() {
                    Ok(output) => {
                        match String::from_utf8(output.stdout) {
                            Ok(text) => {
                                Ok(text)
                            }
                            Err(e) => Err(format!("!!! - Post-process isn't outputting valid UTF-8 text: {}", e))
                        }
                    }
                    Err(e) => Err(format!("!!! - Post-process failed to execute: {}", e))
                }
            }
            Err(e) => Err(format!("!!! - \"command\" exists but its value is malformed: {}", e))
        }
        None => Err(format!("!!! - \"command\" key not found in table"))
    }
}