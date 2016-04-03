use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
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
    pending_events: AtomicUsize,
}

impl FilterEventBroker {

    pub fn new() -> Self {
        FilterEventBroker {
            events: Arc::new(MsQueue::new()),
            receiving_events: AtomicBool::new(true),
            condvar: Condvar::new(),
            mutex: Mutex::new(false),
            pending_events: AtomicUsize::new(0),
        }
    }

    pub fn send(&self, filter_event: String) {
        self.events.push(filter_event);
        self.condvar.notify_one();
        self.pending_events.fetch_add(1, Ordering::Relaxed);
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
                Some(event) => {
                    return_value = Some(event);
                    self.pending_events.fetch_sub(1, Ordering::Relaxed);
                },
                None  => { done = true; }
            }
        }
        return_value
    }

    pub fn recv(&self) -> Result<String, &str>  {
        let mut return_event = Err("I dont't know");
        match self.try_recv() {
            Some(event) => { return_event = Ok(event); }
            None => {
                let mut done = false;
                while !done {
                    let mutex_guard = self.mutex.lock().unwrap();
                    let _ = self.condvar.wait(mutex_guard).unwrap();
                    if !self.receiving_events.load(Ordering::Relaxed) {
                        done = true;
                        return Err("no longer receiving events"); //TODO send a real error type
                    }
                    match self.try_recv() {
                        Some(event) =>  {
                            done = true;
                            return_event = Ok(event);
                        },
                        None => {}
                    }
                }
            }
        }
        return_event
    }

    pub fn has_pending_events(&self) -> bool {
        self.pending_events.load(Ordering::Relaxed) > 0
    }
}
