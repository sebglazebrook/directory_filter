extern crate directory_scanner;
extern crate regex;

use directory_scanner::{ScannerBuilder, Directory};
use std::path::PathBuf;
use regex::Regex;

struct FilteredDirectory<'b> {
    directory: &'b Directory,
    matches: Vec<String>, // TODO this should be a collection of references/pointers to paths in the directory
}

impl<'b> FilteredDirectory<'b> {

    pub fn len(&self) -> usize {
        self.matches.len()
    }

}

struct SimpleFilter<'a> {
    directory: &'a Directory,
    filter_string: &'a str,
}

impl<'a> SimpleFilter<'a> {

    pub fn new(directory: &'a Directory, filter_string: &'a str) -> Self {
        SimpleFilter {
            directory: directory,
            filter_string: filter_string
        }
    }

    pub fn execute(&self) -> FilteredDirectory {
        FilteredDirectory {
           matches: self.find_matches(self.directory),
           directory: self.directory,
        }
    }

    //----------- private ---------//

    // TODO make this faster use a thread pool or something
    fn find_matches(&self, directory: &Directory) -> Vec<String> {
        let mut matches = vec![];
        if self.is_match(&directory.path, self.filter_string) {
            matches.extend(self.directory.contents());
        } else {
            for file in self.directory.files.clone() {
                if self.is_match(&file.path(), self.filter_string) {
                    matches.push(file.as_string());
                }
            }
            for directory in self.directory.sub_directories.clone() {
                matches.extend(self.find_matches(&directory))
            }
        }
        matches
    }

    fn is_match(&self, path: &PathBuf, filter_string: &'a str) -> bool {
        let regex = Regex::new(filter_string).unwrap();
        regex.is_match(path.to_str().unwrap())
    }

}

#[test]
fn simple_filtering_example() {
    let mut scanner_builder = ScannerBuilder::new();
    scanner_builder = scanner_builder.start_from_path("test/fixture_dir/");
    scanner_builder = scanner_builder.max_threads(1);
    let directory = scanner_builder.build().scan();

    let filter = SimpleFilter::new(&directory, "fixture_dir");
    let filtered_directory = filter.execute();

    assert_eq!(filtered_directory.len(), 10);
}

//// step 2 - dynamic data that subscribes and publishes events

//let filter = ContinuousFilter::new(event_bus) // subscribes to new directory item event
                                              //// subscribes to filter string change events
                                              //// publishes events when a new directory item matches
                                              //// the filter string

//directory_filter.start();

//// it updates the listeners as new results match the filter string
//// it resets when the filter string is updated

//directory_filter.stop();
