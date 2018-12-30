use regex::Regex;

pub fn parse(response_text: &mut String) -> (Vec<String>, Vec<String>) {

    //removes junk text
    remove_scripts(response_text); //JavaScript elements
    remove_style(response_text); //CSS
    remove_html_nodes(response_text); //HTML nodes
    remove_html_text(response_text); //HTML encoded text (e.g. "&nbsp;")
    remove_numbers(response_text); //any numbers

    let critical = find_critical_words(response_text);
    let extracted = extract_words(response_text);
    (critical, extracted)
}

fn find_critical_words(response_text: &String) -> Vec<String> {
    let cap_rex = Regex::new(r"(?P<cap>(?:[A-Z][\w]*)(?:[ \-]?[A-Z][\w]*)*)").unwrap();

    let mut result : Vec<String> = Vec::new();

    for capture in cap_rex.captures_iter(response_text)
        .filter(|cap| cap.get(0).unwrap().as_str().len() > 4)
        {
            //for every capture (many contain multiple words), pull out the words
            let word_rex = Regex::new(r"[a-zA-Z]+").unwrap();
            //collect them into a vector
            let word_vector = word_rex.captures_iter(&capture[0])
                .map(|cap| cap.get(0).unwrap().as_str().to_string())
                .collect();

            //and find all the slices
            let variations = get_all_slices(&word_vector);

            for variant in variations {
                result.push(variant);
            }
        }

    result
}

fn get_all_slices(vector: &Vec<String>) -> Vec<String> {
    let mut perms: Vec<String> = Vec::new();

    for begin in 0..vector.len() {
        for end in begin..vector.len() { //vector.len()
            let mut perm = String::new();
            for value in &vector[begin..end + 1] { //end+1 so it includes the end
                perm = format!("{}{}", perm, *value)
            }
            perms.push(perm.to_string().clone());
        }
    }

    perms
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
    let rex = Regex::new(r"&.*?;").unwrap();
    *text = rex.replace_all(text, "").to_string();
}

fn remove_numbers(text: &mut String) {
    let rex = Regex::new(r"\d+(?:\.\d+)?").unwrap();
    *text = rex.replace_all(text, "").to_string();
}