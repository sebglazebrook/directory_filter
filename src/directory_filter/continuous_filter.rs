use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::Sender;
use std::sync::atomic::{AtomicBool, Ordering};

use regex::Regex;
use crossbeam;
use directory_scanner::{Directory, DirectoryEventBroker};
use directory_filter::{FilteredDirectory, RegexBuilder, FILTER_EVENT_BROKER};

#[derive(Clone)]
pub struct ContinuousFilter {
    actual_filter: Arc<Mutex<Filter>>,
    done: Arc<AtomicBool>,
    pub finished_lock: Arc<Mutex<bool>>,
    pub finished_condvar: Arc<Condvar>,
    new_directory_item_event_broker: DirectoryEventBroker,
}

impl ContinuousFilter{

    pub fn new(directory: Directory,
               filter_match_transmitter: Arc<Mutex<Sender<FilteredDirectory>>>, new_directory_item_event_broker: DirectoryEventBroker) -> Self {

      let actual_filter = Arc::new(Mutex::new(Filter::new(directory, filter_match_transmitter)));

      let finished_lock = Arc::new(Mutex::new(false));
      let finished_condvar = Arc::new(Condvar::new());

      ContinuousFilter {
          actual_filter: actual_filter,
          done: Arc::new(AtomicBool::new(false)),
          finished_lock: finished_lock,
          finished_condvar: finished_condvar,
          new_directory_item_event_broker: new_directory_item_event_broker,
      }
    }

    pub fn start(&self) { // TODO could this return a FilteredDirectory that gets updated?

        info!("filter scanning started");
        crossbeam::scope(|scope| {

            // listen for filter change events and then kick off scan
            let local_filter = self.actual_filter.clone();
            let done = self.done.clone();
            scope.spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    match FILTER_EVENT_BROKER.recv() {
                        Ok(filter_string)  => {
                            info!("Found new filter string: {}", filter_string);
                            let mut locked_filter = local_filter.lock().unwrap();
                            locked_filter.rescan(RegexBuilder::new(filter_string).build());
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
                    match self.new_directory_item_event_broker.recv() {
                        Ok(_) => {
                            let mut locked_filter = local_filter.lock().unwrap();
                            locked_filter.scan();
                        },
                        Err(_) => {} // TODO handle this nicer?
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

    pub fn is_processing(&self) -> bool {
        FILTER_EVENT_BROKER.has_pending_events() || self.scanning_in_progress()
    }


    //------------ private ----------//

    fn wait_until_finished(&self) {
        let mut finished = self.finished_lock.lock().unwrap();
        while !*finished {
            finished = self.finished_condvar.wait(finished).unwrap();
        }
        self.done.store(true, Ordering::Relaxed);
    }

    fn scanning_in_progress(&self) -> bool {
       self.actual_filter.lock().unwrap().filtering_in_progress.load(Ordering::Relaxed)
    }

}

struct Filter {
    directory: Directory,
    filter_match_transmitter: Arc<Mutex<Sender<FilteredDirectory>>>,
    filtered_directory: FilteredDirectory,
    regex: Regex,
    pub filtering_in_progress: AtomicBool, // TODO make this private
}

impl Filter {

    pub fn new(directory: Directory, filter_match_transmitter: Arc<Mutex<Sender<FilteredDirectory>>>) -> Self {

      let initial_regex = Regex::new("").unwrap();
      let filtered_directory = FilteredDirectory::new(directory.clone(), initial_regex.clone());

      Filter {
          directory: directory.clone(),
          filter_match_transmitter: filter_match_transmitter,
          filtered_directory: filtered_directory,
          regex: initial_regex,
          filtering_in_progress: AtomicBool::new(false),
      }
    }

    pub fn scan(&mut self) {
        self.filtering_in_progress.store(true, Ordering::Relaxed);
        info!("Filter scanning");
        let mut new_filtered_directory = FilteredDirectory::new(self.directory.clone(), self.regex.clone());
        new_filtered_directory.run_filter();
        if self.filtered_directory.file_matches != new_filtered_directory.file_matches {
            info!("Filter found matches to be different from previous emitting event");
            self.filtered_directory = new_filtered_directory;
            let _ = self.filter_match_transmitter.lock().unwrap().send(self.filtered_directory.clone());
        }
        self.filtering_in_progress.store(false, Ordering::Relaxed);
    }

    pub fn rescan(&mut self, new_regex: Regex)  {
        self.filtering_in_progress.store(true, Ordering::Relaxed);
        info!("Filter rescanning using new regex: {:?}", new_regex);
        self.regex = new_regex.clone();
        self.filtered_directory.re_filter(new_regex);
        //if self.filtered_directory.file_matches != new_filtered_directory.file_matches {
            info!("Filter found matches to be different from previous emitting event");
            let _ = self.filter_match_transmitter.lock().unwrap().send(self.filtered_directory.clone());
        //}
        self.filtering_in_progress.store(false, Ordering::Relaxed);
    }

}
