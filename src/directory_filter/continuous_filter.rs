use regex::Regex;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;

use directory_scanner::Directory;
use crossbeam;

use directory_filter::FilteredDirectory;
use directory_filter::matchers::*;

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
            let filter_change_receiver;
            let new_directory_item_receiver ;
            {
                let mut locked_filter = self.actual_filter.lock().unwrap();
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
                            let mut locked_filter = local_filter.lock().unwrap();
                            locked_filter.regex = Regex::new(&filter_string).unwrap();
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
                        Ok(new_directory_item) => {
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
        self.results.matches = find_matches(self.directory, self.regex.clone());
        let _ = self.filter_match_transmitter.send(self.results.clone()); // TODO only send if there is a difference? or only send the delta?
    }

}
