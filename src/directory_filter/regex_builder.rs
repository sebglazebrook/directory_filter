use regex::Regex;

pub struct RegexBuilder {
    string: String,
}

impl RegexBuilder {

    pub fn new(string: String) -> Self {
        RegexBuilder { string: string }
    }

    pub fn build(&self) -> Regex {
        let mut new_string = self.string.chars().fold(String::new(), |mut acc, character|{
            acc.push_str(self.global_flag());
            acc.push_str(".*");
            acc.push(character);
            acc
        });
        new_string.push_str(".*");
        Regex::new(&new_string).unwrap()
    }

    //----------- private -----------//

    fn global_flag(&self) -> &'static str {
        let mut prefix = "(?i)";
        if Regex::new("[A-Z]+").unwrap().is_match(&self.string) {
            prefix = "";
        }
        prefix
    }

}
