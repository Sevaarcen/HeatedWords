use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

pub fn search(engines: &Vec<String>, query: &str) -> Vec<Result<(String, String), String>> {

    println!("Received query: {}", query);
    let encoded_query = utf8_percent_encode(query, DEFAULT_ENCODE_SET).to_string();
    println!("Encoded query: {}", encoded_query);

    let mut responses: Vec<Result<(String, String), String>> = Vec::new();
    for engine in engines {
        let query_url = engine.replace("{}", encoded_query.as_str());
        responses.push(make_request(query_url.as_str()));
    }

    responses
}

pub fn make_request(url: &str) -> Result<(String, String), String> {

    match reqwest::get(url) {
        Ok(mut response) => {
            if response.status()  == reqwest::StatusCode::OK {
                match response.text() {
                    Ok(text) => Ok((url.to_string(), text)),
                    Err(e) => Err(e.to_string())
                }
            }
            else{
                Err(format!("Received status code {}: {}", response.status(), url))
            }
        },
        Err(e) => Err(e.to_string())
    }
}