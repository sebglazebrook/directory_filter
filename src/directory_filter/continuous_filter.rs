use std::thread;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;

use directory_scanner::Directory;
use crossbeam;

use directory_filter::FilteredDirectory;

pub struct ContinuousFilter<'a> {
    actual_filter: Arc<Mutex<Filter<'a>>>,
    done: Arc<AtomicBool>,
    pub finished_transmitter: Sender<bool>,
    finished_receiver: Receiver<bool>,
}

impl<'a> ContinuousFilter<'a> {

    pub fn new(directory: &'a Directory, filter_change_receiver: Receiver<String>,
               new_directory_item_receiver: Receiver<Directory>, filter_match_transmitter: Sender<FilteredDirectory<'a>>) -> Self {

      let actual_filter = Arc::new(
          Mutex::new(
              Filter::new(
                directory,
                filter_change_receiver,
                new_directory_item_receiver,
                filter_match_transmitter
              )
          )
      );

      let (tx, rx) = channel();
      ContinuousFilter {
          actual_filter: actual_filter,
          done: Arc::new(AtomicBool::new(false)),
          finished_transmitter: tx,
          finished_receiver: rx,
      }
    }

    pub fn start(&mut self) {

        crossbeam::scope(|scope| {
            let mut locked_filter = self.actual_filter.lock().unwrap();
            let local_filter = self.actual_filter.clone();

            // listen for filter change events and then kick off scan
            let filter_change_receiver = locked_filter.filter_change_receiver.clone();
            let done = self.done.clone();
            scope.spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    println!("waiting for filter changes");
                    let filter_string = filter_change_receiver.lock().unwrap().recv().unwrap(); // TODO handle this better
                    println!("new filter string: {}", filter_string);
                    println!("Rescanning!");
                    local_filter.lock().unwrap().scan();
                }
            });

            // listen for new directory item events and then kick off scan
            let new_directory_item_receiver = locked_filter.new_directory_item_receiver.clone();
            let local_filter = self.actual_filter.clone();
            let done = self.done.clone();
            scope.spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    println!("waiting for directory changes");
                    let new_directory_item = new_directory_item_receiver.lock().unwrap().recv().unwrap(); // TODO handle this better
                    println!("new directory item: {:?}", new_directory_item);
                    println!("Rescanning!");
                    local_filter.lock().unwrap().scan();
                }
            });

            // initial scan
            locked_filter.scan();


            self.finished_receiver.recv().unwrap();
            self.done.store(true, Ordering::Relaxed);
        });
    }

}

struct Filter<'a> {
    directory: &'a Directory,
    filter_change_receiver: Arc<Mutex<Receiver<String>>>,
    new_directory_item_receiver: Arc<Mutex<Receiver<Directory>>>,
    filter_match_transmitter: Sender<FilteredDirectory<'a>>,
    results: FilteredDirectory<'a>,
    regex: Regex,
}

impl<'a> Filter<'a> {

    pub fn new(directory: &'a Directory, filter_change_receiver: Receiver<String>,
               new_directory_item_receiver: Receiver<Directory>, filter_match_transmitter: Sender<FilteredDirectory<'a>>) -> Self {

      let filtered_directory = FilteredDirectory {
           matches: vec![],
           directory: directory,
      };

      Filter {
          directory: directory,
          filter_change_receiver: Arc::new(Mutex::new(filter_change_receiver)),
          new_directory_item_receiver: Arc::new(Mutex::new(new_directory_item_receiver)),
          filter_match_transmitter: filter_match_transmitter,
          results: filtered_directory,
          regex: Regex::new("").unwrap(),
      }
    }

    pub fn scan(&mut self) {
        self.results.directory = self.directory;
        self.results.matches = self.find_matches(self.directory);
        println!("Sending matches: {:?}", self.results.matches.len());
        self.filter_match_transmitter.send(self.results.clone()); // TODO only send if there is a difference? or only send the delta?
    }

    // -------- private ----------- //

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
