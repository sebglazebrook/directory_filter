use directory_scanner::{Directory, File};
use std::sync::{Arc, Mutex};
use regex::Regex;
use directory_filter::matchers::*;

#[derive(Clone)]
pub struct FilteredDirectory<'a> {
    directory: Arc<Mutex<Directory>>,
    pub matches: Vec<String>, // TODO this should be a collection of references/pointers to paths in the directory
    regex: Regex, // TODO gonna need to store the filterstring/regex here
    pub match_references: Vec<&'a File>, // TODO make this actually work
}

impl<'a> FilteredDirectory<'a> {

    pub fn new(directory: Arc<Mutex<Directory>>, regex: Regex) -> Self {
      FilteredDirectory {
           matches: vec![],
           directory: directory,
           regex: regex,
           match_references: vec![],
      }
    }

    pub fn len(&self) -> usize {
        self.matches.len()
    }

    pub fn run_filter(&mut self) {
        // TODO don't find matches if regex is empty/match all
        let new_matches = find_matches(&self.directory, self.regex.clone()); // TODO don't return matches update the FilteredDirectory??
        self.matches = new_matches;
        info!("Filter found {} matches", self.len());
    }

    // TODO  implement trait std::iter::Iterator
    // TODO implement eq trait for this one
}
