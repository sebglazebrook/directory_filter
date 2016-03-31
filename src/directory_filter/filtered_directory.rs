use directory_scanner::{Directory, File};
use std::sync::{Arc, Mutex};
use regex::Regex;
use directory_filter::matchers::*;

#[derive(Clone)]
pub struct FilteredDirectory<'a> {
    directory: Arc<Mutex<Directory>>,
    regex: Regex, // TODO gonna need to store the filterstring/regex here
    pub match_references: Vec<&'a File>, // TODO make this actually work
    pub file_matches: Vec<File>,
}

impl<'a> FilteredDirectory<'a> {

    pub fn new(directory: Arc<Mutex<Directory>>, regex: Regex) -> Self {
      FilteredDirectory {
           directory: directory,
           regex: regex,
           match_references: vec![],
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
            self.file_matches = find_matches(&self.directory, self.regex.clone()); // TODO don't return matches update the FilteredDirectory??
            info!("Filter found {} matches", self.len());
        }
    }

    // TODO  implement trait std::iter::Iterator
    // TODO implement eq trait for this one
}
