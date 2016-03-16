use regex::Regex;
use directory_scanner::Directory;

use directory_filter::FilteredDirectory;
use directory_filter::matchers::*;

pub struct SimpleFilter<'a> {
    directory: &'a Directory,
    regex: Regex,
}

impl<'a> SimpleFilter<'a> {

    pub fn new(directory: &'a Directory, filter_string: &'a str) -> Self {
        let regex = Regex::new(filter_string).unwrap();
        SimpleFilter {
            directory: directory,
            regex: regex,
        }
    }

    pub fn execute(&self) -> FilteredDirectory {
        FilteredDirectory {
           matches: find_matches(self.directory, self.regex.clone()),
           directory: self.directory,
        }
    }
}
