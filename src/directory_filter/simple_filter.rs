use regex::Regex;
use std::path::PathBuf;
use directory_scanner::Directory;

use directory_filter::FilteredDirectory;

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
           matches: self.find_matches(self.directory),
           directory: self.directory,
        }
    }

    //----------- private ---------//

    fn find_matches(&self, directory: &Directory) -> Vec<String> {
        let mut matches = vec![];
        if self.is_match(&directory.path) {
            matches.extend(self.directory.contents());
        } else {
            for file in self.directory.files.clone() {
                if self.is_match(&file.path()) {
                    matches.push(file.as_string());
                }
            }
            for directory in self.directory.sub_directories.clone() {
                // TODO use a thread pool or something
                // for directories to make things quicker
                matches.extend(self.find_matches(&directory))
            }
        }
        matches
    }

    fn is_match(&self, path: &PathBuf) -> bool {
        self.regex.is_match(path.to_str().unwrap())
    }

}
