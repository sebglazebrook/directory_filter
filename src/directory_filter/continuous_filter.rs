use regex::Regex;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;

use directory_scanner::Directory;
use crossbeam;

use directory_filter::{FilteredDirectory, RegexBuilder, FILTER_EVENT_BROKER};

pub struct ContinuousFilter<'a> {
    actual_filter: Arc<Mutex<Filter<'a>>>,
    done: Arc<AtomicBool>,
    pub finished_transmitter: Sender<bool>,
    finished_receiver: Receiver<bool>,
}

impl<'a> ContinuousFilter<'a>{

    pub fn new(directory: Arc<Mutex<Directory>>, new_directory_item_receiver: Arc<Mutex<Receiver<Directory>>>,
               filter_match_transmitter: Arc<Mutex<Sender<FilteredDirectory<'a>>>>) -> Self {

      let actual_filter = Arc::new(
          Mutex::new(
              Filter::new(
                directory,
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

    pub fn start(&mut self) { // TODO could this return a FilteredDirectory that gets updated?

        info!("filter scanning started");
        crossbeam::scope(|scope| {
            let new_directory_item_receiver ;
            {
                let locked_filter = self.actual_filter.lock().unwrap();
                new_directory_item_receiver = locked_filter.new_directory_item_receiver.clone();
            }

            // listen for filter change events and then kick off scan
            let local_filter = self.actual_filter.clone();
            let done = self.done.clone();
            scope.spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    match FILTER_EVENT_BROKER.recv() {
                        Ok(filter_string)  => {
                            info!("Found new filter string: {}", filter_string);
                            let mut locked_filter = local_filter.lock().unwrap();
                            locked_filter.regex = RegexBuilder::new(filter_string).build();
                            locked_filter.scan();
                        },
                        Err(_) => {
                            done.store(true, Ordering::Relaxed);
                        }
                    }
                }
            });

            // listen for new directory item events and then kick off scan
            let local_filter = self.actual_filter.clone();
            let done = self.done.clone();
            scope.spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    match new_directory_item_receiver.lock().unwrap().recv() {
                        Ok(_) => {
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

// TODO remove new_directory_item_receiver from her
struct Filter<'a> {
    directory: Arc<Mutex<Directory>>,
    new_directory_item_receiver: Arc<Mutex<Receiver<Directory>>>,
    filter_match_transmitter: Arc<Mutex<Sender<FilteredDirectory<'a>>>>,
    filtered_directory: FilteredDirectory<'a>,
    regex: Regex,
}

impl<'a> Filter<'a> {

    pub fn new(directory: Arc<Mutex<Directory>>, new_directory_item_receiver: Arc<Mutex<Receiver<Directory>>>,
               filter_match_transmitter: Arc<Mutex<Sender<FilteredDirectory<'a>>>>) -> Self {

      let initial_regex = Regex::new("").unwrap();
      let filtered_directory = FilteredDirectory::new(directory.clone(), initial_regex.clone());

      Filter {
          directory: directory.clone(),
          new_directory_item_receiver: new_directory_item_receiver,
          filter_match_transmitter: filter_match_transmitter,
          filtered_directory: filtered_directory,
          regex: initial_regex,
      }
    }

    pub fn scan(&mut self) {
        info!("Filter is scanning through directory with");
        let mut new_filtered_directory = FilteredDirectory::new(self.directory.clone(), self.regex.clone());
        new_filtered_directory.run_filter();
        //if self.filtered_directory.matches != new_filtered_directory.matches {
        if self.filtered_directory.file_matches != new_filtered_directory.file_matches {
            info!("Filter found matches to be different from previous emitting event");
            self.filtered_directory = new_filtered_directory;
            let _ = self.filter_match_transmitter.lock().unwrap().send(self.filtered_directory.clone());
        }
    }

}
