use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam::sync::MsQueue;
use std::sync::Condvar;

lazy_static! {
    pub static ref FILTER_EVENT_BROKER: FilterEventBroker = {
        FilterEventBroker::new()
    };
}


pub struct FilterEventBroker {
    events: Arc<MsQueue<String>>,
    receiving_events: AtomicBool,
    mutex: Mutex<bool>,
    condvar: Condvar,
}

impl FilterEventBroker {

    pub fn new() -> Self {
        FilterEventBroker {
            events: Arc::new(MsQueue::new()),
            receiving_events: AtomicBool::new(true),
            condvar: Condvar::new(),
            mutex: Mutex::new(false),
        }
    }

    pub fn send(&self, filter_event: String) {
        self.events.push(filter_event);
        self.condvar.notify_one();
    }

    pub fn close(&self) {
        self.receiving_events.store(false, Ordering::Relaxed);
        self.condvar.notify_all();
    }

    pub fn try_recv(&self) -> Option<String> {
        let mut return_value = None;
        let mut done = false;
        while !done {
            match self.events.try_pop() {
                Some(event) => { return_value = Some(event); },
                None  => { done = true; }
            }
        }
        return_value
    }

    pub fn recv(&self) -> Result<String, &str>  {
        match self.try_recv() {
            Some(event) => { Ok(event) }
            None => {
                let mutex_guard = self.mutex.lock().unwrap();
                let _ = self.condvar.wait(mutex_guard).unwrap();
                if !self.receiving_events.load(Ordering::Relaxed) {
                    return Err("no longer receiving events"); //TODO send a real error type
                }
                let mut event = self.events.pop();
                let mut found_most_recent_event = false;
                while !found_most_recent_event {
                    if !self.receiving_events.load(Ordering::Relaxed) {
                    }
                    match self.events.try_pop() {
                        Some(newer_event) => { event = newer_event; },
                        None => { found_most_recent_event = true; }
                    }
                }
                Ok(event) // TODO handle when there is an error
            }
        }
    }
}
