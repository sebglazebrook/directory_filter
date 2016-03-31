use directory_scanner::{Directory, File};
use std::sync::{Arc, Mutex};
use regex::Regex;
use directory_filter::matchers::*;

#[derive(Clone)]
pub struct FilteredDirectory {
    directory: Arc<Mutex<Directory>>,
    regex: Regex,
    pub file_matches: Vec<File>,
}

impl FilteredDirectory {

    pub fn new(directory: Arc<Mutex<Directory>>, regex: Regex) -> Self {
      FilteredDirectory {
           directory: directory,
           regex: regex,
           file_matches: vec![],
      }
    }

    pub fn len(&self) -> usize {
        self.file_matches.len()
    }

    pub fn run_filter(&mut self) {
        if self.regex.to_string() == "" {
            self.file_matches = self.directory.lock().unwrap().file_contents();
        } else {
            self.file_matches = find_matches(&self.directory, self.regex.clone());
            info!("Filter found {} matches", self.len());
        }
    }

    pub fn re_filter(&mut self, new_regex: Regex) {
        if self.new_regex_is_addative(&new_regex) {
            self.file_matches = find_file_matches(&self.file_matches, new_regex.clone());
            self.regex = new_regex.clone();
        } else {
            self.regex = new_regex.clone();
            self.run_filter();
        }
    }

    //---------- private ---------//

    fn new_regex_is_addative(&self, new_regex: &Regex) -> bool {
        if self.regex.as_str() == "" {
            return false;
        }
        new_regex.as_str().starts_with(self.regex.as_str())
    }

    // TODO  implement trait std::iter::Iterator
    // TODO implement eq trait for this one
}
