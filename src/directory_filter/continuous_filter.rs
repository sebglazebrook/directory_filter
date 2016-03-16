use std::thread;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use directory_scanner::Directory;
use std::path::PathBuf;

use directory_filter::FilteredDirectory;

pub struct ContinuousFilter<'a> {

    directory: &'a Directory,
    filter_change_receiver: Arc<Mutex<Receiver<String>>>,
    new_directory_item_receiver: Arc<Mutex<Receiver<Directory>>>,
    filter_match_transmitter: Sender<FilteredDirectory<'a>>,
    results: FilteredDirectory<'a>,
    regex: Regex,
}

impl<'a> ContinuousFilter<'a> {

    pub fn new(directory: &'a Directory, filter_change_receiver: Receiver<String>, 
               new_directory_item_receiver: Receiver<Directory>, filter_match_transmitter: Sender<FilteredDirectory<'a>>) -> Self {

      let filtered_directory = FilteredDirectory {
           matches: vec![],
           directory: directory,
      };

      ContinuousFilter {
          directory: directory,
          filter_change_receiver: Arc::new(Mutex::new(filter_change_receiver)),
          new_directory_item_receiver: Arc::new(Mutex::new(new_directory_item_receiver)),
          filter_match_transmitter: filter_match_transmitter,
          results: filtered_directory,
          regex: Regex::new("").unwrap(),
      }
    }

    // TODO get this to return the rescan handler
    pub fn start(&mut self) {
        self.listen_for_events();
        self.scan();
    }

    pub fn stop(&self) {
        // stop listening for new events
    }

    //---------- private ----------//

    fn listen_for_events(&self) {
        // listen for filter change events and then kick off scan
        let filter_change_receiver = self.filter_change_receiver.clone();
        let _ = thread::spawn(move || {
            let filter_string = filter_change_receiver.lock().unwrap().recv().unwrap(); // TODO handle this better
            println!("new filter string: {}", filter_string);
            // TODO  send event to rescan
            // loop
        });

        // listen for new directory item events and then kick off scan
        let new_directory_item_receiver = self.new_directory_item_receiver.clone();
        let _ = thread::spawn(move || {
            let new_directory_item = new_directory_item_receiver.lock().unwrap().recv().unwrap(); // TODO handle this better
            println!("new directory item: {:?}", new_directory_item);
        });
    }

    fn scan(&mut self) {
        println!("Scanning");
        self.results.directory = self.directory;
        self.results.matches = self.find_matches(self.directory);
        println!("Scanning complete");
    }

    // TODO remove this duplication
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

    // TODO remove this duplication
    fn is_match(&self, path: &PathBuf) -> bool {
        self.regex.is_match(path.to_str().unwrap())
    }

}
