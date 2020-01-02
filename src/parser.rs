use regex::Regex;

pub fn parse(response_text: &mut String) -> (Vec<String>, Vec<String>) {

    //removes junk text
    remove_scripts(response_text); // remove JavaScript elements
    remove_style(response_text); // remove CSS
    remove_html_nodes(response_text); // remove HTML nodes
    remove_html_text(response_text); // remove HTML encoded text (e.g. "&nbsp;")
    remove_numbers(response_text); // remove any numbers

    let critical = gather_critical_words(response_text);
    let extracted = extract_words(response_text);

    //return tuple
    (critical, extracted)
}

// 'critical words' are things like pronouns or important nouns that are likely to be passwords
fn gather_critical_words(response_text: &String) -> Vec<String> {
    let cap_rex = Regex::new(r"(?P<cap>(?:[A-Z][\w]*)(?:[ \-]?[A-Z][\w]*)*)").unwrap();

    let mut result: Vec<String> = Vec::new();

    for capture in cap_rex.captures_iter(response_text) {
        //for every capture (many contain multiple words), pull out the words
        let word_rex = Regex::new(r"[a-zA-Z]+").unwrap();
        //collect them into a vector
        let word_vector: Vec<String> = word_rex.captures_iter(&capture[0])
            .map(|cap| cap.get(0).unwrap().as_str().to_string())
            .collect();

        //and find the powerset of each (all variations)
        for word in word_vector {
            let split_word = split_string_to_vector(&word);
            let word_variations = get_powerset_of_vector(&split_word);

            for variant in word_variations {
                result.push(variant);
            }
        }
    }

    result
}

fn split_string_to_vector(string: &String) -> Vec<String> {
    string.split_whitespace().map(|val| val.to_string()).collect()
}

fn get_powerset_of_vector(vector: &Vec<String>) -> Vec<String> {
    let length = vector.len();
    let mut result = Vec::new();

    for setnum in 0..1<<length { // bit shift for ^2 of the length to get correct size of powerset
        // create the list for the individual powerset
        let mut set = Vec::new();
        // build each possible element
        for index in 0..setnum {
            if setnum & (1<<index) > 0 {
                set.push(vector[index].clone());
            }
        }
        // flatten the powerset and join it into the vector of all powersets
        let mut flattened_set = String::new();
        for value in set {
            flattened_set = format!("{} {}", flattened_set, value);
        }
        result.push(flattened_set.trim().to_string());
    }

    // return the list of powerset elements
    result
}

fn extract_words(text: &String) -> Vec<String> {
    let rex = Regex::new(r"\w+").unwrap();
    let result: Vec<String> = rex.captures_iter(text)
        .filter(|cap| cap.get(0).unwrap().as_str().len() > 4)
        .map(|cap| cap.get(0).unwrap().as_str().to_string())
        .collect();
    result
}

fn remove_scripts(text: &mut String) {
    let rex = Regex::new(r"<script[\s\S]*?>[\s\S]*?</script>").unwrap();
    *text = rex.replace_all(text, "").to_string();
}

fn remove_style(text: &mut String) {
    let rex = Regex::new(r"<style[\s\S]*?>[\s\S]*?</style>").unwrap();
    *text = rex.replace_all(text, "").to_string();
}

fn remove_html_nodes(text: &mut String) {
    let rex = Regex::new(r"<[\s\S]*?>").unwrap();
    *text = rex.replace_all(text, "").to_string();
}

fn remove_html_text(text: &mut String) {
    // replace non-breaking spaces with actual spaces
    *text = text.replace("&nbsp;", " ");

    // just remove all other html characters
    let rex = Regex::new(r"&.*?;").unwrap();
    *text = rex.replace_all(text, "").to_string();
}

fn remove_numbers(text: &mut String) {
    let rex = Regex::new(r"\d+(?:\.\d+)?").unwrap();
    *text = rex.replace_all(text, "").to_string();
}