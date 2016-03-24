//use regex::Regex;
//use directory_scanner::Directory;

//use directory_filter::{FilteredDirectory, RegexBuilder};
//use directory_filter::matchers::*;

//pub struct SimpleFilter<'a> {
    //directory: &'a Directory,
    //regex: Regex,
//}

//impl<'a> SimpleFilter<'a> {

    //pub fn new(directory: &'a Directory, filter_string: &'a str) -> Self {
        //SimpleFilter {
            //directory: directory,
            //regex: RegexBuilder::new(filter_string.to_string()).build(),
        //}
    //}

    //pub fn execute(&self) -> FilteredDirectory {
        //FilteredDirectory {
           //matches: find_matches(self.directory, self.regex.clone()),
           //directory: Arc::new(Mutex::new(self.directory)),
        //}
    //}
//}
