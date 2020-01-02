use super::configuration;

use config::Value;

use std::fs;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::process::Command;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;

pub fn finish_link_vector(link_vector: Vec<String>) {
    match configuration::CONFIGURATION.read() {
        Ok(config) => {
            match config.get_str("filenames.links") {
                Ok(value) => {
                    match File::create(&value) {
                        Ok(mut file) => {
                            for link in link_vector {
                                match write!(file, "{}\n", link) {
                                    Ok(_) => (),
                                    Err(e) => println!("!!!  Could not write to file: {}", e)
                                }
                            }
                        }
                        Err(e) => println!("!!!  Could not create/write to file \"{}\": {}", value, e)
                    }
                }
                Err(_) => println!("!!!  The \"links\" key is missing from the \"filenames\" table in config.toml.\
                The link vector won't be processed into a file.")
            }
        }
        Err(e) => println!("!!!  Could not read config and finalize link vector handling: {}", e)
    }
}

pub fn finish_wordlist(end_list: &mut Vec<String>) {
    let minimum_word_length
        = match configuration::CONFIGURATION.read().unwrap().get_int("minimum_word_length") {
        Ok(value) => value as usize,
        Err(_) => {
            println!("!!!  \"minimum_word_length\" key missing from configuration. \
                Defaulting to 6");
            6 as usize
        }
    };
    let maximum_word_length
        = match configuration::CONFIGURATION.read().unwrap().get_int("maximum_word_length") {
        Ok(value) => value as usize,
        Err(_) => {
            println!("!!!  \"maximum_word_length\" key missing from configuration. \
                Defaulting to 12");
            12 as usize
        }
    };

    end_list.retain(|s| s.len() <= maximum_word_length && s.len() >= minimum_word_length);

    println!("###  Deduping list");
    // dedup
    for start in 0..end_list.len() {
        let mut end = start;
        while end < end_list.len() { //use a while loop rather than for since removing changes index
            if start != end && end_list[start].eq_ignore_ascii_case(end_list[end].as_str()) {
                end_list.remove(end);
            } else {
                end += 1;
            }
        }
    }

    if configuration::read_debug() {
        println!("###  Wordlist is now of length: {}", end_list.len());
    }

    println!("###  Removing blacklisted words");
    match fs::read_dir("./blacklists/") {
        Ok(contents) => {
            for entry in contents {
                let path = entry.unwrap().path();
                let filename = path.to_str().unwrap();
                match File::open(filename) {
                    Ok(blacklist_file) => {
                        if configuration::read_debug() {
                            println!("###  Using blacklist from path: {}", filename);
                        }
                        let reader = BufReader::new(blacklist_file);

                        for line in reader.lines() {
                            let entry = line.unwrap();
                            let mut index: usize = 0;

                            while index < end_list.len() {
                                if entry.eq_ignore_ascii_case(end_list[index].as_str()) {
                                    end_list.remove(index);
                                    break; //because the list doesn't have duplicates we can break
                                }
                                index += 1;
                            }
                        }
                    }
                    Err(e) => println!("!!!  Blacklist file could not be opened: {}", e)
                }
            }
        }
        Err(_) => println!("!!!  Directory \"./blacklists/\" is missing")
    }

    println!("$$$  wordlist was finalized to length {}", end_list.len());

    //then write to file
    match configuration::CONFIGURATION.read() {
        Ok(config) => {
            match config.get_str("filenames.wordlist") {
                Ok(value) => {
                    match File::create(&value) {
                        Ok(mut file) => {
                            if configuration::read_debug() {
                                println!("~~~  Writing wordlist of length: {}", end_list.len());
                            }
                            
                            for word in end_list {
                                match write!(file, "{}\n", word) {
                                    Ok(_) => (),
                                    Err(e) => println!("!!!  Could not write to file: {}", e)
                                }
                            }
                        }
                        Err(e) => println!("!!!  Could not create file \"{}\": {}", value, e)
                    }
                }
                Err(_) => println!("The \"wordlist\" key is missing \
                    from the \"filenames\" table in config.toml.\
                    The word list won't be processed into a file... \
                    which kinda defeats the purpose of running this program")
            }
        }
        Err(e) => println!("Could not read config and finalize wordlist handling: {}", e)
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
                Err(_) => println!("!!!  Config Table \"post-processing\" is missing/malformed and no post-processing will occur")
            }
        }
        Err(e) => println!("!!!  Could not read config for post-processing: {}", e)
    }
}

fn run_process(table: HashMap<String, Value, RandomState>) -> Result<String, String> {
    match table.get("description") {
        Some(desc) => println!("###  Running post-process: {}", desc),
        None => println!("!!!  Error finding \"description\" in config.toml")
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
                                    Err(e) => {
                                        println!("!!!  arg {} could not be converted into a string: {}", arg, e);
                                        &mut command
                                    }
                                };
                            }
                        }
                        Err(e) => println!("!!!  \"args\" exists but its value is malformed (it should be an array): {}", e)
                    }
                    None => println!("!!!  \"args\" key not found in table")
                }

                //run and get output, direct to stdout
                match command.output() {
                    Ok(output) => {
                        match String::from_utf8(output.stdout) {
                            Ok(text) => {
                                Ok(text)
                            }
                            Err(e) => Err(format!("!!!  Post-process isn't outputting valid UTF-8 text: {}", e))
                        }
                    }
                    Err(e) => Err(format!("!!!  Post-process failed to execute: {}", e))
                }
            }
            Err(e) => Err(format!("!!!  \"command\" exists but its value is malformed: {}", e))
        }
        None => Err(format!("!!!  \"command\" key not found in table"))
    }
}