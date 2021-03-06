use directory_scanner::{Directory, File};
use regex::Regex;
use directory_filter::matchers::*;

#[derive(Clone)]
pub struct FilteredDirectory {
    directory: Directory,
    regex: Regex,
    pub file_matches: Vec<File>,
}

impl FilteredDirectory {

    pub fn new(directory: Directory, regex: Regex) -> Self {
      FilteredDirectory {
           directory: directory,
           regex: regex,
           file_matches: vec![],
      }
    }

    pub fn len(&self) -> usize {
        self.file_matches.len()
    }

    pub fn total_len(&self) -> usize {
        self.directory.len()
    }

    pub fn run_filter(&mut self) {
        info!("Running filter with {:?}", self.regex);
        if self.regex.to_string() == "" || self.regex.to_string() == ".*" {
            info!("Match all filter found, returning all files");
            self.file_matches = self.directory.file_contents();
        } else {
            self.file_matches = find_matches(&self.directory, self.regex.clone());
            info!("Filter found {} matches", self.len());
        }
    }

    pub fn re_filter(&mut self, new_regex: Regex) {
        if self.new_regex_is_addative(&new_regex) {
            info!("Additive filter found, filtering only on previous matches");
            self.file_matches = find_file_matches(&self.file_matches, new_regex.clone());
            self.regex = new_regex.clone();
        } else {
            info!("Non addative filter found, filtering over all files");
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

    // TODO implement eq trait for this one
}

impl IntoIterator for FilteredDirectory {
    type Item = File;
    type IntoIter = FilteredDirectoryIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        FilteredDirectoryIntoIterator { filtered_directory: self, index: 0 }
    }

}

pub struct FilteredDirectoryIntoIterator  {
    filtered_directory: FilteredDirectory,
    index: usize,
}

impl Iterator for FilteredDirectoryIntoIterator {
    type Item = File;

    fn next(&mut self) -> Option<File> {
        match self.filtered_directory.file_matches.get(self.index) {
            Some(result) => {
                self.index += 1;
                Some(result.clone())
            },
            None => None
        }
    }
}
