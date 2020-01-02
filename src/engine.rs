extern crate regex;
extern crate url;
extern crate reqwest;

use std::fmt;
use regex::Regex;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use url::Url;

use super::configuration;

#[derive(Clone, Debug)]
pub struct Engine {
    base_url: Url,
    link_regex: Regex
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
                contents_pattern.as_str()
            ).unwrap(),
        }
    }

    pub fn dispatch(&self) -> Result<Vec<String>, String> {
        let query_as_string = match configuration::CONFIGURATION.read().unwrap().get_str("query") {
            Ok(result) => result.clone(),
            Err(e) => panic!("Could not read query from configuration: {}", e)
        };
        let query = query_as_string.as_str();

        println!("###  {} dispatched with query: {}", self, query);
        let encoded_query = utf8_percent_encode(query, DEFAULT_ENCODE_SET).to_string();
        let url = self.base_url.as_str().replace("{}", encoded_query.as_str());

        match self.generate_links(url.as_str()) {
            Ok(mut links) => {
                if configuration::read_debug() {
                    println!("{} full url = {:?}", self, url);
                    println!("{} extracted the following links with the regex \"{}\":", self, self.link_regex);
                    for link in &links {
                        println!("{}", link);
                    }
                }

                filter_links(&mut links, query);
                let max_links = match configuration::CONFIGURATION.read() {
                    Ok(config) => {
                        match config.get_int("sensitivity.max_links") {
                            Ok(max) =>
                                if max != -1 && max as usize <= links.len() {
                                    println!("###  {} received {} links but will only return the first {}", self, links.len(), max);
                                    max as usize
                                } else {
                                    links.len()
                                },
                            Err(_) => links.len()
                        }
                    }
                    Err(_) => links.len()
                };

                Ok(links[0..max_links].to_vec())
            }
            Err(e) => Err(e)
        }
    }

    //runs actual search via search engine
    fn generate_links(&self, url: &str) -> Result<Vec<String>, String> {

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
                Err(e) => panic!("Could not create Engine Client: {}", e)
            };
        
        if configuration::read_debug() {
            println!("~~~  {} has finished building the request client", self);
        }

        // actually make the request
        let response = match request_client.get(url).send() {
            Ok(resp) => resp,
            Err(e) => return Err(format!("!!!  {} failed to make request due to error: {}", self, e))
        };

        // check if the status code is a 2XX
        if response.status() != reqwest::StatusCode::OK {
            return Err(format!("!!!  {} made request but received status code '{}'", self, response.status()))
        }

        if configuration::read_debug() {
            println!("~~~  {} has finished making the request to '{}'", self, url);
        }

        // get the response text (html webpage)
        let response_text = match response.text() {
            Ok(text) => text,
            Err(e) => return Err(format!("!!!  {} could not retrieve text from response due to error: {}", self, e))
        };

        if configuration::read_debug() {
            println!("~~~  {} has received the response text", self);
            println!("\n\n\n{}\n\n\n", response_text);
        }

        // extract the links to each page from the HTML
        Ok(self.extract_links(response_text))  // return the vector of absolute URLs
    }

    //returns a vector of the URLs
    fn extract_links(&self, search_results: String) -> Vec<String> {
        let links =
            self.link_regex
                .captures_iter(search_results.as_str())
                .map(|cap| match self.base_url.join(cap.name("link").unwrap().as_str()) { //try joining the link with the base_url
                    Ok(full_url) => {
                        if configuration::read_debug() {
                            println!("RL: {}", cap.name("link").unwrap().as_str());
                        }
                        full_url.to_string()
                    } //if sucessful, the link was relative
                    Err(_) => cap.name("link").unwrap().as_str().to_string(), //otherwise its absolute and can be returned
                })
                .collect();

        links
    }
}

fn filter_links(links: &mut Vec<String>, query: &str) {
    let page_rex = Regex::new(r"https?://[^/]+/(?P<page>.+)$").unwrap();
    let alphanumeric_rex = Regex::new(r"[a-zA-Z0-9]+").unwrap();

    let min_match_threshold = match
        configuration::CONFIGURATION
            .read()
            .unwrap()
            .get_float("sensitivity.match_threshold") {
        Ok(value) => value,
        Err(_) => {
            println!("!!!  Could not read value of \"sensitivity.match_threshhold\". Defaulting to 0.00");
            0.00
        }
    };
    let max_extra_threshold = match
        configuration::CONFIGURATION
            .read()
            .unwrap()
            .get_float("sensitivity.extra_threshold") {
        Ok(value) => value,
        Err(_) => {
            println!("!!!  Could not read value of \"sensitivity.extra_threshold\". Defaulting to 1.00");
            1.00
        }
    };
    let required_words = match
        configuration::CONFIGURATION
            .read()
            .unwrap()
            .get_array("required_words") {
        Ok(value) => value,
        Err(_) => Vec::new()
    };
    let excluded_words = match
        configuration::CONFIGURATION
            .read()
            .unwrap()
            .get_array("exclude words") {
        Ok(value) => value,
        Err(_) => Vec::new()
    };
    let ignored_patterns_list = match
        configuration::CONFIGURATION
            .read()
            .unwrap()
            .get_array("sensitivity.ignore_link_patterns") {
        Ok(array) => array.iter().map(|value| {
            let value_copy = value.clone();
            match value_copy.into_str() {
                Ok(string) => {
                    match Regex::new(string.as_str()) {
                        Ok(regex) => regex,
                        Err(e) => {
                            println!("!!!  ignore_link_pattern isn't valid regex: {}", e);
                            Regex::new("").unwrap()
                        }
                    }
                }
                Err(e) => {
                    println!("!!!  ignore_link_pattern isn't a valid string: {}", e);
                    Regex::new("").unwrap()
                }
            }
        }).collect(),
        Err(_) => {
            println!("!!!  The key \"ignore_link_patterns\" doesn't exist in the sensitivity table. Ignoring...");
            Vec::new()
        }
    };


    links.retain(|link| {
        if configuration::read_debug() {
            println!("###  Original link {:?} before ignoring patterns", link);
        }

        let query_word_count = alphanumeric_rex.captures_iter(query).count();
        let path_word_count = match page_rex.captures(link.as_str()) {
            Some(page_cap) => {
                let mut path = page_cap.name("page").unwrap().as_str().to_string();

                //filter out ignored words before counting them
                for pattern in &ignored_patterns_list {
                    path = pattern.replace_all(path.as_str(), "").to_string();
                }

                alphanumeric_rex.captures_iter(path.as_str()).count()
            }
            None => 0
        };

        //check to see if the counts should cause a bypass
        //bypass if the page doesn't have a path
        if path_word_count == 0 {
            if configuration::read_debug() {
                println!("###  {} bypassed QA due to not having a path after ignored patterns"
                         , link);
            }
            return true; //assume if the page doesn't have a name its a dedicate site (good)
        }

        let bypass_limit =
            match configuration::CONFIGURATION.read()
                .unwrap()
                .get_int("sensitivity.word_bypass_limit") {
                Ok(value) => value as usize,
                Err(_) => {
                    println!("!!!  No word bypass limit specified, defaulting to 0");
                    0
                }
            };
        if configuration::read_debug() {
            println!("### -Using word bypass limit of: {}", bypass_limit);
        }
        //check if link should be bypassed due to low word count
        if path_word_count <= bypass_limit && path_word_count < query_word_count {
            if configuration::read_debug() {
                println!("### -{:?} bypassed QA due to the word requirement after ignored patterns"
                         , link);
            }
            return true;
        }

        //since length was checked beforehand, we guarantee that unwrapping it is safe
        let link_path = page_rex.captures(link.as_str()).unwrap().name("page").unwrap().as_str();

        //run the ignore again on just the path for the domain
        let mut path_after_ignore: String = link_path.clone().to_string();
        for pattern in &ignored_patterns_list {
            path_after_ignore =
                pattern
                    .replace_all(path_after_ignore.as_str(), "")
                    .into_owned();
        }

        if configuration::read_debug() {
            println!("--- PERFORMING QA CHECK ON LINK ---");
            println!("Link text: {:?}", link);
            println!("Actual Path: {:?}", link_path);
            println!("Ignored patterns: {:?}", ignored_patterns_list);
            println!("Path after ignored patterns: {:?}", path_after_ignore);
            //TODO clean this up so the regex isn't done twice. Keep the arrays and then use them for the check
            println!("Words in query: {:?}",
                     alphanumeric_rex
                         .captures_iter(query)
                         .map(|cap| cap.get(0).unwrap().as_str().to_string())
                         .collect::<Vec<String>>());
            println!("Words in path: {:?}",
                     alphanumeric_rex
                         .captures_iter(path_after_ignore.as_str())
                         .map(|cap| cap.get(0).unwrap().as_str().to_string())
                         .collect::<Vec<String>>());
            println!("Required words: {:?}", required_words);

        }

        //make sure link contains required words
        for required in required_words.to_owned() {
            match required.into_str() {
                Ok(string) => {
                    //if required word isn't in link
                    if !path_after_ignore.to_lowercase().contains(string.to_lowercase().as_str()) {
                        return false; //returns to retains method call up top
                    }
                }
                Err(e) => println!("!!!  malformed required word isn't a string: {}", e)
            }
        }

        //make sure link doesn't include excluded words
        for excluded in excluded_words.to_owned() {
            match excluded.into_str() {
                Ok(string) => {
                    //if required word isn't in link
                    if link_path.to_lowercase().as_str().contains(string.to_lowercase().as_str()) {
                        return false; //returns to retains method call up top
                    }
                }
                Err(e) => println!("!!!  malformed excluded word isn't a string: {}", e)
            }
        }

        let mut match_counter: usize = 0;
        let mut not_match_counter: usize = 0;

        'outer: for query_cap in alphanumeric_rex.captures_iter(query) {
            let query_word = query_cap.get(0).unwrap().as_str();

            'inner: for link_cap in alphanumeric_rex.captures_iter(link_path) {
                let link_cap_word = link_cap.get(0).unwrap().as_str();

                if query_word.eq_ignore_ascii_case(link_cap_word) {
                    match_counter += 1;
                    continue 'outer;
                }
            }
            not_match_counter += 1;
        }

        let match_percent = if not_match_counter != 0 {
            match_counter as f64 / query_word_count as f64
        } else {
            1.00 as f64
        };
        let extra_percent = (path_word_count - match_counter) as f64 / path_word_count as f64;

        if configuration::read_debug() {
            println!("Words in query: {}", query_word_count);
            println!("Words in path: {}", path_word_count);
            println!("Matched words: {}", match_counter);
            println!("Unmatched words: {}", not_match_counter);
            println!("Ratio of match: {:.2} (min allowed is {:.2})", match_percent, min_match_threshold);
            println!("Ratio of extra words: {:.2} (max allowed is {:.2})", extra_percent, max_extra_threshold);
        }

        if match_percent >= min_match_threshold && extra_percent <= max_extra_threshold {
            if configuration::read_debug() {
                println!("Verdict: PASS  {:?}", link);
            }
            true
        } else {
            if configuration::read_debug() {
                println!("Verdict: FAIL  {:?}", link);
            }
            false
        }
    });
}

pub fn build_engines() -> Vec<Engine> {
    let mut engines: Vec<Engine> = Vec::new();

    match configuration::CONFIGURATION.read() {
        Ok(config) => {
            match config.get_table("engines") {
                Ok(table_list) => {
                    for table
                        in
                        table_list
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
                Err(_) => panic!("!!!  The \"engines\" table and its relevant sub-tables are missing from the configuration.")
            }
        }
        Err(e) => panic!("!!!  Configuration could not be read : {}", e)
    }

    println!("$$$  Successfully built {} search Engines", engines.len());

    engines
}