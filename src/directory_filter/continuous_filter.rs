use regex::Regex;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;

use directory_scanner::Directory;
use crossbeam;

use directory_filter::{FilteredDirectory, RegexBuilder};
use directory_filter::matchers::*;

pub struct ContinuousFilter {
    actual_filter: Arc<Mutex<Filter>>,
    done: Arc<AtomicBool>,
    pub finished_transmitter: Sender<bool>,
    finished_receiver: Receiver<bool>,
}

impl ContinuousFilter {

    pub fn new(directory: Arc<Mutex<Directory>>, filter_change_receiver: Arc<Mutex<Receiver<String>>>,
               new_directory_item_receiver: Arc<Mutex<Receiver<Directory>>>, filter_match_transmitter: Arc<Mutex<Sender<FilteredDirectory>>>) -> Self {

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

        info!("filter scanning started");
        crossbeam::scope(|scope| {
            let filter_change_receiver;
            let new_directory_item_receiver ;
            {
                let locked_filter = self.actual_filter.lock().unwrap();
                filter_change_receiver = locked_filter.filter_change_receiver.clone();
                new_directory_item_receiver = locked_filter.new_directory_item_receiver.clone();
            }

            // listen for filter change events and then kick off scan
            let local_filter = self.actual_filter.clone();
            let done = self.done.clone();
            scope.spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    match filter_change_receiver.lock().unwrap().recv() {
                        Ok(filter_string) => {
                            info!("Found new filter string: {}", filter_string);
                            let mut locked_filter = local_filter.lock().unwrap();
                            locked_filter.regex = RegexBuilder::new(filter_string).build();
                            locked_filter.scan();
                        },
                        Err(_) => {},
                    }
                }
            });

            // listen for new directory item events and then kick off scan
            let local_filter = self.actual_filter.clone();
            let done = self.done.clone();
            scope.spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    match new_directory_item_receiver.lock().unwrap().recv() {
                        Ok(directory) => {
                            let mut locked_filter = local_filter.lock().unwrap();
                            locked_filter.scan();
                        },
                        Err(_) => {}
                    }
                }
            });

            {
                let mut locked_filter = self.actual_filter.lock().unwrap();
                // initial scan
                locked_filter.scan();
            }

            self.wait_until_finished();
        });
    }


    //------------ private ----------//

    fn wait_until_finished(&self) {
        self.finished_receiver.recv().unwrap();
        self.done.store(true, Ordering::Relaxed);
    }

}

struct Filter {
    directory: Arc<Mutex<Directory>>,
    filter_change_receiver: Arc<Mutex<Receiver<String>>>,
    new_directory_item_receiver: Arc<Mutex<Receiver<Directory>>>,
    filter_match_transmitter: Arc<Mutex<Sender<FilteredDirectory>>>,
    results: FilteredDirectory,
    regex: Regex,
}

impl Filter {

    pub fn new(directory: Arc<Mutex<Directory>>, filter_change_receiver: Arc<Mutex<Receiver<String>>>,
               new_directory_item_receiver: Arc<Mutex<Receiver<Directory>>>, filter_match_transmitter: Arc<Mutex<Sender<FilteredDirectory>>>) -> Self {

      let filtered_directory = FilteredDirectory {
           matches: vec![],
           directory: directory.clone(),
      };

      Filter {
          directory: directory.clone(),
          filter_change_receiver: filter_change_receiver,
          new_directory_item_receiver: new_directory_item_receiver,
          filter_match_transmitter: filter_match_transmitter,
          results: filtered_directory,
          regex: Regex::new("").unwrap(),
      }
    }

    pub fn append(&mut self, directory: Directory) {
        self.directory.lock().unwrap().extend(&directory)
    }

    pub fn scan(&mut self) {
        info!("Filter is scanning through directory with");
        self.results.directory = self.directory.clone();
        let new_matches = find_matches(&self.directory, self.regex.clone());
        info!("Filter found {} matches", new_matches.len());
        if self.results.matches != new_matches {
            info!("Filter found matches to be different from previous emitting event");
            self.results.matches = new_matches;
            let _ = self.filter_match_transmitter.lock().unwrap().send(self.results.clone()); // TODO only send if there is a difference? or only send the delta?
        }
    }

}
