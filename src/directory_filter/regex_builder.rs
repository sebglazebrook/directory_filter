use regex::Regex;

pub struct RegexBuilder {
    string: String,
}

impl RegexBuilder {

    pub fn new(string: String) -> Self {
        RegexBuilder { string: string }
    }

    pub fn build(&self) -> Regex {
        let mut new_string = String::new();
        for character in self.string.chars() {
            new_string.push_str(".*");
            new_string.push(character);
        }
        new_string.push_str(".*");
        Regex::new(&new_string).unwrap()
    }

}
